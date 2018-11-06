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

    impl PartialEq for Doc {
        fn eq(&self, other: &Doc) -> bool {
            self.short_description == other.short_description
                && self.long_description == other.long_description
                && self.descriptors == other.descriptors
                && self.params == other.params
                && self.returns == other.returns
        }
    }

    impl Doc {
        /// # Build a `Doc` from an array of strings
        /// Parse `Doc` fields.
        pub fn make_doc(vector: &[String], delims: Delimiters) -> Doc {
            let mut result: Doc = Default::default();
            for line in vector.iter() {
                if line == &vector[0] {
                    result.short_description.push_str(line);
                } else if line.contains(delims.params) {
                    let splitted: Vec<_> = line.split_whitespace().map(|x| x.to_string()).collect();
                    let rest: String = splitted[2..].join(" ");
                    result.params.insert(splitted[1].replace(":", ""), rest);
                } else if line.contains(delims.ret) {
                    let splitted: Vec<_> = line.split_whitespace().map(|x| x.to_string()).collect();
                    let rest: String = splitted[2..].join(" ");
                    result.returns.insert(splitted[1].replace(":", ""), rest);
                } else if line.contains(delims.opt) {
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
    fn get_info(p: &Path, delims: Delimiters) -> Vec<Vec<String>> {
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
            if curr_line.contains(delims.start) {
                can_add = true;
                continue;
            } else if curr_line.contains(delims.end) {
                can_add = false;
                index += 1;
                result.push(Vec::new());
            }
            if can_add {
                if curr_line.contains(delims.opt) {
                    result[index].push(curr_line);
                } else {
                    result[index].push(curr_line.replace(delims.comm, ""));
                }
            }
        }
        result
    }

    fn generate_doc_file(docs: &[Vec<String>], fname: String, delims: Delimiters) -> DocFile {
        let mut all_docs: DocFile = Default::default();
        all_docs.filename = fname;
        for doc in docs.iter() {
            if doc.to_vec().is_empty() {
                continue;
            }
            let as_bash_doc = Doc::make_doc(&doc.to_vec(), delims);
            all_docs.add(as_bash_doc);
        }
        all_docs
    }

    pub fn start(p: &str, is_directory: bool, delims: Delimiters) -> Vec<DocFile> {
        let dir = p.replace("~", home_dir().unwrap().to_str().unwrap());
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
                }).collect();
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

    pub fn to_json(docstrings: &[DocFile], file_name: &str) {
        let json = serde_json::to_string_pretty(&docstrings).expect("Could not convert to JSON");
        let path_as_str = file_name.replace("~", home_dir().unwrap().to_str().unwrap());
        let path = Path::new(&path_as_str);
        let mut file = File::create(Path::new(&path)).expect("Invalid file path.");
        file.write_all(&json.as_bytes())
            .expect("Could not write to file.");
    }

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
                    let mut as_static: &'static str = Box::leak(to_convert.into_boxed_str());
                    let sorted: Delimiters = toml::from_str(&as_static).unwrap();
                    sorted
                }
                None => {
                    let mut delimiters = Delimiters::default();
                    let content = toml::to_string_pretty(&delimiters)
                        .expect("Could not be converted to TOML");
                    let mut path = home_dir().unwrap();
                    path.push(".bashdocrc");
                    fs::write(path.to_str().unwrap(), content).unwrap();
                    delimiters
                }
            }
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        macro_rules! map(
        { $($key:expr => $value:expr),+ } => {
                {
                    let mut m = ::std::collections::HashMap::new();
                    $(
                        m.insert($key, $value);
                    )+
                    m
                }
            };
        );

        #[test]
        fn make_doc() {
            let input: &[String] = &vec![
                "runner()".to_string(),
                "This is the beginning".to_string(),
                "@params filename: don\'t test me".to_string(),
                "@params location: where to put it".to_string(),
                "@returns nothing:".to_string(),
            ];
            let result = Doc::make_doc(&input, Delimiters::get_delims());
            let mut expected = Doc::default();
            expected.short_description = "runner()".to_string();
            expected.long_description = "This is the beginning".to_string();
            expected.descriptors = HashMap::new();
            expected.params = map!(
                "location".to_string() => "where to put it".to_string(),
                "filename".to_string() => "don\'t test me".to_string()
                );
            expected.returns = map!("nothing".to_string() => String::new());
            assert_eq!(result, expected);
        }

        #[test]
        fn docfile_add() {
            let mut docfile = DocFile::default();
            let mut expected = Doc::default();
            expected.short_description = "runner()".to_string();
            expected.long_description = "This is the beginning".to_string();
            expected.descriptors = HashMap::new();
            expected.params = map!(
                "location".to_string() => "where to put it".to_string(),
                "filename".to_string() => "don\'t test me".to_string()
                );
            expected.returns = map!("nothing".to_string() => String::new());

            let mut result = Doc::default();
            result.short_description = "runner()".to_string();
            result.long_description = "This is the beginning".to_string();
            result.descriptors = HashMap::new();
            result.params = map!(
                "location".to_string() => "where to put it".to_string(),
                "filename".to_string() => "don\'t test me".to_string()
                );
            result.returns = map!("nothing".to_string() => String::new());
            assert_eq!(0, docfile.thedocs.len());
            docfile.add(expected);
            assert_eq!(1, docfile.thedocs.len());
            assert_eq!(result, docfile.thedocs[0]);
        }

        #[test]
        fn test_get_info() {
            let p = Path::new("example.sh");
            let result = get_info(&p, Delimiters::get_delims());
            let expected: Vec<Vec<String>> = vec![
                [
                    "runner()".to_string(),
                    "This is the beginning".to_string(),
                    "# - CTRL-O pushs the boundaries".to_string(),
                ]
                    .to_vec(),
                [
                    "runner()".to_string(),
                    "This is the beginning".to_string(),
                    "@params filename: don\'t test me".to_string(),
                    "@params location: where to put it".to_string(),
                    "@returns nothing:".to_string(),
                ]
                    .to_vec(),
                [].to_vec(),
            ];
            assert_eq!(expected, result);
        }
    }
}
