use std::io::fs::{File, PathExtensions, unlink};
use std::io::{Write, Append};
use getopts::Matches;

use super::paths::Paths;

static MANIFEST_KEY: &'static str = "manifest";

pub struct Manifest {
    output: Path,
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

    Manifest {
        paths: get_all_paths(&source),
        output: output
    }
}

// XXX: Refactor this function
fn expand(mut paths: Vec<Path>) -> Vec<Path> {
    use glob::glob;
    use std::os::getcwd;

    for path in paths.iter() { println!("path: {}", path.display()); }

    let wildcards: Vec<Path> = paths
        .iter()
        .skip_while(|path| (**path).is_file())
        .map(|path| path.clone())
        .collect();

    paths.iter().map(|path| getcwd().join(path)).collect::<Vec<Path>>();
    paths.retain(|path| path.is_file() && path.exists() );

    for wc in wildcards.iter() {
        for wc_path in glob(wc.as_str().unwrap()) {
            if  wc_path.extension() == Some("js".as_bytes()) &&
                 !paths.iter().any(|path| path.filename() == wc_path.filename()) {
                paths.push(wc_path);
            }
        }
    }

    for path in paths.iter() { println!("path: {}", path.display()); }

    paths
}


impl Manifest {
    pub fn paths<'a>(&'a mut self) -> Paths<'a, Path> {
        Paths::new(self.paths.as_slice())
    }

    pub fn write(&mut self, data: &[u8]) {
        let mut file = if self.output.is_file() {
            File::open_mode(&self.output, Append, Write)
        } else {
            File::create(&self.output)
        };

        match file.write(data) {
            Ok(_) => {},
            Err(e) => fail!("Couldnt write to file: {}", e)
        }
    }
}

fn get_all_paths(source: &Path) -> Vec<Path> {
        use serialize::json;

        let mut file = File::open(source).unwrap();

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
