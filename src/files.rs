use std::{
    fs,
    path::{Path, PathBuf},
};

pub fn check_if_path_exists(path: &PathBuf) -> Result<bool, std::io::Error> {
    match fs::metadata(path) {
        Ok(metadata) => Ok(true),
        Err(e) => match e.kind() {
            std::io::ErrorKind::NotFound => Ok(false),
            _ => Err(e),
        },
    }
}

pub fn copy_dir_contents(src: &Path, dst: &Path) -> std::io::Result<()> {
    if src.is_dir() {
        if !dst.exists() {
            fs::create_dir(dst)?;
        }

        for entry in fs::read_dir(src)? {
            let entry = entry?;
            let path = entry.path();
            let file_name = path.file_name().unwrap().to_str().unwrap();
            let dst_path = dst.join(file_name);

            if entry.file_type()?.is_dir() {
                copy_dir_contents(&path, &dst_path)?;
            } else {
                fs::copy(&path, &dst_path)?;
            }
        }
    }

    Ok(())
}

pub fn remove_files_in_dir(dir_path: &Path) {
    if dir_path.exists() && dir_path.is_dir() {
        let inner_paths = fs::read_dir(dir_path).unwrap();
        for inner_path in inner_paths {
            let inner_path = inner_path.unwrap();
            if inner_path.file_type().unwrap().is_dir() {
                let inner_dir_name = inner_path.file_name();
                let inner_dir_path = dir_path.join(inner_dir_name);
                remove_files_in_dir(&inner_dir_path);
            } else {
                let file_name = inner_path.file_name();
                let file_path = dir_path.join(file_name);
                if file_path.exists() && file_path.is_file() {
                    fs::remove_file(file_path).unwrap();
                }
            }
        }
    }
}
