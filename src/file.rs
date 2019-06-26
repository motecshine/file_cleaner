use std::ops::Add;
use std::path::Path;
use std::thread::sleep;
use std::time::Duration;
use std::{
    fs,
    fs::File,
    io,
    io::{prelude::*, SeekFrom},
};

const WATCH_TICKER: u64 = 6;
const CHUNK_FILE_SIZE: u64 = (2 << 20) * 2;
const MONTHLY: u64 = 60 * 60 * 24 * 30;
pub struct FileWatcher<'a> {
    // 指定多少时间刷新获取一次目录. 默认10分钟
    pub watcher_ticker: u64,
    pub path: &'a mut Vec<Box<&'a Path>>,
    pub exclude_path: &'a mut Vec<Box<&'a Path>>,
}

pub fn new<'a>(
    path: &'a mut Vec<Box<&'a Path>>,
    exclude_path: &'a mut Vec<Box<&'a Path>>,
) -> FileWatcher<'a> {
    FileWatcher {
        watcher_ticker: WATCH_TICKER,
        path,
        exclude_path,
    }
}

impl<'a> FileWatcher<'a> {
    // 读取遍历目录
    fn traverse_dir(&mut self) -> &Self {
        // @todo exclude path logic
        for p in self.path.clone() {
            let _ = self.read_dir(&p);
        }
        self
    }

    fn read_dir(&mut self, p: &Path) -> io::Result<()> {
        if p.is_dir() {
            for entry in fs::read_dir(p).unwrap() {
                let file_or_path = entry.unwrap().path();
                if file_or_path.is_dir() {
                    self.read_dir(&file_or_path).unwrap();
                } else {}
            }
        }
        Ok(())
    }
}