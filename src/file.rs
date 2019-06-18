use std::ops::Add;
use std::path::Path;
use std::thread::sleep;
use std::time::Duration;
use std::{
    fs,
    fs::File,
    io,
    io::{prelude::*, BufReader, SeekFrom},
};

const WATCH_TICKER: u64 = 6;
const CHUNK_FILE_SIZE: u64 = (2 << 20) * 10; // 100mb
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
            for entry in fs::read_dir(p)? {
                let file_or_path = entry?.path();
                if file_or_path.is_dir() {
                    self.read_dir(&file_or_path)?;
                } else {
                    // 获取当前file的metadata
                    let file_meta = fs::metadata(&file_or_path).unwrap();

                    // 如果file一个月没有改动过了那就删除
                    let last_modified_time = file_meta.modified().unwrap();
                    if std::time::SystemTime::now()
                        > (last_modified_time.add(std::time::Duration::from_secs(MONTHLY)))
                    {
                        println!("need_remove:{:?}", &file_or_path);
                        // 移交所有权后需要跳出循环
                        continue;
                    }

                    // 切割文件
                    if file_meta.len() > CHUNK_FILE_SIZE {
                        println!(
                            "need_chunk:{:?}, MAX_FILE_SIZE: {:?}",
                            &file_or_path, CHUNK_FILE_SIZE
                        );
                        // chunkfile完成才会释放fd 貌似有风险
                        let mut fd = File::open(p).unwrap();
                        // 第一次seek从0开始
                        let _ = self.chunk_file(&mut fd, &file_meta, &mut 0);
                    }
                }
            }
        }
        Ok(())
    }

    // 当file.metadata.filesize大于1GB的时候进行切割.
    fn chunk_file(
        &mut self,
        fd: &mut File,
        file_meta: &fs::Metadata,
        seek: &mut u64,
    ) -> io::Result<()> {
        // 如果((file_meta.len()) - seek <= CHUNK_FILE_SIZE), 不够切割了, 分配内存为((file_meta.len()) - seek)
        // 如果现在的((file_meta.len()) - seek > CHUNK_FILE_SIZE), 计算下一次seek(seek + CHUNK_FILE_SIZE)的偏移, 分配内存为100mb,继续切割.
        if (file_meta.len() - *seek) <= CHUNK_FILE_SIZE {
            fd.seek(SeekFrom::Start(*seek))?;
            let mut buf: Vec<u8> = Vec::with_capacity(file_meta.len() as usize);
            // 读取 | start  -> end  | buf contents
            fd.read(&mut buf).unwrap();
            // u8 转换成string 然后存入文件

            // 读取内容
            return Ok(());
        } else {
            fd.seek(SeekFrom::Start(*seek))?;
            let end_seek = (*seek + CHUNK_FILE_SIZE) as i64;
            fd.seek(SeekFrom::End(end_seek))?;
            let mut buf: Vec<u8> = Vec::with_capacity(CHUNK_FILE_SIZE as usize);
            // 读取 | start  -> end  | buf contents
            fd.read(&mut buf).unwrap();
            // u8 转换成string 然后存入文件

            // 递归下一次
            let _ = self.chunk_file(fd, file_meta, &mut (end_seek as u64));
        }
        Ok(())
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
