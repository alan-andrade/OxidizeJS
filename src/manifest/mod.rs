// Manifest
//
// This library will look for "manifest.json" and dig into the
// "manifest" key. The value should be an array of files.
//
// This is an example of how the manifest.json should look like.
//
// "manifest": [
//      "js/foo.js",
//      "js/bar.js"
// ]
extern crate serialize;

use self::serialize::json;
use std::io::{IoResult, IoError, IoErrorKind};
use std::io::fs::{File, PathExtensions};
use std::slice::bytes::copy_memory;

static FILE_NAME: &'static str = "manifest.json";
static JSON_ROOT: &'static str = "manifest";

pub struct Manifest {
    source: File
}

impl Manifest {
    pub fn new() -> Manifest {
        let path = Path::new(FILE_NAME);

        if !path.is_file() {
            panic!("File {} doesn't exist, perhaphs create it?", path.display())
        }

        if path.extension_str() != Some("json") {
            panic!("Manifest File must be in json format.")
        }

        match File::open(&path) {
            Ok(file) => {
                Manifest { source: file }
            }
            Err(_) => {
                panic!("There was an error reading the file: {}", path.display())
            }
        }
    }
}

impl Reader for Manifest {
    fn read (&mut self, buf: &mut [u8]) -> IoResult<uint> {
        let contents  = try!(self.source.read_to_string());

        if contents.len() == 0 {
            return Err(IoError {
                kind: IoErrorKind::EndOfFile,
                desc: "eof",
                detail: None
            })
        }

        match json::from_str(contents.as_slice()) {
            Ok(json) => {
                let root = match json.find(JSON_ROOT) {
                    Some(r) => r,
                    None => panic!("No root found!")
                };

                let file_list = root.as_array().unwrap();
                let mut buffer = Vec::new();
                for file_name in file_list.iter() {
                    let path = Path::new(file_name.as_string().unwrap());

                    match path.stat() {
                        Ok(_) => {
                            let mut file = File::open(&path);
                            let content = try!(file.read_to_end());
                            buffer.push_all(content.as_slice());
                        },
                        Err(_) => warn!("Couldn't find: {}", path.display())
                    }
                }

                copy_memory(buf, buffer.as_slice());
                Ok(buf.len())
            },

            Err(_) => Err(IoError::last_error())
        }
    }
}
