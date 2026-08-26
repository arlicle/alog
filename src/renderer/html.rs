use super::template::{
    render_list, render_page, render_post, CommentsConfigTemplate, PageLink, PostNav,
};
use crate::config::Config;
use crate::parser::BlogPost;
use anyhow::{Context, Result};
use chrono::Datelike;
use std::fs;
use std::path::{Path, PathBuf};

const POSTS_PREFIX: &str = "/p";
const LIST_PREFIX: &str = "/p/list";
const POSTS_PER_PAGE: usize = 15;

pub fn generate_site(posts: &[BlogPost], pages: &[BlogPost], config: &Config) -> Result<()> {
    fs::create_dir_all(&config.output_dir).context("Failed to create output directory")?;
    clean_generated_output(config)?;
    copy_theme_assets(config)?;

    let mut sorted_posts: Vec<&BlogPost> = posts
        .iter()
        .filter(|post| post.metadata.kind == "post" && post.is_public())
        .collect();
    sorted_posts.sort_by(|left, right| {
        right
            .metadata
            .date
            .cmp(&left.metadata.date)
            .then_with(|| right.metadata.created_at.cmp(&left.metadata.created_at))
            .then_with(|| left.metadata.title.cmp(&right.metadata.title))
    });

    let public_pages: Vec<&BlogPost> = pages
        .iter()
        .filter(|page| page.metadata.kind == "page" && page.is_public())
        .collect();
    let page_links = build_page_links(&public_pages);
    let comments_config = build_comments_config(config);

    if let Some(latest) = sorted_posts.first() {
        let html = render_post(
            latest,
            &page_links,
            &config.site_title,
            None,
            sorted_posts.get(1).map(|post| post_nav(post)),
            config.comments.enabled,
            comments_config.clone(),
        )?;
        write_output(&config.output_dir.join("index.html"), html)?;
    }

    for (index, post) in sorted_posts.iter().enumerate() {
        let previous = index
            .checked_sub(1)
            .and_then(|position| sorted_posts.get(position));
        let next = sorted_posts.get(index + 1).copied();
        let html = render_post(
            post,
            &page_links,
            &config.site_title,
            previous.map(|post| post_nav(post)),
            next.map(post_nav),
            config.comments.enabled,
            comments_config.clone(),
        )?;
        write_output(&post_output_path(&config.output_dir, post), html)?;
    }

    for page in &public_pages {
        let html = render_page(
            page,
            &page_links,
            &config.site_title,
            config.comments.enabled,
            comments_config.clone(),
        )?;
        write_output(&page_output_path(&config.output_dir, page), html)?;
    }

    generate_list_pages(
        &sorted_posts,
        &page_links,
        &config.site_title,
        &config.output_dir,
        LIST_PREFIX,
        "永远保持初学者的心",
    )?;

    for category in get_all_categories(&sorted_posts) {
        let category_posts: Vec<&BlogPost> = sorted_posts
            .iter()
            .copied()
            .filter(|post| {
                post.metadata
                    .categories
                    .iter()
                    .any(|value| value.eq_ignore_ascii_case(&category))
            })
            .collect();
        let base_url = format!("{POSTS_PREFIX}/cat/{}", sanitize_segment(&category));
        generate_list_pages(
            &category_posts,
            &page_links,
            &config.site_title,
            &config.output_dir,
            &base_url,
            &format!("分类：{category}"),
        )?;
    }

    for tag in get_all_tags(&sorted_posts) {
        let tag_posts: Vec<&BlogPost> = sorted_posts
            .iter()
            .copied()
            .filter(|post| {
                post.metadata
                    .tags
                    .iter()
                    .any(|value| value.eq_ignore_ascii_case(&tag))
            })
            .collect();
        let base_url = format!("{POSTS_PREFIX}/tag/{}", sanitize_segment(&tag));
        generate_list_pages(
            &tag_posts,
            &page_links,
            &config.site_title,
            &config.output_dir,
            &base_url,
            &format!("标签：{tag}"),
        )?;
    }

    super::rss::save_rss_feed(
        &sorted_posts.iter().copied().cloned().collect::<Vec<_>>(),
        config,
        "http://localhost:7878",
    )?;

    Ok(())
}

fn build_page_links(pages: &[&BlogPost]) -> Vec<PageLink> {
    let mut pages: Vec<&BlogPost> = pages.to_vec();
    pages.sort_by(|left, right| {
        left.metadata
            .order
            .cmp(&right.metadata.order)
            .then_with(|| left.metadata.title.cmp(&right.metadata.title))
    });

    pages
        .into_iter()
        .map(|page| PageLink {
            label: page
                .metadata
                .label
                .clone()
                .unwrap_or_else(|| page.metadata.title.clone()),
            url: page.metadata.url.clone(),
        })
        .collect()
}

fn build_comments_config(config: &Config) -> Option<CommentsConfigTemplate> {
    if !config.comments.enabled || config.comments.system != "giscus" {
        return None;
    }

    config
        .comments
        .giscus
        .as_ref()
        .map(|giscus| CommentsConfigTemplate {
            repo: giscus.repo.clone(),
            repo_id: giscus.repo_id.clone(),
            category: giscus.category.clone(),
            category_id: giscus.category_id.clone(),
            mapping: giscus.mapping.clone(),
            strict: giscus.strict.clone(),
            reactions_enabled: giscus.reactions_enabled.clone(),
            emit_metadata: giscus.emit_metadata.clone(),
            input_position: giscus.input_position.clone(),
            theme: giscus.theme.clone(),
            lang: giscus.lang.clone(),
        })
}

