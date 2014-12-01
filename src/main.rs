extern crate getopts;
extern crate glob;
extern crate serialize;
extern crate debug;

use std::comm::channel;
use std::io::Command;
use std::os;
use getopts::{usage, optopt, getopts, optflag};

pub mod manifest;

fn main () {
    let args = os::args();
    let program = args[0].clone();

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

    let mut manifest = manifest::with_options(&matches);

    let (tx, rx) = channel();

    let mut counter = 0u;
    for chunks in manifest.paths() {
        let filenames: Vec<&str> = chunks.
            iter().
            map(|c| c.as_str().unwrap()).
            collect();

        println!("processing: {:?}", filenames.as_slice());
        counter += 1u;
        tx.send(Command::new("uglifyjs").args(filenames.as_slice()).spawn());
    }

    for _ in  range(0u, counter) {
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
