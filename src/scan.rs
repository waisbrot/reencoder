mod file;
mod ffprobe;

use std::io;
use std::path::Path;
use std::fs::{self, DirEntry};
use std::time::Duration;
use std::thread::sleep;
use file::ScannedFile;
use postgres::Connection;

// code from the Rust book
fn visit_dirs(dir: &Path, visitor: &Fn(&DirEntry) -> io::Result<()>) -> io::Result<()> {
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            visit_dirs(&path, visitor)?;
        } else {
            let metadata = path.symlink_metadata()?;
            let file_type = metadata.file_type();
            if file_type.is_symlink() {
                continue
            } else {
                visitor(&entry)?;
            }
        }
    }
    Ok(())
}

fn scan(root: &String, connection: &Connection) -> io::Result<()> {
    let visitor = |dir: &DirEntry| -> io::Result<()> {
        let path = dir.path();
        let path = path.as_path();
        let file = ScannedFile::new(path, &connection)?;
        let result = file.store(&connection);
        match result {
            Ok(i) => {
                debug!("Wrote {} rows for {}", &i, &file.path);
                Ok(())
            },
            Err(e) => {
                warn!("Error {} while trying to store file {:?}", &e, &file);
                Ok(())
            }
        }
    };
    let root_path = Path::new(root);
    if root_path.is_dir() {
        info!("Scanning from {}", &root);
        visit_dirs(Path::new(root), &visitor)?;
    } else {
        warn!("Root path {} does not appear to be a directory", &root);
    }
    Ok(())
}

fn scan_all(connection: &Connection) -> io::Result<()> {
    let mut i = 0;
    for row in &connection.query("SELECT root FROM roots WHERE active ORDER BY root ASC", &[]).unwrap() {
        let root: String = row.get(0);
        scan(&root, connection)?;
        i += 1;
    }
    info!("Scanned {} roots", &i);
    Ok(())
}

pub fn scan_loop(connection: &Connection) -> io::Result<()> {
    let interval_s: i32 = connection.query("SELECT (config->'interval')::int FROM config WHERE service = 'scan'", &[])?.get(0).get(0);
    let interval = Duration::from_secs(interval_s as u64);
    loop {
        scan_all(&connection)?;
        sleep(interval);
    }
}
