use crate::scan::file::ScannedFile;
use postgres::Connection;
use std::fs;
use std::path::Path;
use subprocess::Exec;
use subprocess::Redirection;

pub struct Reencode {}
impl crate::module::Module for Reencode {
    fn module_name(&self) -> &str {
        "reencode"
    }
    fn module_iteration(&self, connection: &Connection) -> () {
        info!("Searching for targets to reencode");
        let mut done = false;
        let target_extension = self.config_string(&connection, "target_extension");
        let target_codec = self.config_string(&connection, "target_codec");
        while !done {
            done = true;
            debug!(
                "Selecting paths where extension+codec do not match {}+{}",
                &target_extension, &target_codec
            );
            let rows = connection
                .query(
                    "\
                UPDATE paths SET in_progress = true WHERE id = ( \
                    SELECT id FROM paths \
                    INNER JOIN video_extensions USING(extension) \
                    WHERE (extension != $1 or codec != $2) AND NOT in_progress \
                    LIMIT 1 \
                )\
                RETURNING id, path, bytes
                ",
                    &[&target_extension, &target_codec],
                )
                .unwrap();
            debug!("Got {:?}", rows);
            for row in rows.iter() {
                // always just one unless its zero
                done = false;
                let _id: i64 = row.get(0);
                let source_path_s: String = row.get(1);
                let source_path = Path::new(&source_path_s);
                let source_temp_path = Path::new("/tmp/in");
                let original_bytes: i64 = row.get(2);
                let target_path = source_path.with_extension(&target_extension);
                let temp_path = Path::new("/tmp/converting.x").with_extension(&target_extension);
                info!("Copy {:?} to temp", &source_path);
                fs::copy(&source_path, &source_temp_path).unwrap();
                info!("Converting {:?}", &source_path);
                let captured = Exec::cmd("ffmpeg")
                    .arg("-y")
                    .arg("-loglevel")
                    .arg("warning")
                    .arg("-i")
                    .arg(source_temp_path.to_str().unwrap().to_string())
                    .arg("-c:v")
                    .arg(&target_codec)
                    .arg("-c:a")
                    .arg("aac")
                    .arg("-hide_banner")
                    .arg("-nostats")
                    .arg(&temp_path)
                    .stdout(Redirection::Pipe)
                    .stderr(Redirection::Pipe)
                    .capture()
                    .unwrap();
                if captured.success() {
                    info!("cp {:?} {:?}", &temp_path, &target_path);
                    fs::copy(&temp_path, &target_path).unwrap();
                    let new_file = ScannedFile::new(&target_path, &connection).unwrap();
                    info!(
                        "Bytes {:?} -> {:?} = {:?}",
                        original_bytes,
                        new_file.bytes,
                        new_file.bytes - original_bytes
                    );
                    let _store_result = new_file.store(&connection);
                    if source_path != target_path {
                        info!("rm {:?}", &source_path);
                        fs::remove_file(&source_path).unwrap();
                        fs::remove_file(&source_temp_path).unwrap();
                    }
                } else {
                    warn!("ffmpeg failed: {}", &captured.stderr_str());
                }
            }
        }
        ()
    }
}
