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
extern crate colored;
extern crate dirs;
extern crate glob;
extern crate rayon;
#[macro_use]
extern crate clap;
use clap::App;

use colored::*;
use glob::glob;
use rayon::prelude::*;
use std::collections::HashMap;
use std::fs::File;
use std::io::prelude::*;
use std::io::BufReader;
use std::path::Path;

const START_DELIM: &str = "#;";
const END_DELIM: &str = "#\"";
const PAR_DELIM: &str = "@param";
const RET_DELIM: &str = "@return";
const OPT_DELIM: &str = "# -";
const COMM_DELIM: &str = "# ";

/// Represents a docstring
/// contains:
///
/// - short description (name of function)
/// - long description
/// - `HashMap` of options to their descriptions
/// - `HashMap` of parameters to their descriptions
/// - `HashMap` of return values to their descriptions
#[derive(Debug, Default)]
pub struct Doc {
    short_description: String,
    long_description: String,
    descriptors: HashMap<String, String>,
    params: HashMap<String, String>,
    returns: HashMap<String, String>,
}

impl Doc {
    /// # Build a `Doc` from an array of strings
    /// Parse `Doc` fields.
    pub fn make_doc(vector: &[String]) -> Doc {
        let mut result: Doc = Default::default();
        for line in vector.iter() {
            if line == &vector[0] {
                result.short_description.push_str(line);
            } else if line.contains(PAR_DELIM) {
                let splitted: Vec<_> = line.split_whitespace().map(|x| x.to_string()).collect();
                let rest: String = splitted[2..].join(" ");
                result.params.insert(splitted[1].replace(":", ""), rest);
            } else if line.contains(RET_DELIM) {
                let splitted: Vec<_> = line.split_whitespace().map(|x| x.to_string()).collect();
                let rest: String = splitted[2..].join(" ");
                result.returns.insert(splitted[1].replace(":", ""), rest);
            } else if line.contains(OPT_DELIM) {
                let splitted: Vec<_> = line.split_whitespace().map(|x| x.to_string()).collect();
                let rest: String = splitted[3..].join(" ");
                result
                    .descriptors
                    .insert(splitted[2].replace(":", ""), rest);
            } else {
                result.long_description.push_str(line);
            }
        }
        result
    }
}

/// # Represents all documentation in a file
#[derive(Debug, Default)]
pub struct AllDocs {
    thedocs: Vec<Doc>,
    filename: String,
}

impl AllDocs {
    /// Append the given `Doc` to this `AllDoc`
    pub fn add(&mut self, doc: Doc) -> () {
        self.thedocs.push(doc);
        ()
    }

    /// # Pretty print this `AllDocs`
    ///
    /// Given an `AllDoc`:
    /// ```
    ///[
    ///    Doc {
    ///        short_description: "runner()",
    ///        long_description: "This is the beginning",
    ///        descriptors: {
    ///            "CTRL-O": "pushs the boundaries"
    ///        },
    ///        params: {},
    ///        returns: {}
    ///    },
    ///    Doc {
    ///        short_description: "runner()",
    ///        long_description: "This is the beginning",
    ///        descriptors: {},
    ///        params: {
    ///            "location": "where to put it",
    ///            "filename": "don\'t test me"
    ///        },
    ///        returns: {
    ///            "nothing": ""
    ///        }
    ///    }
    ///]
    /// ```
    /// The following will be printed to the `STDOUT` with color
    /// ```
    /// Help
    /// runner: This is the beginning
    ///     CTRL-O pushs the boundaries
    /// runner - location, filename: This is the beginning
    /// ```
    pub fn colorize(&self) -> () {
        for doc in &self.thedocs {
            let mut params: Vec<_> = doc.params.keys().map(|x| x.to_string()).collect();
            let as_string = params.join(", ");
            print!("{}", doc.short_description.replace("()", "").blue().bold());
            if doc.params.is_empty() {
                println!(": {}", doc.long_description);
            } else {
                println!(" - {}: {}", as_string.cyan(), doc.long_description);
            }
            if !doc.descriptors.is_empty() {
                for sub in doc.descriptors.keys() {
                    println!("\t{} {}", sub.yellow().bold(), &doc.descriptors[sub])
                }
            }
        }
    }

