#![warn(
    clippy::all,
    clippy::pedantic,
    clippy::nursery,
    clippy::cargo
)]
#![allow(
    clippy::non_ascii_literal,
    clippy::missing_docs_in_private_items,
    clippy::implicit_return,
    clippy::print_stdout,
    clippy::result_expect_used,
    clippy::option_expect_used
)]
pub mod macos;
pub mod windows;

use crate::types::*;

use crate::APP_INFO;
use app_dirs::*;

use std::io::prelude::*;
use std::io::{Seek, Write};
use std::iter::Iterator;

use anyhow::{ensure, Context, Result};
use std::collections::HashSet;
use std::fs::File;
use std::path::{Path, PathBuf};
use walkdir::{DirEntry, WalkDir};
use zip::result::ZipError;
use zip::write::FileOptions;

/// Get the folder name of where a version of LÖVE is stored in the app cache
pub fn get_love_version_file_name(
    version: LoveVersion,
    platform: Platform,
    bitness: Bitness,
) -> String {
    match (version, platform, bitness) {
        (LoveVersion::V11_3, Platform::Windows, Bitness::X64) => "love-11.3-win64",
        (LoveVersion::V11_3, Platform::Windows, Bitness::X86) => "love-11.3-win32",

        (LoveVersion::V11_2, Platform::Windows, Bitness::X64) => "love-11.2.0-win64",
        (LoveVersion::V11_2, Platform::Windows, Bitness::X86) => "love-11.2.0-win32",

        (LoveVersion::V11_1, Platform::Windows, Bitness::X64) => "love-11.1.0-win64",
        (LoveVersion::V11_1, Platform::Windows, Bitness::X86) => "love-11.1.0-win32",

        (LoveVersion::V11_0, Platform::Windows, Bitness::X64) => "love-11.0.0-win64",
        (LoveVersion::V11_0, Platform::Windows, Bitness::X86) => "love-11.0.0-win32",

        (LoveVersion::V0_10_2, Platform::Windows, Bitness::X64) => "love-0.10.2-win64",
        (LoveVersion::V0_10_2, Platform::Windows, Bitness::X86) => "love-0.10.2-win32",

        (_, Platform::MacOs, _) => "love.app",
    }
    .to_string()
}

/// Get file name of the .love file (same for all platforms)
pub fn get_love_file_name(project: &Project) -> String {
    format!("{}.love", project.title.to_owned())
}

/// Get file name for individual binary based on platform and bitness
pub fn get_output_filename(project: &Project, platform: Platform, bitness: Bitness) -> String {
    match (platform, bitness) {
        (Platform::Windows, _) => format!("{}.exe", project.package_name),
        (Platform::MacOs, _) => format!("{}.app", project.title),
    }
}

/// Get file name of the distributed .zip file based on platform and bitness
pub fn get_zip_output_filename(project: &Project, platform: Platform, bitness: Bitness) -> String {
    match (platform, bitness) {
        (Platform::Windows, Bitness::X64) => format!("{}-win64", project.title),
        (Platform::Windows, Bitness::X86) => format!("{}-win32", project.title),
        (Platform::MacOs, _) => format!("{}-macos", project.title),
    }
}

/// Get a platform-specific path to the app cache directory where LÖVE is stored.
pub fn get_love_version_path(
    version: LoveVersion,
    platform: Platform,
    bitness: Bitness,
) -> Result<PathBuf> {
    let filename = get_love_version_file_name(version, platform, bitness);

    // @DoNotFix: The forward slash here is intentional. It will get escaped by
    // get_app_dir automatically to match the OS preference.
    let subdirectory = format!("{}/{}", version.to_string(), &filename);
    Ok(get_app_dir(AppDataType::UserData, &APP_INFO, &subdirectory)
        .context("Could not get app directory")?)
}

pub fn scan_files(project: &Project) -> Result<()> {
    // Check for main.lua in directory root
    let main_lua_file = PathBuf::new().join(&project.directory).join("main.lua");

    ensure!(
        main_lua_file.exists(),
        "Could not find main.lua in project root."
    );

    Ok(())
}

