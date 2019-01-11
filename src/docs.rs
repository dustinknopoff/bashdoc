use self::delims::*;
use self::doc::*;
use self::docfile::*;
use self::kv::*;
use self::outputs::*;
use clap::ArgMatches;
use dirs::home_dir;
use glob::glob;
use handlebars::{to_json, Handlebars};
use nom::types::CompleteStr;
use nom::*;
use nom_locate::{position, LocatedSpan};
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    env, fs,
    fs::File,
    path::{Path, PathBuf},
    process::exit,
};

/// "Main" of bashdoc
pub mod runners {
    use super::*;
    use notify::{DebouncedEvent, RecommendedWatcher, RecursiveMode, Watcher};
    use std::{sync::mpsc::channel, time::Duration};

    /// Given the arguments received via CLI from clap, setup and run with requested delimiters, file or directory, etc.
    pub fn generate<'a>(matches: &'a ArgMatches<'a>) {
        let delims = match matches.subcommand() {
            ("override", Some(sub_m)) => Delimiters::override_delims(sub_m),
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
        let path: String = if matches.value_of("INPUT").unwrap().contains('~') {
            home_dir()
                .expect("Could not find home directory.")
                .join(
                    Path::new(matches.value_of("INPUT").unwrap())
                        .strip_prefix("~")
                        .expect("Could not strip shortcut."),
                )
                .to_str()
                .unwrap()
                .to_string()
        } else {
            String::from(matches.value_of("INPUT").unwrap())
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
}

/// Functions and declarations for general Key,Value Pair
mod kv {
    use super::*;
    /// Represents a simple Key, Value pair
    #[derive(Debug, Default, Serialize, Deserialize, Clone)]
    pub struct KV {
        pub key: String,
        pub value: String,
    }

    impl PartialEq for KV {
        fn eq(&self, other: &KV) -> bool {
            self.key == other.key && self.value == other.value
        }
    }

    impl KV {
        #[allow(dead_code)]
        pub fn new(key: String, value: String) -> Self {
            KV { key, value }
        }
    }

    /// Nom function to convert a given string into a `KV`
    ///
    /// # Example
    ///
    /// ```
    /// let example = "# @param filename: don't test me";
    /// as_kv(example) // returns [KV {key: "filename", value: "don't test me"}]
    /// ```
    pub fn as_kv(input: &str) -> Result<KV, nom::ErrorKind> {
        let parts: Vec<_> = if input.contains(':') {
            input.split(": ").collect()
        } else {
            input.split_whitespace().collect()
        };
        let result = KV {
            key: parts[0].trim().to_string(),
            value: parts[1..].join(" ").to_string(),
        };
        Ok(result)
    }
}

/// Functions and declarations for Docs and parsing from strings
mod doc {
    use super::*;
    /// Represents a docstring
    #[derive(Debug, Serialize, Deserialize, Clone, Default)]
    pub struct Doc {
        pub short_description: String,
        pub long_description: String,
        pub descriptors: Vec<KV>,
        pub params: Vec<KV>,
        pub returns: Vec<KV>,
        pub position: u32,
    }

    impl PartialEq for Doc {
        fn eq(&self, other: &Doc) -> bool {
            self.short_description == other.short_description
                && self.long_description == other.long_description
                && self.descriptors == other.descriptors
                && self.params == other.params
                && self.returns == other.returns
        }
    }

    /// Nom function to convert a given string in to a `Doc`
    #[allow(clippy::cyclomatic_complexity)]
    pub fn parse_doc<'a>(input: &'a str, delims: Delimiters) -> IResult<&'a str, Doc> {
        do_parse!(
            input,
            short:
                preceded!(
                    take_until_and_consume!(delims.comm),
                    take_until_and_consume!("\n")
                )
                >> long: opt!(preceded!(
                    take_until_and_consume!(delims.comm),
                    take_until_and_consume!("\n")
                ))
                >> desc: opt!(many0!(complete!(map_res!(
                    preceded!(
                        take_until_and_consume!(delims.opt),
                        take_until_and_consume!("\n")
                    ),
                    as_kv
                ))))
                >> par: opt!(many0!(complete!(map_res!(
                    preceded!(
                        take_until_and_consume!(delims.params),
                        take_until_and_consume!("\n")
                    ),
                    as_kv
                ))))
                >> ret: opt!(many0!(complete!(map_res!(
                    preceded!(
                        take_until_and_consume!(delims.ret),
                        take_until_and_consume!("\n")
                    ),
                    as_kv
                ))))
                >> (Doc {
                    short_description: short.to_string(),
                    long_description: long.unwrap_or("").to_string(),
                    descriptors: desc.unwrap_or_default(),
                    params: par.unwrap_or_default(),
                    returns: ret.unwrap_or_default(),
                    position: 0
                })
        )
    }

