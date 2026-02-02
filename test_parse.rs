use alog::parser::BlogPost;
use std::path::Path;

fn main() {
    let path = Path::new("/Users/edison/code/alog/md/pages/projects.md");
    match BlogPost::from_file(&path) {
        Ok(post) => println!("Success: {}", post.metadata.title),
        Err(e) => println!("Error: {:?}", e),
    }
}
