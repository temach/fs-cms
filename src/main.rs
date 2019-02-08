use std::env;
use std::fs;

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

struct RenderInfo<'a> {
    fstem: &'a str,
    fext: &'a str,
    input_dir: &'a str,
    output_dir: &'a str,
    path: &'a str,
}

const HTML_TEMPLATE_PLUGIN_TXT: &str = r#"<p>{{ content }}</p>"#;

const HTML_TEMPLATE_PLUGIN_PNG: &str = r#"<img src="{{ content }}"/>"#;

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
    // set tera auto-escape to true because we are putting user supplied text into HTML
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
    // read png image data
    let png_data: Vec<u8> = match fs::read(ri.path) {
        Ok(contents) => contents,
        Err(error_desc) => {
            return Err(format!(
                "Error: plugin could not read file {:?}, because of error: {:?}",
                ri.path, error_desc
            ))
        }
    };

    // name of file without directory path

    let mut context = Context::new();
    let key = "content";
    let value = format!("./{}.{}", ri.fstem, ri.fext);
    context.insert(key, &value);
    // set tera auto-escape to false because we are putting the os supplied path to png into HTML
    let html = match Tera::one_off(HTML_TEMPLATE_PLUGIN_PNG, &context, false) {
        Ok(value) => value,
        Err(tera_error) => {
            return Err(format!(
                "Error in plugin_txt with artifact {:#?}. Tera render failed: {}",
                ri.path, tera_error
            ))
        }
    };

    // copy png data to output file
    let os_output_path = format!("{}/{}.{}", ri.output_dir, ri.fstem, ri.fext);
    if let Err(io_err) = fs::write(&os_output_path, png_data) {
        return Err(format!("IO error when writing png output: {}", io_err));
    };

    return Ok(html);
}

fn plugin_html(ri: &RenderInfo) -> FscmsRes<String> {
    let html_content: String = match fs::read_to_string(ri.path) {
        Ok(contents) => contents,
        Err(error_desc) => {
            return Err(format!(
                "Error: plugin could not read file {:?}, because of error: {:?}",
                ri.path, error_desc
            ))
        }
    };

    // just return the html directly
    return Ok(html_content);
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
        "html" => plugin_html,
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
        && (fext.contains("txt") || fext.contains("html") || fext.contains("png"));
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
    let mut input_dir: String = "./examples/gallery/input".to_string();
    let mut output_dir = "./examples/gallery/output".to_string();

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
    let tera_glob = format!("{}/*template*.html", input_dir);
    let mut tera = tera::compile_templates!(&tera_glob);

    // disable tera autoescaping completely
    tera.autoescape_on(vec![]);

    println!("{:#?}", tera);

    let mut rendered_artifacts = Vec::new();

    for art_path in artifacts.iter() {
        let plugin: ArtifactPlugin = get_plugin_for_artifact(art_path)?;

        // unwrap guaranteed because input was sanitised in previous steps
        let (art_stem, art_ext) = get_name_extension(art_path).unwrap();

        let ri = RenderInfo {
            fstem: art_stem,
            fext: art_ext,
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
        println!("Writing output to {}", output_dir);
    }

    let out_index = PathBuf::from(format!("{}/{}", output_dir, "index.html"));

    if verbose {
        println!("Output index file {}", out_index.to_string_lossy());
    }

    if let Err(io_err) = fs::write(out_index, html) {
        return Err(format!("IO error when writing output: {}", io_err));
    };

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
