extern crate postgres;
extern crate crypto;
extern crate chrono;
#[macro_use] extern crate log;
#[macro_use] extern crate lazy_static;
extern crate pretty_env_logger;
extern crate subprocess;
extern crate serde_json;
extern crate regex;
extern crate clap;
extern crate crossbeam_utils;
extern crate cadence;

mod scan;
mod clean;
mod reencode;
mod module;

use std::io;
use postgres::params::Builder;
use postgres::params::Host::Tcp;
use clap::{Arg, App};
use module::Module;
use cadence::StatsdClient;

fn main() -> io::Result<()> {
    pretty_env_logger::init();

    let args = App::new("Video converter")
        .version("0.1")
        .author("Nathaniel Waisbrot")
        .about("Find and convert video to hvec")
        .arg(Arg::with_name("username")
             .help("Postgres username")
             .long("username")
             .takes_value(true)
             .required(true))
        .arg(Arg::with_name("password")
             .help("Postgres password")
             .long("password")
             .takes_value(true)
             .required(true))
        .arg(Arg::with_name("host")
             .help("Postgres hostname")
             .long("host")
             .takes_value(true)
             .required(true))
        .arg(Arg::with_name("modules")
             .help("Modules to activate")
             .long("modules")
             .required(false)
             .takes_value(true)
             .multiple(true)
             .require_delimiter(true)
             .default_value("scan,clean,reencode"))
        .arg(Arg::with_name("statsd-host")
             .help("hostname:port for StatsD server")
             .long("statsd-host")
             .required(false)
             .takes_value(true)
             .default_value("localhost:8125"))
        .get_matches();

    // Postgres setup
    let postgres_config = Builder::new()
        .user(args.value_of("username").unwrap(), args.value_of("password"))
        .build(Tcp(args.value_of("host").unwrap().to_string()));

    // StatsD setup
    let statsd_host = args.value_of("statsd-host").unwrap();

    // Modules
    let modules = args.values_of("modules").unwrap();

    fn modules_contains(modules: &clap::Values, target: &str) -> bool {
        modules.clone().filter(|&x| x == target).next().is_some()
    }

    let mut all_modules: Vec<&Module> = Vec::new();
    all_modules.push(&scan::Scan{});
    all_modules.push(&clean::Clean{});
    all_modules.push(&reencode::Reencode{});
    crossbeam_utils::thread::scope(|scope| {
        for m in all_modules.iter() {
            let name = m.module_name();
            if modules_contains(&modules, name) {
                let connection = postgres::Connection::connect(postgres_config.clone(), postgres::TlsMode::None).unwrap();
                let statsd_prefix = format!("reencode.app.{}", &name);
                let stats_client = StatsdClient::from_udp_host(&statsd_prefix, statsd_host).unwrap();
                scope
                    .builder()
                    .name(name.to_string())
                    .spawn(move |_| {
                        m.module_loop(connection, stats_client)
                    }).unwrap();
            }
        }
    }).unwrap();

    info!("All modules have been skipped or failed. END OF LINE");
    Ok(())
}
