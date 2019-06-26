use std::ops::Add;
use std::path::Path;
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
            match self.chunk(&p) {
                Ok(_) => println!("nothing todo"),
                Err(err) => println!("err: {:?}", err.to_string()),
            }
        }
        self
    }

    fn chunk(&mut self, p: &Path) -> io::Result<()> {
        if p.is_dir() {
            for entry in fs::read_dir(p).unwrap() {
                let file_or_path = entry.unwrap().path();
                if file_or_path.is_dir() {
                    self.chunk(&file_or_path).unwrap()
                } else {
                    println!("需要切割的文件: {:?}", file_or_path);
                    let mut fd = File::open(&file_or_path).unwrap();
                    let origin_file_name = file_or_path.file_name().unwrap().to_str().unwrap();
                    let origin_file_size = file_or_path.metadata().unwrap().len();
                    println!("文件大小: {:?}", origin_file_size);
                    let remaining_size = origin_file_size % CHUNK_FILE_SIZE;
                    println!("不能被整除, 剩余量: {:?}bytes", remaining_size);
                    let suffix = 0;
                    // 先处理剩余的
                    if remaining_size > 0 {
                        let new_file_name = self.new_file_name(origin_file_name, suffix);
                        println!("新的文件名: {:?}", new_file_name);
                        // 从源文件头部读取remaining_size大小的buf
                        let mut remaining_buf = Box::new(vec![0; remaining_size as usize]);
                        match fd.read(&mut remaining_buf) {
                            Ok(_) => {
                                let mut new_file = File::create(new_file_name).unwrap();
                                new_file.write(&mut remaining_buf).unwrap();
                                println!("处理剩余buf成功! {:?}", remaining_buf.len());
                                // 处理剩余的文件
                                let chunk_count = origin_file_size / CHUNK_FILE_SIZE;
                                println!("chunk_count! {:?}", chunk_count);
                                let mut seek_flag = remaining_size;
                                for index in 1..(chunk_count + 1) {
                                    fd.seek(SeekFrom::Start(seek_flag)).unwrap();
                                    let mut buf = Box::new(vec![0; CHUNK_FILE_SIZE as usize]);
                                    match fd.read(&mut buf) {
                                        Ok(_) => {
                                            let new_file_name = self.new_file_name(origin_file_name, index as i32);
                                            println!("文件名! {:?}", new_file_name);
                                            let mut new_file = File::create(new_file_name)?;
                                            new_file.write(&mut buf).unwrap();
                                            println!("处理剩余buf成功! {:?}", buf.len());
                                            // 增加偏移量
                                            seek_flag += CHUNK_FILE_SIZE;
                                        }
                                        Err(err) => println!("err: {:?}", err.to_string()),
                                    }
                                }
                            }
                            Err(err) => println!("err: {:?}", err.to_string()),
                        }
                    }
                }
            }
        } else {
            println!("p: {:?} ", p);
        }
        Ok(())
    }

    fn new_file_name(&mut self, origin_file_name: &str, suffix: i32) -> String {
        PATH.to_owned() + origin_file_name + "_" + suffix.to_string().as_str()
    }

    pub fn run(&mut self) {
        self.traverse_dir();
    }
}
