extern crate manifest;

use std::comm::channel;
use std::io::Command;
use std::io::fs::File;
use manifest::Manifest;

fn main () {
    let mut manifest = Manifest::new();
    let file_chunks = manifest.split(1);

    let (tx, rx) = channel();

    for (num, files) in file_chunks.iter().enumerate() {
        let filenames = pluck_filenames(*files);
        print!("batch {}:", num+1);
        println!(" {}", filenames.as_slice());
        tx.send(Command::new("uglifyjs").args(filenames.as_slice()).spawn());
    }

    let mut file = File::create(&Path::new("test/with_channels.js"));
    for _ in range(0, file_chunks.len()) {
        match rx.recv() {
            Ok(process) => { file.write(process.wait_with_output().unwrap().output.as_slice()); },
            Err(f) => { fail!("{}", f) }
        }
    }
}

fn pluck_filenames (files: &[Path]) -> Vec<String> {
    files.iter().map(|f| String::from_str(f.as_str().unwrap())).collect()
}
