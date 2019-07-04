use postgres::Connection;
use std::path::Path;
use cadence::StatsdClient;
use cadence::prelude::*;

pub struct Clean {}
impl crate::module::Module for Clean {
    fn module_name(&self) -> &str {
        "clean"
    }
    fn module_iteration(&self, connection: &Connection, statsd: &StatsdClient) -> () {
        info!("Checking all paths for non-existant files");
        let mut done = false;
        let mut offset: i32 = 0;
        let limit: i32 = 100;
        while !done {
            done = true;
            debug!("Selecting paths");
            let rows = connection.query("SELECT path FROM paths ORDER BY path DESC OFFSET $1::int4 LIMIT $2::int4", &[&offset, &limit]).unwrap();
            debug!("Got {:?}", rows);
            for row in rows.iter() {
                done = false;
                let path: String = row.get(0);
                debug!("Checking {}", &path);
                if !Path::new(&path).is_file() {
                    info!("{} does not exist; removing it from the database", &path);
                    connection.execute("DELETE FROM paths WHERE path = $1", &[&path]).unwrap();
                    statsd.incr("deleted").unwrap();
                }
            }
            offset += limit;
        }
        ()
    }
}
