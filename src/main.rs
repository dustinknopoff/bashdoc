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
use clap::{load_yaml, App};

fn main() {
    let yaml = load_yaml!("../cli.yml");
    let matches = App::from_yaml(yaml).get_matches();
    let delims = if matches.is_present("override") {
        Delimiters::override_delims(matches.value_of("override").unwrap().to_string())
    } else {
        Delimiters::get_delims()
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
