// Manifest
//
// This library will look for "manifesto.json" and dig into the
// "manifest" key. The value should be an array of files.
//
// This is an example of how the manifest.json should look.
//
// "manifest": [
//      "js/foo.js",
//      "js/bar.js",
//      "js/*.js"
// ]
//
// You can use wildcards.
//
// The manifesto gives us information about which files should be
// printed at the beginning of the resulting file.
#![crate_id = "manifest#0.0.2"]
#![crate_type = "rlib"]

extern crate serialize;
extern crate glob;

use serialize::json;
use glob::glob;

pub struct Manifest {
    list: Vec<Path>
}

impl Manifest {
    pub fn new (filename: &str) -> Manifest {
        use std::io::fs::File;

        let file_path = Path::new(filename);

        let json_str = match File::open(&file_path).read_to_str() {
            Ok(s)   => s,
            Err(e)  => fail!("{}", e)
        };

        Manifest { list: Manifest::read(json_str.as_slice()) }
    }

    fn read (data: &str) -> Vec<Path> {
        let json = match json::from_str(data) {
            Ok(s)   => s,
            Err(e)  => fail!("{}", e)
        };

        let key = StrBuf::from_str("manifest");
        let value = match json.find(&key) {
            Some(v) => {
                match v.as_list() {
                    Some(l) => l,
                    None    => fail!("The value of the manifest key doesn't contain an array.")
                }
            },
            None => fail!("json file doesn't contain the key manifest")
        };

        let paths: Vec<Path> = value.iter().map(|filename| {
            match filename.as_string() {
                Some(f) => Path::new(f),
                None    => fail!("Array contents aren't strings.")
            }
        }).collect();

        Manifest::expand(&paths)
    }

    fn expand(paths: &Vec<Path>) -> Vec<Path> {
        let mut collector = Vec::new();

        for path in paths.iter() {
            if path.is_file() {
                collector.push(path.clone());
            } else {
                for exp_path in glob(path.as_str().unwrap()) {
                    let same_name = {|p:&&Path| p.filename() == exp_path.filename()};

                    if paths.iter().find(same_name).is_none() {
                       collector.push(exp_path.clone())
                    }

                }
            }
        }

        collector
    }

    pub fn split<'a> (&'a self, cores: uint) -> Vec<&'a [Path]> {
        let mut collector = vec!();
        for i in self.list.as_slice().chunks(cores) {
            collector.push(i.clone());
        }
        collector
    }
}

#[cfg(test)]
mod test {
    use Manifest;

    #[test]
    fn test_read () {
        let json_str = "{ \"manifest\": [\"test/a.js\"] }";
        let read = Manifest::read(json_str);
        assert_eq!(read.get(0).filename_str().unwrap(), "a.js");


        let json_str = "{ \"manifest\": [\"test/a.js\", \"test/b.js\"] }";
        let read = Manifest::read(json_str);
        assert_eq!(read.get(0).filename_str().unwrap(), "a.js");
        assert_eq!(read.get(1).filename_str().unwrap(), "b.js");

        let json_str = "{ \"manifest\": [] }";
        assert!(Manifest::read(json_str) == vec!());
    }

    #[test]
    #[should_fail]
    fn test_read_fails_wrong_key () {
        let json_str = "{ \"wrong_key\": [\"test/a.js\", \"test/b.js\"] }";
        Manifest::read(json_str);
    }

    #[test]
    #[should_fail]
    fn test_read_fails_when_no_array () {
        let json_str = "{ \"manifest\": \"test/a.js\" }";
        Manifest::read(json_str);
    }

    #[test]
    fn test_expand () {
        let glob_path = Path::new("test/*.js");
        let man = Manifest::expand(&vec!(glob_path));

        assert_eq!(man.len(), 3);
    }

    #[test]
    fn test_read_with_expand () {
        let json_str = "{ \"manifest\": [\"test/a.js\", \"test/*.js\"] }";
        let paths = Manifest::read(json_str);

        assert_eq!(paths.get(0).filename_str().unwrap(), "a.js");
        assert_eq!(paths.get(1).filename_str().unwrap(), "b.js");
        assert_eq!(paths.get(2).filename_str().unwrap(), "c.js");
    }

    #[test]
    fn test_split_for_parallel_compilation () {
        let m = Manifest {
            list: vec!(Path::new("js/one.js"),
                       Path::new("js/two.js"),
                       Path::new("js/thr.js"))
        };
        assert_eq!(m.split(1).len(), 3);
        assert_eq!(m.split(2).len(), 2);
        assert_eq!(m.split(3).len(), 1);
        assert_eq!(m.split(4).len(), 1);
    }

    #[test]
    fn test_new () {
        let filename = "tester.json";

        create_json(filename);
        let manifesto = Manifest::new(filename);
        assert_eq!(manifesto.list.get(0).filename_str().unwrap(), "a.js");
        delete_json(filename);
    }

    fn create_json (filename: &str) {
        #![allow(unused_must_use)]

        use std::io::fs::File;
        match File::create(&Path::new(filename)) {
            Ok(mut f) => {
                f.write(bytes!("{\"manifest\": [\"test/a.js\"]}"));
            },
            Err(e) => { fail!("{}", e) }
        };
    }

    fn delete_json (filename: &str) {
        #![allow(unused_must_use)]

        use std::io::Command;
        match Command::new("rm").arg(filename).spawn() {
            Ok(mut child) => { child.wait(); },
            Err(_) => { println!("manifest.json didn't exist.") }
        };
    }
}
