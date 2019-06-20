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
mod module;

use std::io;
use postgres::params::Builder;
use postgres::params::Host::Tcp;
use clap::{Arg, App};
use module::Module;

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

    let scan_thread = scan::Scan{}.spawn_module(&postgres_config, &modules);
    let clean_thread = clean::Clean{}.spawn_module(&postgres_config, &modules);

    match scan_thread {
        Some(handle) => handle.join().unwrap(),
        None => ()
    };
    match clean_thread {
        Some(handle) => handle.join().unwrap(),
        None => ()
    };
    info!("All modules have been skipped or failed. END OF LINE");
    Ok(())
}
