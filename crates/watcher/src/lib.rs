use archiver;
use notify::{
    Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher as NorifyWatcher,
    event::ModifyKind,
};
use std::{
    collections::HashMap,
    path::{Path, PathBuf},
    time::{Duration, Instant},
};
use tokio::sync::mpsc;

pub struct WatcherConfig {
    pub timeout: Duration,
}

pub struct Watcher {
    root: PathBuf,
    config: WatcherConfig,
}

impl Watcher {
    pub fn new(root: PathBuf) -> Self {
        Self {
            root,
            config: WatcherConfig {
                timeout: (Duration::from_secs(5)),
            },
        }
    }

    pub async fn run(&self) -> notify::Result<()> {
        let (tx, mut rx) = mpsc::unbounded_channel::<Event>();

        let mut watcher: RecommendedWatcher =
            notify::recommended_watcher(move |res: notify::Result<Event>| match res {
                Ok(event) => {
                    let _ = tx.send(event);
                }
                Err(e) => println!("watch error: {:?}", e),
            })?;

        watcher.watch(&self.root, RecursiveMode::Recursive)?;
        println!("watching: {}", self.root.display());

        let mut projects: HashMap<PathBuf, Instant> = HashMap::new();

        loop {
            tokio::select! {
                Some(event) = rx.recv() => {
                    println!("event occured: {:?}", event);

                    match event.kind {
                        EventKind::Create(_) | EventKind::Modify(ModifyKind::Any) | EventKind::Modify(ModifyKind::Data(_)) => {
                            for path in event.paths {
                                if let Some(project) = find_project(&path) {
                                    projects.insert(project, Instant::now());
                                }
                            }
                        }
                        _ => {}
                    }
                }
                _ = tokio::time::sleep(Duration::from_secs(1)) => {
                    let mut finished = Vec::new();

                    for (project, last_change) in projects.iter() {
                        if last_change.elapsed() > self.config.timeout {
                            finished.push(project.clone());
                        }
                    }

                    for project in finished {
                        projects.remove(&project);

                        match archiver::archive_project(&project) {
                            Ok(archive) => {
                                println!("archive project: {} -> {}", project.display(), archive.display());
                            }
                            Err(e) => {
                                eprintln!("archive failed for project {}: {:?}", project.display(), e);

                            }

                        }
                        println!("render finished: {}", project.display());
                    }
                }

            }
        }
    }
}

fn find_project(path: &Path) -> Option<PathBuf> {
    let mut cur = path;

    while let Some(parent) = cur.parent() {
        if path.extension().and_then(|e| e.to_str()) != Some("wav") {
            continue;
        }

        if cur.file_name()?.to_str()?.eq_ignore_ascii_case("audio") {
            println!("path found");
            return Some(parent.to_path_buf());
        }

        cur = parent;
    }

    None
}
