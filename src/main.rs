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
//!![](./example/zshrc.png)
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
//!**NOTE: Must use Rust 2018 Edition**
//!
//!_update with `rustup update stable`_
//!
//!```bash
//!git clone https://github.com/dustinknopoff/bashdoc
//!cd bashdoc
//!cargo install
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
