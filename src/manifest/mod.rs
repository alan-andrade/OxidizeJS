// Manifest
//
// This library will look for "manifesto.json" and dig into the
// "manifest" key. The value should be an array of files.
//
// This is an example of how the manifest.json should look like.
//
// "manifest": [
//      "js/foo.js",
//      "js/bar.js",
//      "js/*.js"
// ]
use getopts::Matches;
use std::io::{Write, Append};
use std::io::fs::{File, PathExtensions};

static MANIFEST_KEY: &'static str = "manifest";

pub struct ManifestConfig {
    source: Path,
    output: Path
}

pub struct Manifest {
    config: ManifestConfig
}

pub fn with_options (matches: &Matches) -> Manifest {
    let source = match matches.opt_str("f") {
        Some(path) => Path::new(path),
        None => { fail!("no manifest file given") }
    };

    let output = match matches.opt_str("o") {
        Some(path) => { Path::new(path) }
        None => {
            let p = Path::new("oxidized.js");
            println!("- Output file defaulting to: {}", p.display());
            p
        }
    };

    let config = ManifestConfig {
        source: source,
        output: output
    };

    Manifest { config: config }
}

fn expand(mut paths: Vec<Path>) -> Vec<Path> {
    use glob::glob;

    let wildcards: Vec<Path> = paths
        .iter()
        .skip_while(|path| (**path).is_file())
        .map(|path| path.clone())
        .collect();

    paths.retain(|path| path.is_file());

    for wc in wildcards.iter() {
        for wc_path in glob(wc.as_str().unwrap()) {
            if  wc_path.extension() == Some("js".as_bytes()) &&
                 !paths.iter().any(|path| path.filename() == wc_path.filename()) {
                paths.push(wc_path);
            }
        }
    }

    paths
}

static CORES: uint = 4;

struct Paths {
    collection: Vec<Path>,
    index: uint,
    per_core: uint
}

impl Paths {
    fn new(col: Vec<Path>) -> Paths {
        Paths {
            per_core: ((col.len() / CORES) as f32).ceil() as uint,
            collection: col,
            index: 0
        }
    }
}

impl Iterator<Vec<Path>> for Paths {
    fn next(&mut self) -> Option<Vec<Path>> {
        if self.index < self.collection.len() {
            let current = self.index;
            self.index  = self.index + self.per_core;
            Some(self.collection.slice(current, self.index).to_vec())
        } else {
            None
        }
    }
}

impl Manifest {
    pub fn paths(&mut self) -> Paths {
        use serialize::json;
        use serialize::json::String;

        let mut file = File::open(&self.config.source).unwrap();

        let json = match json::from_reader(&mut file as &mut Reader) {
            Ok(json) => json,
            Err(e) => fail!("{}", e)
        };

        let value = match json.find(&MANIFEST_KEY.to_string()) {
            Some(v) => v,
            None => fail!("Key {} is missing.", MANIFEST_KEY)
        };

        let list = match value.as_list() {
            Some(v) => v,
            None => fail!("The value of {}, should be an array.", MANIFEST_KEY)
        };

        let paths = list.iter().map(|item|
            match item.as_string() {
                Some(s) => { Path::new(s.as_slice()) }
                None    => { fail!("couldnt convert to string") }
            }
        ).collect();

        Paths::new(expand(paths))
    }

    pub fn write(&mut self, data: &[u8]) {
        match File::open_mode(&self.config.output, Append, Write) {
            Ok(mut f) => {
                match f.write(data) {
                    Ok(_) => {},
                    Err(e) => { fail!("{}", e) }
                }
            },
            Err(e) => fail!("{}", e)
        };
    }


    //fn expand<'a>(&'a mut self) -> &'a Vec<Path> {
        //use glob::glob;

        //let wildcards: Vec<Path> = self.paths
            //.iter()
            //.skip_while(|path| (**path).is_file())
            //.map(|path| path.clone())
            //.collect();

        //self.paths.retain(|path| path.is_file());

        //for wc in wildcards.iter() {
            //for wc_path in glob(wc.as_str().unwrap()) {
                //if  wc_path.extension() == Some("js".as_bytes()) &&
                    // !self.paths.iter().any(|path| path.filename() == wc_path.filename()) {
                    //self.paths.push(wc_path);
                //}
            //}
        //}

        //&self.paths
    //}

    //pub fn split<'a> (&'a mut self, cores: uint) -> Vec<&'a [Path]> {
        //let mut collector = vec!();
        //for i in self.paths().as_slice().chunks(cores) {
            //collector.push(i);
        //}
        //collector
    //}
}

#[cfg(test)]
mod test {
    use super::{ManifestConfig,Manifest};

    #[test]
    fn test_explicit_json () {
        let config = ManifestConfig {
            source: Path::new("tests/manifest/explicit.json"),
            output: Path::new("")
        };

        let mut manifest = Manifest { config: config };
        assert_eq!(manifest.paths().count(), 3);
    }

    #[test]
    fn test_wildcards () {
        let mut config = ManifestConfig {
            source: Path::new("tests/manifest/wildcards.json"),
            output: Path::new("")
        };

        let mut manifest = Manifest { config: config };
        assert_eq!(manifest.paths().count(), 4);
    }
}
