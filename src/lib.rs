use notify::{RecommendedWatcher, RecursiveMode, Watcher, Config, EventKind, Event, INotifyWatcher};
use std::fs::{File};
use std::io::{Seek, SeekFrom, BufRead};
use std::sync::mpsc::{self, Receiver};
use std::time::Duration;
use std::path::{Path, PathBuf};
use colored::{self, ColoredString};
use std::io;
use colored::Colorize;
use std::collections::HashMap;

#[derive(PartialEq, Debug)]
pub enum MageLogIssues {
    WARNING,
    ERROR,
    CRITICAL,
    ALL
}

pub struct MageLog {
    pub is_all: bool,
    pub path: String,
    pub issues: Vec<MageLogIssues>,
    pub files: Vec<String>,
    pub watchers: Vec<INotifyWatcher>,
    pub positions: HashMap<String, u64>
}

impl MageLog {
    pub fn new() -> Self {
        let mage_log = MageLog {
            is_all: false,
            path: String::from(""),
            issues: vec![],
            files: vec![],
            watchers: vec![],
            positions: HashMap::new()
        };

        mage_log
    }

    pub fn set_path(self: &mut Self, path: String) -> &mut Self {
        self.files.clear();
        self.path = path;
        self
    }

    pub fn set_is_all(self: &mut Self, is_all: bool) -> &mut Self {
        if is_all {
            self.is_all = true;
            self.issues.push(MageLogIssues::ERROR);
        }

        self
    }

    pub fn set_error_issue(self: &mut Self, is_error: bool) -> &mut Self {
        if is_error {
            self.issues.push(MageLogIssues::ERROR);
        }
        
        self
    }

    pub fn set_warning_issue(self: &mut Self, is_warning: bool) -> &mut Self {
        if is_warning {
            self.issues.push(MageLogIssues::WARNING);
        }
        
        self
    }

    pub fn set_critical_issue(self: &mut Self, is_critical: bool) -> &mut Self {
        if is_critical {
            self.issues.push(MageLogIssues::CRITICAL);
        }
        
        self
    }

    pub fn calculate_positions(self: &mut Self) -> &mut Self {
        let filenames = [
            String::from("exception.log"),
            String::from("debug.log"),
            String::from("system.log")
        ];

        for filename in filenames {
            let filepath = self.path.clone() + &filename;
            self.files.push(filepath.clone());
            // self.positions.entry(filepath).or_insert(contents.len() as u64);

            let f = File::open(&filepath).unwrap();

            // let content_len = contents.len() as u64;
            let content_len = f.metadata().unwrap().len() as u64;

            self.positions.insert(
                filepath,
                content_len,
            );
        }

        self
    }

    pub fn run_watchers(self: &mut Self) -> Receiver<Result<Event, notify::Error>> {
        
        let (tx, rx) = mpsc::channel();

        let mut config = Config::default();
        config = Config::with_poll_interval(config, Duration::from_millis(100));

        for filename in self.files.iter() {
            let cloned_tx = tx.clone();

            let mut watcher = match RecommendedWatcher::new(cloned_tx, config) {
                Ok(watcher) => {watcher},
                Err(_) => {panic!("cannot create watcher");}
            };
            
            let path = Path::new(filename);

            watcher.watch(path, RecursiveMode::NonRecursive).unwrap_or(());
            self.watchers.push(watcher);
        }
        
        rx
    }

    pub fn watch(self: &mut Self) -> ! {
        self.calculate_positions();

        let rx = self.run_watchers();

        loop {
            match rx.recv() {
                Ok(event) => {
                    let unwrapped_event = event.unwrap();
                    match unwrapped_event {
                        Event {kind: EventKind::Modify(_), paths: paths_vec, ..} => {
                            let newpaths = paths_vec.first().unwrap();
                            let f = File::open(&newpaths).unwrap();
                            self.handle_output(f, newpaths);
                        },
                        _ => {}
                    }
                }
                Err(err) => {
                    eprintln!("Error: {:?}", err);
                    std::process::exit(1);
                }
            }
        }
    }

    fn handle_output(self: &mut Self, mut f: File, newpaths: &PathBuf) {
        let fullpath = newpaths.to_string_lossy().to_string();
        let pos = self.positions.get(&fullpath).unwrap();
        let newpos = f.metadata().unwrap().len() as u64;
        f.seek(SeekFrom::Start(*pos)).unwrap();

        if *pos < newpos {

            self.positions.entry(fullpath.clone()).and_modify(move |localpos| *localpos = newpos);
            let filename = fullpath.split("/").last().unwrap().to_owned();
    
            let lines = self.read_lines(f)
                .unwrap()
                .filter_map(|line| {
                    for issue in &self.issues {
                        match issue {
                            MageLogIssues::ALL => { return self.handle_line(MageLogIssues::ALL, filename.clone().normal().clear(), line.as_ref().unwrap())}
                            MageLogIssues::CRITICAL => { return self.handle_line(MageLogIssues::ALL, filename.clone().normal().clear(), line.as_ref().unwrap())}
                            MageLogIssues::WARNING => { return self.handle_line(MageLogIssues::ALL, filename.clone().normal().clear(), line.as_ref().unwrap())}
                            MageLogIssues::ERROR => { return self.handle_line(MageLogIssues::ALL, filename.clone().normal().clear(), line.as_ref().unwrap())}
                        }
                    }

                    None::<String>
                });
    
            for line in lines {
                println!("{}", line);
            }
        }
    }

    fn read_lines(&self, file: File) -> io::Result<io::Lines<io::BufReader<File>>>
    {
        Ok(io::BufReader::new(file).lines())
    }

    fn handle_line(&self, issue: MageLogIssues, filename: ColoredString, line: &String) -> Option<String> {
        if (line.contains("main.ERROR") || line.contains("main.CRITICAL")) &&
          (self.is_all || issue == MageLogIssues::CRITICAL)
        {
            return Some(format!("{} --> {}", filename, line.red()));
        }

        if line.contains("main.WARNING") && (self.is_all || issue == MageLogIssues::WARNING) {
            return Some(format!("{} --> {}", filename, line.yellow()));
        }

        return None;
    }
}
