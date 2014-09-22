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

pub struct ManifestConfig {
    key: String,
    pub source: Path,
    pub output: Path
}

impl ManifestConfig {
    pub fn new() -> ManifestConfig {
        ManifestConfig {
            key:  String::from_str("manifest"),
            source: Path::new("manifest.json"),
            output: Path::new("oxidized.js")
        }
    }
}

pub struct Manifest {
    paths: Vec<Path>,
    config: ManifestConfig
}

impl Manifest {
    pub fn with_options (matches: &Matches) -> Manifest {
        let mut config = ManifestConfig::new();

        match matches.opt_str("f") {
            Some(file) => config.source = Path::new(file),
            None => { fail!("no manifest file given") }
        }

        match matches.opt_str("o") {
            Some(file) => { config.output = Path::new(file) }
            None => {
                println!("- Output file defaulting to: {}", config.output.display())
            }
        }

        File::create(&config.output); // Wipes out old file

        Manifest {
            config: config,
            paths: vec!()
        }
    }

    pub fn write(&mut self, data: &[u8]) {
        match File::open_mode(&self.config.output, Append, Write) {
            Ok(mut f) => {
                match f.write(data) {
                    Err(e) => fail!("{}", e),
                    _ => {}
                }
            },
            Err(e) => fail!("{}", e)
        }
    }

    pub fn extract_paths<'a>(&'a mut self) -> &'a Vec<Path> {
        use serialize::json;
        use serialize::json::String;

        match File::open(&self.config.source) {
            Ok(ref mut file) => {
                let json = match json::from_reader(file as &mut Reader) {
                    Ok(json) => json,
                    Err(e) => fail!("{}", e)
                };

                let value = match json.find(&self.config.key) {
                    Some(v) => v,
                    None => fail!("Key {} is missing.", self.config.key)
                };

                let list = match value.as_list() {
                    Some(v) => v,
                    None => fail!("The value of {}, should be an array.", self.config.key)
                };

                let paths = list.iter().map(|item|
                    match *item {
                        String(ref s) => Path::new(s.as_slice()),
                        _ => { fail!("couldnt convert to string") }
                    }
                ).collect();

                self.paths = paths;
            },
            Err(e) => fail!(e)
        };

        self.expand()
    }

    fn expand<'a>(&'a mut self) -> &'a Vec<Path> {
        use glob::glob;

        let wildcards: Vec<Path> = self.paths
            .iter()
            .skip_while(|path| (**path).is_file())
            .map(|path| path.clone())
            .collect();

        self.paths.retain(|path| path.is_file());

        for wc in wildcards.iter() {
            for wc_path in glob(wc.as_str().unwrap()) {
                if  wc_path.extension() == Some("js".as_bytes()) &&
                    !self.paths.iter().any(|path| path.filename() == wc_path.filename()) {
                    self.paths.push(wc_path);
                }
            }
        }

        &self.paths
    }

    pub fn split<'a> (&'a mut self, cores: uint) -> Vec<&'a [Path]> {
        let mut collector = vec!();
        for i in self.extract_paths().as_slice().chunks(cores) {
            collector.push(i);
        }
        collector
    }
}

#[cfg(test)]
mod test {
    use super::{ManifestConfig,Manifest};

    #[test]
    fn test_explicit_json () {
        let mut config = ManifestConfig::new();
        config.source = Path::new("tests/manifest/explicit.json");

        let mut manifest = Manifest {
            config: config,
            paths: vec!()
        };

        let paths = manifest.extract_paths();
        assert_eq!(paths.len(), 3);
    }

    #[test]
    fn test_wildcards () {
        let mut config = ManifestConfig::new();
        config.source = Path::new("tests/manifest/wildcards.json");

        let mut manifest = Manifest {
            config: config,
            paths: vec!()
        };

        let paths = manifest.extract_paths();
        assert_eq!(paths.len(), 3);
    }
}
