use archiver;
use notify::{
    Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher as NorifyWatcher,
    event::ModifyKind,
};
use std::{
    collections::{HashMap, HashSet},
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
        let mut finished_projects: HashSet<PathBuf> = HashSet::new();

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

                    match event.kind {
                        EventKind::Create(_)
                        | EventKind::Modify(ModifyKind::Any)
                        | EventKind::Modify(ModifyKind::Data(_)) => {
                            for path in event.paths {
                                if let Some(project) = find_project(&path) {
                                    if !finished_projects.contains(&project) {
                                        projects.insert(project, Instant::now());
                                    }
                                }
                            }
                        }
                        _ => {}
                    }
                }
                _ = tokio::time::sleep(Duration::from_secs(1)) => {
                    let mut rendered = Vec::new();

                    for (project, last_change) in projects.iter() {
                        if last_change.elapsed() > self.config.timeout {
                            rendered.push(project.clone());
                        }
                    }

                    for project in rendered {
                        projects.remove(&project);
                        finished_projects.insert(project.clone());

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
    if path.extension()?.to_str()? != "wav" {
        return None;
    }

    for ancestor in path.ancestors() {
        if let Some(name) = ancestor.file_name().and_then(|n| n.to_str()) {
            if name.eq_ignore_ascii_case("audio") {
                return ancestor.parent().map(|p| p.to_path_buf());
            }
        }
    }

    None
}