    impl Doc {
        /// Build a `Doc` from an array of strings
        /// Parse `Doc` fields.
        pub fn make_doc(vector: &Extracted, delims: Delimiters) -> Result<Doc, nom::ErrorKind> {
            // println!("{:#?}", vector);
            let parsed = parse_doc(&vector.content, delims);
            let mut result = match parsed {
                Ok(e) => e.1,
                Err(_) => Default::default(),
            };
            result.position = vector.position.line + 1;
            Ok(result)
        }
    }
}

/// Functions and declarations for DocFile's and parsing
mod docfile {
    use super::*;
    use rayon::prelude::*;
    use std::io::prelude::*;
    /// Represents all documentation in a file
    #[derive(Debug, Default, Serialize, Deserialize)]
    pub struct DocFile {
        pub thedocs: Vec<Doc>,
        pub filename: String,
    }

    impl DocFile {
        /// Append the given `Doc` to this `AllDoc`
        #[allow(dead_code)]
        pub fn add(&mut self, doc: Doc) {
            self.thedocs.push(doc)
        }
    }

    pub type Span<'a> = LocatedSpan<CompleteStr<'a>>;
    /// Represents the string extracted from a file, including it's location in the file found.
    pub struct Extracted<'a> {
        pub position: Span<'a>,
        pub content: String,
    }

    /// Nom function to extract all docstring from a file.
    pub fn parse_strings_from_file(
        input: Span<'static>,
        delims: Delimiters,
    ) -> IResult<Span<'static>, Vec<Extracted<'static>>> {
        many0!(
            input,
            do_parse!(
                content:
                    complete!(preceded!(
                        take_until_and_consume!(delims.start),
                        take_until_and_consume!(delims.end)
                    ))
                    >> pos: position!()
                    >> (Extracted {
                        position: pos,
                        content: content.to_string()
                    })
            )
        )
    }

    /// Gets all `START_DELIM->END_DELIM` comments in the zshrc
    ///
    /// This goes through every line finding the start of the docstring
    /// and adds every line to a `Vec` until the end delimiter.
    ///
    /// A final `Vec` of the collected comment strings is returned.
    pub fn get_strings_from_file<'a>(
        p: &Path,
        delims: Delimiters,
    ) -> Result<Vec<Extracted<'a>>, String> {
        let mut file = File::open(p).map_err(|e| e.to_string())?;
        let mut contents = String::new();
        file.read_to_string(&mut contents)
            .map_err(|e| e.to_string())?;
        let used = Box::leak(contents.into_boxed_str());
        let x = parse_strings_from_file(Span::new(CompleteStr(used)), delims)
            .map_err(|err| err.to_string())?;
        Ok(x.1)
    }

    /// Given a `Vec<str>` make a `DocFile`
    pub fn generate_doc_file(
        docs: &[Extracted<'static>],
        fname: &Path,
        delims: Delimiters,
    ) -> DocFile {
        let mut all_docs: DocFile = Default::default();
        all_docs.filename = String::from(fname.file_stem().unwrap().to_str().unwrap());
        let collected: Vec<Doc> = docs
            .par_iter()
            .filter(|x| !x.content.is_empty())
            .map(|x| Doc::make_doc(x, delims).unwrap())
            .collect();
        all_docs.thedocs = collected;
        all_docs
    }

    fn extract_all_paths(p: &str) -> Result<Vec<PathBuf>, String> {
        let as_path = Path::new(p);
        let pth = if p.contains('~') {
            home_dir().expect("Could not find home directory.").join(
                as_path
                    .strip_prefix("~")
                    .expect("Could not strip shortcut."),
            )
        } else {
            match as_path.canonicalize() {
                Ok(o) => o,
                Err(e) => {
                    println!("{}", e.to_string());
                    exit(1);
                }
            }
        };
        let files: Vec<_> = if p.contains('*') {
            glob(pth.to_str().unwrap())
                .unwrap()
                .filter_map(|x| x.ok())
                .collect()
        } else {
            vec![pth]
        };
        Ok(files)
    }

    /// Given a file path and delimiters, generate a DocFile for all files requested.
    pub fn start(p: &str, delims: Delimiters) -> Result<Vec<DocFile>, String> {
        let x: Vec<PathBuf> = extract_all_paths(p).map_err(|e| e.to_string())?;
        Ok(x.par_iter()
            .map(|entry| {
                let docs = match get_strings_from_file(&entry, delims) {
                    Ok(o) => o,
                    Err(e) => {
                        println!("{}", e.to_string());
                        exit(1);
                    }
                };
                generate_doc_file(&docs, &entry, delims)
            })
            .collect())
    }
}

