extern crate renert;
extern crate clap;
extern crate colored;
//extern crate regex;
extern crate serde_json;
//#[macro_use]
//extern crate chan;
//extern crate chan_signal;
//#[macro_use]
//extern crate lazy_static;

use renert::*;
use std::fs::OpenOptions;
use std::io::Read;
//use std::process::Command;
//use std::sync::RwLock;
use std::collections::BTreeMap;
use colored::*;
//use serde_json::Value;
//use std::thread;
//use chan_signal::Signal;
use clap::{App, Arg, SubCommand};

fn help() {
    println!("\
USAGE:
    shandue [SUBCOMMAND]
shandue -h for help\
");
}

fn enqueue(matches: &clap::ArgMatches) -> Result<i32, String> {
    println!("{}", "[+]start.".red().bold());
    let enqueue_subcmd = matches.subcommand_matches("enqueue").unwrap();
    let json_files = enqueue_subcmd.values_of("json_file").unwrap();
    for json_file in json_files {
        rpush(json_file)?;
    }
    println!("{}", "[+]finish.".red().bold());
    return Ok(0);
}

fn rpush(json_file: &str) -> Result<i32, String> {
    let mut f = OpenOptions::new().read(true).open(json_file).map_err(|e| format!("{}: \"{}\"", e, json_file))?;
    let mut json_data: String = String::new();
    f.read_to_string(&mut json_data).map_err(|e| e.to_string())?;
    let json_data: BTreeMap<String, String> = serde_json::from_str(&json_data).map_err(|e| format!("Failed to parse json: {}", e))?;
    let cmd = json_data.get("cmd").ok_or("Key not found: \"cmd\"")?.to_string();
    let cgroup = json_data.get("cgroup");
    let cmd: String = match cgroup {
        Some(x) => ["sh -c \"echo \\$$ | sudo tee /sys/fs/cgroup/", x, "/tasks 1> /dev/null && ", &cmd, "\""].join(""),
        None => cmd
    };
    println!("{}", cmd);
    match system_on_shell(&["redis-cli rpush commands '", &cmd, "'"].join("")) {
        Ok(o) => println!("{}", o.stdout),
        Err(o) => {
            my_eprint(o.stderr);
            std::process::exit(0);
        }
    }
    //match system_on_shell(&cmd) {
    //    Ok(o) => println!("{}", o.stdout),
    //    Err(o) => {
    //        my_eprint(o.stderr);
    //        std::process::exit(0);
    //    }
    //}
    return Ok(0);
}

fn main() {
    let matches = App::new("shandue")
        .version("0.0.1")
        .author("miyagaw61 <miyagaw61@gmail.com>")
        .about("shell command queue")
        .subcommand(SubCommand::with_name("enqueue")
                    .arg(Arg::with_name("json_file")
                         .help("json_file - help message")
                         .takes_value(true)
                         .required(true)
                         .multiple(true)
                         )
                    )
        .get_matches();
    let sub_command = matches.subcommand_name().unwrap_or("");
    match sub_command {
        "enqueue" => {
            match enqueue(&matches) {
                Ok(_) => {},
                Err(e) => {
                    eprintln!("ERROR: {}", e);
                }
            }
        },
        _ => help()
    }
}
