// HTML generation module
use super::template::{render_index, render_post, render_category, render_tag, render_tags};
use crate::config::Config;
use crate::parser::BlogPost;
use anyhow::{Context, Result};
use std::fs;
use std::path::PathBuf;

const TOP_TAGS_COUNT: usize = 30;

pub fn generate_site(posts: &[BlogPost], config: &Config) -> Result<()> {
    // Create output directory
    fs::create_dir_all(&config.output_dir)
        .context("Failed to create output directory")?;

    // Copy CSS files from templates directory to output directory
    copy_template_assets(config)?;

    // Get categories and tags
    let categories = get_all_categories(posts);
    let tags = get_all_tags(posts);
    let posts_per_page = config.pagination.posts_per_page;
    
    // Get top 30 tags for sidebar
    let top_tags: Vec<(String, usize)> = tags.iter().take(TOP_TAGS_COUNT).cloned().collect();

    // Sort posts by date (ascending order for chronological navigation)
    let mut sorted_posts: Vec<&BlogPost> = posts.iter().collect();
    sorted_posts.sort_by(|a, b| {
        match a.metadata.date.cmp(&b.metadata.date) {
            std::cmp::Ordering::Equal => match a.metadata.title.cmp(&b.metadata.title) {
                std::cmp::Ordering::Equal => a.metadata.created_at.cmp(&b.metadata.created_at),
                other => other,
            },
            other => other,
        }
    });

    // Generate index.html with pagination
    generate_paginated_index(posts, &categories, &top_tags, config, posts_per_page)?;

    // Generate tags.html (all tags listing)
    let tags_html = render_tags(&tags, &categories)?;
    let tags_path = config.output_dir.join("tags.html");
    fs::write(&tags_path, tags_html)
        .context("Failed to write tags.html")?;

    // Generate category pages with pagination
    for category in &categories {
        let category_posts = get_posts_by_category(posts, category);
        generate_paginated_pages(
            &category_posts,
            &categories,
            &top_tags,
            config,
            posts_per_page,
            &format!("/category/{}", sanitize_filename(category)),
            category,
            "category",
        )?;
    }

    // Generate tag pages with pagination
    for (tag, _) in &tags {
        let tag_posts = get_posts_by_tag(posts, tag);
        generate_paginated_pages(
            &tag_posts,
            &categories,
            &top_tags,
            config,
            posts_per_page,
            &format!("/tag/{}", sanitize_filename(tag)),
            tag,
            "tag",
        )?;
    }

    // Generate individual post pages
    for post in posts {
        // Find prev and next posts (sorted by date ascending)
        let prev_post = sorted_posts
            .iter()
            .position(|p| std::ptr::eq(*p, post))
            .and_then(|i| {
                if i > 0 {
                    Some(sorted_posts[i - 1])
                } else {
                    None
                }
            });

        let next_post = sorted_posts
            .iter()
            .position(|p| std::ptr::eq(*p, post))
            .and_then(|i| {
                if i + 1 < sorted_posts.len() {
                    Some(sorted_posts[i + 1])
                } else {
                    None
                }
            });

        let prev_nav = prev_post.map(|p| super::template::PostNav {
            title: p.metadata.title.clone(),
            url: format!(
                "/{}/{}/{}/{}.html",
                p.metadata.year, p.metadata.month, p.metadata.day, p.metadata.slug
            ),
            date: p.metadata.formatted_date.clone(),
            created_at: p.metadata.created_at.clone(),
        });

        let next_nav = next_post.map(|p| super::template::PostNav {
            title: p.metadata.title.clone(),
            url: format!(
                "/{}/{}/{}/{}.html",
                p.metadata.year, p.metadata.month, p.metadata.day, p.metadata.slug
            ),
            date: p.metadata.formatted_date.clone(),
            created_at: p.metadata.created_at.clone(),
        });

        // Prepare comments config
        let comments_config = if config.comments.enabled && config.comments.system == "giscus" {
            config.comments.giscus.as_ref().map(|giscus| super::template::CommentsConfigTemplate {
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
        } else {
            None
        };

        let post_html = render_post(
            post,
            &categories,
            &top_tags,
            prev_nav,
            next_nav,
            config.comments.enabled,
            comments_config,
        )?;
        
        // Organize by year/month/day: YYYY/MM/DD/slug.html
        let post_dir = config
            .output_dir
            .join(&post.metadata.year)
            .join(&post.metadata.month)
            .join(&post.metadata.day);
        
        fs::create_dir_all(&post_dir)
            .context("Failed to create post directory")?;
        
        let post_path = post_dir.join(format!("{}.html", post.metadata.slug));
        fs::write(&post_path, post_html)
            .context("Failed to write post HTML")?;
    }

    // Generate RSS feed
    let base_url = "http://localhost:7878";
    super::rss::save_rss_feed(posts, config, base_url)?;

    Ok(())
}

fn generate_paginated_index(
    all_posts: &[BlogPost],
    categories: &[String],
    tags: &[(String, usize)],
    config: &Config,
    posts_per_page: usize,
) -> Result<()> {
    let total_posts = all_posts.len();
    let total_pages = (total_posts + posts_per_page - 1) / posts_per_page;

    for page in 1..=total_pages {
        let start = (page - 1) * posts_per_page;
        let end = std::cmp::min(start + posts_per_page, total_posts);
        let page_posts: Vec<&BlogPost> = all_posts[start..end].iter().collect();

        let html = render_index(page_posts, categories, tags, page, total_pages)?;

        let output_path = if page == 1 {
            config.output_dir.join("index.html")
        } else {
            config.output_dir.join(format!("page/{}.html", page))
        };

        fs::create_dir_all(output_path.parent().unwrap())
            .context("Failed to create page directory")?;
        fs::write(&output_path, html)
            .context("Failed to write paginated HTML")?;
    }

    Ok(())
}

fn generate_paginated_pages(
    all_posts: &[&BlogPost],
    categories: &[String],
    tags: &[(String, usize)],
    config: &Config,
    posts_per_page: usize,
    base_path: &str,
    identifier: &str,
    page_type: &str,
) -> Result<()> {
    let total_posts = all_posts.len();
    let total_pages = (total_posts + posts_per_page - 1) / posts_per_page;

    for page in 1..=total_pages {
        let start = (page - 1) * posts_per_page;
        let end = std::cmp::min(start + posts_per_page, total_posts);
        let page_posts: Vec<&BlogPost> = all_posts[start..end].to_vec();

        let html = if page_type == "category" {
            render_category(identifier, page_posts, categories, tags, page, total_pages)?
        } else {
            render_tag(identifier, page_posts, categories, tags, page, total_pages)?
        };

        let output_path = if page == 1 {
            config.output_dir.join(&format!("{}.html", &base_path[1..]))
        } else {
            config.output_dir.join(&format!("{}/page/{}.html", &base_path[1..], page))
        };

        fs::create_dir_all(output_path.parent().unwrap())
            .context("Failed to create page directory")?;
        fs::write(&output_path, html)
            .context("Failed to write paginated HTML")?;
    }

    Ok(())
}

fn sanitize_filename(name: &str) -> String {
    name.chars()
        .map(|c| if c.is_alphanumeric() || c == '-' { c } else { '_' })
        .collect()
}

pub fn collect_markdown_files(input_dir: &PathBuf) -> Result<Vec<PathBuf>> {
    let mut markdown_files = Vec::new();

    fn collect_recursive(dir: &PathBuf, files: &mut Vec<PathBuf>) -> Result<()> {
        for entry in fs::read_dir(dir)
            .with_context(|| format!("Failed to read directory: {}", dir.display()))?
        {
            let entry = entry?;
            let path = entry.path();

            if path.is_file() {
                if let Some(ext) = path.extension() {
                    if ext == "md" {
                        files.push(path);
                    }
                }
            } else if path.is_dir() {
                collect_recursive(&path, files)?;
            }
        }
        Ok(())
    }

    collect_recursive(input_dir, &mut markdown_files)?;
    Ok(markdown_files)
}

pub fn get_posts_by_category<'a>(posts: &'a [BlogPost], category: &str) -> Vec<&'a BlogPost> {
    posts
        .iter()
        .filter(|p| p.metadata.category.to_lowercase() == category.to_lowercase())
        .collect()
}

