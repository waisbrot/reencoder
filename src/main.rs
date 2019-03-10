extern crate postgres;
extern crate crypto;

use std::io;
use std::fs::{self, DirEntry};
use std::path::Path;
use postgres::{Connection, TlsMode};
use crypto::digest::Digest;
use crypto::sha3::Sha3;

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
fn visit_dirs(dir: &Path, visitor: &Fn(&DirEntry) -> io::Result<()>) -> io::Result<()> {
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

fn hash_from_path(path: &Path) -> io::Result<String> {
    let mut hasher = Sha3::sha3_256();
    let data = format!("{}", path.display());
    hasher.input_str(&data);
    Ok(hasher.result_str())
}

fn main() -> io::Result<()> {
    let connection = Connection::connect("postgres://media:media@tularemia.local", TlsMode::None)?;
    let root = Path::new("/Users/waisbrot/git/scan-to-postgres");
    let print_dir = |dir: &DirEntry| -> io::Result<()> {
        let path = dir.path();
        let path = path.as_path();
        let fake_file = File::new(hash_from_path(path)?, path);
        let updates = connection.execute("INSERT INTO paths (hash, path) VALUES ($1, $2)", &[&fake_file.hash, &fake_file.path]);
        match updates {
            Ok(_) => { Ok(()) },
            Err(e) => { println!("Error: {}", &e); Ok(()) }
        }
    };
    visit_dirs(root, &print_dir)?;
    //let fake_file = File::new(String::from("Fake"), root);
    //let updates = connection.execute("INSERT INTO paths (hash, path) VALUES ($1, $2)", &[&fake_file.hash, &fake_file.path])?;
    //println!("Inserted {} rows", updates);
    Ok(())
}