/// Functions for presenting bashdocs to STDOUT, as JSON, or HTML
mod outputs {
    use super::*;
    use colored::*;
    use std::io::prelude::*;
    /// Pretty print an `DocFile`
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
    pub fn printer(thedocs: &DocFile, use_color: bool) {
        if use_color {
            println!(
                "{}: {}",
                "Help".green().underline(),
                thedocs.filename.green().underline()
            );
            for doc in &thedocs.thedocs {
                let params: Vec<&str> = doc.params.iter().map(|x| x.key.as_str()).collect();
                let as_string = params.join(", ");
                print!("{}", doc.short_description.replace("()", "").blue().bold());
                if doc.params.is_empty() {
                    println!(": {}", doc.long_description);
                } else {
                    println!(" - {}: {}", as_string.cyan(), doc.long_description);
                }
                if !doc.descriptors.is_empty() {
                    doc.descriptors
                        .iter()
                        .for_each(|x| println!("\t{} {}", &x.key.yellow().bold(), x.value));
                }
            }
        } else {
            println!("Help: {}", thedocs.filename);
            for doc in &thedocs.thedocs {
                let params: Vec<&str> = doc.params.iter().map(|x| x.key.as_str()).collect();
                let as_string = params.join(", ");
                print!("{}", doc.short_description.replace("()", ""));
                if doc.params.is_empty() {
                    println!(": {}", doc.long_description);
                } else {
                    println!(" - {}: {}", as_string, doc.long_description);
                }
                if !doc.descriptors.is_empty() {
                    doc.descriptors
                        .iter()
                        .for_each(|x| println!("\t{} {}", &x.key, x.value));
                }
            }
        }
    }

    /// Given a list of `DocFile` and a file path, write the JSON representation to a file.
    pub fn write_json(docstrings: &[DocFile], file_name: &str) {
        let mut map = HashMap::new();
        map.insert("docs", docstrings);
        let json = serde_json::to_string_pretty(&map).expect("Could not convert to JSON");
        let path_as_str = if cfg!(windows) {
            String::from(file_name)
        } else {
            file_name.replace("~", home_dir().unwrap().to_str().unwrap())
        };
        let path = Path::new(&path_as_str);
        let mut file = File::create(Path::new(&path)).expect("Invalid file path.");
        file.write_all(&json.as_bytes())
            .expect("Could not write to file.");
    }

    pub fn to_html(docstrings: &[DocFile], dir: Option<&str>, template_loc: Option<&str>) {
        for dfile in docstrings {
            let json = to_json(dfile);
            let handlebars = Handlebars::new();
            let mut template = match template_loc {
                Some(m) => match File::open(m) {
                    Ok(o) => o,
                    Err(_) => {
                        println!("Provided path is invalid");
                        exit(1);
                    }
                },
                None => File::open("./static/template.hbs").unwrap(),
            };
            // let mut template = File::open("./static/template.hbs").unwrap();
            let mut output = match dir {
                Some(d) if Path::new(d).is_dir() => {
                    File::create(format!("{}/{}.html", d, dfile.filename).as_str())
                        .expect("File could not be created")
                }
                None | Some(_) => {
                    println!("Provided path is invalid");
                    exit(1);
                }
            };
            // let mut output = if dir.len() == 1 {
            //     File::create(format!("{}.html", dfile.filename).as_str())
            //         .expect("File cannot be created")
            // } else {
            //     File::create(format!("{}/{}.html", dir, dfile.filename).as_str())
            //         .expect("File cannot be created")
            // };
            handlebars
                .render_template_source_to_write(&mut template, &json, &mut output)
                .expect("Could not generate documentation");
        }
    }
}

