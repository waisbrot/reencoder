extern crate postgres;

use std::io;
use std::fs::{self, DirEntry};
use std::path::Path;
use postgres::{Connection, TlsMode};

struct File {
    hash: String,
    path: String,
}

impl File {
    fn new(hash: String, path: &Path) -> File {
        File { hash: hash, path: format!("{}", path.display()) }
    }
}

// one possible implementation of walking a directory only visiting files
fn visit_dirs(dir: &Path, visitor: fn(&DirEntry) -> io::Result<()>) -> io::Result<()> {
    if dir.is_dir() {
        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir() {
                visit_dirs(&path, visitor)?;
            } else {
                visitor(&entry)?;
            }
        }
    }
    Ok(())
}

fn print_dir(dir: &DirEntry) -> io::Result<()> {
    println!("{:?}", dir.path());
    Ok(())
}

fn main() -> io::Result<()> {
    let connection = Connection::connect("postgres://media:media@tularemia.local", TlsMode::None)?;
    let root = Path::new("/Users/waisbrot/git/scan-to-postgres");
    visit_dirs(root, print_dir)?;
    let fake_file = File::new(String::from("Fake"), root);
    let updates = connection.execute("INSERT INTO paths (hash, path) VALUES ($1, $2)", &[&fake_file.hash, &fake_file.path])?;
    println!("Inserted {} rows", updates);
    Ok(())
}
