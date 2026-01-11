use std::{collections::HashMap, path::{Path, PathBuf}, sync::{Arc, Mutex}, time::{Duration, Instant}};

use notify::{Event, RecommendedWatcher, RecursiveMode, Watcher as NorifyWatcher};

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
            config: WatcherConfig { timeout: (Duration::from_secs(5)) }
        }
    }

    pub async fn run(&self) -> notify::Result<()> {
        let projects: Arc<Mutex<HashMap<PathBuf, Instant>>> = Arc::new(Mutex::new(HashMap::new()));

        let projects_clone = projects.clone();

        let mut watcher: RecommendedWatcher = 
            notify::recommended_watcher(move |res: notify::Result<Event>| {
                match res {
                    Ok(event) => {
                        println!("event occured: {:?}", event);
                        for path in event.paths {
                            if let Some(project) = find_project(&path) {
                                let mut map = projects_clone.lock().unwrap();
                                map.insert(project, Instant::now());
                            }
                        }
                    }
                    Err(e) => println!("watch error: {:?}", e)
                } 
            })?;

        watcher.watch(&self.root, RecursiveMode::Recursive)?;
        println!("watching: {}", self.root.display());

        loop {
            tokio::time::sleep(Duration::from_secs(1)).await;

            let mut finished = Vec::new();

            {
                let map = projects.lock().unwrap();
                for (project, last_change) in map.iter() {
                    if last_change.elapsed() > self.config.timeout {
                        finished.push(project.clone());
                    }
                }
            }

            if !finished.is_empty() {
                let mut map = projects.lock().unwrap();
                
                for project in finished {
                    map.remove(&project);
                    println!("render finished: {}", project.display())
                }
            }
        }
    }
}

fn find_project(path: &Path) -> Option<PathBuf> {
    let mut cur = path;

    while let Some(parent) = cur.parent() {
        if cur.file_name()?.to_str()? == "Audio" {
            return Some(parent.to_path_buf())
        }

        cur = parent;
    }

    None
}