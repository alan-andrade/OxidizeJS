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
//
// You can use wildcards.
#![crate_id = "manifest#0.0.2"]
#![crate_type = "rlib"]

extern crate serialize;
extern crate glob;
use std::io::fs::File;

pub struct ManifestConfig {
    key: String,
    path: Path,
    ext: String
}

impl ManifestConfig {
    pub fn new() -> ManifestConfig {
        ManifestConfig {
            key:  String::from_str("manifest"),
            path: Path::new("manifest.json"),
            ext: String::from_str("js")
        }
    }

    fn set_path(&mut self, path: Path) {
        self.path = path
    }
}

pub struct Manifest {
    pub paths: Vec<Path>,
    config: ManifestConfig
}

impl Manifest {
    pub fn new () -> Manifest {
        Manifest {
            config: ManifestConfig::new(),
            paths: vec!()
        }
    }

    fn get_paths<'a>(&'a mut self) -> &'a Vec<Path> {
        self.extract_paths()
    }

    pub fn extract_paths<'a>(&'a mut self) -> &'a Vec<Path> {
        use serialize::json;
        use serialize::json::String;

        match File::open(&self.config.path) {
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
                if  wc_path.extension() == Some(self.config.ext.as_bytes()) &&
                    !self.paths.iter().any(|path| path.filename() == wc_path.filename()) {
                    self.paths.push(wc_path);
                }
            }
        }

        &self.paths
    }

    pub fn split<'a> (&'a mut self, cores: uint) -> Vec<&'a [Path]> {
        let mut collector = vec!();
        for i in self.get_paths().as_slice().chunks(cores) {
            collector.push(i);
        }
        collector
    }
}

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

        let paths = manifesto.get_paths();
        assert_eq!(paths.len(), 3);
    }

    #[test]
    fn test_wildcards () {
        let mut config = ManifestConfig::new();
        config.set_path(Path::new("test/wildcards.json"));

        let mut manifesto = Manifest::new();
        manifesto.config = config;

        let paths = manifesto.get_paths();
        assert_eq!(paths.len(), 3);
    }
}
