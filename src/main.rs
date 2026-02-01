use alog::cli::{Cli, Commands};
use alog::config::Config;
use alog::parser::BlogPost;
use alog::renderer::html::{collect_markdown_files, generate_site};
use alog::server::start_server;
use alog::watcher::start_watcher;
use anyhow::Result;
use clap::Parser;
use tracing_subscriber;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    let cli = Cli::parse();

    match cli.command {
        Commands::Build {
            input_dir,
            output_dir,
        } => {
            let config = Config {
                input_dir,
                output_dir,
                ..Default::default()
            };
            build_site(&config)?;
            println!("Site built successfully!");
        }
        Commands::Serve {
            port,
            input_dir,
            output_dir,
        } => {
            let mut config = Config {
                input_dir,
                output_dir,
                ..Default::default()
            };
            config.server.port = port;

            // Initial build
            build_site(&config)?;

            // Start file watcher in a separate thread
            let (tx, mut rx) = tokio::sync::mpsc::channel(100);
            let input_dir_clone = config.input_dir.clone();
            
            std::thread::spawn(move || {
                if let Ok((_watcher, rx_sync)) = start_watcher(&input_dir_clone) {
                    loop {
                        match rx_sync.recv() {
                            Ok(_event) => {
                                // Send signal to async task
                                let _ = tx.blocking_send(());
                            }
                            Err(_) => {
                                // Watcher stopped
                                break;
                            }
                        }
                    }
                }
            });

            // Spawn watcher task
            let input_dir_clone = config.input_dir.clone();
            let output_dir_clone = config.output_dir.clone();
            tokio::spawn(async move {
                while let Some(_) = rx.recv().await {
                    println!("Detected file change, rebuilding...");
                    let build_config = Config {
                        input_dir: input_dir_clone.clone(),
                        output_dir: output_dir_clone.clone(),
                        ..Default::default()
                    };
                    if let Err(e) = build_site(&build_config) {
                        eprintln!("Build error: {}", e);
                    } else {
                        println!("Site rebuilt successfully!");
                    }
                }
            });

            // Start server
            start_server(config).await?;
        }
    }

    Ok(())
}

fn build_site(config: &Config) -> Result<()> {
    println!("Building site...");
    println!("Input directory: {}", config.input_dir.display());
    println!("Output directory: {}", config.output_dir.display());

    // Collect markdown files
    let markdown_files = collect_markdown_files(&config.input_dir)?;
    println!("Found {} markdown files", markdown_files.len());

    if markdown_files.is_empty() {
        println!("No markdown files found. Nothing to build.");
        return Ok(());
    }

    // Parse all posts
    let mut posts: Vec<BlogPost> = Vec::new();
    for file_path in markdown_files {
        match BlogPost::from_file(&file_path) {
            Ok(post) => {
                println!("Processed: {}", post.metadata.title);
                posts.push(post);
            }
            Err(e) => {
                eprintln!("Error processing {}: {}", file_path.display(), e);
            }
        }
    }

    // Sort posts by date (newest first)
    posts.sort_by(|a, b| b.metadata.date.cmp(&a.metadata.date));

    // Generate site
    generate_site(&posts, config)?;

    println!("Generated {} posts", posts.len());

    Ok(())
}
