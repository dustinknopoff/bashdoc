//!# BashDoc
//!
//!A tool for generating documentation/help menu for user defined bash functions.
//!
//!## Syntax
//!
//!### Example
//!
//!```bash
//!#;
//!# cd()
//!# moves to given directory
//!# @param directory: folder to move to
//!# @return void
//!#"
//!cd() {
//!    cd $1
//!}
//!```
//!
//!Outputs
//!
//!![](https://github.com/dustinknopoff/bashdoc/raw/master/example/zshrc.png)
//!
//!with lots of color!
//!
//!### Global Delimiters
//!
//!`START_DELIM = #;`
//!
//!`END_DELIM = #"`
//!
//!`PAR_DELIM = @param`
//!
//!`RET_DELIM = @return`
//!
//!`OPT_DELIM = # -`
//!
//!`COMM_DELIM = #`
//!
//!These can be modifed in your `.bashdocrc`.
//!
//!## Install
//!
//! ```bash
//! cargo install bashdoc
//! ```
//!
//! or from source
//!
//!**NOTE: Must use Rust 2018 Edition**
//!
//!_update with `rustup update stable`_
//!
//!```bash
//!git clone https://github.com/dustinknopoff/bashdoc
//!cd bashdoc
//!cargo install --path . --force
//!```
//!
//!## Usage
//!
//!```bash
//!bashdoc 1.0
//!Creates a "javadoc" like structure for bash. See github repo github.com/dustinknopoff/bashdoc for information on
//!formatting.
//!
//!USAGE:
//!    bashdoc [FLAGS] [OPTIONS] <INPUT> [SUBCOMMAND]
//!
//!FLAGS:
//!    -c, --color        toggles color
//!    -d, --directory    pass a glob pattern to run on.
//!        --help         Prints help information
//!    -V, --version      Prints version information
//!    -w, --watch        continuously update on change
//!
//!OPTIONS:
//!    -h, --html <html>    output html documentation
//!    -j, --json <FILE>    print result as JSON
//!
//!ARGS:
//!    <INPUT>    Sets the input file to use
//!
//!SUBCOMMANDS:
//!    help        Prints this message or the help of the given subcommand(s)
//!    override    override the delimiters
//!```
//!
//! See the [examples](https://github.com/dustinknopoff/bashdoc/tree/master/example) folder for more.
//!
//! See the [changelog](https://github.com/dustinknopoff/bashdoc/blob/master/CHANGELOG.md) for updates
mod docs;
use crate::docs::*;
use clap::{load_yaml, App, ArgMatches};
use dirs::home_dir;
use notify::{DebouncedEvent, RecommendedWatcher, RecursiveMode, Watcher};
use std::{process::exit, sync::mpsc::channel, time::Duration};

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
    } else if matches.is_present("location") {
        to_html(
            &all_em,
            matches.value_of("location"),
            matches.value_of("template"),
        );
    } else {
        for doc in &all_em {
            if matches.is_present("color") {
                printer(doc, true);
            } else {
                printer(doc, false);
            }
        }
    }
}

fn watcher<'a>(matches: &'a ArgMatches<'a>) {
    generate(matches);
    let (tx, rx) = channel();
    let mut watcher: RecommendedWatcher = match Watcher::new(tx, Duration::from_secs(2)) {
        Ok(d) => d,
        Err(_) => {
            println!("Provided path is invalid");
            exit(1);
        }
    };
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
                if let DebouncedEvent::Write(e) = event {
                    println!(
                        "Bashdoc updated to match changes to {}.",
                        e.as_path().file_name().unwrap().to_str().unwrap()
                    );
                }
            }
            Err(e) => println!("watch error: {:?}", e),
        }
    }
}
