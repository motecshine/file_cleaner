use failure::Error;
use std::env;
use std::path::Path;
use std::time::Duration;
use std::{
    fs,
    fs::File,
    io,
    io::{prelude::*, SeekFrom},
};

#[derive(Clone)]
pub struct FileCleaner {
    chunk_size: u64,
}

#[inline]
pub fn new_cleaner() -> Result<FileCleaner, Error> {
    let chunk_size = env::var("CHUNK_FILE_SIZE")?.parse::<u64>()?;
    Ok(FileCleaner { chunk_size })
}

impl FileCleaner {
    pub fn clean(&mut self, path_string: String) -> io::Result<()> {
        let path = Path::new(&path_string);
        let origin_file_size = path.metadata()?.len();
        let can_be_move_outdate_file = env::var("REMOVE_OUTDATE_FILE").unwrap().parse::<bool>().unwrap();

        if can_be_move_outdate_file {
            let last_modified_time = path.metadata().unwrap().modified().unwrap();
            let outdate_duration = env::var("OUTDATE_DURATION").unwrap().parse::<u64>().unwrap();
            if std::time::SystemTime::now() > (last_modified_time + Duration::from_secs(outdate_duration))  {
                fs::remove_file(&path_string)?;
                return Ok(());
            }
        }

        if origin_file_size <= self.chunk_size {
            println!("[{:?}], 不需要处理!", path.file_name());
            return Ok(());
        }
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
                path.to_str().unwrap(),
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
        let can_be_move = env::var("FILE_CAN_BE_MOVE").unwrap().parse::<bool>().unwrap();
        if can_be_move {
            fs::remove_file(path_string)?;
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
        match fd.write(&mut buf) {
            Ok(_) => Ok(()),
            Err(err) => Err(err),
        }
    }

    fn new_file_name(&mut self, origin_file_name: &str, suffix: u64) -> String {
        origin_file_name.to_owned() + "_" + suffix.to_string().as_str()
    }
}
