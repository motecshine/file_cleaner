use std::env;
use std::path::{Path, PathBuf};

use failure::Error;
use std::{
    fs,
    fs::File,
    io,
    io::{prelude::*, SeekFrom},
};

pub struct FileWatcher {
    pub path: Vec<String>,
    pub chunk_size: u64,
}

#[inline]
pub fn new() -> Result<FileWatcher, Error> {
    let path: Vec<String> = env::var("SCAN_PATH")?
        .split(",")
        .map(|s| s.to_string())
        .collect();
    let chunk_size = env::var("CHUNK_FILE_SIZE")?.parse::<u64>()?;
    Ok(FileWatcher { path, chunk_size })
}

impl FileWatcher {
    fn scan_parent_dir(&mut self, path: &str) -> &Self {
        match self.recursive_child_dir(Path::new(path)) {
            Ok(_) => println!("nothing todo"),
            Err(err) => println!("some error occur.{:?}", err.to_string())
        }
        self
    }

    fn recursive_child_dir(&mut self, p: &Path) -> io::Result<()> {
        if p.is_dir() {
            for entry in fs::read_dir(p)? {
                let file_or_path = entry?.path();
                if file_or_path.is_dir() {
                    self.recursive_child_dir(&file_or_path)?
                } else {
                    self.chunk(file_or_path)?;
                }
            }
        } else {
            self.chunk(PathBuf::from(p))?;
        }
        Ok(())
    }

    fn chunk(&mut self, path: PathBuf) -> io::Result<()> {
        let origin_file_size = path.metadata()?.len();
        let remaining_size = origin_file_size % self.chunk_size;
        let mut origin_fd = File::open(&path)?;
        let mut chunk_count = origin_file_size / self.chunk_size;
        let mut seek_flag: u64 = 0;
        let mut chunk_start = 0;
        if remaining_size > 0 {
            self.create_new_file(
                remaining_size,
                0,
                seek_flag,
                &mut origin_fd,
                path.file_name().unwrap().to_str().unwrap(),
            )?;
            seek_flag += remaining_size;
            chunk_start += 1;
            chunk_count += 1;
        }

        for index in chunk_start..chunk_count {
            self.create_new_file(
                self.chunk_size,
                index,
                seek_flag,
                &mut origin_fd,
                path.to_str().unwrap(),
            )?;
            seek_flag += self.chunk_size;
        }
        Ok(())
    }

    #[inline]
    fn create_new_file(
        &mut self,
        buf_size: u64,
        file_suffix: u64,
        seek_flag: u64,
        origin_fd: &mut File,
        origin_file_name: &str,
    ) -> io::Result<()> {
        let file_name = self.new_file_name(origin_file_name, file_suffix);
        let mut buf = Box::new(vec![0; buf_size as usize]);
        let mut fd = File::create(file_name)?;
        origin_fd.seek(SeekFrom::Start(seek_flag))?;
        origin_fd.read(&mut buf)?;
        match  fd.write(&mut buf) {
            Ok(_) => Ok(()),
            Err(err) => {Err(err)},
        }
    }

    fn new_file_name(&mut self, origin_file_name: &str, suffix: u64) -> String {
        origin_file_name.to_owned() + "_" + suffix.to_string().as_str()
    }

    pub fn run(&mut self) {
        for path in self.path.clone() {
            self.scan_parent_dir(&path);
        }
    }
}
