use failure::Error;
use std::env;
use std::path::Path;
use std::{
    fs::File,
    io,
    io::{prelude::*, SeekFrom},
};

#[derive(Clone)]
pub struct FileCutter {
    chunk_size: u64,
}

#[inline]
pub fn new_cutter() -> Result<FileCutter, Error> {
    let chunk_size = env::var("CHUNK_FILE_SIZE")?.parse::<u64>()?;
    Ok(FileCutter { chunk_size })
}

impl FileCutter {
    pub fn chunk(&mut self, path_string: String) -> io::Result<()> {
        let path = Path::new(&path_string);
        let origin_file_size = path.metadata()?.len();
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
