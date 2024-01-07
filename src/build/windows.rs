#![allow(clippy::too_many_lines)]
use crate::build::{Iterator, collect_zip_directory, get_love_file_name, get_love_version_path, get_output_filename, get_zip_output_filename};
use crate::types::{Bitness, BuildSettings, BuildStatistics, LoveVersion, Platform, Project};
use glob::glob;
use remove_dir_all::remove_dir_all;

use anyhow::{anyhow, ensure, Context, Result};
use std::collections::HashSet;
use std::fs::File;
use std::io::{Read, Write};
use std::path::PathBuf;

//
// Windows .exe build
//
pub fn create_exe(
    project: &Project,
    build_settings: &BuildSettings,
    version: LoveVersion,
    bitness: Bitness,
) -> Result<BuildStatistics> {
    // Stats
    let start = std::time::Instant::now();

    let app_dir_path = get_love_version_path(version, Platform::Windows, bitness)?;

    let mut app_dir_path_clone = PathBuf::new();
    app_dir_path_clone.clone_from(&app_dir_path);

    let mut love_exe_path = app_dir_path;
    love_exe_path.push("love.exe");
    ensure!(love_exe_path.exists(), format!("love.exe not found at '{}'\nhint: You may need to download LÖVE first: `boon love download {}`", love_exe_path.display(), version.to_string()));

    let exe_file_name = get_output_filename(project, Platform::Windows, bitness);
    let zip_output_file_name = &get_zip_output_filename(project, Platform::Windows, bitness);
    let mut output_path = project.get_release_path(build_settings);
    output_path.push(zip_output_file_name);

    if output_path.exists() {
        println!("Removing existing directory {}", output_path.display());
        std::fs::remove_dir_all(&output_path).with_context(|| {
            format!(
                "Could not remove output directory '{}'",
                output_path.display()
            )
        })?;
    }

    // Create temp directory to be zipped and removed later
    std::fs::create_dir(&output_path).with_context(|| {
        format!(
            "Could not create build directory '{}'",
            output_path.display()
        )
    })?;

    output_path.push(exe_file_name);

    println!("Copying love from {}", love_exe_path.display());

    println!("Outputting exe to {}", output_path.display());
    let mut output_file = File::create(&output_path)
        .with_context(|| format!("Could not create output file '{}'", output_path.display()))?;

    let love_file_name = get_love_file_name(project);
    let mut local_love_file_path = project.get_release_path(build_settings);
    local_love_file_path.push(love_file_name);

    println!(
        "Copying project .love from {}",
        local_love_file_path.display()
    );

    let mut copy_options = fs_extra::file::CopyOptions::new();
    copy_options.overwrite = true;

    // copy all .dll, .txt, and .ico files from the love source
    let dll_glob = glob(
        app_dir_path_clone
            .join("*.dll")
            .to_str()
            .context("Could not convert string")?,
    )?;
    let txt_glob = glob(
        app_dir_path_clone
            .join("*.txt")
            .to_str()
            .context("Could not convert string")?,
    )?;
    let ico_glob = glob(
        app_dir_path_clone
            .join("*.ico")
            .to_str()
            .context("Could not convert string")?,
    )?;
    for entry in dll_glob.chain(txt_glob).chain(ico_glob) {
        match entry {
            Ok(path) => {
                let local_file_name = path
                    .file_name()
                    .with_context(|| {
                        format!("Could not get file name from path '{}'", path.display())
                    })?
                    .to_str()
                    .context("Could not do string conversion")?;

                fs_extra::file::copy(
                    &path,
                    &project
                        .get_release_path(build_settings)
                        .join(zip_output_file_name)
                        .join(local_file_name),
                    &copy_options,
                )?;
            }

            // if the path matched but was unreadable,
            // thereby preventing its contents from matching
            Err(e) => {
                return Err(anyhow!(
                    "Path matched for '{}' but file was unreadable: {}",
                    e.path().display(),
                    e.error()
                ))
            }
        }
    }

    let paths = &[love_exe_path.as_path(), local_love_file_path.as_path()];

    let mut buffer = Vec::new();
    for path in paths {
        if path.is_file() {
            let mut file = File::open(path)?;
            file.read_to_end(&mut buffer)?;
            output_file.write_all(&buffer)?;
            buffer.clear();
        }
    }

    // Time to zip up the whole directory
    let zip_output_file_name = get_zip_output_filename(project, Platform::Windows, bitness);
    let output_path = project
        .get_release_path(build_settings)
        .join(zip_output_file_name);

    let src_dir = output_path.clone();
    let src_dir = src_dir.to_str().context("Could not do string conversion")?;

    let mut dst_file_path = output_path;
    dst_file_path.set_extension("zip");
    let dst_file = dst_file_path
        .to_str()
        .context("Could not do string conversion")?;

    collect_zip_directory(
        src_dir,
        dst_file,
        zip::CompressionMethod::Deflated,
        &HashSet::new(),
    )
    .with_context(|| {
        format!(
            "Error while zipping files from `{src_dir}` to `{dst_file}`"
        )
    })??;
    let path = PathBuf::new().join(src_dir);
    println!("Removing {}", path.display());
    remove_dir_all(&path)?;

    let build_metadata = std::fs::metadata(dst_file)
        .with_context(|| format!("Failed to read file metadata for '{dst_file}'"))?;

    Ok(BuildStatistics {
        name: format!("Windows {bitness}"),
        // @TODO: There is probably a better way here
        file_name: dst_file_path
            .file_name()
            .unwrap()
            .to_str()
            .unwrap()
            .to_string(),
        time: start.elapsed(),
        size: build_metadata.len(),
    })
}
