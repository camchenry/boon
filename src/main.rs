extern crate clap;
extern crate zip;
extern crate walkdir;

use clap::{App, Arg, SubCommand};

use std::io::prelude::*;
use std::iter::Iterator;
use std::io::{Write, Seek};
use zip::result::ZipError;
use zip::write::FileOptions;
use std::process::Command;

use walkdir::{WalkDir, DirEntry};
use std::path::Path;
use std::fs::File;

fn main() {
    let targets = &["love", "windows"];

    let app_m = App::new("love-kit")
        .version("1.0")
        .author("Cameron McHenry")
        .about("LÖVE2D Kit build/deploy tool")
        .subcommand(SubCommand::with_name("build")
                    .about("Build game for a target platform (win32, macos, ...)")
                    .arg(Arg::with_name("DIRECTORY")
                        .required(true)
                        .takes_value(true)
                        )
                    .arg(Arg::with_name("target")
                         .short("-t")
                         .required(true)
                         .possible_values(targets)
                         .max_values(8)
                         .min_values(1)
                    ))
        .get_matches();

    match app_m.subcommand() {
        ("build", Some(sub_m)) => {
            let directory = sub_m.value_of("DIRECTORY");
            let target = sub_m.value_of("target");
            match target {
                Some("love") => {
                    println!("Building target `{}` from directory `{}`", target.unwrap(), directory.unwrap());
                    match build_love(directory.unwrap().to_string()) {
                        _ => {}
                    }
                }
                Some("windows") => {
                    println!("Building target `{}` from directory `{}`", target.unwrap(), directory.unwrap());
                    match build_windows(directory.unwrap().to_string()) {
                        _ => {}
                    }
                }
                _ => {}
            }
        },

        // No command used
        _ => {
            println!("{}", app_m.usage());
        },
    }
}

fn build_love(directory: String) {
    const METHOD_DEFLATED: Option<zip::CompressionMethod> = Some(zip::CompressionMethod::Deflated);
    let method = METHOD_DEFLATED;

    let src_dir = &directory;
    let dst_file: &str = "test.zip";

    match doit(src_dir, dst_file, method.unwrap()) {
        Ok(_) => {
            println!("done: {} written to {}", src_dir, dst_file);
        },
        Err(e) => {
            println!("Error: {:?}", e);
        }
    }
}

fn build_windows(directory: String) {
    build_love(directory);

    let love_path = "C:/Program Files/LOVE/love.exe";

    let result: &str = &format!("{}+{}", love_path, "test.love");

    println!("Building for windows..{}", result);

    let output = if cfg!(target_os = "windows") {
        Command::new("cmd")
                .args(&["copy", "/b", result, "mygame.exe"])
                .output()
                .expect("failed to execute process")
    } else {
        // no clue if this will or should work
        Command::new("cat")
            .args(&[love_path, "test.love", ">", "mygame.exe"])
            .output()
            .expect("failed to execute process")
    };

    println!("{}", output.status);
}

fn zip_dir<T>(it: &mut Iterator<Item=DirEntry>, prefix: &str, writer: T, method: zip::CompressionMethod)
              -> zip::result::ZipResult<()>
    where T: Write+Seek
{
    let mut zip = zip::ZipWriter::new(writer);
    let options = FileOptions::default()
        .compression_method(method)
        .unix_permissions(0o644);

    let mut buffer = Vec::new();
    for entry in it {
        let path = entry.path();
        let name = path.strip_prefix(Path::new(prefix))
            .unwrap()
            .to_str()
            .unwrap();

        if path.is_file() {
            println!("adding {:?} as {:?} ...", path, name);
            zip.start_file(name, options)?;
            let mut f = File::open(path)?;

            f.read_to_end(&mut buffer)?;
            zip.write_all(&*buffer)?;
            buffer.clear();
        }
    }
    zip.finish()?;
    Result::Ok(())
}

fn doit(src_dir: &str, dst_file: &str, method: zip::CompressionMethod) -> zip::result::ZipResult<()> {
    if !Path::new(src_dir).is_dir() {
        return Err(ZipError::FileNotFound);
    }

    let path = Path::new(dst_file);
    let file = File::create(&path).unwrap();

    let walkdir = WalkDir::new(src_dir.to_string());
    let it = walkdir.into_iter();

    zip_dir(&mut it.filter_map(|e| e.ok()), src_dir, file, method)?;

    Ok(())
}
