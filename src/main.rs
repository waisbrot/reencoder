extern crate chrono;
extern crate crypto;
extern crate postgres;
#[macro_use]
extern crate log;
#[macro_use]
extern crate lazy_static;
extern crate clap;
extern crate crossbeam_utils;
extern crate pretty_env_logger;
extern crate regex;
extern crate serde_json;
extern crate subprocess;

mod clean;
mod module;
mod reencode;
mod scan;

use clap::{App, Arg};
use module::Module;
use postgres::params::Builder;
use postgres::params::Host::Tcp;
use std::io;

fn main() -> io::Result<()> {
    pretty_env_logger::init();
    info!("Starting main thread");

    let args = App::new("Video converter")
        .version("0.1")
        .author("Nathaniel Waisbrot")
        .about("Find and convert video to hvec")
        .arg(
            Arg::with_name("username")
                .help("Postgres username")
                .long("username")
                .takes_value(true)
                .required(true),
        )
        .arg(
            Arg::with_name("password")
                .help("Postgres password")
                .long("password")
                .takes_value(true)
                .required(true),
        )
        .arg(
            Arg::with_name("host")
                .help("Postgres hostname")
                .long("host")
                .takes_value(true)
                .required(true),
        )
        .arg(
            Arg::with_name("modules")
                .help("Modules to activate")
                .long("modules")
                .required(false)
                .takes_value(true)
                .multiple(true)
                .require_delimiter(true)
                .default_value("scan,clean,reencode"),
        )
        .arg(
            Arg::with_name("loop")
                .help("Continue to run forever?")
                .long("loop")
                .required(false)
                .takes_value(false),
        )
        .get_matches();

    // Postgres setup
    let postgres_config = Builder::new()
        .user(
            args.value_of("username").unwrap(),
            args.value_of("password"),
        )
        .build(Tcp(args.value_of("host").unwrap().to_string()));

    // Modules
    let modules = args.values_of("modules").unwrap();

    fn modules_contains(modules: &clap::Values, target: &str) -> bool {
        modules.clone().filter(|&x| x == target).next().is_some()
    }

    let do_loop = args.is_present("loop");
    let mut all_modules: Vec<&dyn Module> = Vec::new();
    all_modules.push(&scan::Scan {});
    all_modules.push(&clean::Clean {});
    all_modules.push(&reencode::Reencode {});
    debug!("Starting threads for {:?}", &modules);
    crossbeam_utils::thread::scope(|scope| {
        for m in all_modules.iter() {
            let name = m.module_name();
            debug!("Checking if we should start a thread for {}", &name);
            if modules_contains(&modules, name) {
                debug!("Connecting to postgres");
                let connection =
                    postgres::Connection::connect(postgres_config.clone(), postgres::TlsMode::None)
                        .unwrap();
                info!("Starting thread {}", &name);
                scope
                    .builder()
                    .name(name.to_string())
                    .spawn(move |_| m.module_loop(connection, do_loop))
                    .unwrap();
            }
        }
        info!("All threads started")
    })
    .unwrap();

    info!("All modules have been skipped or failed. END OF LINE");
    Ok(())
}
