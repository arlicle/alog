// CLI module - command line argument parsing
pub mod cli {
    use clap::{Parser, Subcommand};
    use std::path::PathBuf;

    #[derive(Parser, Debug)]
    #[command(name = "alog")]
    #[command(about = "A blazing fast blog generator", long_about = None)]
    pub struct Cli {
        #[command(subcommand)]
        pub command: Commands,
    }

    #[derive(Subcommand, Debug)]
    pub enum Commands {
        /// Build the blog (generate HTML from Markdown)
        Build {
            /// Directory containing markdown files
            #[arg(short, long, default_value = "./md")]
            input_dir: PathBuf,
            /// Output directory for generated site
            #[arg(short, long, default_value = "./www")]
            output_dir: PathBuf,
        },
        /// Start development server
        Serve {
            /// Port number
            #[arg(short, long, default_value = "7878")]
            port: u16,
            /// Directory containing markdown files
            #[arg(short, long, default_value = "./md")]
            input_dir: PathBuf,
            /// Output directory for generated site
            #[arg(short, long, default_value = "./www")]
            output_dir: PathBuf,
        },
    }
}

pub use cli::{Cli, Commands};