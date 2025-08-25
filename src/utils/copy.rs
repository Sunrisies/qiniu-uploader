use std::fs;
use std::io;
use std::path::{Path, PathBuf};
pub fn copy_file(src: &Path, dst: &Path) -> Result<PathBuf, io::Error> {
    fs::copy(src, dst)?;
    Ok(dst.to_path_buf())
}
