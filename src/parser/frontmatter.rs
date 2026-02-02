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
    #[serde(default)]
    pub order: Option<usize>,
    #[serde(default)]
    pub label: Option<String>,
    #[serde(default)]
    pub path: Option<String>,
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

/// Page item for sidebar navigation
#[derive(Debug, Clone, serde::Serialize, Deserialize)]
pub struct PageItem {
    pub order: Option<usize>,
    pub label: String,
    pub path: String,
    pub filename: String,
}

/// Parse a markdown file from the pages directory
pub fn parse_page_frontmatter(content: &str, filename: &str) -> Result<PageItem> {
    let trimmed = content.trim_start();

    // Parse frontmatter if exists
    let (order, label, path) = if trimmed.starts_with("---") {
        if let Some(end_delimiter) = trimmed[3..].find("---") {
            let yaml_content = &trimmed[3..3 + end_delimiter];

            // Parse only the fields we need for pages
            #[derive(Deserialize)]
            struct PageFrontMatter {
                #[serde(default)]
                order: Option<usize>,
                #[serde(default)]
                label: Option<String>,
                #[serde(default)]
                title: Option<String>,
                #[serde(default)]
                path: Option<String>,
            }

            if let Ok(page_fm) = serde_yaml::from_str::<PageFrontMatter>(yaml_content) {
                let label = page_fm.label.or(page_fm.title).unwrap_or_else(|| {
                    // Use filename without .md extension as fallback
                    filename.strip_suffix(".md").unwrap_or(filename).to_string()
                });

                let path = page_fm.path.unwrap_or_else(|| {
                    // Use filename without .md extension as path
                    filename.strip_suffix(".md").unwrap_or(filename).to_string()
                });

                (page_fm.order, label, path)
            } else {
                // Failed to parse, use defaults
                let label = filename.strip_suffix(".md").unwrap_or(filename).to_string();
                let path = filename.strip_suffix(".md").unwrap_or(filename).to_string();
                (None, label, path)
            }
        } else {
            // Invalid frontmatter, use defaults
            let label = filename.strip_suffix(".md").unwrap_or(filename).to_string();
            let path = filename.strip_suffix(".md").unwrap_or(filename).to_string();
            (None, label, path)
        }
    } else {
        // No frontmatter, use defaults
        let label = filename.strip_suffix(".md").unwrap_or(filename).to_string();
        let path = filename.strip_suffix(".md").unwrap_or(filename).to_string();
        (None, label, path)
    };

    Ok(PageItem {
        order,
        label,
        path,
        filename: filename.to_string(),
    })
}

#[cfg(test)]
mod tests_pages {
    use super::*;

    #[test]
    fn test_parse_page_frontmatter_with_all_fields() {
        let content = r#"---
order: 1
label: About Me
path: about
title: About Page
---

Content here."#;

        let page = parse_page_frontmatter(content, "test.md").unwrap();
        assert_eq!(page.order, Some(1));
        assert_eq!(page.label, "About Me");
        assert_eq!(page.path, "about");
    }

    #[test]
    fn test_parse_page_frontmatter_without_label() {
        let content = r#"---
order: 2
path: contact
title: Contact Us
---

Content here."#;

        let page = parse_page_frontmatter(content, "test.md").unwrap();
        assert_eq!(page.order, Some(2));
        assert_eq!(page.label, "Contact Us"); // Falls back to title
        assert_eq!(page.path, "contact");
    }

    #[test]
    fn test_parse_page_frontmatter_without_order() {
        let content = r#"---
label: FAQ
path: faq
---

Content here."#;

        let page = parse_page_frontmatter(content, "test.md").unwrap();
        assert_eq!(page.order, None);
        assert_eq!(page.label, "FAQ");
        assert_eq!(page.path, "faq");
    }

    #[test]
    fn test_parse_page_frontmatter_without_frontmatter() {
        let content = "Just content without frontmatter";
        let page = parse_page_frontmatter(content, "test.md").unwrap();
        assert_eq!(page.order, None);
        assert_eq!(page.label, "test");
        assert_eq!(page.path, "test");
    }
}