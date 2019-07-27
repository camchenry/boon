use crate::build::*;
use crate::types::*;
use glob::glob;
use remove_dir_all::*;

use std::fs::File;
use std::io::{Read, Write};
use std::path::PathBuf;

//
// Windows .exe build
//
pub fn create_exe(
    project: &Project,
    build_settings: &BuildSettings,
    version: &LoveVersion,
    bitness: &Bitness,
) -> BuildStatistics {
    // Stats
    let start = std::time::Instant::now();

    let app_dir_path = get_love_version_path(version, &Platform::Windows, bitness);

    let mut app_dir_path_clone = PathBuf::new();
    app_dir_path_clone.clone_from(&app_dir_path);

    let mut love_exe_path = PathBuf::from(app_dir_path);
    love_exe_path.push("love.exe");
    if !love_exe_path.exists() {
        eprintln!("\nlove.exe not found at '{}'\nYou may need to download LÖVE first: `boon love download {}`", love_exe_path.display(), version.to_string());
        std::process::exit(1);
    }

    let exe_file_name = get_output_filename(project, &Platform::Windows, bitness);
    let zip_output_file_name = &get_zip_output_filename(project, &Platform::Windows, bitness);
    let mut output_path = project.get_release_path(build_settings);
    output_path.push(zip_output_file_name);

    println!("Removing existing directory {}", output_path.display());
    if output_path.exists() {
        match std::fs::remove_dir_all(&output_path) {
            Ok(_) => {}
            Err(err) => {
                eprintln!("Could not remove directory: '{}'", err);
                std::process::exit(1);
            }
        };
    }

    // Create temp directory to be zipped and removed later
    match std::fs::create_dir(&output_path) {
        Ok(_) => {}
        Err(err) => {
            eprintln!("Could not create build directory: '{}'", err);
            std::process::exit(1);
        }
    };

    output_path.push(exe_file_name);

    println!("Copying love from {}", love_exe_path.display());

    println!("Outputting exe to {}", output_path.display());
    let mut output_file = match File::create(&output_path) {
        Ok(file) => file,
        Err(why) => {
            eprintln!("Unable to create file `{}`: {}", output_path.display(), why);
            std::process::exit(1);
        }
    };

    let love_file_name = get_love_file_name(&project);
    let mut local_love_file_path = PathBuf::from(project.get_release_path(build_settings));
    local_love_file_path.push(love_file_name);

    println!(
        "Copying project .love from {}",
        local_love_file_path.display()
    );

    let mut copy_options = fs_extra::file::CopyOptions::new();
    copy_options.overwrite = true;

    // copy all .dll, .txt, and .ico files from the love source
    let search_for_files_dll = app_dir_path_clone.join("*.dll");
    let search_for_files_txt = app_dir_path_clone.join("*.txt");
    let search_for_files_ico = app_dir_path_clone.join("*.ico");
    for entry in glob(search_for_files_dll.to_str().unwrap())
        .unwrap()
        .chain(glob(search_for_files_txt.to_str().unwrap()).unwrap())
        .chain(glob(search_for_files_ico.to_str().unwrap()).unwrap())
    {
        match entry {
            Ok(path) => {
                let local_file_name = path.file_name().unwrap().to_str().unwrap();
                //println!("Local file name: {}", local_file_name);
                //println!("copying {:?} to {}", path.display(), project.get_release_path().join(zip_output_file_name).join(local_file_name).display());

                match fs_extra::file::copy(
                    &path,
                    &project
                        .get_release_path(build_settings)
                        .join(zip_output_file_name)
                        .join(local_file_name),
                    &copy_options,
                ) {
                    Ok(_) => {}
                    Err(err) => {
                        eprintln!("{:?}", err);
                        std::process::exit(1);
                    }
                };
            }

            // if the path matched but was unreadable,
            // thereby preventing its contents from matching
            Err(e) => println!("{:?}", e),
        }
    }

    let paths = &[love_exe_path.as_path(), local_love_file_path.as_path()];

    let mut buffer = Vec::new();
    for path in paths {
        if path.is_file() {
            let mut file = match File::open(path) {
                Ok(file) => file,
                Err(why) => {
                    eprintln!("Could not open file: {}", why);
                    std::process::exit(1);
                }
            };

            match file.read_to_end(&mut buffer) {
                Ok(_) => {}
                Err(why) => {
                    eprintln!("Could not read file: {}", why);
                    std::process::exit(1);
                }
            };

            match output_file.write_all(&*buffer) {
                Ok(_) => {}
                Err(why) => {
                    eprintln!("Could not write output file: {}", why);
                    std::process::exit(1);
                }
            };

            buffer.clear();
        }
    }

    // Time to zip up the whole directory
    let zip_output_file_name = get_zip_output_filename(project, &Platform::Windows, bitness);
    let output_path = project
        .get_release_path(build_settings)
        .join(zip_output_file_name);

    let src_dir = output_path.clone();
    let src_dir = src_dir.to_str().unwrap();

    let mut dst_file = output_path.clone();
    dst_file.set_extension("zip");
    let dst_file = dst_file.to_str().unwrap();

    let method = zip::CompressionMethod::Deflated;
    let ignore_list: &Vec<String> = &vec![];
    match collect_zip_directory(src_dir, dst_file, method, ignore_list) {
        Ok(_) => {
            println!("done: {} written to {}", src_dir, dst_file);
        }
        Err(e) => {
            println!("Error: {:?}", e);
        }
    }
    let path = PathBuf::new().join(src_dir);
    println!("Removing {}", path.display());
    match remove_dir_all(&path) {
        Ok(_) => {}
        Err(err) => {
            eprintln!("{:?}", err);
            std::process::exit(1);
        }
    };

    BuildStatistics {
        build_name: String::from(format!("Windows {}", bitness.to_string())),
        build_time: start.elapsed(),
    }
}
