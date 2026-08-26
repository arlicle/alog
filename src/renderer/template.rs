use crate::parser::BlogPost;
use askama::Template;
use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct PageLink {
    pub label: String,
    pub url: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct PostNav {
    pub title: String,
    pub url: String,
}

#[derive(Serialize, Clone, Debug)]
pub struct CommentsConfigTemplate {
    pub repo: String,
    pub repo_id: String,
    pub category: String,
    pub category_id: String,
    pub mapping: String,
    pub strict: String,
    pub reactions_enabled: String,
    pub emit_metadata: String,
    pub input_position: String,
    pub theme: String,
    pub lang: String,
}

#[derive(Template)]
#[template(path = "post.html")]
pub struct PostTemplate<'a> {
    pub post: &'a BlogPost,
    pub pages: &'a [PageLink],
    pub site_title: &'a str,
    pub prev_post: Option<PostNav>,
    pub next_post: Option<PostNav>,
    pub comments_enabled: bool,
    pub comments_config: Option<CommentsConfigTemplate>,
}

#[derive(Template)]
#[template(path = "page.html")]
pub struct PageTemplate<'a> {
    pub page: &'a BlogPost,
    pub pages: &'a [PageLink],
    pub site_title: &'a str,
    pub comments_enabled: bool,
    pub comments_config: Option<CommentsConfigTemplate>,
}

#[derive(Template)]
#[template(path = "list.html")]
pub struct ListTemplate<'a> {
    pub posts: Vec<&'a BlogPost>,
    pub pages: &'a [PageLink],
    pub site_title: &'a str,
    pub page: usize,
    pub total_pages: usize,
    pub heading: &'a str,
    pub base_url: &'a str,
}

pub fn render_post<'a>(
    post: &'a BlogPost,
    pages: &'a [PageLink],
    site_title: &'a str,
    prev_post: Option<PostNav>,
    next_post: Option<PostNav>,
    comments_enabled: bool,
    comments_config: Option<CommentsConfigTemplate>,
) -> askama::Result<String> {
    PostTemplate {
        post,
        pages,
        site_title,
        prev_post,
        next_post,
        comments_enabled,
        comments_config,
    }
    .render()
}

pub fn render_page<'a>(
    page: &'a BlogPost,
    pages: &'a [PageLink],
    site_title: &'a str,
    comments_enabled: bool,
    comments_config: Option<CommentsConfigTemplate>,
) -> askama::Result<String> {
    PageTemplate {
        page,
        pages,
        site_title,
        comments_enabled,
        comments_config,
    }
    .render()
}

pub fn render_list<'a>(
    posts: Vec<&'a BlogPost>,
    pages: &'a [PageLink],
    site_title: &'a str,
    page: usize,
    total_pages: usize,
    heading: &'a str,
    base_url: &'a str,
) -> askama::Result<String> {
    ListTemplate {
        posts,
        pages,
        site_title,
        page,
        total_pages,
        heading,
        base_url,
    }
    .render()
}
