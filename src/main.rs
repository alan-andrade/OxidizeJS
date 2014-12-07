#![feature(phase)]
#[phase(plugin, link)] extern crate log;

use std::io::process::Command;
use manifest::Manifest;
use std::io::File;

mod manifest;

fn main () {
    let mut manifest = Manifest::new();

    // Might need to escape quotation marks here.
    let content  = match manifest.read_to_string() {
        Ok(c) => {
            let mut bytes = c.into_bytes();
            bytes.retain(|&c| c != 0 && c != 10 && c != 11); // Remove LF
            String::from_utf8(bytes).unwrap()
        }
        Err(e) => panic!(e)
    };
    println!("js from manifest: {}", content);

    let source_code = format!("print(
        JSON.stringify(
            Reflect.parse(\"{}\")
        ));\n", content);

    println!("Source code: \n{}", source_code);
    let source_code_path = Path::new(".oxidize-source");
    let mut source_code_file = File::create(&source_code_path);
    source_code_file.write(source_code.as_bytes()).ok();

    let ast_path = Path::new(".oxidize-out");
    match Command::new("js").
            env("JS_STDOUT", ".oxidize-out").
            env("JS_STDERR", ".oxidize-error").
            arg("-f").
            arg(source_code_path).
            status() {
                Ok(s) => println!("{}", s),
                Err(e) => panic!(e)
            }


    let result =
        match File::open(&ast_path) {
            Ok(mut f) => {
                let ast = f.read_to_string().ok().unwrap();
                println!("ast: {}", ast);
                let code = format!("
                    var esmangle  = require('esmangle')
                      , escodegen = require('escodegen');

                    var minified = esmangle.mangle({});
                    console.log(escodegen.generate(minified))
                ", ast);

                let mut minify = File::create(&Path::new(".oxidize-minify")).ok().unwrap();
                minify.write(code.as_bytes());

                let output = Command::new("node").
                    arg(".oxidize-minify").
                    output().
                    unwrap().
                    output;

                String::from_utf8(output).unwrap()
            }
            Err(e) => panic!("{}", e)
        };

    println!("result: {}", result);
}
