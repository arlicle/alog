// Markdown parsing module
use super::frontmatter::{extract_directory_date, extract_filename_date, parse_frontmatter};
use anyhow::{Context, Result};
use chrono::NaiveDate;
use pulldown_cmark::{html, Parser as MdParser};
use serde::Serialize;
use std::path::Path;

#[derive(Debug, Clone, Serialize)]
pub struct PostMetadata {
    pub title: String,
    pub date: NaiveDate,
    pub category: String,
    pub tags: Vec<String>,
    pub summary: Option<String>,
    pub slug: String,
    pub created_at: String,
    // Formatted fields for templates
    pub year: String,
    pub month: String,
    pub day: String,
    pub formatted_date: String,
    pub html_content: String,
}

fn generate_summary(markdown_content: &str) -> Option<String> {
    // Get the first paragraph or first 200 characters
    let content = markdown_content.trim();
    
    // Try to extract first paragraph
    if let Some(end_pos) = content.find("\n\n") {
        let first_para = &content[..end_pos];
        // Remove markdown syntax for summary
        let clean = first_para
            .replace("# ", "")
            .replace("## ", "")
            .replace("### ", "")
            .replace("**", "")
            .replace("*", "")
            .replace("`", "")
            .trim()
            .to_string();
        
        // Limit to 200 characters
        if clean.len() > 200 {
            Some(format!("{}...", &clean[..200]))
        } else {
            Some(clean)
        }
    } else {
        // Fallback to first 200 characters
        let clean = content
            .replace("# ", "")
            .replace("## ", "")
            .replace("### ", "")
            .replace("**", "")
            .replace("*", "")
            .replace("`", "")
            .trim()
            .to_string();
        
        if clean.len() > 200 {
            Some(format!("{}...", &clean[..200]))
        } else {
            Some(clean)
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct BlogPost {
    pub metadata: PostMetadata,
    pub content: String,
}

impl BlogPost {
    pub fn from_file(path: &Path) -> Result<Self> {
        let content = std::fs::read_to_string(path)
            .with_context(|| format!("Failed to read markdown file: {}", path.display()))?;

        let metadata = std::fs::metadata(path)?;
        let modified = metadata.modified()?;
        let created_at: String = chrono::DateTime::<chrono::Utc>::from(modified)
            .format("%Y-%m-%d %H:%M:%S")
            .to_string();

        let filename = path
            .file_name()
            .and_then(|s| s.to_str())
            .context("Invalid filename")?;

        let frontmatter = parse_frontmatter(&content)?;
        let (metadata, markdown_content) = if let Some(fm) = frontmatter {
            // Priority 1: Try frontmatter date
            let date = NaiveDate::parse_from_str(&fm.date, "%Y-%m-%d")
                .ok()
                // Priority 2: Try filename date
                .or_else(|| extract_filename_date(filename).ok())
                // Priority 3: Try directory date
                .or_else(|| extract_directory_date(path))
                // Priority 4: Try file creation time
                .unwrap_or_else(|| {
                    // Extract date from file creation time
                    chrono::DateTime::<chrono::Utc>::from(modified)
                        .date_naive()
                });
            
            let slug = filename.strip_suffix(".md").unwrap_or(filename);
            
            // Remove frontmatter from content
            let remaining_content = if content.starts_with("---") {
                if let Some(end_pos) = content[3..].find("---") {
                    content[3 + end_pos + 3..].trim().to_string()
                } else {
                    content
                }
            } else {
                content.clone()
            };
            
            // Auto-generate summary if not provided
            let summary = fm.summary.or_else(|| generate_summary(&remaining_content));
            
            let metadata = PostMetadata {
                title: fm.title,
                date,
                category: fm.category,
                tags: fm.tags,
                summary,
                slug: slug.to_string(),
                created_at: created_at.clone(),
                year: date.format("%Y").to_string(),
                month: date.format("%m").to_string(),
                day: date.format("%d").to_string(),
                formatted_date: date.format("%B %d, %Y").to_string(),
                html_content: String::new(),
            };
            
            (metadata, remaining_content)
        } else {
            // Priority 2: Try filename date
            // Priority 3: Try directory date
            // Priority 4: Try file creation time
            let date = extract_filename_date(filename)
                .ok()
                .or_else(|| extract_directory_date(path))
                .unwrap_or_else(|| {
                    // Extract date from file creation time
                    chrono::DateTime::<chrono::Utc>::from(modified)
                        .date_naive()
                });
            let slug = filename.strip_suffix(".md").unwrap_or(filename);
            
            // Auto-generate summary
            let summary = generate_summary(&content);
            
            let metadata = PostMetadata {
                title: slug.to_string(),
                date,
                category: String::new(),
                tags: vec![],
                summary,
                slug: slug.to_string(),
                created_at: created_at,
                year: date.format("%Y").to_string(),
                month: date.format("%m").to_string(),
                day: date.format("%d").to_string(),
                formatted_date: date.format("%B %d, %Y").to_string(),
                html_content: String::new(),
            };
            
            (metadata, content.clone())
        };

        let html_content = markdown_to_html(&markdown_content);
        
        // Set the html_content in metadata
        let mut metadata = metadata;
        metadata.html_content = html_content;

        Ok(BlogPost {
            metadata,
            content: markdown_content,
        })
    }
}

pub fn markdown_to_html(markdown: &str) -> String {
    let parser = MdParser::new(markdown);
    let mut html_output = String::new();
    html::push_html(&mut html_output, parser);
    html_output
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_markdown_to_html() {
        let markdown = "# Hello\n\nThis is **bold** text.";
        let html = markdown_to_html(markdown);
        assert!(html.contains("<h1>Hello</h1>"));
        assert!(html.contains("<strong>bold</strong>"));
    }
}