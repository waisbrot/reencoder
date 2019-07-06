use postgres::Connection;
use std::path::Path;
use subprocess::Exec;
use subprocess::Redirection;
use std::fs;

pub struct Reencode{}
impl crate::module::Module for Reencode {
    fn module_name(&self) -> &str {
        "reencode"
    }
    fn module_iteration(&self, connection: &Connection) -> () {
        info!("Searching for targets to reencode");
        let mut done = false;
        let mut offset: i32 = 0;
        let limit: i32 = 100;
        let target_extension = self.config_string(&connection, "target_extension");
        let target_codec = self.config_string(&connection, "target_codec");
        while !done {
            done = true;
            debug!("Selecting paths where extension+codec do not match {}+{}", &target_extension, &target_codec);
            let rows = connection.query("SELECT path FROM paths INNER JOIN video_extensions USING(extension) WHERE extension != $1 or codec != $2 ORDER BY path DESC OFFSET $3::int4 LIMIT $4::int4", &[&target_extension, &target_codec, &offset, &limit]).unwrap();
            debug!("Got {:?}", rows);
            offset += limit;
            for row in rows.iter() {
                done = false;
                let source_path: String = row.get(0);
                let target_path = Path::new(&source_path).with_extension(&target_extension);
                let temp_path = Path::new("/tmp/converting.x").with_extension(&target_extension);
                info!("Converting {}", &source_path);
                let captured = Exec::cmd("ffmpeg")
                    .arg("-y")
                    .arg("-loglevel").arg("warning")
                    .arg("-i").arg(&source_path)
                    .arg("-c:v").arg(&target_codec)
                    .arg("-c:a").arg("aac")
                    .arg("-hide_banner")
                    .arg("-nostats")
                    .arg(&temp_path)
                    .stdout(Redirection::Pipe)
                    .stderr(Redirection::Pipe)
                    .capture()
                    .unwrap();
                if captured.success() {
                    fs::copy(&temp_path, &target_path).unwrap();
                    fs::remove_file(&source_path).unwrap();
                    info!("{} -> {:?}", &source_path, &target_path);
                } else {
                    warn!("ffmpeg failed: {}", &captured.stderr_str());
                }
            }
        }
        ()
    }
}
