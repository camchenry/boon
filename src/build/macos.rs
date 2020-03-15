#![allow(clippy::result_expect_used)]
use crate::build::*;
use crate::types::*;
use anyhow::{ensure, Result};
use std::io::{Read, Write};

//
// macOS .app build
//
pub fn create_app(
    project: &Project,
    build_settings: &BuildSettings,
    version: LoveVersion,
    bitness: Bitness,
) -> Result<BuildStatistics> {
    // Stats
    let start = std::time::Instant::now();

    let love_path = get_love_version_path(version, Platform::MacOs, bitness)?;
    ensure!(love_path.exists(), format!("LÖVE not found at '{}'\nhint: You may need to download LÖVE first: `boon love download {}`", love_path.display(), version.to_string()));

    let output_file_name = get_output_filename(project, Platform::MacOs, bitness);
    let output_path = project.get_release_path(build_settings);
    let mut final_output_path = project.get_release_path(build_settings);
    final_output_path.push(output_file_name);

    println!(
        "Copying LÖVE from {} to {}",
        love_path.display(),
        output_path.display()
    );

    let mut copy_options = fs_extra::dir::CopyOptions::new();
    copy_options.overwrite = true;
    fs_extra::dir::copy(&love_path, &output_path, &copy_options)?;

    let mut local_love_app_path = project.get_release_path(build_settings);
    local_love_app_path.push(
        love_path
            .file_name()
            .context("Could not get file name")?
            .to_str()
            .context("Could not do string conversion")?,
    );

    if final_output_path.exists() {
        println!("Removing output path '{}'", final_output_path.display());
        std::fs::remove_dir_all(&final_output_path)?;
    }

    println!(
        "Renaming LÖVE from {} to {}",
        local_love_app_path.display(),
        final_output_path.display()
    );
    std::fs::rename(&local_love_app_path, &final_output_path).with_context(|| {
        format!(
            "Failed to rename '{}' to '{}'",
            local_love_app_path.display(),
            final_output_path.display()
        )
    })?;

    let love_file_name = get_love_file_name(project);
    let mut local_love_file_path = project.get_release_path(build_settings);
    local_love_file_path.push(love_file_name);
    let mut resources_path = PathBuf::from(&final_output_path);
    resources_path.push("Contents");
    resources_path.push("Resources");
    resources_path.push(get_love_file_name(project));
    println!(
        "Copying .love file from {} to {}",
        local_love_file_path.display(),
        resources_path.display()
    );

    let mut copy_options = fs_extra::file::CopyOptions::new();
    copy_options.overwrite = true;
    fs_extra::file::copy(local_love_file_path, resources_path, &copy_options)?;

    // Rewrite plist file
    let mut plist_path = PathBuf::from(&final_output_path);
    plist_path.push("Contents");
    plist_path.push("Info.plist");

    println!("Rewriting {}", plist_path.display());

    let mut file = std::fs::OpenOptions::new().read(true).open(&plist_path)?;

    let buffer = rewrite_app_files(project, &mut file).with_context(|| {
        format!(
            "Could not rewrite macOS application info in '{}'",
            plist_path.display()
        )
    })?;

    let mut file = std::fs::OpenOptions::new()
        .write(true)
        .truncate(true)
        .open(&plist_path)?;

    file.write_all(buffer.as_bytes())?;

    Ok(BuildStatistics {
        build_name: format!("macOS {}", bitness.to_string()),
        build_time: start.elapsed(),
    })
}

/// Rewrites the macOS application files to contain the project's info
fn rewrite_app_files(project: &Project, file: &mut File) -> Result<String> {
    let mut buffer = String::new();
    file.read_to_string(&mut buffer)?;
    let re = regex::Regex::new("(CFBundleIdentifier.*\n\t<string>)(.*)(</string>)")
        .context("Could not create regex")?;
    buffer = re
        .replace(buffer.as_str(), |caps: &regex::Captures| {
            [
                caps.get(1).expect("Could not get capture").as_str(),
                project.uti.as_str(),
                caps.get(3).expect("Could not get capture").as_str(),
            ]
            .join("")
        })
        .to_string();
    let re = regex::Regex::new("(CFBundleName.*\n\t<string>)(.*)(</string>)")
        .context("Could not create regex")?;
    buffer = re
        .replace(buffer.as_str(), |caps: &regex::Captures| {
            [
                caps.get(1).expect("Could not get capture").as_str(),
                project.title.as_str(),
                caps.get(3).expect("Could not get capture").as_str(),
            ]
            .join("")
        })
        .to_string();
    let re = regex::RegexBuilder::new("^\t<key>UTExportedTypeDeclarations.*(\n.*)+\t</array>\n")
        .multi_line(true)
        .build()
        .context("Could not build regex")?;
    buffer = re.replace(buffer.as_str(), "").to_string();
    Ok(buffer)
}
