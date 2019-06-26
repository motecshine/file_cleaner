use std::ops::Add;
use std::path::{Path, PathBuf};
use std::{
    fs,
    fs::File,
    io,
    io::{prelude::*, SeekFrom},
};

const WATCH_TICKER: u64 = 6;
const CHUNK_FILE_SIZE: u64 = 1024 * 1024 * 2; // 2mb
const MONTHLY: u64 = 60 * 60 * 24 * 30;
pub const PATH: &str = "/Users/zhuhaifeng/Desktop/workspace/rustcode/simple-file-watcher/file/";
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
            match self.recursive_dir(&p) {
                Ok(_) => println!("nothing todo"),
                Err(err) => println!("err: {:?}", err.to_string()),
            }
        }
        self
    }


    fn recursive_dir(&mut self, p: &Path) -> io::Result<()> {
        if p.is_dir() {
            for entry in fs::read_dir(p).unwrap() {
                let file_or_path = entry.unwrap().path();
                if file_or_path.is_dir() {
                    self.recursive_dir(&file_or_path).unwrap()
                } else {
                    self.chunk(file_or_path)
                }
            }
        } else {
            self.chunk(PathBuf::from(p))
        }
        Ok(())
    }

    fn chunk(&mut self, path: PathBuf) {
        let mut origin_fd = File::open(&path).unwrap();
        let origin_file_name = path.file_name().unwrap().to_str().unwrap();
        let origin_file_size = path.metadata().unwrap().len();
        println!("文件大小: {:?}", origin_file_size);
        let remaining_size = origin_file_size % CHUNK_FILE_SIZE;
        println!("不能被整除, 剩余量: {:?}bytes", remaining_size);
        let mut chunk_count = origin_file_size / CHUNK_FILE_SIZE;
        let suffix = 0;
        let mut seek_flag:u64 = 0;
        let mut chunk_start = 0;
        // 先处理剩余的
        if remaining_size > 0 {
            self.create_new_file(remaining_size, suffix, seek_flag, &mut origin_fd, origin_file_name);
            seek_flag += remaining_size;
            chunk_start += 1;
            chunk_count += 1;
        }

        for index in chunk_start..chunk_count {
            self.create_new_file(CHUNK_FILE_SIZE, index as i32, seek_flag, &mut origin_fd, origin_file_name);
        }
    }

    fn create_new_file(&mut self, buf_size: u64, file_suffix: i32, seek_flag: u64, origin_fd: &mut File, origin_file_name: &str) {
        origin_fd.seek(SeekFrom::Start(seek_flag)).unwrap();
        let file_name = self.new_file_name(origin_file_name, file_suffix);
        // 从源文件头部读取remaining_size大小的buf
        let mut buf = Box::new(vec![0; buf_size as usize]);
        origin_fd.read(&mut buf).unwrap();
        let mut fd = File::create(file_name).unwrap();
        fd.write(&mut buf).unwrap();
    }

    fn new_file_name(&mut self, origin_file_name: &str, suffix: i32) -> String {
        PATH.to_owned() + origin_file_name + "_" + suffix.to_string().as_str()
    }

    pub fn run(&mut self) {
        self.traverse_dir();
    }
}
