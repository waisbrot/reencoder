extern crate chrono;
extern crate crypto;
extern crate postgres;
#[macro_use]
extern crate log;
#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate prometheus;
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

use clap::{parser::ValuesRef, Arg, ArgAction, Command};
use module::Module;
use postgres::Client;
use std::{io, string};

fn main() -> io::Result<()> {
    pretty_env_logger::init();
    info!("Starting main thread");

    let args = Command::new("Video converter")
        .version("0.1")
        .author("Nathaniel Waisbrot")
        .about("Find and convert video to hvec")
        .arg(
            Arg::new("username")
                .help("Postgres username")
                .long("username")
                .required(true),
        )
        .arg(
            Arg::new("password")
                .help("Postgres password")
                .long("password")
                .required(true),
        )
        .arg(
            Arg::new("host")
                .help("Postgres hostname")
                .long("host")
                .required(true),
        )
        .arg(
            Arg::new("modules")
                .help("Modules to activate")
                .long("modules")
                .required(false)
                .value_delimiter(',')
                .default_value("scan,clean,reencode"),
        )
        .arg(
            Arg::new("loop")
                .help("Continue to run forever?")
                .long("loop")
                .action(ArgAction::SetTrue)
                .required(false),
        )
        .get_matches();

    // Postgres setup
    let mut postgres_config = Client::configure();
    postgres_config
        .user(
            args.get_one::<String>("username")
                .expect("missing username"),
        )
        .password(
            args.get_one::<String>("password")
                .expect("missing password"),
        )
        .host(args.get_one::<String>("host").expect("missing hostname"));

    // Modules
    let modules = args
        .get_many::<String>("modules")
        .expect("missing modules to run");

    fn modules_contains(modules: &ValuesRef<String>, target: &str) -> bool {
        modules.clone().filter(|&x| x == target).next().is_some()
    }

    let do_loop = args.get_flag("loop");
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
                let mut connection = postgres_config.connect(postgres::NoTls).unwrap();
                info!("Starting thread {}", &name);
                scope
                    .builder()
                    .name(name.to_string())
                    .spawn(move |_| m.module_loop(&mut connection, do_loop))
                    .unwrap();
            }
        }
        info!("All threads started")
    })
    .unwrap();

    info!("All modules have been skipped or failed. END OF LINE");
    Ok(())
}
