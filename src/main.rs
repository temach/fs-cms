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
type Artifact = String;

use std::path::Path;
use std::path::PathBuf;

fn example_unimplemented() -> bool {
    unimplemented!();
}

struct RenderInfo<'a> {
    input_dir: &'a str,
    output_dir: &'a str,
    path: &'a str,
}

const HTML_TEMPLATE_PLUGIN_TXT: &str = r#"<p>{{ content }}</p>"#;

const HTML_TEMPLATE_PLUGIN_PNG: &str = r#"<img src=>{{ content }}</p>"#;

type ArtifactPlugin = fn(&RenderInfo) -> FscmsRes<String>;

fn read_to_string(fpath: &str) -> FscmsRes<String> {
    let fcontent: String = match fs::read_to_string(fpath) {
        Ok(contents) => contents,
        Err(error_desc) => {
            return Err(format!(
                "Error: plugin could not read file {:?}, because of error: {:?}",
                fpath, error_desc
            ))
        }
    };

    Ok(fcontent.trim().to_string())
}

fn convert_to_rust_str(path: &Path) -> FscmsRes<&str> {
    if !is_system_file(path) {
        return Err(format!(
            "Should only call this function for system files, not for {:#?}",
            path,
        ));
    }

    let pstring = match path.to_str() {
        Some(v) => v,
        None => {
            return Err(format!(
                "Error: could not get path for system file {:#?}",
                path
            ))
        }
    };

    Ok(pstring)
}

fn plugin_txt(ri: &RenderInfo) -> FscmsRes<String> {
    let fcontent: String = match fs::read_to_string(ri.path) {
        Ok(contents) => contents,
        Err(error_desc) => {
            return Err(format!(
                "Error: plugin could not read file {:?}, because of error: {:?}",
                ri.path, error_desc
            ))
        }
    };

    let fcontent = fcontent.trim().to_string();

    let mut context = Context::new();
    let key = "content";
    let value = fcontent;
    context.insert(key, &value);
    let html = match Tera::one_off(HTML_TEMPLATE_PLUGIN_TXT, &context, true) {
        Ok(value) => Ok(value),
        Err(tera_error) => {
            return Err(format!(
                "Error in plugin_txt with artifact {:#?}. Tera render failed: {}",
                ri.path, tera_error
            ))
        }
    };

    return html;
}

fn plugin_png(ri: &RenderInfo) -> FscmsRes<String> {
    let fdata: Vec<u8> = match fs::read(ri.path) {
        Ok(contents) => contents,
        Err(error_desc) => {
            return Err(format!(
                "Error: plugin could not read file {:?}, because of error: {:?}",
                ri.path, error_desc
            ))
        }
    };

    let mut context = Context::new();
    let key = "content";
    let value = "";
    context.insert(key, &value);
    let html = match Tera::one_off(HTML_TEMPLATE_PLUGIN_TXT, &context, true) {
        Ok(value) => Ok(value),
        Err(tera_error) => {
            return Err(format!(
                "Error in plugin_txt with artifact {:#?}. Tera render failed: {}",
                ri.path, tera_error
            ))
        }
    };

    return html;
}

