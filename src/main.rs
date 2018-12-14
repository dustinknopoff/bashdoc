//! # BashDoc
//!
//! A tool for generating documentation/help menu for user defined bash functions.
//!
//! ## Syntax
//!
//! ### Example
//!
//! ```bash
//! #;
//! # cd()
//! # moves to given directory
//! # @param directory: folder to move to
//! # @return void
//! #"
//! cd() {
//!     cd $1
//! }
//! ```
//!
//! Outputs
//!
//! ```
//! Help
//! cd - directory: moves to given directory
//! ```
//!
//! with lots of color!
//!
//! ### Global Delimiters
//!
//! `START_DELIM = #;`
//!
//! `END_DELIM = #"`
//!
//! `PAR_DELIM = @param`
//!
//! `RET_DELIM = @return`
//!
//! `OPT_DELIM = # -`
//!
//! `COMM_DELIM = # `
//!
//! These can be modifed in the code to your preference.
//!
mod docs;
use crate::docs::*;
use clap::{load_yaml, App, ArgMatches};
use dirs::home_dir;
use notify::{DebouncedEvent, RecommendedWatcher, RecursiveMode, Watcher};
use std::sync::mpsc::channel;
use std::time::Duration;

fn main() {
    let yaml = load_yaml!("../cli.yml");
    let matches = App::from_yaml(yaml).get_matches();
    if matches.is_present("watch") {
        watcher(&matches);
    } else {
        generate(&matches);
    }
}

fn generate<'a>(matches: &'a ArgMatches<'a>) {
    let delims = match matches.subcommand() {
        ("override", Some(sub_m)) => Delimiters::override_delims(sub_m),
        _ => Delimiters::get_delims(),
    };
    let all_em = if matches.is_present("directory") {
        start(
            matches.value_of("INPUT").expect("directory glob not found"),
            true,
            delims,
        )
    } else {
        start(
            matches.value_of("INPUT").expect("no file found."),
            false,
            delims,
        )
    };
    if matches.is_present("json") {
        write_json(&all_em, matches.value_of("json").unwrap());
    } else if matches.is_present("html") {
        to_html(&all_em, matches.value_of("html").unwrap());
    } else {
        for doc in &all_em {
            if matches.is_present("color") {
                colorize(doc);
            } else {
                printer(doc);
            }
        }
    }
}

fn watcher<'a>(matches: &'a ArgMatches<'a>) {
    generate(matches);
    let (tx, rx) = channel();
    let mut watcher: RecommendedWatcher = Watcher::new(tx, Duration::from_secs(2)).unwrap();
    let path: String = if cfg!(windows) {
        String::from(matches.value_of("INPUT").unwrap())
    } else {
        matches
            .value_of("INPUT")
            .unwrap()
            .replace("~", home_dir().unwrap().to_str().unwrap())
    };
    watcher.watch(&path, RecursiveMode::Recursive).unwrap();
    println!("Watching for changes in {}...", path);
    loop {
        match rx.recv() {
            Ok(event) => {
                generate(&matches);
                match event {
                    DebouncedEvent::Write(e) => println!(
                        "Bashdoc updated to match changes to {}.",
                        e.as_path().file_name().unwrap().to_str().unwrap()
                    ),
                    _ => (),
                }
            }
            Err(e) => println!("watch error: {:?}", e),
        }
    }
}
