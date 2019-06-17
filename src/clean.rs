use postgres::Connection;
use std::path::Path;
use std::thread::sleep;
use std::time::Duration;
use std::io::Result;

pub fn clean_loop(connection: &Connection) -> Result<()> {
    let interval_s: i32 = connection.query("SELECT (config->'interval')::int FROM config WHERE service = 'clean'", &[])?.get(0).get(0);
    let interval = Duration::from_secs(interval_s as u64);
    loop {
        info!("Checking all paths for non-existant files");
        clean_all(&connection)?;
        sleep(interval);
    }
}

fn clean_all(connection: &Connection) -> Result<()> {
    let mut done = false;
    let mut offset: i32 = 0;
    let limit: i32 = 100;
    while !done {
        done = true;
        debug!("Selecting paths");
        let rows = connection.query("SELECT path FROM paths ORDER BY path DESC OFFSET $1::int4 LIMIT $2::int4", &[&offset, &limit])?;
        debug!("Got {:?}", rows);
        for row in rows.iter() {
            done = false;
            let path: String = row.get(0);
            debug!("Checking {}", &path);
            if !Path::new(&path).is_file() {
                info!("{} does not exist; removing it from the database", &path);
                connection.execute("DELETE FROM paths WHERE path = $1", &[&path])?;
            }
        }
        offset += limit;
    }
    Ok(())
}
