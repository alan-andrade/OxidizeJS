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
use std::io::fs::{File, PathExtensions, unlink};

static MANIFEST_KEY: &'static str = "manifest";

pub struct ManifestConfig {
    source: Path,
    output: Path
}

pub struct Manifest {
    config: ManifestConfig,
    paths: Vec<Path>
}

pub fn with_options(matches: &Matches) -> Manifest {
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

    unlink(&output);

    let config = ManifestConfig {
        source: source,
        output: output
    };

    Manifest::new(config)
}

fn get_all_paths(config: &ManifestConfig) -> Vec<Path> {
        use serialize::json;

        let mut file = File::open(&config.source).unwrap();

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

        expand(paths)
}

fn expand(mut paths: Vec<Path>) -> Vec<Path> {
    use glob::glob;

    let wildcards: Vec<Path> = paths
        .iter()
        .skip_while(|path| (**path).is_file())
        .map(|path| path.clone())
        .collect();

    paths.retain(|path| path.is_file() && path.exists() );

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

struct Paths<'a, T: 'static> {
    collection: &'a [T],
    index: uint,
    per_core: uint
}

impl<'a, T> Paths<'a, T> {
    fn new(col: &'a [T]) -> Paths<'a, T> {
        Paths {
            per_core: (col.len() as f32 / CORES as f32).ceil() as uint,
            collection: col,
            index: 0
        }
    }
}

impl<'a, T> Iterator<&'a [T]> for Paths<'a, T> {
    fn next(&mut self) -> Option<&'a [T]> {
        if self.index < self.collection.len() && self.per_core > 0 {
            let current = self.index;
            self.index  = self.index + self.per_core;
            Some(self.collection.slice(current, self.index))
        } else {
            None
        }
    }
}

impl Manifest {
    fn new(c: ManifestConfig) -> Manifest {
        Manifest {
            paths: get_all_paths(&c),
            config: c
        }
    }

    pub fn paths<'a>(&'a mut self) -> Paths<'a, Path> {
        Paths::new(self.paths.as_slice())
    }

    pub fn write(&mut self, data: &[u8]) {
        let mut file = if self.config.output.is_file() {
            File::open_mode(&self.config.output, Append, Write)
        } else {
            File::create(&self.config.output)
        };

        match file.write(data) {
            Ok(_) => {},
            Err(e) => fail!("Couldnt write to file: {}", e)
        }
    }
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
