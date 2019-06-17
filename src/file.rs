use std::{io, fs};
use std::path::Path;
use std::thread::sleep;
use std::time::Duration;

const WATCHE_TICKER: u64 = 6;
const MAX_FILE_SIZE: u64 = 1024 * 1024;
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
        watcher_ticker: WATCHE_TICKER,
        path: path,
        exclude_path: exclude_path,
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

    // 递归遍历子目录下的所有目录 这里其实不需要返回值太丑了
    fn read_dir(&mut self, p: &Path) -> io::Result<()> {
        if p.is_dir() {
            for entry in fs::read_dir(p)? {
                let file_or_path = entry?.path();
                if file_or_path.is_file() {
                    // 获取当前file的metadata
                    let fmeta = fs::metadata(file_or_path).unwrap(); 
                    // 如果file一个月没有改动过了那就删除
                    let mut last_modified_time = fmeta.modified();
                     if std::time::Instant::now() > (last_modified_time + std::time::Duration::from_secs(60 *60 *24 *30))  {
                        
                     }
                    // 可能需要chunk
                    println!("{:?}", fmeta.modified());
                    
                } else {
                    self.read_dir(&file_or_path)?;
                }
            }
        }
        Ok(())
    }

    // 当file.metadata.filesize大于1GB的时候进行切割.
    fn chunk_file(&mut self) -> &Self {
        self
    }

    // 按创建时间排序移除大于一个月的日志文件
    fn remove(&mut self) -> &Self {
        self
    }

    // 启动
    pub fn run(&mut self) {
        loop {
            sleep(Duration::from_secs(self.watcher_ticker));
            self.traverse_dir();
        }
    }
}
