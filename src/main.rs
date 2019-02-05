use std::env;
use std::fs;
use std::io;
use std::io::Write;
use std::path::Path;

extern crate argparse;
use argparse::{ArgumentParser, Store, StoreTrue};

extern crate tera;
use tera::{Context, Tera};

extern crate walkdir;
use walkdir::WalkDir;

type FscmsRes<T> = Result<T, String>;

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
                "Artifact {:?}: bad path or file extension. Please use utf-8 characters and specify extensions for artifact files.",
                path
            ));
        }
    };

    // return the result of processing artifact with the plugin
    match ext {
        "txt" => plugin_txt(fpath),
        "png" => plugin_png(fpath),
        _ => Err(format!(
            "Artifact {:?}: no plugin for extension {}.",
            path, ext
        )),
    }
}

type FileFilter = Fn(&str) -> bool;

fn get_working_paths(input_dir: &str) -> FscmsRes<Vec<&Path>> {
    let mut results = Vec::new();

    for walkdir_result in WalkDir::new(input_dir).into_iter() {
        // check that walkdir returned valid entry
        let valid_entry = match walkdir_result {
            Ok(entry) => entry,
            Err(e) => {
                return Err(format!(
                    "Searching inside {}: directory entry {} is invalid due to error {}",
                    input_dir, walkdir_result, e
                ));
            }
        };

        // convert walkdir entry to std::path::Path
        let path = match valid_entry.path() {
            Ok(p) => p,
            Err(e) => {
                return Err(format!(
                    "Searching inside {}: directory entry {} is not a path due to error {}",
                    input_dir, valid_entry, e
                ));
            }
        };

        results.push(&path);
    }

    // return vector of paths
    Ok(results)
}


fn find_artifact_files(path: Path) -> bool {
    let fpath = path.to_string_lossy();
    return path.is_file() && (!fpath.contains("template"));
}

fn find_template_files(path: Path) -> bool {
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

    // let paths = fs::read_dir(&input_dir).expect( &format!("Could not find input directory {}", input_dir) );

    let working_paths = get_working_paths(&input_dir)?;
    let tpaths = working_paths.
    let tpaths: Vec<_> = find_template_files(&input_dir)
        .expect("Error while finding templates, did not return a valid vector");

    // for path in paths {
    //     println!("Found input file: {}", path.expect("could not read name of one of the files in the input directory").path().display());
    // }

    for path in tpaths {
        println!("Found template file: {}", path);
    }

    let apaths: Vec<_> = find_artifact_files(&input_dir)
        .expect("Error while finding artifacts, did not return a valid vector");
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

fn main() {
    std::process::exit(match run() {
        Ok(_) => 0,
        Err(err) => {
            println!("Error: {:?}", err);
            1
        }
    });
}
