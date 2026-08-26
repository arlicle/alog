// File watcher module
use anyhow::Result;
use notify::{RecommendedWatcher, RecursiveMode, Watcher};
use std::path::PathBuf;
use std::sync::mpsc::channel;
use std::time::Duration;

#[derive(Debug, Clone)]
pub enum WatchEvent {
    FileChanged(PathBuf),
    FileCreated(PathBuf),
    FileDeleted(PathBuf),
}

pub fn start_watcher(
    input_dirs: &[PathBuf],
) -> Result<(impl Watcher, std::sync::mpsc::Receiver<WatchEvent>)> {
    let (tx, rx) = channel();

    let mut watcher: RecommendedWatcher = Watcher::new(
        move |res: Result<notify::Event, notify::Error>| {
            if let Ok(event) = res {
                for path in event.paths {
                    if let Some(ext) = path.extension() {
                        if ext == "md" {
                            let event = match event.kind {
                                notify::EventKind::Create(_) => {
                                    WatchEvent::FileCreated(path.clone())
                                }
                                notify::EventKind::Modify(_) => {
                                    WatchEvent::FileChanged(path.clone())
                                }
                                notify::EventKind::Remove(_) => {
                                    WatchEvent::FileDeleted(path.clone())
                                }
                                _ => WatchEvent::FileChanged(path.clone()),
                            };
                            let _ = tx.send(event);
                        }
                    }
                }
            }
        },
        notify::Config::default().with_poll_interval(Duration::from_millis(200)),
    )?;

    for input_dir in input_dirs {
        if input_dir.exists() {
            watcher.watch(input_dir, RecursiveMode::Recursive)?;
        }
    }

    Ok((watcher, rx))
}
