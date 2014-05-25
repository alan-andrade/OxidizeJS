extern crate manifest;

use std::comm::channel;
use std::io::Command;
use std::io::fs::File;
use manifest::Manifest;

fn main () {
    let manifest = Manifest::new("manifest.json");
    let file_chunks = manifest.split(4);

    let (tx, rx) = channel();

    for (num, files) in file_chunks.iter().enumerate() {
        println!("chunk: {}", num+1);
        let filenames = pluck_filenames(*files);
        println!("filenames: {}", filenames.as_slice());
        tx.send(Command::new("uglifyjs").args(filenames.as_slice()).output());
    }

    let mut file = File::create(&Path::new("with_channels.js"));
    for _ in range(0, file_chunks.len()) {
        match rx.recv() {
            Ok(p_out) => { file.write(p_out.output.as_slice()); },
            Err(f) => { fail!("{}", f) }
        }
    }
}

fn pluck_filenames (files: &[Path]) -> Vec<StrBuf> {
    files.iter().map(|f| StrBuf::from_str(f.as_str().unwrap())).collect()
}
