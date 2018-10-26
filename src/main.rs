extern crate clap;
extern crate colored;
extern crate dirs;
extern crate glob;
extern crate rayon;
extern crate serde;
extern crate serde_derive;
extern crate serde_json;
use clap::load_yaml;
mod doc_structure;
use clap::App;
use doc_structure::docs::*;

fn main() {
    let yaml = load_yaml!("../cli.yml");
    let matches = App::from_yaml(yaml).get_matches();
    let all_em = if matches.is_present("directory") {
        start(
            matches.value_of("INPUT").expect("directory glob not found"),
            true,
        )
    } else {
        start(matches.value_of("INPUT").expect("no file found."), false)
    };
    if matches.is_present("json") {
        export_json(&all_em, matches.value_of("json").unwrap());
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
