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

mod scan;
mod clean;

use std::io;
use postgres::{Connection, TlsMode};
use postgres::params::Builder;
use postgres::params::Host::Tcp;
use clap::{Arg, App};
use std::thread;

fn modules_contains(modules: &clap::Values, target: &str) -> bool {
    modules.clone().filter(|&x| x == target).next().is_some()
}

fn spawn_scan(postgres_config: &postgres::params::ConnectParams, modules: &clap::Values) -> Option<thread::JoinHandle<io::Result<()>>> {
    if modules_contains(modules, "scan") {
        let connection = Connection::connect(postgres_config.clone(), TlsMode::None).unwrap();
        let handle = thread::Builder::new()
            .name("scan".to_string())
            .spawn(move || {
                scan::scan_loop(&connection)
            }).unwrap();
        Some(handle)
    } else {
        None
    }
}

fn spawn_clean(postgres_config: &postgres::params::ConnectParams, modules: &clap::Values) -> Option<thread::JoinHandle<io::Result<()>>> {
    if modules_contains(modules, "clean") {
        let connection = Connection::connect(postgres_config.clone(), TlsMode::None).unwrap();
        let handle = thread::Builder::new()
            .name("clean".to_string())
            .spawn(move || {
                clean::clean_loop(&connection)
            }).unwrap();
        Some(handle)
    } else {
        None
    }
}

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
             .default_value("scan,clean"))
        .get_matches();

    let postgres_config = Builder::new()
        .user(args.value_of("username").unwrap(), args.value_of("password"))
        .build(Tcp(args.value_of("host").unwrap().to_string()));

    let modules = args.values_of("modules").unwrap();

    let scan_thread = spawn_scan(&postgres_config, &modules);
    let clean_thread = spawn_clean(&postgres_config, &modules);

    match scan_thread {
        Some(handle) => handle.join().unwrap(),
        None => Ok(())
    }?;
    match clean_thread {
        Some(handle) => handle.join().unwrap(),
        None => Ok(())
    }?;
    info!("All modules have been skipped or failed. END OF LINE");
    Ok(())
}