pub fn get_posts_by_tag<'a>(posts: &'a [BlogPost], tag: &str) -> Vec<&'a BlogPost> {
    posts
        .iter()
        .filter(|p| p.metadata.tags.iter().any(|t| t.to_lowercase() == tag.to_lowercase()))
        .collect()
}

pub fn get_all_categories(posts: &[BlogPost]) -> Vec<String> {
    let mut categories: Vec<String> = posts
        .iter()
        .filter(|p| !p.metadata.category.is_empty())
        .map(|p| p.metadata.category.clone())
        .collect();
    categories.sort();
    categories.dedup();
    categories
}

pub fn get_all_tags(posts: &[BlogPost]) -> Vec<(String, usize)> {
    let mut tag_counts: std::collections::HashMap<String, usize> = std::collections::HashMap::new();

    for post in posts {
        for tag in &post.metadata.tags {
            *tag_counts.entry(tag.clone()).or_insert(0) += 1;
        }
    }

    let mut tags: Vec<(String, usize)> = tag_counts.into_iter().collect();
    tags.sort_by(|a, b| b.1.cmp(&a.1)); // Sort by count descending
    tags
}

fn copy_template_assets(config: &Config) -> Result<()> {
    let templates_dir = PathBuf::from("templates");

    // Check if templates directory exists
    if !templates_dir.exists() {
        return Ok(());
    }

    // Find all CSS files in templates directory
    let css_files = find_css_files(&templates_dir)?;

    for css_file in css_files {
        let file_name = css_file
            .file_name()
            .context("Invalid CSS file name")?;

        let dest_path = config.output_dir.join(file_name);

        // Copy the CSS file
        fs::copy(&css_file, &dest_path)
            .with_context(|| format!("Failed to copy CSS file from {:?} to {:?}", css_file, dest_path))?;

        println!("Copied {} to output directory", file_name.to_string_lossy());
    }

    Ok(())
}

fn find_css_files(dir: &PathBuf) -> Result<Vec<PathBuf>> {
    let mut css_files = Vec::new();

    for entry in fs::read_dir(dir)
        .with_context(|| format!("Failed to read directory: {}", dir.display()))?
    {
        let entry = entry?;
        let path = entry.path();

        if path.is_file() {
            if let Some(ext) = path.extension() {
                if ext == "css" {
                    css_files.push(path);
                }
            }
        }
    }

    Ok(css_files)
}
