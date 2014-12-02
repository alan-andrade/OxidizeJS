#![feature(phase)]
#[phase(plugin, link)] extern crate log;

use std::io::process::Command;
use manifest::Manifest;
use std::io::net::pipe::UnixStream;
use std::io::File;

mod manifest;

fn main () {
    // First runs spidermonkey parser
    let source_code = ["Reflect.parse(", ");"];

    let mut manifest = Manifest::new();

    // Might need to escape quotation marks here.
    let content  = match manifest.read_to_string() {
        Ok(c) => c,
        Err(e) => panic!(e)
    };

    println!("js gotten: {}", content);

    let source_code =
        format!(
            "'print(JSON.stringify(Reflect.parse(\"{}\")))'",
            content);

    println!("Source code: \n{}", source_code);

    let stdout = Command::new("uname").arg("-v").output().ok().unwrap();

    println!("uname: {}", String::from_utf8_lossy(stdout.output.as_slice()))

    //let file_path = Path::new("~/.oxidize.out");
    //let output_file = File::create(&file_path);

    //let command = Command::new("js").
        //arg("-e").
        //arg(source_code.as_slice()).
        //env("JS_STDOUT", file_path.clone()).
        //spawn().
        //ok().
        //unwrap();

    //let mut file = File::open(&file_path);
    //let ast = file.read_to_string().ok().unwrap();

    //println!("ast: {}", ast);

    //match Command::new("js").args(&["-e", source_code.as_slice()]).output() {
        //Ok(output) => println!("output: {}", output.output),
        //Err(e) => println!("ererer: {}", e)
    //}
}
