use colored::*;
use dirs::home_dir;
use glob::glob;
use nom::*;
use rayon::prelude::*;
use serde_derive::*;
use std::env;
use std::fs;
use std::fs::File;
use std::io::prelude::*;
use std::path::Path;

/// Represents a simple Key, Value pair
#[derive(Debug, Default, Serialize, Deserialize, Clone)]
struct KV {
    key: String,
    value: String,
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

/// Represents a docstring
#[derive(Debug, Default, Serialize, Deserialize, Clone)]
pub struct Doc {
    short_description: String,
    long_description: String,
    descriptors: Vec<KV>,
    params: Vec<KV>,
    returns: Vec<KV>,
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

/// Nom function to convert a given string into a `KV`
///
/// # Example
///
/// ```
/// let example = "# @param filename: don't test me";
/// as_kv(example) // returns [KV {key: "filename", value: "don't test me"}]
/// ```
fn as_kv(input: &str) -> Result<KV, nom::ErrorKind> {
    let parts: Vec<_> = input.split(": ").collect();
    let result = KV {
        key: parts[0].trim().to_string(),
        value: parts[1..].join("").to_string(),
    };
    Ok(result)
}

/// Nom function to convert a given string in to a `Doc`
fn parse_doc<'a>(input: &'a str, delims: Delimiters) -> IResult<&'a str, Doc> {
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
                descriptors: desc.unwrap_or(Vec::new()),
                params: par.unwrap_or(Vec::new()),
                returns: ret.unwrap_or(Vec::new()),
            })
    )
}

impl Doc {
    /// Build a `Doc` from an array of strings
    /// Parse `Doc` fields.
    pub fn make_doc(vector: String, delims: Delimiters) -> Doc {
        // println!("{:#?}", vector);
        let result = parse_doc(&vector, delims);
        result.expect("Parsing error.").1
    }
}

/// Represents all documentation in a file
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct DocFile {
    thedocs: Vec<Doc>,
    filename: String,
}

impl DocFile {
    /// Append the given `Doc` to this `AllDoc`
    #[allow(dead_code)]
    pub fn add(&mut self, doc: Doc) {
        self.thedocs.push(doc)
    }
}

/// Nom function to extract all docstring from a file.
fn getinfo(input: &'static str, delims: Delimiters) -> IResult<&'static str, Vec<&'static str>> {
    many0!(
        input,
        complete!(preceded!(
            take_until_and_consume!(delims.start),
            take_until_and_consume!(delims.end)
        ))
    )
}

/// Gets all `START_DELIM->END_DELIM` comments in the zshrc
///
/// This goes through every line finding the start of the docstring
/// and adds every line to a `Vec` until the end delimiter.
///
/// A final `Vec` of the collected comment strings is returned.
fn get_info<'a>(p: &Path, delims: Delimiters) -> Vec<&'a str> {
    let mut f = File::open(&p).expect("file not found.");
    let mut buffer = String::new();
    f.read_to_string(&mut buffer).unwrap();
    let used = Box::leak(buffer.into_boxed_str());
    // println!("{:#?}", used);
    let result = getinfo(used, delims);
    // println!("{:#?}", result);
    result.unwrap().1
}

/// Given a `Vec<str>` make a `DocFile`
fn generate_doc_file(docs: &[&str], fname: String, delims: Delimiters) -> DocFile {
    let mut all_docs: DocFile = Default::default();
    all_docs.filename = fname;
    let collected: Vec<Doc> = docs
        .par_iter()
        .filter(|x| !x.is_empty())
        .map(|x| Doc::make_doc(x.to_string(), delims))
        .collect();
    all_docs.thedocs = collected;
    all_docs
}

/// Given a file path and delimiters, generate a DocFile for all files requested.
pub fn start(p: &str, is_directory: bool, delims: Delimiters) -> Vec<DocFile> {
    let dir = if cfg!(windows) {
        String::from(p)
    } else {
        p.replace("~", home_dir().unwrap().to_str().unwrap())
    };
    if is_directory {
        let files: Vec<_> = glob(&dir).unwrap().filter_map(|x| x.ok()).collect();
        let every_doc: Vec<DocFile> = files
            .par_iter()
            .map(|entry| {
                let docs = get_info(&entry, delims);
                generate_doc_file(
                    &docs,
                    entry.file_name().unwrap().to_str().unwrap().to_string(),
                    delims,
                )
            })
            .collect();
        every_doc
    } else {
        let docs = get_info(&Path::new(&p), delims);
        let all_docs = generate_doc_file(
            &docs,
            Path::new(&dir)
                .file_name()
                .unwrap()
                .to_str()
                .unwrap()
                .to_string(),
            delims,
        );
        let result = vec![all_docs];
        result
    }
}

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
pub fn colorize(thedocs: &DocFile) {
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
}

/// Color free version of `colorize`
pub fn printer(thedocs: &DocFile) {
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

/// Given a list of `DocFile` and a file path, write the JSON representation to a file.
pub fn to_json(docstrings: &[DocFile], file_name: &str) {
    let json = serde_json::to_string_pretty(&docstrings).expect("Could not convert to JSON");
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

/// Represents the necessary delimiters for a `bashdoc`
#[derive(Debug, Serialize, Deserialize, Copy, Clone)]
pub struct Delimiters<'a> {
    start: &'a str,
    end: &'a str,
    params: &'a str,
    ret: &'a str,
    opt: &'a str,
    comm: &'a str,
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
    pub fn override_delims(overrides: String) -> Self {
        let mut result: Delimiters = Delimiters::default();
        let splitted: Vec<_> = Box::leak(overrides.into_boxed_str())
            .split_whitespace()
            .collect();
        if splitted.len() != 6 {
            panic!("Please enter the proper number of delimiters");
        }
        result.start = &splitted[0];
        result.end = &splitted[1];
        result.params = &splitted[2];
        result.ret = &splitted[3];
        result.opt = &splitted[4];
        result.comm = &splitted[5];
        result
    }

    /// Read/Write contents of `$BASHDOC_CONFIG_PATH` for use as Delimiters.
    pub fn get_delims() -> Self {
        let mut contents = String::new();
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
                let content =
                    toml::to_string_pretty(&delimiters).expect("Could not be converted to TOML");
                let mut path = home_dir().unwrap();
                path.push(".bashdocrc");
                fs::write(path.to_str().unwrap(), content).unwrap();
                delimiters
            }
        }
    }
}
