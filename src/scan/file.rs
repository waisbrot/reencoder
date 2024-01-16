use crate::scan::ffprobe;
use chrono::offset::Local;
use chrono::DateTime;
use postgres;
use postgres::row::Row;
use sha256::Sha256Digest;
use std::cmp::{max, min};
use std::error::Error;
use std::fs::File;
use std::io::Read;
use std::path::Path;

const FILE_SAMPLE_LENGTH: usize = 1024;

#[derive(Debug)]
enum Operation {
    INSERT,
    UPDATE,
}

#[derive(Debug)]
pub struct ScannedFile {
    hash: String,
    pub path: String,
    codec: Option<String>,
    height: Option<i32>,
    width: Option<i32>,
    kbps: Option<f32>,
    extension: Option<String>,
    pub bytes: i64,
    last_modified: DateTime<Local>,
    operation: Option<Operation>,
}

impl ScannedFile {
    pub fn new(
        path: &Path,
        connection: &mut postgres::Client,
    ) -> Result<ScannedFile, Box<dyn Error>> {
        let mut file = File::open(path)?;
        let path_string = format!("{}", path.display());
        let last_modified = last_modified(&file)?;
        let existing_files = connection.query("SELECT hash, last_modified, codec, height, width, kbps, extension, bytes FROM paths WHERE path = $1", &[&path_string])
            .or(Err("failed to query for known paths"))?;
        Self::new_from_result(&mut file, path_string, last_modified, &existing_files)
    }
    fn new_from_result(
        file: &mut File,
        path_string: String,
        last_modified: DateTime<Local>,
        existing_files: &Vec<Row>,
    ) -> Result<ScannedFile, Box<dyn Error>> {
        if existing_files.is_empty() {
            Self::new_from_file(file, path_string, last_modified, Some(Operation::INSERT))
        } else {
            let found = existing_files.get(0);
            let db_last_modified: DateTime<Local> =
                found.expect("found no files").get("last_modified");
            // Postgres timestamps are less precise than I get from the OS here, so look only at whole ms resolution
            let delta = last_modified - db_last_modified;
            let delta_ms = delta.num_milliseconds();
            if delta_ms < 1 {
                debug!("Last modified in the DB is newer or same; no change");
                Self::new_from_row(&found.expect("found no rows"), path_string, None)
            } else {
                debug!(
                    "Last modified in the DB is older ({} < {}); needs update",
                    &db_last_modified, &last_modified
                );
                Self::new_from_file(file, path_string, last_modified, Some(Operation::UPDATE))
            }
        }
    }
    fn new_from_file(
        file: &mut File,
        path_string: String,
        last_modified: DateTime<Local>,
        operation: Option<Operation>,
    ) -> Result<ScannedFile, Box<dyn Error>> {
        let hash = hash(file)?;
        let path = path_string;
        let (codec, height, width, kbps) = ffprobe::probe(&path)?;
        let extension = file_extension(&path);
        let bytes = file_bytes(&file);
        Ok(ScannedFile {
            hash,
            path,
            codec,
            height,
            width,
            kbps,
            extension,
            bytes,
            last_modified,
            operation,
        })
    }
    fn new_from_row(
        row: &Row,
        path_string: String,
        operation: Option<Operation>,
    ) -> Result<ScannedFile, Box<dyn Error>> {
        let hash = row.get("hash");
        let last_modified = row.get("last_modified");
        let codec = row.get("codec");
        let height = row.get("height");
        let width = row.get("width");
        let kbps = row.get("kbps");
        let extension = row.get("extension");
        let bytes = row.get("bytes");
        let path = path_string;
        Ok(ScannedFile {
            hash,
            path,
            codec,
            height,
            width,
            kbps,
            extension,
            bytes,
            last_modified,
            operation,
        })
    }
    pub fn store(
        &self,
        connection: &mut postgres::Client,
    ) -> core::result::Result<u64, postgres::Error> {
        match &self.operation {
            Some(Operation::INSERT) => connection.execute("INSERT INTO paths (hash, path, last_modified, codec, height, width, kbps, extension, bytes) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)", &[&self.hash, &self.path, &self.last_modified, &self.codec, &self.height, &self.width, &self.kbps, &self.extension, &self.bytes]),
            Some(Operation::UPDATE) => connection.execute("UPDATE paths SET (hash, last_modified) = ($1, $3) WHERE path = $2", &[&self.hash, &self.path, &self.last_modified]),
            None => Ok(0)
        }
    }
}

fn file_extension(path: &String) -> Option<String> {
    match Path::new(&path).extension() {
        None => None,
        Some(os_str) => match os_str.to_os_string().into_string() {
            Ok(string) => Some(string),
            Err(_) => None,
        },
    }
}

fn last_modified(file: &File) -> Result<DateTime<Local>, Box<dyn Error>> {
    let metadata = file.metadata()?;
    let created = metadata.created();
    let modified = metadata.modified();
    match (created, modified) {
        (Ok(t), Err(_)) => Ok(DateTime::<Local>::from(t)),
        (Err(_), Ok(t)) => Ok(DateTime::<Local>::from(t)),
        (Ok(t1), Ok(t2)) => Ok(max(
            DateTime::<Local>::from(t1),
            DateTime::<Local>::from(t2),
        )),
        (Err(e1), Err(e2)) => {
            panic!(
                "created_at says '{}'; modified_at says '{}'. Can't work with no timestamps,",
                e1, e2
            );
        }
    }
}

fn file_bytes(file: &File) -> i64 {
    file.metadata().unwrap().len() as i64
}

fn hash(file: &mut File) -> Result<String, Box<dyn Error>> {
    let mut chunk: [u8; FILE_SAMPLE_LENGTH] = [0; FILE_SAMPLE_LENGTH];
    let metadata = file.metadata()?;
    let file_size = metadata.len();
    let read_length: usize = min(file_size as usize, FILE_SAMPLE_LENGTH);
    let slice = &mut chunk[0..read_length];
    file.read_exact(slice)?;
    Ok(slice.digest())
}
