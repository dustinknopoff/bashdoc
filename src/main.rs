//!# BashDoc
//!
//!A tool for generating documentation/help menu for ~~user defined bash functions~~ any folder or file with 6 generic delimiters defined.
//!
//!## Syntax
//!
//!### Example
//!
//! Using syntax similar to below
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
//!on my `zshrc` Outputs
//!
//!![](https://github.com/dustinknopoff/bashdoc/raw/master/example/zshrc.png)
//!
//!with lots of color!
//!
//!### Global Delimiters
//!
//! The default delimiters to use are as follows:
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
//!bashdoc 0.4.10
//!Creates a "javadoc" like structure for bash. See github repo github.com/dustinknopoff/bashdoc for information on formatting.
//!
//!USAGE:
//!bashdoc [FLAGS] [OPTIONS] <INPUT> [SUBCOMMAND]
//!
//!FLAGS:
//!-c, --color      toggles color
//!-h, --help       Prints help information
//!-V, --version    Prints version information
//!-w, --watch      continuously update on change
//!
//!OPTIONS:
//!-j, --json <FILE>            print result as JSON
//!-l, --location <location>    location to save HTML
//!-t, --template <template>    .hbs template to use for generation of documentation
//!
//!ARGS:
//!<INPUT>    Sets the input file or glob pattern to use
//!
//!SUBCOMMANDS:
//!help        Prints this message or the help of the given subcommand(s)
//!override    override the delimiters
//!```
//!
//! See the [examples](https://github.com/dustinknopoff/bashdoc/tree/master/example) folder for more.
//!
//! See the [changelog](https://github.com/dustinknopoff/bashdoc/blob/master/CHANGELOG.md) for updates
//!
//! # Changelog
//!
//!- v0.4.0 - Added to crates.io
//!- v0.4.1/v0.4.2 - Better descriptions for crates.io
//!- v.0.4.5 - Fix error where bashdoc would not function for users without a `~/.bashdocrc`
//!- v.0.4.6 - Improved Error handling, `--html` argument removed replaced with `--location`, `--template` argument added for supplying custom `.hbs`
//!- v0.4.7 - Fix required location for all inputs and not exclusive to `--location`
//!- v0.4.8 - Clearer README, link to docs.rs documentation
//!- v0.4.9 - Improved error path handling
//!- v0.4.10 - Support for windows file paths again
//! - v0.4.11 - support for overriding global `.bashdocrc` within a directory.
//! - v0.4.12 - descriptors can be split on ':' or whitespace
mod docs;
use crate::docs::delims::override_delims;
use clap::{load_yaml, App, ArgMatches};
use notify::{DebouncedEvent, RecommendedWatcher, RecursiveMode, Watcher};
use std::{sync::mpsc::channel, time::Duration};

fn main() {
    let yaml = load_yaml!("../cli.yml");
    let matches = App::from_yaml(yaml).get_matches();
    if matches.is_present("watch") {
        watcher(&matches);
    } else {
        generate(&matches);
    }
}

/// Given the arguments received via CLI from clap, setup and run with requested delimiters, file or directory, etc.
pub fn generate<'a>(matches: &'a ArgMatches<'a>) {
    let delims = match matches.subcommand() {
        ("override", Some(sub_m)) => override_delims(sub_m),
        _ => Delimiters::get_delims(),
    };
    let all_em = start(
        matches.value_of("INPUT").expect("directory glob not found"),
        delims,
    )
    .unwrap();
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

/// Given a request to watch files, Call `generate` on file write.
pub fn watcher<'a>(matches: &'a ArgMatches<'a>) {
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
