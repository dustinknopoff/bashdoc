extern crate colored;
extern crate dirs;
extern crate glob;
extern crate rayon;
#[macro_use]
extern crate clap;
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
    for doc in &all_em {
        if matches.is_present("color") {
            colorize(doc);
        } else {
            printer(doc);
        }
    }
}