pub fn init(project: &Project, build_settings: &BuildSettings) -> Result<()> {
    // Currently does nothing. This step would be where the build process
    // would be halted for some reason (dirty files, etc.).
    scan_files(project).context("Error found while scanning project files")?;

    // Ensure release directory exists.
    let release_dir_path = project.get_release_path(build_settings);

    if !release_dir_path.exists() {
        println!("Creating release directory {}", release_dir_path.display());

        std::fs::create_dir(&release_dir_path).with_context(|| {
            format!(
                "Could not create release directory `{}`",
                release_dir_path.display()
            )
        })?;
    }

    Ok(())
}

//
// LÖVE .love build
//
pub fn create_love(project: &Project, build_settings: &BuildSettings) -> Result<BuildStatistics> {
    // Stats
    let start = std::time::Instant::now();

    let method = zip::CompressionMethod::Deflated;

    let src_dir = &project.directory;
    let output_file_name = get_love_file_name(project);
    let love_path = project
        .get_release_path(build_settings)
        .join(&output_file_name);
    let dst_file = love_path
        .to_str()
        .context("Could not do string conversion")?;
    println!("Outputting LÖVE as {}", dst_file);

    collect_zip_directory(src_dir, dst_file, method, &build_settings.ignore_list).with_context(
        || {
            format!(
                "Error while zipping files from `{}` to `{}`",
                src_dir, dst_file
            )
        },
    )??;

    let build_metadata = std::fs::metadata(dst_file)
        .with_context(|| format!("Failed to read file metadata for '{}'", dst_file))?;

    Ok(BuildStatistics {
        name: String::from("LÖVE"),
        file_name: output_file_name,
        time: start.elapsed(),
        size: build_metadata.len(),
    })
}

fn should_exclude_file(file_name: &str, ignore_list: &HashSet<String>) -> bool {
    for exclude_pattern in ignore_list {
        // @Performance @TODO: Could cache regex in a multi-build to
        // avoid recompiling the same patterns
        let re = regex::Regex::new(exclude_pattern).expect("Could not compile regex pattern");
        if re.is_match(file_name) {
            return true;
        }
    }

    false
}

fn zip_directory<T>(
    it: &mut dyn Iterator<Item = DirEntry>,
    prefix: &str,
    writer: T,
    method: zip::CompressionMethod,
    ignore_list: &HashSet<String>,
) -> zip::result::ZipResult<()>
where
    T: Write + Seek,
{
    let mut zip = zip::ZipWriter::new(writer);
    let options = FileOptions::default()
        .compression_method(method)
        .unix_permissions(0o644);

    let mut buffer = Vec::new();
    for entry in it {
        let path = entry.path();
        let name = path
            .strip_prefix(Path::new(prefix))
            .expect("Could not get path suffix");

        if path.is_file()
            && !should_exclude_file(
                name.to_str().expect("Could not do string conversion"),
                ignore_list,
            )
        {
            zip.start_file_from_path(name, options)?;
            let mut f = File::open(path)?;

            f.read_to_end(&mut buffer)?;
            zip.write_all(&*buffer)?;
            buffer.clear();
        }
    }
    zip.finish()?;
    Result::Ok(())
}

fn collect_zip_directory(
    src_dir: &str,
    dst_file: &str,
    method: zip::CompressionMethod,
    ignore_list: &HashSet<String>,
) -> Result<zip::result::ZipResult<()>> {
    if !Path::new(src_dir).is_dir() {
        return Err(anyhow::Error::from(ZipError::FileNotFound));
    }

    let path = Path::new(dst_file);
    let file = File::create(&path)
        .with_context(|| format!("Could not create file path: '{}'", path.display()))?;

    let walkdir = WalkDir::new(src_dir);
    let it = walkdir.into_iter();

    zip_directory(
        &mut it.filter_map(std::result::Result::ok),
        src_dir,
        file,
        method,
        ignore_list,
    )?;

    Ok(Ok(()))
}

impl Project {
    fn get_release_path(&self, build_settings: &BuildSettings) -> PathBuf {
        let mut path = Path::new(self.directory.as_str())
            .canonicalize()
            .expect("Could not get canonical directory path");
        path.push(build_settings.output_directory.as_str());
        path
    }
}
