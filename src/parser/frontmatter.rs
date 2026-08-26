use anyhow::{Context, Result};
use chrono::NaiveDate;
use serde::{de::Deserializer, Deserialize};
use std::path::Path;

#[derive(Debug, Clone, Default, Deserialize)]
pub struct FrontMatter {
    #[serde(default)]
    pub title: Option<String>,
    #[serde(default)]
    pub date: Option<String>,
    #[serde(rename = "post-date", default)]
    pub post_date: Option<String>,
    #[serde(default)]
    pub category: Option<String>,
    #[serde(default, deserialize_with = "deserialize_string_vec")]
    pub categories: Vec<String>,
    #[serde(default, deserialize_with = "deserialize_string_vec")]
    pub tags: Vec<String>,
    #[serde(default)]
    pub summary: Option<String>,
    #[serde(default)]
    pub order: Option<usize>,
    #[serde(default)]
    pub label: Option<String>,
    #[serde(default)]
    pub path: Option<String>,
    #[serde(default)]
    pub layout: Option<String>,
    #[serde(default)]
    pub slug: Option<String>,
    #[serde(default)]
    pub draft: bool,
    #[serde(default)]
    pub private: bool,
}

fn deserialize_string_vec<'de, D>(deserializer: D) -> Result<Vec<String>, D::Error>
where
    D: Deserializer<'de>,
{
    let value = Option::<serde_yaml::Value>::deserialize(deserializer)?;

    match value {
        None | Some(serde_yaml::Value::Null) => Ok(Vec::new()),
        Some(serde_yaml::Value::String(value)) => Ok(vec![value]),
        Some(serde_yaml::Value::Sequence(values)) => Ok(values
            .into_iter()
            .filter_map(|value| value.as_str().map(ToOwned::to_owned))
            .collect()),
        Some(value) => Err(serde::de::Error::custom(format!(
            "expected a string or list of strings, got {value:?}"
        ))),
    }
}

/// Split a Markdown file into optional YAML frontmatter and body.
///
/// Delimiters are recognized by complete lines, so a `---` occurring inside
/// the article body cannot accidentally terminate the metadata block.
pub fn split_frontmatter(content: &str) -> (Option<&str>, &str) {
    let mut lines = content.split_inclusive('\n');
    let Some(first_line) = lines.next() else {
        return (None, content);
    };

    if first_line.trim() != "---" {
        return (None, content);
    }

    let yaml_start = first_line.len();
    let mut cursor = yaml_start;

    for line in lines {
        if line.trim() == "---" {
            let yaml = &content[yaml_start..cursor];
            let body = content[cursor + line.len()..].trim_start();
            return (Some(yaml), body);
        }
        cursor += line.len();
    }

    (None, content)
}

pub fn parse_frontmatter(content: &str) -> Result<Option<FrontMatter>> {
    let Some(yaml_content) = split_frontmatter(content).0 else {
        return Ok(None);
    };

    let frontmatter =
        serde_yaml::from_str(yaml_content).context("Failed to parse frontmatter YAML")?;

    Ok(Some(frontmatter))
}

pub fn extract_filename_date(filename: &str) -> Result<NaiveDate> {
    let stem = Path::new(filename)
        .file_stem()
        .and_then(|s| s.to_str())
        .context("Invalid filename")?;

    let candidates = [
        stem.chars().take(10).collect::<String>(),
        stem.chars().take(8).collect::<String>(),
    ];

    for candidate in candidates {
        let format = if candidate.len() == 10 && candidate.as_bytes().get(4) == Some(&b'-') {
            "%Y-%m-%d"
        } else {
            "%Y%m%d"
        };

        if let Ok(date) = NaiveDate::parse_from_str(&candidate, format) {
            return Ok(date);
        }
    }

    anyhow::bail!("Filename does not contain a supported date")
}

pub fn extract_directory_date(path: &Path) -> Option<NaiveDate> {
    let parent = path.parent()?;
    let parent_name = parent.file_name()?.to_str()?;

    if let Ok(year) = parent_name.parse::<i32>() {
        if (1000..=9999).contains(&year) {
            let month = parent
                .parent()
                .and_then(|p| p.file_name())
                .and_then(|s| s.to_str())
                .and_then(|value| value.parse::<u32>().ok())?;
            return NaiveDate::from_ymd_opt(year, month, 1);
        }
    }

    if parent_name.len() == 2 {
        let year = parent
            .parent()
            .and_then(|p| p.file_name())
            .and_then(|s| s.to_str())
            .and_then(|value| value.parse::<i32>().ok())?;
        let month = parent_name.parse::<u32>().ok()?;
        return NaiveDate::from_ymd_opt(year, month, 1);
    }

    None
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct PageItem {
    pub order: Option<usize>,
    pub label: String,
    pub path: String,
    pub filename: String,
    pub url: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_ablog_frontmatter() {
        let content = r#"---
title: Test Post
post-date: "2024-1-15 10:20"
categories: [rust, blog]
tags: rust
draft: false
---

This is the content."#;

        let frontmatter = parse_frontmatter(content).unwrap().unwrap();
        assert_eq!(frontmatter.title.as_deref(), Some("Test Post"));
        assert_eq!(frontmatter.post_date.as_deref(), Some("2024-1-15 10:20"));
        assert_eq!(frontmatter.categories, vec!["rust", "blog"]);
        assert_eq!(frontmatter.tags, vec!["rust"]);
    }

    #[test]
    fn test_split_frontmatter() {
        let content = "---\ntitle: Test\n---\n\nBody";
        let (yaml, body) = split_frontmatter(content);
        assert!(yaml.unwrap().contains("title: Test"));
        assert_eq!(body, "Body");
    }

    #[test]
    fn test_extract_filename_date() {
        assert_eq!(
            extract_filename_date("2024-01-15-my-post.md")
                .unwrap()
                .to_string(),
            "2024-01-15"
        );
        assert_eq!(
            extract_filename_date("20240115-my-post.md")
                .unwrap()
                .to_string(),
            "2024-01-15"
        );
    }
}