fn post_nav(post: &BlogPost) -> PostNav {
    PostNav {
        title: post.metadata.title.clone(),
        url: post.metadata.url.clone(),
    }
}

fn post_output_path(output_dir: &Path, post: &BlogPost) -> PathBuf {
    output_dir
        .join("p")
        .join(&post.metadata.year)
        .join(post.metadata.date.month().to_string())
        .join(post.metadata.date.day().to_string())
        .join(&post.metadata.slug)
        .join("index.html")
}

fn page_output_path(output_dir: &Path, page: &BlogPost) -> PathBuf {
    output_dir
        .join(page.metadata.url.trim_start_matches('/'))
        .join("index.html")
}

fn generate_list_pages(
    posts: &[&BlogPost],
    pages: &[PageLink],
    site_title: &str,
    output_dir: &Path,
    base_url: &str,
    heading: &str,
) -> Result<()> {
    let total_pages = std::cmp::max(1, posts.len().div_ceil(POSTS_PER_PAGE));

    for page_number in 1..=total_pages {
        let start = (page_number - 1) * POSTS_PER_PAGE;
        let end = std::cmp::min(start + POSTS_PER_PAGE, posts.len());
        let page_posts = posts.get(start..end).unwrap_or_default().to_vec();
        let html = render_list(
            page_posts,
            pages,
            site_title,
            page_number,
            total_pages,
            heading,
            base_url,
        )?;

        let relative = if page_number == 1 {
            PathBuf::from(base_url.trim_start_matches('/'))
        } else {
            PathBuf::from(base_url.trim_start_matches('/'))
                .join("page")
                .join(page_number.to_string())
        };
        write_output(&output_dir.join(relative).join("index.html"), html)?;
    }

    Ok(())
}

fn sanitize_segment(value: &str) -> String {
    value
        .split_whitespace()
        .collect::<Vec<_>>()
        .join("-")
        .replace(['/', '\\'], "-")
}

fn get_all_categories(posts: &[&BlogPost]) -> Vec<String> {
    let mut categories: Vec<String> = posts
        .iter()
        .flat_map(|post| post.metadata.categories.iter().cloned())
        .filter(|category| !category.is_empty())
        .collect();
    categories.sort();
    categories.dedup();
    categories
}

fn get_all_tags(posts: &[&BlogPost]) -> Vec<String> {
    let mut tags: Vec<String> = posts
        .iter()
        .flat_map(|post| post.metadata.tags.iter().cloned())
        .filter(|tag| !tag.is_empty())
        .collect();
    tags.sort();
    tags.dedup();
    tags
}

fn write_output(path: &Path, content: String) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("Failed to create output directory: {}", parent.display()))?;
    }
    fs::write(path, content)
        .with_context(|| format!("Failed to write generated file: {}", path.display()))?;
    Ok(())
}

fn clean_generated_output(config: &Config) -> Result<()> {
    let generated_posts = config.output_dir.join("p");
    if generated_posts.exists() {
        fs::remove_dir_all(&generated_posts).with_context(|| {
            format!(
                "Failed to clean generated directory: {}",
                generated_posts.display()
            )
        })?;
    }

    for filename in ["index.html", "rss.xml"] {
        let path = config.output_dir.join(filename);
        if path.exists() {
            fs::remove_file(&path)
                .with_context(|| format!("Failed to remove generated file: {}", path.display()))?;
        }
    }

    Ok(())
}

fn copy_theme_assets(config: &Config) -> Result<()> {
    let theme_dir = PathBuf::from("theme/default");
    for (source, destination) in [
        (theme_dir.join("css"), config.output_dir.join("css")),
        (theme_dir.join("js"), config.output_dir.join("js")),
        (theme_dir.join("img"), config.output_dir.join("img")),
    ] {
        if source.exists() {
            copy_directory(&source, &destination)?;
        }
    }
    Ok(())
}

fn copy_directory(source: &Path, destination: &Path) -> Result<()> {
    for entry in fs::read_dir(source)
        .with_context(|| format!("Failed to read theme directory: {}", source.display()))?
    {
        let entry = entry?;
        let source_path = entry.path();
        let destination_path = destination.join(entry.file_name());
        if source_path.is_dir() {
            copy_directory(&source_path, &destination_path)?;
        } else {
            fs::create_dir_all(destination).with_context(|| {
                format!(
                    "Failed to create theme output directory: {}",
                    destination.display()
                )
            })?;
            fs::copy(&source_path, &destination_path).with_context(|| {
                format!(
                    "Failed to copy theme asset from {} to {}",
                    source_path.display(),
                    destination_path.display()
                )
            })?;
        }
    }
    Ok(())
}

pub fn collect_markdown_files(input_dir: &Path) -> Result<Vec<PathBuf>> {
    collect_content_files(&input_dir.join("posts"))
}

pub fn collect_page_files(input_dir: &Path) -> Result<Vec<PathBuf>> {
    collect_content_files(&input_dir.join("pages"))
}

fn collect_content_files(content_dir: &Path) -> Result<Vec<PathBuf>> {
    if !content_dir.exists() {
        return Ok(Vec::new());
    }

    let mut markdown_files = Vec::new();
    collect_recursive(content_dir, &mut markdown_files)?;
    markdown_files.sort();
    Ok(markdown_files)
}

fn collect_recursive(dir: &Path, files: &mut Vec<PathBuf>) -> Result<()> {
    for entry in
        fs::read_dir(dir).with_context(|| format!("Failed to read directory: {}", dir.display()))?
    {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            collect_recursive(&path, files)?;
        } else if path.extension().and_then(|value| value.to_str()) == Some("md") {
            files.push(path);
        }
    }
    Ok(())
}
