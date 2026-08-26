// RSS feed generation module
use crate::config::Config;
use crate::parser::BlogPost;
use anyhow::Result;
use chrono::DateTime;
use rss::{ChannelBuilder, ItemBuilder};

const RSS_ITEM_LIMIT: usize = 20;

pub fn generate_rss_feed(posts: &[BlogPost], config: &Config, base_url: &str) -> Result<String> {
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

pub fn save_rss_feed(posts: &[BlogPost], config: &Config, base_url: &str) -> Result<()> {
    let rss_content = generate_rss_feed(posts, config, base_url)?;
    let rss_path = config.output_dir.join("rss.xml");
    std::fs::write(&rss_path, rss_content)?;
    Ok(())
}
