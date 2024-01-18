mod ffprobe;
pub(crate) mod file;

use file::ScannedFile;
use postgres::Client;
use std::error::Error;
use std::fs::{self, DirEntry};
use std::path::Path;

type VoidResult = Result<(), Box<dyn Error>>;

// code from the Rust book
fn visit_dirs(dir: &Path, visitor: &mut dyn FnMut(&DirEntry) -> VoidResult) -> VoidResult {
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            visit_dirs(&path, visitor)?;
        } else {
            let metadata = path.symlink_metadata()?;
            let file_type = metadata.file_type();
            if file_type.is_symlink() {
                continue;
            } else {
                visitor(&entry)?;
            }
        }
    }
    Ok(())
}

fn scan(root: &String, connection: &mut Client) -> VoidResult {
    let mut visitor = |dir: &DirEntry| -> VoidResult {
        let path = dir.path();
        let path = path.as_path();
        let file = ScannedFile::new(path, connection)?;
        let result = file.store(connection);
        match result {
            Ok(i) => {
                debug!("Wrote {} rows for {}", &i, &file.path);
                Ok(())
            }
            Err(e) => {
                warn!("Error {} while trying to store file {:?}", &e, &file);
                Ok(())
            }
        }
    };
    let root_path = Path::new(root);
    if root_path.is_dir() {
        info!("Scanning from {}", &root);
        visit_dirs(Path::new(root), &mut visitor)?;
    } else {
        warn!("Root path {} does not appear to be a directory", &root);
    }
    Ok(())
}

pub struct Scan {}
impl crate::module::Module for Scan {
    fn module_name(&self) -> &str {
        "scan"
    }
    fn module_iteration(&self, connection: &mut Client) {
        let mut i = 0;
        for row in &connection
            .query("SELECT root FROM roots WHERE active ORDER BY root ASC", &[])
            .unwrap()
        {
            let root: String = row.get(0);
            scan(&root, connection).unwrap();
            i += 1;
        }
        info!("Scanned {} roots", &i);
    }
}
