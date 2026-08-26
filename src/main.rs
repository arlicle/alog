use alog::cli::{Cli, Commands};
use alog::config::Config;
use alog::parser::{BlogPost, ContentKind};
use alog::renderer::html::{collect_markdown_files, collect_page_files, generate_site};
use alog::server::start_server;
use alog::watcher::start_watcher;
use anyhow::Result;
use clap::Parser;
use std::path::{Path, PathBuf};

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    let cli = Cli::parse();

    match cli.command {
        Commands::Build {
            input_dir,
            output_dir,
        } => {
            let mut config = load_config();
            apply_cli_overrides(&mut config, input_dir, output_dir, None);
            build_site(&config)?;
            println!("Site built successfully!");
        }
        Commands::Serve {
            port,
            input_dir,
            output_dir,
        } => {
            let mut config = load_config();
            apply_cli_overrides(&mut config, input_dir, output_dir, Some(port));
            build_site(&config)?;

            let watched_dirs = vec![
                config.input_dir.join("posts"),
                config.input_dir.join("pages"),
            ];
            let (tx, mut rx) = tokio::sync::mpsc::channel(100);
            let watcher_dirs = watched_dirs.clone();
            std::thread::spawn(move || {
                if let Ok((_watcher, rx_sync)) = start_watcher(&watcher_dirs) {
                    while rx_sync.recv().is_ok() {
                        let _ = tx.blocking_send(());
                    }
                }
            });

            let rebuild_config = config.clone();
            tokio::spawn(async move {
                while rx.recv().await.is_some() {
                    println!("Detected content change, rebuilding...");
                    if let Err(error) = build_site(&rebuild_config) {
                        eprintln!("Build error: {error}");
                    } else {
                        println!("Site rebuilt successfully!");
                    }
                }
            });

            start_server(config).await?;
        }
    }

    Ok(())
}

fn load_config() -> Config {
    let config_path = PathBuf::from("config.toml");
    if config_path.exists() {
        Config::from_file(&config_path).unwrap_or_else(|error| {
            eprintln!("Warning: Failed to load config.toml: {error}, using defaults");
            Config::default()
        })
    } else {
        Config::default()
    }
}

fn apply_cli_overrides(
    config: &mut Config,
    input_dir: PathBuf,
    output_dir: PathBuf,
    port: Option<u16>,
) {
    if input_dir != Path::new(".") {
        config.input_dir = input_dir;
    }
    if output_dir != Path::new("./www") {
        config.output_dir = output_dir;
    }
    if let Some(port) = port.filter(|port| *port != 7878) {
        config.server.port = port;
    }
}

fn build_site(config: &Config) -> Result<()> {
    println!("Building site...");
    println!("Content root: {}", config.input_dir.display());
    println!("Output root: {}", config.output_dir.display());

    let post_files = collect_markdown_files(&config.input_dir)?;
    let page_files = collect_page_files(&config.input_dir)?;
    println!(
        "Found {} post files and {} page files",
        post_files.len(),
        page_files.len()
    );

    let mut posts = Vec::new();
    for path in post_files {
        match BlogPost::from_file_with_kind(&path, ContentKind::Post) {
            Ok(post) => {
                if post.is_public() {
                    println!("Processed post: {}", post.metadata.title);
                } else {
                    println!("Skipped draft/private post: {}", post.metadata.title);
                }
                posts.push(post);
            }
            Err(error) => eprintln!("Error processing {}: {error}", path.display()),
        }
    }

    let mut pages = Vec::new();
    for path in page_files {
        match BlogPost::from_file_with_kind(&path, ContentKind::Page) {
            Ok(page) => {
                if page.is_public() {
                    println!("Processed page: {}", page.metadata.title);
                } else {
                    println!("Skipped draft/private page: {}", page.metadata.title);
                }
                pages.push(page);
            }
            Err(error) => eprintln!("Error processing {}: {error}", path.display()),
        }
    }

    generate_site(&posts, &pages, config)?;

    let published_posts = posts.iter().filter(|post| post.is_public()).count();
    let published_pages = pages.iter().filter(|page| page.is_public()).count();
    println!("Generated {published_posts} public posts and {published_pages} public pages");

    Ok(())
}
