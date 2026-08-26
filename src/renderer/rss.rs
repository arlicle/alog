// RSS feed generation module
use crate::config::Config;
use crate::parser::BlogPost;
use anyhow::Result;
use chrono::DateTime;
use rss::{ChannelBuilder, ItemBuilder};

const RSS_ITEM_LIMIT: usize = 20;

pub fn generate_rss_feed(posts: &[BlogPost], config: &Config) -> Result<String> {
    let base_url = config.site_url.trim_end_matches('/');

    // Only include the latest 20 posts
    let latest_posts: Vec<&BlogPost> = posts.iter().take(RSS_ITEM_LIMIT).collect();

    let items: Vec<rss::Item> = latest_posts
        .iter()
        .map(|post| {
            let link = format!("{}{}", base_url.trim_end_matches('/'), post.metadata.url);

            let pub_date = DateTime::<chrono::Utc>::from_naive_utc_and_offset(
                post.metadata.date.and_hms_opt(0, 0, 0).unwrap(),
                chrono::Utc,
            );

            let description = post.metadata.summary.clone().unwrap_or_default();

            ItemBuilder::default()
                .title(post.metadata.title.clone())
                .link(link)
                .description(description)
                .pub_date(pub_date.to_rfc2822())
                .build()
        })
        .collect();

    let channel = ChannelBuilder::default()
        .title(config.site_title.clone())
        .link(base_url)
        .description(format!("{} generated with alog", config.site_title))
        .language(Some("en-us".to_string()))
        .items(items)
        .build();

    Ok(channel.to_string())
}

pub fn save_rss_feed(posts: &[BlogPost], config: &Config) -> Result<()> {
    let rss_content = generate_rss_feed(posts, config)?;
    let rss_path = config.output_dir.join("rss.xml");
    std::fs::write(&rss_path, rss_content)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::{BlogPost, PostMetadata};
    use chrono::NaiveDate;

    fn test_post(number: usize) -> BlogPost {
        let date = NaiveDate::from_ymd_opt(2024, 1, number as u32).unwrap();
        BlogPost {
            metadata: PostMetadata {
                title: format!("Post {number}"),
                date,
                category: String::new(),
                categories: Vec::new(),
                tags: Vec::new(),
                summary: Some(format!("Summary {number}")),
                slug: format!("post-{number}"),
                url: format!("/p/2024/1/{number}/post-{number}/"),
                created_at: "2024-01-01 00:00:00".to_string(),
                year: "2024".to_string(),
                month: "01".to_string(),
                day: format!("{number:02}"),
                formatted_date: date.format("%Y-%m-%d").to_string(),
                html_content: String::new(),
                order: None,
                label: None,
                path: None,
                layout: None,
                draft: false,
                private: false,
                kind: "post".to_string(),
            },
            content: String::new(),
        }
    }

    #[test]
    fn feed_uses_configured_site_url_and_summary() {
        let mut config = Config::default();
        config.site_title = "Example Blog".to_string();
        config.site_url = "https://example.com/".to_string();

        let feed = generate_rss_feed(&[test_post(1)], &config).unwrap();
        let channel = rss::Channel::read_from(feed.as_bytes()).unwrap();

        assert_eq!(channel.title(), "Example Blog");
        assert_eq!(channel.link(), "https://example.com");
        assert_eq!(channel.items().len(), 1);
        assert_eq!(channel.items()[0].title(), Some("Post 1"));
        assert_eq!(
            channel.items()[0].link(),
            Some("https://example.com/p/2024/1/1/post-1/")
        );
        assert_eq!(channel.items()[0].description(), Some("Summary 1"));
    }

    #[test]
    fn feed_contains_at_most_twenty_posts_in_input_order() {
        let config = Config::default();
        let posts: Vec<BlogPost> = (1..=21).rev().map(test_post).collect();

        let feed = generate_rss_feed(&posts, &config).unwrap();
        let channel = rss::Channel::read_from(feed.as_bytes()).unwrap();

        assert_eq!(channel.items().len(), 20);
        assert_eq!(channel.items()[0].title(), Some("Post 21"));
        assert_eq!(channel.items()[19].title(), Some("Post 2"));
        assert!(!channel
            .items()
            .iter()
            .any(|item| item.title() == Some("Post 1")));
    }

    #[test]
    fn empty_feed_is_valid_rss() {
        let config = Config::default();

        let feed = generate_rss_feed(&[], &config).unwrap();
        let channel = rss::Channel::read_from(feed.as_bytes()).unwrap();

        assert!(channel.items().is_empty());
    }
}
