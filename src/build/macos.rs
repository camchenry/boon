use crate::build::*;
use crate::types::*;
use std::io::{Read, Write};

//
// macOS .app build
//
pub fn create_app(
    project: &Project,
    build_settings: &BuildSettings,
    version: &LoveVersion,
    bitness: &Bitness,
) -> BuildStatistics {
    // Stats
    let start = std::time::Instant::now();

    let love_path = get_love_version_path(version, &Platform::MacOs, bitness);
    if !love_path.exists() {
        eprintln!("\nLÖVE not found at '{}'\nYou may need to download LÖVE first: `boon love download {}`", love_path.display(), version.to_string());
        std::process::exit(1);
    }

    let output_file_name = get_output_filename(project, &Platform::MacOs, bitness);
    let output_path = project.get_release_path(build_settings);
    let mut final_output_path = PathBuf::from(project.get_release_path(build_settings));
    final_output_path.push(output_file_name);

    println!(
        "Copying LÖVE from {} to {}",
        love_path.display(),
        output_path.display()
    );

    let mut copy_options = fs_extra::dir::CopyOptions::new();
    copy_options.overwrite = true;
    match fs_extra::dir::copy(&love_path, &output_path, &copy_options) {
        Ok(_) => {}
        Err(err) => {
            eprintln!("{:?}", err);
            std::process::exit(1);
        }
    };

    let mut local_love_app_path = PathBuf::from(project.get_release_path(build_settings));
    local_love_app_path.push(love_path.file_name().unwrap().to_str().unwrap());

    if final_output_path.exists() {
        println!("Removing {}", final_output_path.display());
        match std::fs::remove_dir_all(&final_output_path) {
            Ok(_) => {}
            Err(err) => {
                eprintln!("{:?}", err);
                std::process::exit(1);
            }
        };
    }

    println!(
        "Renaming LÖVE from {} to {}",
        local_love_app_path.display(),
        final_output_path.display()
    );
    match std::fs::rename(&local_love_app_path, &final_output_path) {
        Ok(_) => {}
        Err(err) => {
            eprintln!("{:?}", err);
            std::process::exit(1);
        }
    };

    let love_file_name = get_love_file_name(&project);
    let mut local_love_file_path = PathBuf::from(project.get_release_path(build_settings));
    local_love_file_path.push(love_file_name);
    let mut resources_path = PathBuf::from(&final_output_path);
    resources_path.push("Contents");
    resources_path.push("Resources");
    resources_path.push(get_love_file_name(&project));
    println!(
        "Copying .love file from {} to {}",
        local_love_file_path.display(),
        resources_path.display()
    );

    let mut copy_options = fs_extra::file::CopyOptions::new();
    copy_options.overwrite = true;
    match fs_extra::file::copy(local_love_file_path, resources_path, &copy_options) {
        Ok(_) => {}
        Err(err) => {
            eprintln!("{:?}", err);
            std::process::exit(1);
        }
    };

    // Rewrite plist file
    let mut plist_path = PathBuf::from(&final_output_path);
    plist_path.push("Contents");
    plist_path.push("Info.plist");

    println!("Rewriting {}", plist_path.display());

    let mut buffer = String::new();
    let mut file = match std::fs::OpenOptions::new().read(true).open(&plist_path) {
        Ok(file) => file,
        Err(why) => {
            eprintln!("Could not open file: {}", why);
            std::process::exit(1);
        }
    };

    match file.read_to_string(&mut buffer) {
        Ok(_) => {}
        Err(why) => {
            eprintln!("Could not read file: {}", why);
            std::process::exit(1);
        }
    };

    let re = regex::Regex::new("(CFBundleIdentifier.*\n\t<string>)(.*)(</string>)").unwrap();
    buffer = re
        .replace(buffer.as_str(), |caps: &regex::Captures| {
            [&caps[1], project.uti.as_str(), &caps[3]].join("")
        })
        .to_string();

    let re = regex::Regex::new("(CFBundleName.*\n\t<string>)(.*)(</string>)").unwrap();
    buffer = re
        .replace(buffer.as_str(), |caps: &regex::Captures| {
            [&caps[1], project.title.as_str(), &caps[3]].join("")
        })
        .to_string();

    let re = regex::RegexBuilder::new("^\t<key>UTExportedTypeDeclarations.*(\n.*)+\t</array>\n")
        .multi_line(true)
        .build()
        .unwrap();
    buffer = re.replace(buffer.as_str(), "").to_string();

    let mut file = match std::fs::OpenOptions::new()
        .write(true)
        .truncate(true)
        .open(&plist_path)
    {
        Ok(file) => file,
        Err(why) => {
            eprintln!("Could not open file: {}", why);
            std::process::exit(1);
        }
    };

    match file.write_all(buffer.as_bytes()) {
        Ok(_) => {}
        Err(why) => {
            eprintln!("Could not write output file: {}", why);
            std::process::exit(1);
        }
    };

    BuildStatistics {
        build_name: String::from(format!("macOS {}", bitness.to_string())),
        build_time: start.elapsed(),
    }
}