    pub fn printer(&self) -> () {
        for doc in &self.thedocs {
            let mut params: Vec<_> = doc.params.keys().map(|x| x.to_string()).collect();
            let as_string = params.join(", ");
            print!("{}", doc.short_description.replace("()", ""));
            if doc.params.is_empty() {
                println!(": {}", doc.long_description);
            } else {
                println!(" - {}: {}", as_string, doc.long_description);
            }
            if !doc.descriptors.is_empty() {
                for sub in doc.descriptors.keys() {
                    println!("\t{} {}", sub, &doc.descriptors[sub])
                }
            }
        }
    }
}

/// Gets all `START_DELIM->END_DELIM` comments in the zshrc
///
/// This goes through every line finding the start of the docstring
/// and adds every line to a `Vec` until the end delimiter.
///
/// A final `Vec` of the collected comment strings is returned.
pub fn get_info(p: &Path) -> Vec<Vec<String>> {
    // let mut p = dirs::home_dir().unwrap();
    // p.push(".zshrc");
    let f = File::open(&p).expect("file not found.");
    let f = BufReader::new(f);
    let mut result: Vec<Vec<String>> = Vec::new();
    result.push(Vec::new());
    let mut can_add = false;
    let mut index = 0;
    for line in f.lines() {
        let curr_line = line.expect("Line cannot be accessed.");
        if curr_line.contains(START_DELIM) {
            can_add = true;
            continue;
        } else if curr_line.contains(END_DELIM) {
            can_add = false;
            index += 1;
            result.push(Vec::new());
        }
        if can_add {
            if curr_line.contains(OPT_DELIM) {
                result[index].push(curr_line);
            } else {
                result[index].push(curr_line.replace(COMM_DELIM, ""));
            }
        }
    }
    result
}

fn main() {
    let yaml = load_yaml!("../cli.yml");
    let matches = App::from_yaml(yaml).get_matches();
    if matches.is_present("directory") {
        let dir = matches
            .value_of("directory")
            .unwrap()
            .replace("~", dirs::home_dir().unwrap().to_str().unwrap());
        let files: Vec<_> = glob(&dir).unwrap().filter_map(|x| x.ok()).collect();
        let every_doc: Vec<AllDocs> = files
            .par_iter()
            .map(|entry| {
                let docs = get_info(&entry);
                generate(
                    &docs,
                    entry
                        .file_name()
                        .unwrap()
                        .clone()
                        .to_str()
                        .unwrap()
                        .to_string(),
                )
            }).collect();
        for doc in every_doc {
            if matches.is_present("color") {
                println!("Help: {}", doc.filename);
                doc.printer();
            } else {
                println!(
                    "{}: {}",
                    "Help".green().underline(),
                    doc.filename.green().underline()
                );
                doc.colorize();
            }
        }
    } else {
        let dir = matches
            .value_of("INPUT")
            .expect("Enter a valid file")
            .replace("~", dirs::home_dir().unwrap().to_str().unwrap());
        let docs = get_info(&Path::new(&dir));
        let all_docs = generate(
            &docs,
            Path::new(&dir)
                .file_name()
                .unwrap()
                .clone()
                .to_str()
                .unwrap()
                .to_string(),
        );
        if matches.is_present("color") {
            println!("Help");
            all_docs.printer();
        } else {
            println!("{}", "Help".green().underline());
            all_docs.colorize();
        }
    }
    // println!("{:#?}", docs);
}

fn generate(docs: &Vec<Vec<String>>, fname: String) -> AllDocs {
    let mut all_docs: AllDocs = Default::default();
    all_docs.filename = fname;
    for doc in docs.iter() {
        if doc.to_vec().is_empty() {
            continue;
        }
        let as_bash_doc = Doc::make_doc(&doc.to_vec());
        all_docs.add(as_bash_doc);
    }
    return all_docs;
}