/// Functions and declarations for generating/overriding delimiters
mod delims {
    use super::*;
    use std::io::prelude::*;
    /// Represents the necessary delimiters for a `bashdoc`
    #[derive(Debug, Serialize, Deserialize, Copy, Clone)]
    pub struct Delimiters<'a> {
        pub start: &'a str,
        pub end: &'a str,
        pub params: &'a str,
        pub ret: &'a str,
        pub opt: &'a str,
        pub comm: &'a str,
    }

    impl<'a> Default for Delimiters<'a> {
        fn default() -> Delimiters<'a> {
            Delimiters {
                start: "#;",
                end: "#\"",
                params: "@param",
                ret: "@return",
                opt: "# -",
                comm: "# ",
            }
        }
    }
    impl<'a> Delimiters<'a> {
        /// Override default delimiters with passed in values
        pub fn override_delims(overrides: &'a ArgMatches<'a>) -> Self {
            let mut result: Delimiters = Delimiters::default();
            for key in overrides.args.keys() {
                match key.as_ref() {
                    "start" => result.start = overrides.value_of(key).unwrap(),
                    "end" => result.end = overrides.value_of(key).unwrap(),
                    "descriptor" => result.opt = overrides.value_of(key).unwrap(),
                    "params" => result.params = overrides.value_of(key).unwrap(),
                    "returns" => result.ret = overrides.value_of(key).unwrap(),
                    "comment" => result.comm = overrides.value_of(key).unwrap(),
                    _ => {}
                }
            }
            result
        }

        /// Read/Write contents of `$BASHDOC_CONFIG_PATH` for use as Delimiters.
        pub fn get_delims() -> Self {
            let mut contents = String::new();
            if env::current_dir().unwrap().join(".bashdocrc").is_file() {
                let mut config =
                    File::open(Path::new(&env::current_dir().unwrap().join(".bashdocrc")))
                        .expect("Invalid path");
                config
                    .read_to_string(&mut contents)
                    .expect("could not read from file.");
                let mut to_convert = String::new();
                to_convert.push_str(&contents);
                let as_static: &'static str = Box::leak(to_convert.into_boxed_str());
                let sorted: Delimiters = toml::from_str(&as_static).unwrap();
                sorted
            } else {
                match env::var_os("BASHDOC_CONFIG_PATH") {
                    Some(val) => {
                        let mut config = File::open(Path::new(&val)).expect("Invalid path");
                        config
                            .read_to_string(&mut contents)
                            .expect("could not read from file.");
                        let mut to_convert = String::new();
                        to_convert.push_str(&contents);
                        let as_static: &'static str = Box::leak(to_convert.into_boxed_str());
                        let sorted: Delimiters = toml::from_str(&as_static).unwrap();
                        sorted
                    }
                    None => {
                        let delimiters = Delimiters::default();
                        let content = toml::to_string_pretty(&delimiters)
                            .expect("Could not be converted to TOML");
                        let mut path = home_dir().unwrap();
                        path.push(".bashdocrc");
                        fs::write(path.to_str().unwrap(), content).unwrap();
                        env::set_var("BASHDOC_CONFIG_PATH", path);
                        delimiters
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    mod kv_tests {
        use super::*;
        #[test]
        fn new_kv() {
            let kv = KV::new(String::from("a"), String::from("b"));
            assert_eq!(String::from("a"), kv.key);
            assert_eq!(String::from("b"), kv.value);
        }

        #[test]
        fn cmp_kv() {
            let kv1 = KV::new(String::from("a"), String::from("b"));
            let kv2 = KV::new(String::from("a"), String::from("b"));
            let kv = KV::new(String::from("b"), String::from("a"));
            assert_eq!(kv1, kv2);
            assert_ne!(kv1, kv);
        }

        #[test]
        fn is_as_kv() {
            let conv = as_kv("type: mp4 or gif");
            assert_eq!(
                KV {
                    key: String::from("type"),
                    value: String::from("mp4 or gif")
                },
                conv.unwrap()
            );
        }

        #[test]
        fn is_as_kv_white() {
            let conv = as_kv("CTRL-O to open with `open` command,");
            assert_eq!(
                KV {
                    key: String::from("CTRL-O"),
                    value: String::from("to open with `open` command,")
                },
                conv.unwrap()
            );
        }
    }

    mod docfile_tests {
        use super::*;
        #[test]
        fn test_add() {
            let mut dfile = DocFile {
                thedocs: Vec::new(),
                filename: String::from("zshrc"),
            };
            dfile.add(Doc {
                short_description: String::from("lala"),
                long_description: String::from("rawr"),
                descriptors: Vec::new(),
                params: Vec::new(),
                returns: Vec::new(),
                position: 0,
            });
            assert_eq!(
                dfile.thedocs,
                [Doc {
                    short_description: String::from("lala"),
                    long_description: String::from("rawr"),
                    descriptors: Vec::new(),
                    params: Vec::new(),
                    returns: Vec::new(),
                    position: 0,
                }]
            );
        }
    }
}
