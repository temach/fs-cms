use std::env;
use std::fs;
use std::io;
use std::io::Write;

extern crate argparse;
use argparse::{ArgumentParser, Store, StoreTrue};

extern crate tera;
use tera::{Context, Tera};

extern crate walkdir;
use walkdir::WalkDir;

type FscmsRes<T> = Result<T, String>;

use std::path::Path;
use std::path::PathBuf;

fn example_unimplemented() -> bool {
    unimplemented!();
}

fn plugin_txt(path: &str) -> FscmsRes<String> {
    match fs::read_to_string(path) {
        Ok(contents) => Ok(contents),
        Err(error_desc) => Err(format!(
            "Plugin could not open file {:?}, because of error: {:?}",
            path, error_desc
        )),
    }
}

fn plugin_png(path: &str) -> FscmsRes<String> {
    match fs::read_to_string(path) {
        Ok(contents) => Ok(contents),
        Err(error_desc) => Err(format!(
            "Plugin could not open file {:?}, because of error: {:?}",
            path, error_desc
        )),
    }
}

fn render_artifact_to_html(path: &Path) -> FscmsRes<String> {
    // check that file extension exitsts and is valid
    let (ext, fpath) = match (path.extension().and_then(|e| e.to_str()), path.to_str()) {
        // if we have extension and filepath ok, then proceed
        (Some(art_ext), Some(art_path)) => (art_ext, art_path),
        _ => {
            return Err(format!(
                "Artifact {:?}: bad file path or file extension. Please use utf-8 characters and specify extensions for artifact files.",
                path
            ));
        }
    };

    // return the result of processing artifact with the plugin
    match ext {
        "txt" => plugin_txt(fpath),
        "png" => plugin_png(fpath),
        _ => Err(format!(
            "Artifact {:?}: no plugin found for file with extension {}.",
            path, ext
        )),
    }
}

fn get_working_paths(input_dir: &str) -> FscmsRes<Vec<PathBuf>> {
    let mut results = Vec::new();

    for walkdir_result in WalkDir::new(input_dir).into_iter() {
        // check that walkdir returned valid entry
        let valid_entry = match walkdir_result {
            Ok(good_result) => good_result,
            Err(bad_result) => {
                return Err(format!(
                    "Searching inside {}: a directory entry is invalid {:#?}",
                    input_dir, bad_result
                ));
            }
        };

        results.push(valid_entry.into_path());
    }

    // return vector of paths
    Ok(results)
}

fn is_artifact_file(path: &Path) -> bool {
    let fpath = path.to_string_lossy();
    return path.is_file() && (!fpath.contains("template"));
}

fn is_template_file(path: &Path) -> bool {
    let fpath = path.to_string_lossy();
    return path.is_file() && fpath.contains("template") && fpath.contains("html");
}

fn run() -> FscmsRes<()> {
    let mut verbose = false;
    let mut input_dir: String = "./examples/basic/input".to_string();
    let mut output_dir = "./examples/basic/output".to_string();

    {
        // this block limits scope of borrows by ap.refer() method
        let mut ap = ArgumentParser::new();
        ap.set_description("Parse site directory and build a static site from it.");
        ap.refer(&mut verbose)
            .add_option(&["-v", "--verbose"], StoreTrue, "Be verbose");
        ap.refer(&mut input_dir).add_option(
            &["--in"],
            Store,
            "Path to input directory with data for website. By default a path from examples",
        );
        ap.refer(&mut output_dir).add_option(
            &["--out"],
            Store,
            "Path to output directory, serve content from there. By default a path from examples",
        );
        ap.parse_args_or_exit();
    }

    if verbose {
        println!(
            "Running in {:?}.\nStarting site build.",
            env::current_dir().expect("Could not get current working directory.")
        );
    }

    let working_paths = get_working_paths(&input_dir)?;

    let tpaths: Vec<PathBuf> = working_paths
        .iter()
        .filter(|&path| is_template_file(path))
        .cloned()
        .collect();

    let apaths: Vec<PathBuf> = working_paths
        .iter()
        .filter(|path| is_artifact_file(path))
        .cloned()
        .collect();

    if verbose {
        println!("Explored paths: {:#?}", working_paths);
        println!("Found artifacts: {:#?}", apaths);
        println!("Found templates: {:#?}", tpaths);
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

    Ok(())
}

fn main() {
    std::process::exit(match run() {
        Ok(_) => 0,
        Err(err) => {
            println!("Error: {:?}", err);
            1
        }
    });
}
