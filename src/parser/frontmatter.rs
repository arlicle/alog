// Frontmatter parsing module
use anyhow::{Context, Result};
use chrono::NaiveDate;
use serde::Deserialize;
use std::path::Path;

#[derive(Debug, Deserialize)]
pub struct FrontMatter {
    pub title: String,
    pub date: String,
    #[serde(default)]
    pub category: String,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub summary: Option<String>,
}

pub fn parse_frontmatter(content: &str) -> Result<Option<FrontMatter>> {
    let trimmed = content.trim_start();

    if !trimmed.starts_with("---") {
        return Ok(None);
    }

    let end_delimiter = trimmed[3..].find("---").context("Missing closing frontmatter delimiter")?;
    let yaml_content = &trimmed[3..3 + end_delimiter];

    let frontmatter: FrontMatter =
        serde_yaml::from_str(yaml_content).context("Failed to parse frontmatter YAML")?;

    Ok(Some(frontmatter))
}

pub fn extract_filename_date(filename: &str) -> Result<NaiveDate> {
    let stem = Path::new(filename)
        .file_stem()
        .and_then(|s| s.to_str())
        .context("Invalid filename")?;

    // Check if the stem starts with YYYY-MM-DD format
    // We need at least 10 characters for the date
    if stem.len() < 10 {
        anyhow::bail!("Filename too short to contain date");
    }

    // Take the first 10 characters safely
    let date_str: String = stem.chars().take(10).collect();
    
    // Verify it's a valid date format
    NaiveDate::parse_from_str(&date_str, "%Y-%m-%d")
        .context("Failed to parse date from filename, expected YYYY-MM-DD format")
}

pub fn extract_directory_date(path: &Path) -> Option<NaiveDate> {
    // Try to extract date from path structure like md/YYYY/MM/
    let parent = path.parent()?;
    
    // Get the immediate parent (MM folder)
    let month_str = parent.file_name()?.to_str()?;
    if month_str.len() != 2 {
        return None;
    }
    
    // Get the grandparent (YYYY folder)
    let year_str = parent.parent()?.file_name()?.to_str()?;
    if year_str.len() != 4 {
        return None;
    }
    
    // Try to parse YYYY-MM
    let date_str = format!("{}-{}-01", year_str, month_str);
    NaiveDate::parse_from_str(&date_str, "%Y-%m-%d").ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_frontmatter() {
        let content = r#"---
title: Test Post
date: 2024-01-15
tags: [rust, blog]
summary: A test post
---

This is the content."#;

        let frontmatter = parse_frontmatter(content).unwrap();
        assert!(frontmatter.is_some());
        let fm = frontmatter.unwrap();
        assert_eq!(fm.title, "Test Post");
        assert_eq!(fm.date, "2024-01-15");
    }

    #[test]
    fn test_extract_filename_date() {
        let date = extract_filename_date("2024-01-15-my-post.md").unwrap();
        assert_eq!(date.to_string(), "2024-01-15");
    }
}