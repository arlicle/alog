use super::frontmatter::{
    extract_directory_date, extract_filename_date, parse_frontmatter, split_frontmatter,
    FrontMatter,
};
use anyhow::{Context, Result};
use chrono::{Datelike, NaiveDate, NaiveDateTime};
use pulldown_cmark::{html, Parser as MdParser};
use serde::Serialize;
use std::path::Path;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ContentKind {
    Post,
    Page,
}

#[derive(Debug, Clone, Serialize)]
pub struct PostMetadata {
    pub title: String,
    pub date: NaiveDate,
    pub category: String,
    pub categories: Vec<String>,
    pub tags: Vec<String>,
    pub summary: Option<String>,
    pub slug: String,
    pub url: String,
    pub created_at: String,
    pub year: String,
    pub month: String,
    pub day: String,
    pub formatted_date: String,
    pub html_content: String,
    pub order: Option<usize>,
    pub label: Option<String>,
    pub path: Option<String>,
    pub layout: Option<String>,
    pub draft: bool,
    pub private: bool,
    pub kind: String,
}

fn generate_summary(markdown_content: &str) -> Option<String> {
    let content = markdown_content.trim();
    if content.is_empty() {
        return None;
    }

    let first_para = content.split_once("\n\n").map_or(content, |(para, _)| para);
    let clean = first_para
        .replace("# ", "")
        .replace("## ", "")
        .replace("### ", "")
        .replace("**", "")
        .replace(['*', '`'], "")
        .trim()
        .to_string();

    Some(truncate_chars(&clean, 200))
}

fn truncate_chars(value: &str, max_chars: usize) -> String {
    if value.chars().count() <= max_chars {
        return value.to_string();
    }

    let truncated: String = value.chars().take(max_chars).collect();
    format!("{truncated}...")
}

fn parse_date_value(value: &str) -> Option<NaiveDate> {
    let value = value.trim();
    let date_formats = ["%Y-%m-%d", "%Y-%-m-%-d", "%Y/%m/%d", "%Y/%-m/%-d"];
    let datetime_formats = [
        "%Y-%m-%d %H:%M",
        "%Y-%m-%d %H:%M:%S",
        "%Y-%-m-%-d %H:%M",
        "%Y-%-m-%-d %H:%M:%S",
        "%Y/%m/%d %H:%M",
        "%Y/%-m/%-d %H:%M",
        "%Y%m%d%H%M",
    ];

    for format in datetime_formats {
        if let Ok(date) = NaiveDateTime::parse_from_str(value, format) {
            return Some(date.date());
        }
    }

    for format in date_formats {
        if let Ok(date) = NaiveDate::parse_from_str(value, format) {
            return Some(date);
        }
    }

    None
}

fn fallback_title(filename: &str) -> String {
    let stem = Path::new(filename)
        .file_stem()
        .and_then(|value| value.to_str())
        .unwrap_or(filename);

    stem.trim_start_matches(|value: char| value.is_ascii_digit() || value == '-')
        .trim()
        .to_string()
}

fn normalize_slug(value: &str) -> String {
    value
        .split_whitespace()
        .collect::<Vec<_>>()
        .join("-")
        .replace(['/', '\\'], "-")
        .trim_matches('-')
        .to_string()
}

fn derive_slug(filename: &str, frontmatter: Option<&FrontMatter>, kind: ContentKind) -> String {
    let explicit_slug = frontmatter.and_then(|fm| {
        if kind == ContentKind::Page {
            fm.path.as_deref().or(fm.slug.as_deref())
        } else {
            fm.slug.as_deref()
        }
    });

    if let Some(slug) = explicit_slug.filter(|value| !value.trim().is_empty()) {
        return normalize_slug(slug);
    }

    let stem = Path::new(filename)
        .file_stem()
        .and_then(|value| value.to_str())
        .unwrap_or(filename);
    let without_date =
        stem.trim_start_matches(|value: char| value.is_ascii_digit() || value == '-');
    normalize_slug(if without_date.is_empty() {
        stem
    } else {
        without_date
    })
}

