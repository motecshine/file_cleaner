use failure::Error;
use std::env;
use std::path::Path;
use std::sync::mpsc::Sender;
use std::thread::sleep;
use std::time::Duration;
use std::{fs, io};

pub struct DirScanner<'a> {
    dir: Vec<String>,
    sender: &'a mut Sender<String>,
}

#[inline]
pub fn new_dir_scanner(sender: &mut Sender<String>) -> Result<DirScanner, Error> {
    let dir: Vec<String> = env::var("SCAN_PATH")?
        .split(",")
        .map(|s| s.to_string())
        .collect();
    Ok(DirScanner { dir, sender })
}

impl<'a> DirScanner<'a> {
    fn scan_parent_dir(&mut self, path: &str) -> &Self {
        match self.child_dir_scanner(Path::new(path)) {
            Ok(_) => println!("scan child dir success."),
            Err(err) => println!("some error occur{:?}.", err.to_string()),
        }
        self
    }

    fn child_dir_scanner(&mut self, p: &Path) -> io::Result<()> {
        if p.is_dir() {
            for entry in fs::read_dir(p)? {
                let file_or_path = entry?.path();
                if file_or_path.is_dir() {
                    self.child_dir_scanner(file_or_path.as_path())?
                } else {

                    match file_or_path.to_str() {
                        Some(p) => match self.sender.send(p.to_string()) {
                            Ok(_) => {}
                            Err(err) => {
                                println!("sending error: {:?}", err);
                            }
                        },
                        None => println!("sender error: path is not file"),
                    }
                }
            }
        } else {
            match p.to_str() {
                Some(path) => match self.sender.send(path.to_string()) {
                    Ok(_) => {}
                    Err(err) => {
                        println!("sending error: {:?}", err);
                    }
                },
                None => println!("sender error: path is not file"),
            }
        }
        Ok(())
    }

    pub fn run(&mut self) {
        let ticker = env::var("SCAN_DIR_TICKER").unwrap().parse::<u64>().unwrap();
        loop {
            sleep(Duration::from_secs(ticker));
            for path in self.dir.clone() {
                self.scan_parent_dir(&path);
            }
        }
    }
}
