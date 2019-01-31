use std::fs;
use std::fs::File;
use std::io::{Write};

extern crate argparse;
use argparse::{ArgumentParser, StoreTrue, Store};

extern crate tera;
use tera::{Context, Tera};

const LIPSUM: &'static str =
"Lorem ipsum dolor sit amet, consectetur adipiscing elit, sed do eiusmod tempor incididunt ut \
labore et dolore magna aliqua. Ut enim ad minim veniam, quis nostrud exercitation ullamco \
laboris nisi ut aliquip ex ea commodo consequat. Duis aute irure dolor in reprehenderit in \
voluptate velit esse cillum dolore eu fugiat nulla pariatur. Excepteur sint occaecat \
cupidatat non proident, sunt in culpa qui officia deserunt mollit anim id est laborum";


fn main() {
    let mut verbose = false;
    let mut input_dir : String = "./site".to_string();
    let mut output_dir = "./www".to_string();

    {
        // this block limits scope of borrows by ap.refer() method
        let mut ap = ArgumentParser::new();
        ap.set_description("Parse site directory and build a static site from it.");
        ap.refer(&mut verbose)
            .add_option(&["-v", "--verbose"], StoreTrue,
                        "Be verbose");
        ap.refer(&mut input_dir)
            .add_option(&["--in"], Store,
                        "Path to input directory with data for website. By default ./site");
        ap.refer(&mut output_dir)
            .add_option(&["--out"], Store,
                        "Path to output directory, serve content from there. By default ./www");
        ap.parse_args_or_exit();
    }

    if verbose {
        println!("Starting site build.");
    }

    let paths = fs::read_dir(&input_dir).expect("could not find input directory");

    for path in paths {
        println!("Found input file: {}", path.expect("could not read name of one of the files in the input directory").path().display())
    }

    // Use globbing
    input_dir = input_dir.trim_end_matches('/').to_string();
    input_dir.push_str("/**/*");
    if verbose {
        println!("input_dir: {}", &input_dir);
        println!("output_dir: {}", &output_dir);
    }

    let mut tera = tera::compile_templates!(&input_dir);

    let mut ctx = Context::new();
    ctx.insert("title", &"hello world!");
    ctx.insert("content", &LIPSUM);
    ctx.insert("todos", &vec!["buy milk", "walk the dog", "write about tera"]);

    let rendered = tera.render("index.html", &ctx).expect("Failed to render template");

    output_dir.push_str("/index.html");
    let mut output = File::create(output_dir).expect("Could not open output file");
    write!(output, "{}", rendered);

    println!("{}", rendered);

}