fn get_plugin_for_artifact(path: &Path) -> FscmsRes<ArtifactPlugin> {
    // use unwrap because we are expected to sanitise inputs before this step
    let (fname, fext) = get_name_extension(path).unwrap();
    let fpath = match path.to_str() {
        Some(s) => s,
        None => {
            return Err(format!(
                "Error: could not convert path {:#?} to rust utf-8 string.",
                path
            ))
        }
    };

    let selected_plugin = match fext {
        "txt" => plugin_txt,
        "png" => plugin_png,
        _ => {
            return Err(format!(
                "Artifact {:?}: no plugin found for file with extension {}.",
                path, fext
            ))
        }
    };

    return Ok(selected_plugin);
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

fn get_name_extension(path: &Path) -> FscmsRes<(&str, &str)> {
    let (fstem, fext) = match (path.file_stem(), path.extension()) {
        (Some(fs), Some(fe)) => (fs, fe),
        _ => {
            return Err(format!(
                "Error: could not extract file name or extension from path: {:#?}",
                path
            ))
        }
    };
    let (fstem, fext) = match (fstem.to_str(), fext.to_str()) {
        (Some(fs), Some(fe)) => (fs, fe),
        _ => {
            return Err(format!(
                "Error: could not convert os string to rust utf-8 string for path {:#?}",
                path
            ))
        }
    };

    Ok((fstem, fext))
}

fn is_artifact_file(path: &Path) -> bool {
    if !path.is_file() {
        return false;
    }
    let (fname, fext) = get_name_extension(path).unwrap();
    return !is_system_file(path)
        && !fname.starts_with("_")
        && !fname.starts_with(".")
        && (fext.contains("txt") || fext.contains("png") || fext.contains("jpeg"));
}

fn is_system_file(path: &Path) -> bool {
    if !path.is_file() {
        return false;
    };
    let (fname, _) = get_name_extension(path).unwrap();
    return path.is_file() && fname.starts_with("_");
}

fn is_template_file(path: &Path) -> bool {
    if !is_system_file(path) {
        return false;
    }
    let (fname, fext) = get_name_extension(path).unwrap();
    return fname.contains("template") && fext.contains("html");
}

fn is_style_file(path: &Path) -> bool {
    if !is_system_file(path) {
        return false;
    }
    let (fname, fext) = get_name_extension(path).unwrap();
    return fname.contains("style") && fext.contains("css");
}

fn is_layout_file(path: &Path) -> bool {
    if !is_system_file(path) {
        return false;
    }
    let (fname, fext) = get_name_extension(path).unwrap();
    return fname.contains("layout") && fext.contains("css");
}

// strict function that checks all paths, if a path is strange: no name || no extension
// we brake and return error description
fn validate_paths(workpaths: &Vec<PathBuf>) -> FscmsRes<bool> {
    unimplemented!();
    // for wp in workpaths {
    //     if ! path.is_file() {
    //         continue
    //     }

    //     let (fname, fext) = match (path.file_stem(), path.extension()) {
    //         (Some(name), Some(ext)) => (name, ext),
    //         _ => return Err(format!(
    //                 "Found path {:?}: file has bad path, name or extension. Please use utf-8 characters and specify 'file_name.extension' for all files.",
    //                 path
    //         ))
    //     };

    //     let ftype = if is_template_file(path) {
    //         Template
    //     } else if is_layout_file(path) {
    //         Layout
    //     } else if is_style_file(path) {
    //         Style
    //     } else if is_artifact_file(path) {
    //         Artifact
    //     } else {
    //         return Err(format!(
    //                 "Found path {:?}: Could not deternime internal file type for it.",
    //                 path
    //         ));
    //     }

    //     let wrapper = PathWrapper {
    //         path: valid_entry.into_path(),
    //         fname,
    //         fext,
    //     };
    // }

    // Ok(true)
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

    let template: PathBuf = working_paths
        .iter()
        .filter(|&path| is_template_file(path))
        .cloned()
        .collect::<Vec<PathBuf>>()[0]
        .clone();

    let style = working_paths
        .iter()
        .filter(|&path| is_style_file(path))
        .cloned()
        .collect::<Vec<PathBuf>>()[0]
        .clone();

    let layout: PathBuf = working_paths
        .iter()
        .filter(|&path| is_layout_file(path))
        .cloned()
        .collect::<Vec<PathBuf>>()[0]
        .clone();

    let mut artifacts: Vec<PathBuf> = working_paths
        .iter()
        .filter(|path| is_artifact_file(path))
        .cloned()
        .collect();

    // first sort the artifacts so they are inserted into html in order
    artifacts.sort();

    // process non-artifact paths
    let (template_fname, template_fext) = get_name_extension(&template)?;
    let template = convert_to_rust_str(&template)?;
    let layout = convert_to_rust_str(&layout)?;
    let style = convert_to_rust_str(&style)?;

    if verbose {
        println!("Explored paths: {:#?}", working_paths);
        println!("Found artifacts: {:#?}", artifacts);
        println!("Found template: {:#?}", template);
        println!("Found layout: {:#?}", layout);
        println!("Found style: {:#?}", style);
    }

    // actually process all artifacts
    //
    let tera_glob = format!("{}/*", input_dir);
    let mut tera = tera::compile_templates!(&tera_glob);

    // disable autoescaping completely
    tera.autoescape_on(vec![]);

    println!("{:#?}", tera);

    let mut rendered_artifacts = Vec::new();

    for art_path in artifacts.iter() {
        let plugin: ArtifactPlugin = get_plugin_for_artifact(art_path)?;
        let ri = RenderInfo {
            input_dir: &input_dir,
            output_dir: &output_dir,
            // unwrap guaranteed because input was sanitised in previous steps
            path: art_path.to_str().unwrap(),
        };
        // the plugin is responsible for coping files, etc.
        let html = plugin(&ri)?;
        rendered_artifacts.push(html);
    }

    // prepare to render the final html
    let mut context = Context::new();
    let key = "page_title";
    let value = input_dir;
    context.insert(key, &value);

    let key = "artifacts";
    let value = rendered_artifacts;
    context.insert(key, &value);

    let key = "page_style";
    let value = read_to_string(style)?;
    context.insert(key, &value);

    let key = "page_script";
    let value = "";
    context.insert(key, &value);

    let key = "page_layout";
    let value = read_to_string(layout)?;
    context.insert(key, &value);

    let tera_template_name = format!("{}.{}", template_fname, template_fext);
    let html = match tera.render(&tera_template_name, &context) {
        Ok(value) => value,
        Err(tera_error) => {
            return Err(format!(
                "Error: main tera render failed because {}",
                tera_error
            ))
        }
    };

    if verbose {
        println!("Rendered main template:\n{}", html);
    }

    // for templ_path in

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
