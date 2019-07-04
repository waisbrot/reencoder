use postgres::Connection;
use std::thread::sleep;
use std::time::Duration;
use std::time::SystemTime;
use cadence::StatsdClient;
use cadence::prelude::*;

pub trait Module where Self: std::marker::Sync {
    fn module_name(&self) -> &str;
    fn module_iteration(&self, connection: &Connection, statsd: &StatsdClient) -> ();
    fn module_loop(&self, connection: Connection, statsd: StatsdClient) -> () {
        let zero = Duration::from_secs(0);
        loop {
            let interval_s = self.config_int(&connection, "interval");
            let interval = Duration::from_secs(interval_s as u64);
            let iteration_start = SystemTime::now();
            self.module_iteration(&connection, &statsd);
            let iteration_duration = iteration_start.elapsed().unwrap_or(zero);
            statsd.time_duration("loop_time", iteration_duration).unwrap();
            sleep(interval);
        }
    }
    fn config_string(&self, connection: &Connection, key: &str) -> String {
        let name = self.module_name();
        let s: String = connection.query("SELECT (config->$1)::text FROM config WHERE service = $2", &[&key, &name])
            .unwrap()
            .get(0)
            .get(0);
        s.trim_matches('"').to_string()
    }
    fn config_int(&self, connection: &Connection, key: &str) -> i32 {
        let name = self.module_name();
        connection.query("SELECT (config->$1)::int FROM config WHERE service = $2", &[&key, &name])
            .unwrap()
            .get(0)
            .get(0)
    }
}
