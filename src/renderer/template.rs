// Template rendering module
use askama::Template;
use crate::parser::BlogPost;
use serde::Serialize;

#[derive(Serialize)]
pub struct PostNav {
    pub title: String,
    pub url: String,
    pub date: String,
    pub created_at: String,
}

#[derive(Template)]
#[template(path = "index.html")]
pub struct IndexTemplate<'a> {
    pub posts: Vec<&'a BlogPost>,
    pub categories: &'a [String],
    pub tags: &'a [(String, usize)],
    pub page: usize,
    pub total_pages: usize,
    pub current_url: String,
}

#[derive(Template)]
#[template(path = "post.html")]
pub struct PostTemplate<'a> {
    pub post: &'a BlogPost,
    pub categories: &'a [String],
    pub tags: &'a [(String, usize)],
    pub prev_post: Option<PostNav>,
    pub next_post: Option<PostNav>,
    pub comments_enabled: bool,
    pub comments_config: Option<CommentsConfigTemplate>,
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
#[template(path = "category.html")]
pub struct CategoryTemplate<'a> {
    pub category: &'a str,
    pub posts: Vec<&'a BlogPost>,
    pub categories: &'a [String],
    pub tags: &'a [(String, usize)],
    pub page: usize,
    pub total_pages: usize,
}

#[derive(Template)]
#[template(path = "tag.html")]
pub struct TagTemplate<'a> {
    pub tag: &'a str,
    pub posts: Vec<&'a BlogPost>,
    pub categories: &'a [String],
    pub tags: &'a [(String, usize)],
    pub page: usize,
    pub total_pages: usize,
}

#[derive(Template)]
#[template(path = "tags.html")]
pub struct TagsTemplate<'a> {
    pub tags: &'a [(String, usize)],
    pub categories: &'a [String],
}

pub fn render_index<'a>(
    posts: Vec<&'a BlogPost>,
    categories: &'a [String],
    tags: &'a [(String, usize)],
    page: usize,
    total_pages: usize,
) -> askama::Result<String> {
    let template = IndexTemplate {
        posts,
        categories,
        tags,
        page,
        total_pages,
        current_url: "/".to_string(),
    };
    template.render()
}

pub fn render_post<'a>(
    post: &'a BlogPost,
    categories: &'a [String],
    tags: &'a [(String, usize)],
    prev_post: Option<PostNav>,
    next_post: Option<PostNav>,
    comments_enabled: bool,
    comments_config: Option<CommentsConfigTemplate>,
) -> askama::Result<String> {
    let template = PostTemplate {
        post,
        categories,
        tags,
        prev_post,
        next_post,
        comments_enabled,
        comments_config,
    };
    template.render()
}

pub fn render_category<'a>(
    category: &'a str,
    posts: Vec<&'a BlogPost>,
    categories: &'a [String],
    tags: &'a [(String, usize)],
    page: usize,
    total_pages: usize,
) -> askama::Result<String> {
    let template = CategoryTemplate {
        category,
        posts,
        categories,
        tags,
        page,
        total_pages,
    };
    template.render()
}

pub fn render_tag<'a>(
    tag: &'a str,
    posts: Vec<&'a BlogPost>,
    categories: &'a [String],
    tags: &'a [(String, usize)],
    page: usize,
    total_pages: usize,
) -> askama::Result<String> {
    let template = TagTemplate {
        tag,
        posts,
        categories,
        tags,
        page,
        total_pages,
    };
    template.render()
}

pub fn render_tags(tags: &[(String, usize)], categories: &[String]) -> askama::Result<String> {
    let template = TagsTemplate { tags, categories };
    template.render()
}
