use std::path::Path;
pub mod file;

fn main() {
    let mut path = vec![Box::new(Path::new(
        "/Users/zhuhaifeng/Desktop/workspace/rust/filewatcher/file",
    ))];
    let mut exclude_path = vec![Box::new(Path::new(
        "/Users/zhuhaifeng/Desktop/workspace/rust/filewatcher/target",
    ))];
    let mut file_watcher: file::FileWatcher = file::new(&mut path, &mut exclude_path);
    file_watcher.run()
}
