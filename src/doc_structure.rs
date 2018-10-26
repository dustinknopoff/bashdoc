pub mod docs {
    use colored::*;
    use dirs::home_dir;
    use glob::glob;
    use rayon::prelude::*;
    use serde_derive::*;
    use std::collections::HashMap;
    use std::env;
    use std::fs;
    use std::fs::File;
    use std::io::prelude::*;
    use std::io::BufReader;
    use std::path::Path;

    /// Represents a docstring
    /// contains:
    ///
    /// - short description (name of function)
    /// - long description
    /// - `HashMap` of options to their descriptions
    /// - `HashMap` of parameters to their descriptions
    /// - `HashMap` of return values to their descriptions
    #[derive(Debug, Default, Serialize, Deserialize)]
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
            let delims: Delimiters = get_delims();
            let mut result: Doc = Default::default();
            for line in vector.iter() {
                if line == &vector[0] {
                    result.short_description.push_str(line);
                } else if line.contains(delims.params.as_str()) {
                    let splitted: Vec<_> = line.split_whitespace().map(|x| x.to_string()).collect();
                    let rest: String = splitted[2..].join(" ");
                    result.params.insert(splitted[1].replace(":", ""), rest);
                } else if line.contains(delims.ret.as_str()) {
                    let splitted: Vec<_> = line.split_whitespace().map(|x| x.to_string()).collect();
                    let rest: String = splitted[2..].join(" ");
                    result.returns.insert(splitted[1].replace(":", ""), rest);
                } else if line.contains(delims.opt.as_str()) {
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
    #[derive(Debug, Default, Serialize, Deserialize)]
    pub struct DocFile {
        thedocs: Vec<Doc>,
        filename: String,
    }

    impl DocFile {
        /// Append the given `Doc` to this `AllDoc`
        pub fn add(&mut self, doc: Doc) -> () {
            self.thedocs.push(doc);
            ()
        }
    }

    /// Gets all `START_DELIM->END_DELIM` comments in the zshrc
    ///
    /// This goes through every line finding the start of the docstring
    /// and adds every line to a `Vec` until the end delimiter.
    ///
    /// A final `Vec` of the collected comment strings is returned.
    fn get_info(p: &Path) -> Vec<Vec<String>> {
        // let mut p = dirs::home_dir().unwrap();
        // p.push(".zshrc");
        let delims: Delimiters = get_delims();
        let f = File::open(&p).expect("file not found.");
        let f = BufReader::new(f);
        let mut result: Vec<Vec<String>> = Vec::new();
        result.push(Vec::new());
        let mut can_add = false;
        let mut index = 0;
        for line in f.lines() {
            let curr_line = line.expect("Line cannot be accessed.");
            if curr_line.contains(delims.start.as_str()) {
                can_add = true;
                continue;
            } else if curr_line.contains(delims.end.as_str()) {
                can_add = false;
                index += 1;
                result.push(Vec::new());
            }
            if can_add {
                if curr_line.contains(delims.opt.as_str()) {
                    result[index].push(curr_line);
                } else {
                    result[index].push(curr_line.replace(delims.comm.as_str(), ""));
                }
            }
        }
        result
    }

    fn generate_doc_file(docs: &[Vec<String>], fname: String) -> DocFile {
        let mut all_docs: DocFile = Default::default();
        all_docs.filename = fname;
        for doc in docs.iter() {
            if doc.to_vec().is_empty() {
                continue;
            }
            let as_bash_doc = Doc::make_doc(&doc.to_vec());
            all_docs.add(as_bash_doc);
        }
        all_docs
    }

    pub fn start(p: &str, is_directory: bool) -> Vec<DocFile> {
        let dir = p.replace("~", home_dir().unwrap().to_str().unwrap());
        if is_directory {
            let files: Vec<_> = glob(&dir).unwrap().filter_map(|x| x.ok()).collect();
            let every_doc: Vec<DocFile> = files
                .par_iter()
                .map(|entry| {
                    let docs = get_info(&entry);
                    generate_doc_file(
                        &docs,
                        entry.file_name().unwrap().to_str().unwrap().to_string(),
                    )
                }).collect();
            every_doc
        } else {
            let docs = get_info(&Path::new(&p));
            let all_docs = generate_doc_file(
                &docs,
                Path::new(&dir)
                    .file_name()
                    .unwrap()
                    .to_str()
                    .unwrap()
                    .to_string(),
            );
            let result = vec![all_docs];
            result
        }
    }

    /// # Pretty print an `DocFile`
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
    pub fn colorize(thedocs: &DocFile) -> () {
        println!(
            "{}: {}",
            "Help".green().underline(),
            thedocs.filename.green().underline()
        );
        for doc in &thedocs.thedocs {
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

    pub fn printer(thedocs: &DocFile) -> () {
        println!("Help: {}", thedocs.filename);
        for doc in &thedocs.thedocs {
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

    pub fn export_json(docstrings: &[DocFile], file_name: &str) {
        let json = serde_json::to_string_pretty(&docstrings).expect("Could not convert to JSON");
        let path_as_str = file_name.replace("~", home_dir().unwrap().to_str().unwrap());
        let path = Path::new(&path_as_str);
        let mut file = File::create(Path::new(&path)).expect("Invalid file path.");
        file.write_all(&json.as_bytes())
            .expect("Could not write to file.");
    }

    #[derive(Debug, Serialize, Deserialize)]
    struct Delimiters {
        start: String,
        end: String,
        params: String,
        ret: String,
        opt: String,
        comm: String,
    }

    impl Default for Delimiters {
        fn default() -> Delimiters {
            Delimiters {
                start: "#;".to_string(),
                end: "#\"".to_string(),
                params: "@param".to_string(),
                ret: "@return".to_string(),
                opt: "# -".to_string(),
                comm: "# ".to_string(),
            }
        }
    }

    fn get_delims() -> Delimiters {
        let mut contents = String::new();
        match env::var_os("BASHDOC_CONFIG_PATH") {
            Some(val) => {
                let mut config = File::open(Path::new(&val)).expect("Invalid path");
                config
                    .read_to_string(&mut contents)
                    .expect("could not read from file.");
                let mut to_convert = String::new();
                to_convert.push_str(&contents);
                let sorted: Delimiters = toml::from_str(&to_convert.as_str()).unwrap();
                sorted
            }
            None => {
                let mut delimiters = Delimiters::default();
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
