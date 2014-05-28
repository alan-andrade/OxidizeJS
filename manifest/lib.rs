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
#![crate_id = "manifest#0.0.2"]
#![crate_type = "rlib"]

extern crate serialize;
extern crate glob;
extern crate getopts;
use std::io::fs::File;

pub struct ManifestConfig {
    key: String,
    source: Path,
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
    config: ManifestConfig,
    file: File
}

impl Manifest {
    pub fn new () -> Manifest {
        let config = ManifestConfig::new();

        match File::create(&config.output) {
            Ok(f) => {
                Manifest {
                    config: config,
                    paths: vec!(),
                    file: f
                }
            },
            Err(e) => fail!("{}", e)
        }

    }

    pub fn with_options (matches: &getopts::Matches) -> Manifest {
        let mut config = ManifestConfig::new();

        match matches.opt_str("f") {
            Some(file) => config.source = Path::new(file),
            None => {}
        }

        match matches.opt_str("o") {
            Some(file) => config.output = Path::new(file),
            None => {}
        }

        match File::create(&config.output) {
            Ok(f) => {
                Manifest {
                    config: config,
                    paths: vec!(),
                    file: f
                }
            },
            Err(e) => fail!("{}", e)
        }
    }

    pub fn write(&mut self, data: &[u8]) {
        match self.file.write(data) {
            Err(e) => fail!("{}", e),
            _ => {}
        }
    }

    fn paths<'a>(&'a mut self) -> &'a Vec<Path> {
        self.extract_paths()
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
            .skip_while(|path| path.is_file())
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
        for i in self.paths().as_slice().chunks(cores) {
            collector.push(i);
        }
        collector
    }
}

//impl<'a> Iterator<&'a Path> for Manifest {
    //fn next(&'a mut self) -> Option<&'a Path> {
        //Some(self.paths.get(0))
    //}
//}

#[cfg(test)]
mod test {
    use ManifestConfig;
    use Manifest;

    #[test]
    fn test_explicit_json () {
        let mut config = ManifestConfig::new();
        config.set_path(Path::new("test/explicit.json"));

        let mut manifesto = Manifest::new();
        manifesto.config = config;

        let paths = manifesto.paths();
        assert_eq!(paths.len(), 3);
    }

    #[test]
    fn test_wildcards () {
        let mut config = ManifestConfig::new();
        config.set_path(Path::new("test/wildcards.json"));

        let mut manifesto = Manifest::new();
        manifesto.config = config;

        let paths = manifesto.paths();
        assert_eq!(paths.len(), 3);
    }
}