fn ablog_url(date: NaiveDate, slug: &str, kind: ContentKind) -> String {
    match kind {
        ContentKind::Post => format!(
            "/p/{}/{}/{}/{}/",
            date.format("%Y"),
            date.month(),
            date.day(),
            slug
        ),
        ContentKind::Page => format!("/p/{slug}/"),
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct BlogPost {
    pub metadata: PostMetadata,
    pub content: String,
}

impl BlogPost {
    pub fn from_file(path: &Path) -> Result<Self> {
        Self::from_file_with_kind(path, ContentKind::Post)
    }

    pub fn from_file_with_kind(path: &Path, kind: ContentKind) -> Result<Self> {
        let content = std::fs::read_to_string(path)
            .with_context(|| format!("Failed to read markdown file: {}", path.display()))?;

        let file_metadata = std::fs::metadata(path)?;
        let modified = file_metadata.modified()?;
        let created_at = chrono::DateTime::<chrono::Utc>::from(modified)
            .format("%Y-%m-%d %H:%M:%S")
            .to_string();
        let filename = path
            .file_name()
            .and_then(|value| value.to_str())
            .context("Invalid filename")?;

        let (frontmatter, markdown_content) = match parse_frontmatter(&content) {
            Ok(frontmatter) => (frontmatter, split_frontmatter(&content).1.to_string()),
            Err(error) => {
                eprintln!(
                    "Warning: invalid frontmatter in {}: {}; using filename defaults",
                    path.display(),
                    error
                );
                (None, split_frontmatter(&content).1.to_string())
            }
        };

        let date = frontmatter
            .as_ref()
            .and_then(|fm| fm.date.as_deref().or(fm.post_date.as_deref()))
            .and_then(parse_date_value)
            .or_else(|| extract_filename_date(filename).ok())
            .or_else(|| extract_directory_date(path))
            .unwrap_or_else(|| chrono::DateTime::<chrono::Utc>::from(modified).date_naive());

        let title = frontmatter
            .as_ref()
            .and_then(|fm| fm.title.clone())
            .filter(|value| !value.trim().is_empty())
            .unwrap_or_else(|| fallback_title(filename));

        let mut categories = frontmatter
            .as_ref()
            .map(|fm| fm.categories.clone())
            .unwrap_or_default();
        if categories.is_empty() {
            if let Some(category) = frontmatter.as_ref().and_then(|fm| fm.category.clone()) {
                categories.push(category);
            }
        }
        let category = categories.first().cloned().unwrap_or_default();
        let slug = derive_slug(filename, frontmatter.as_ref(), kind);
        let url = ablog_url(date, &slug, kind);
        let html_content = markdown_to_html(&markdown_content);

        let metadata = PostMetadata {
            title,
            date,
            category,
            categories,
            tags: frontmatter
                .as_ref()
                .map(|fm| fm.tags.clone())
                .unwrap_or_default(),
            summary: frontmatter
                .as_ref()
                .and_then(|fm| fm.summary.clone())
                .or_else(|| generate_summary(&markdown_content)),
            slug,
            url,
            created_at,
            year: date.format("%Y").to_string(),
            month: date.format("%m").to_string(),
            day: date.format("%d").to_string(),
            formatted_date: date.format("%Y-%m-%d").to_string(),
            html_content,
            order: frontmatter.as_ref().and_then(|fm| fm.order),
            label: frontmatter.as_ref().and_then(|fm| fm.label.clone()),
            path: frontmatter.as_ref().and_then(|fm| fm.path.clone()),
            layout: frontmatter.as_ref().and_then(|fm| fm.layout.clone()),
            draft: frontmatter.as_ref().map(|fm| fm.draft).unwrap_or(false),
            private: frontmatter.as_ref().map(|fm| fm.private).unwrap_or(false),
            kind: match kind {
                ContentKind::Post => "post".to_string(),
                ContentKind::Page => "page".to_string(),
            },
        };

        Ok(BlogPost {
            metadata,
            content: markdown_content,
        })
    }

    pub fn is_public(&self) -> bool {
        !self.metadata.draft && !self.metadata.private
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
    use std::fs;

    #[test]
    fn test_markdown_to_html() {
        let markdown = "# Hello\n\nThis is **bold** text.";
        let html = markdown_to_html(markdown);
        assert!(html.contains("<h1>Hello</h1>"));
        assert!(html.contains("<strong>bold</strong>"));
    }

    #[test]
    fn test_parse_post_date_without_title_or_date() {
        let temp_path = std::env::temp_dir().join("20190103-The road not taken.md");
        fs::write(
            &temp_path,
            "---\npost-date: \"2019-1-3 10:20\"\n---\n\n中文内容很长。",
        )
        .unwrap();

        let post = BlogPost::from_file(&temp_path).unwrap();
        assert_eq!(post.metadata.date.to_string(), "2019-01-03");
        assert_eq!(post.metadata.title, "The road not taken");
        assert_eq!(post.metadata.slug, "The-road-not-taken");
        let _ = fs::remove_file(temp_path);
    }

    #[test]
    fn test_page_url_and_draft_flag() {
        let temp_path = std::env::temp_dir().join("alog-about-me.md");
        fs::write(
            &temp_path,
            "---\ntitle: About\npath: about-me\ndraft: true\n---\n\nPage",
        )
        .unwrap();

        let page = BlogPost::from_file_with_kind(&temp_path, ContentKind::Page).unwrap();
        assert_eq!(page.metadata.url, "/p/about-me/");
        assert!(!page.is_public());
        let _ = fs::remove_file(temp_path);
    }
}
