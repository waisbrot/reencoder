use postgres::params::ConnectParams;
use clap;
use std::thread::JoinHandle;
use postgres::{Connection, TlsMode};
use std::io;
use std::thread::sleep;
use std::time::Duration;

fn modules_contains(modules: &clap::Values, target: &str) -> bool {
    modules.clone().filter(|&x| x == target).next().is_some()
}

pub trait Module where Self: std::marker::Sync {
    fn spawn_module(&self, postgres_config: &ConnectParams, modules: &clap::Values) -> Option<JoinHandle<()>> {
        let name = self.module_name();
        if modules_contains(modules, name) {
            let connection = Connection::connect(postgres_config.clone(), TlsMode::None).unwrap();
            let handle = std::thread::Builder::new()
                .name(name.to_string())
                .spawn(move || {
                    self.module_loop(&connection)
                }).unwrap();
            Some(handle)
        } else {
            None
        }
    }
    fn module_name(&self) -> &str;
    fn module_iteration(&self, connection: &Connection) -> io::Result<()>;
    fn module_loop(&self, connection: &Connection) -> () {
        let name = self.module_name();
        let interval_s: i32 = connection.query("SELECT (config->'interval')::int FROM config WHERE service = $1", &[&name]).unwrap().get(0).get(0);
        let interval = Duration::from_secs(interval_s as u64);
        loop {
            self.module_iteration(&connection).unwrap();
            sleep(interval);
        }
    }
}
