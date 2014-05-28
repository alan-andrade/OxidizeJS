extern crate manifest;
extern crate getopts;

use std::comm::channel;
use std::io::Command;
use std::os;
use getopts::{usage, optopt, getopts, optflag};
use manifest::Manifest;

fn main () {
    let args = os::args();
    let program = args.get(0).clone();

    let options = [
        optopt("f", "file", "defaults to manifest.json", "input file"),
        optopt("o", "output", "output file", ""),
        optflag("h", "help", "Prints this message")
    ];

    let matches = match getopts(args.tail(), options) {
        Ok(res) => res,
        Err(e) => fail!("{}", e)
    };

    if matches.opt_present("h") {
        println!("{}", usage(program.as_slice(), options));
        return
    }

    let mut manifest = Manifest::with_options(&matches);

    //let file_chunks = manifest.split(1);
    let (tx, rx) = channel();

    for (num, files) in manifest.split(1).iter().enumerate() {
        let filenames = pluck_filenames(*files);
        print!("batch {}:", num+1);
        println!(" {}", filenames.as_slice());
        tx.send(Command::new("uglifyjs").args(filenames.as_slice()).spawn());
    }

    for _ in range(0, 3) {
        match rx.recv() {
            Ok(process) => {
                match process.wait_with_output() {
                    Ok(p) => manifest.write(p.output.as_slice()),
                    Err(e) => fail!("{}", e)
                };
            },
            Err(f) => fail!("{}", f)
        }
    }
}

fn pluck_filenames (files: &[Path]) -> Vec<String> {
    files.iter().map(|f| String::from_str(f.as_str().unwrap())).collect()
}
