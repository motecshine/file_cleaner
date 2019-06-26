use std::path::Path;
pub mod file;
fn main() {
    let mut path = vec![Box::new(Path::new(
        "/Users/zhuhaifeng/Desktop/workspace/rustcode/simple-file-watcher/file",
    ))];
    let mut exclude_path = vec![Box::new(Path::new(
        "/Users/zhuhaifeng/Desktop/workspace/rustcode/simple-file-watcher",
    ))];
    let mut file_watcher: file::FileWatcher = file::new(&mut path, &mut exclude_path);
    file_watcher.run()
}
