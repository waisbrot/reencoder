use postgres::Connection;
use std::thread::sleep;
use std::time::Duration;

pub trait Module
where
    Self: std::marker::Sync,
{
    fn module_name(&self) -> &str;
    fn module_iteration(&self, connection: &Connection) -> ();
    fn module_loop(&self, connection: Connection, doLoop: bool) -> () {
        loop {
            let interval_s = self.config_int(&connection, "interval");
            let interval = Duration::from_secs(interval_s as u64);
            self.module_iteration(&connection);
            if (doLoop) {
                sleep(interval);
            } else {
                break;
            }
        }
    }
    fn config_string(&self, connection: &Connection, key: &str) -> String {
        let name = self.module_name();
        let s: String = connection
            .query(
                "SELECT (config->$1)::text FROM config WHERE service = $2",
                &[&key, &name],
            )
            .unwrap()
            .get(0)
            .get(0);
        s.trim_matches('"').to_string()
    }
    fn config_int(&self, connection: &Connection, key: &str) -> i32 {
        let name = self.module_name();
        connection
            .query(
                "SELECT (config->$1)::int FROM config WHERE service = $2",
                &[&key, &name],
            )
            .unwrap()
            .get(0)
            .get(0)
    }
}
