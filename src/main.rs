use std::io::{Write};
use std::error::Error;
use std::io;
use std::env;
use std::fs;

extern crate argparse;
use argparse::{ArgumentParser, StoreTrue, Store};

extern crate tera;
use tera::{Context, Tera};

extern crate walkdir;
use walkdir::WalkDir;

enum TemplateFilesErrors {
    EntryInvalid,
    NothingFound,
}

fn plugin_txt(path : & str) -> String {

}

fn plugin_png(path : & str) -> String {

}


fn render_artifact_to_html(path : &str) -> String {
    match path.extension() {
        "txt" => plugin_txt(path),
        "png" => plugin_png(path),
    }
}

#[no_mangle]
fn find_artifact_files(input_dir : &str) -> Result<Vec<String>, io::Error> {
    let mut artifact_paths = Vec::new();

    for entry in WalkDir::new(input_dir).into_iter() {
        let entry = entry?.path();
        let fpath = entry.to_string_lossy();
        if ! entry.is_file()
            || fpath.contains("template") {
            continue;
        }
        artifact_paths.push(fpath.to_string());
    }
    Ok(artifact_paths)
}


#[no_mangle]
fn find_template_files(input_dir : &str) -> Result<Vec<String>, io::Error> {
    let mut template_fpaths = Vec::new();

    for entry in WalkDir::new(input_dir).into_iter() {
        let entry = entry?.path();
        let fpath = entry.to_string_lossy();
        if entry.is_file() && fpath.contains("template") && fpath.contains("html") {
            template_fpaths.push(fpath.to_string());
        }
    }
    Ok(template_fpaths)
}

fn main() {
    let mut verbose = false;
    let mut input_dir : String = "./examples/basic/input".to_string();
    let mut output_dir = "./examples/basic/output".to_string();

    {
        // this block limits scope of borrows by ap.refer() method
        let mut ap = ArgumentParser::new();
        ap.set_description("Parse site directory and build a static site from it.");
        ap.refer(&mut verbose)
            .add_option(&["-v", "--verbose"], StoreTrue,
                        "Be verbose");
        ap.refer(&mut input_dir)
            .add_option(&["--in"], Store,
                        "Path to input directory with data for website. By default a path from examples");
        ap.refer(&mut output_dir)
            .add_option(&["--out"], Store,
                        "Path to output directory, serve content from there. By default a path from examples");
        ap.parse_args_or_exit();
    }

    if verbose {
        println!("Running in {:?}.\nStarting site build.", env::current_dir().expect("Could not get current working directory."));
    }

    // let paths = fs::read_dir(&input_dir).expect( &format!("Could not find input directory {}", input_dir) );

    let tpaths : Vec<_> = find_template_files(&input_dir).expect("Error while finding templates, did not return a valid vector");

    // for path in paths {
    //     println!("Found input file: {}", path.expect("could not read name of one of the files in the input directory").path().display());
    // }

    for path in tpaths {
        println!("Found template file: {}", path);
    }

    let apaths : Vec<_> = find_artifact_files(&input_dir).expect("Error while finding artifacts, did not return a valid vector");
    for path in apaths {
        println!("Found artifact file: {}", path);
    }

    // Use globbing
    //
    // input_dir = input_dir.trim_end_matches('/').to_string();
    // input_dir.push_str("/**/*");
    // if verbose {
    //     println!("input_dir: {}", &input_dir);
    //     println!("output_dir: {}", &output_dir);
    // }

    // let mut tera = tera::compile_templates!(&input_dir);

    // let mut ctx = Context::new();
    // ctx.insert("title", &"hello world!");
    // ctx.insert("content", &LIPSUM);
    // ctx.insert("todos", &vec!["buy milk", "walk the dog", "write about tera"]);

    // let rendered = tera.render("index.html", &ctx).expect("Failed to render template");

    // output_dir.push_str("/index.html");
    // let mut output = File::create(output_dir).expect("Could not open output file");
    // write!(output, "{}", rendered);

    // println!("{}", rendered);

}
