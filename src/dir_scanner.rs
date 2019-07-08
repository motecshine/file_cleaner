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
    exclude_dir: Vec<String>,
    file_extension: Vec<String>,
}

#[inline]
pub fn new_dir_scanner(sender: &mut Sender<String>) -> Result<DirScanner, Error> {
    let dir: Vec<String> = env::var("SCAN_PATH")?
        .split(",")
        .map(|s| s.to_string())
        .collect();
    let exclude_dir: Vec<String> = env::var("EXCLUDE_PATH")?
        .split(",")
        .map(|s| s.to_string())
        .collect();
    let file_extension: Vec<String> = env::var("FILE_SUFFIX")?
        .split(",")
        .map(|s| s.to_string())
        .collect();
    Ok(DirScanner {
        dir,
        sender,
        exclude_dir,
        file_extension,
    })
}

impl<'a> DirScanner<'a> {
    fn scan_parent_dir(&mut self, path: &str) {
        match self.child_dir_scanner(Path::new(path)) {
            Ok(_) => {},
            Err(err) => println!("some error occur{:?}.", err.to_string()),
        }
    }

    fn child_dir_scanner(&mut self, p: &Path) -> io::Result<()> {
        if p.is_dir() {
            for entry in fs::read_dir(p)? {
                let file_or_path = entry?.path();
                if file_or_path.is_dir() {
                    for v in &self.exclude_dir {
                        if *v == file_or_path.to_str().unwrap().to_string() {
                            println!("exclude dir: {:?}, current dir: {:?}", *v, file_or_path);
                            continue;
                        }
                    }
                    self.child_dir_scanner(file_or_path.as_path())?
                } else {
                    if !self.check_file_ext(&file_or_path) {
                        println!(
                            "file_suffix cant be handle {:?} {:?}.",
                            file_or_path.file_name(),
                            file_or_path.extension()
                        );
                        continue;
                    }
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
            if !self.check_file_ext(p) {
                println!("file_suffix cant be handle {:?}.", p);
                return Ok(());
            }
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

    pub fn check_file_ext(&mut self, file: &Path) -> bool {
        match file.extension() {
            Some(extension) => {
                for v in &self.file_extension {
                    if extension.to_str().unwrap().contains(v) {
                        println!("actually ext: {:?}, ext: {:?}", extension.to_str().unwrap(), v);
                        return true;
                    }
                }
            }
            None => println!("extension not in whitelist, {:?} ", file),
        }
        false
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
