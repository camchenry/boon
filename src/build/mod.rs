#![warn(
    clippy::all,
    clippy::restriction,
    clippy::pedantic,
    clippy::nursery,
    clippy::cargo
)]
#![allow(
    clippy::missing_docs_in_private_items,
    clippy::print_stdout,
    clippy::non_ascii_literal,
    clippy::implicit_return
)]

pub mod macos;
pub mod windows;

use crate::types::*;

use crate::APP_INFO;
use app_dirs::*;

use std::io::prelude::*;
use std::io::{Seek, Write};
use std::iter::Iterator;

use std::fs::File;
use std::path::{Path, PathBuf};

use walkdir::{DirEntry, WalkDir};
use zip::result::ZipError;
use zip::write::FileOptions;

/// Get the folder name of where a version of LÖVE is stored in the app cache
pub fn get_love_version_file_name(
    version: &LoveVersion,
    platform: &Platform,
    bitness: &Bitness,
) -> String {
    match (version, platform, bitness) {
        (&LoveVersion::V11_2, &Platform::Windows, &Bitness::X64) => "love-11.2.0-win64",
        (&LoveVersion::V11_2, &Platform::Windows, &Bitness::X86) => "love-11.2.0-win32",

        (&LoveVersion::V11_1, &Platform::Windows, &Bitness::X64) => "love-11.1.0-win64",
        (&LoveVersion::V11_1, &Platform::Windows, &Bitness::X86) => "love-11.1.0-win32",

        (&LoveVersion::V11_0, &Platform::Windows, &Bitness::X64) => "love-11.0.0-win64",
        (&LoveVersion::V11_0, &Platform::Windows, &Bitness::X86) => "love-11.0.0-win32",

        (&LoveVersion::V0_10_2, &Platform::Windows, &Bitness::X64) => "love-0.10.2-win64",
        (&LoveVersion::V0_10_2, &Platform::Windows, &Bitness::X86) => "love-0.10.2-win32",

        (_, &Platform::MacOs, _) => "love.app",
    }
    .to_owned()
}

/// Get file name of the .love file (same for all platforms)
pub fn get_love_file_name(project: &Project) -> String {
    format!("{}.love", project.title.to_owned())
}

/// Get file name for individual binary based on platform and bitness
pub fn get_output_filename(project: &Project, platform: &Platform, bitness: &Bitness) -> String {
    match (platform, bitness) {
        (&Platform::Windows, &Bitness::X64) => format!("{}.exe", project.package_name),
        (&Platform::Windows, &Bitness::X86) => format!("{}.exe", project.package_name),
        (&Platform::MacOs, _) => format!("{}.app", project.title),
    }
}

/// Get file name of the distributed .zip file based on platform and bitness
pub fn get_zip_output_filename(
    project: &Project,
    platform: &Platform,
    bitness: &Bitness,
) -> String {
    match (platform, bitness) {
        (&Platform::Windows, &Bitness::X64) => format!("{}-win64", project.title),
        (&Platform::Windows, &Bitness::X86) => format!("{}-win32", project.title),
        (&Platform::MacOs, _) => format!("{}-macos", project.title),
    }
}

/// Get a platform-specific path to the app cache directory where LÖVE is stored.
pub fn get_love_version_path(
    version: &LoveVersion,
    platform: &Platform,
    bitness: &Bitness,
) -> PathBuf {
    let filename = get_love_version_file_name(version, platform, bitness);

    // @DoNotFix: The forward slash here is intentional. It will get escaped by
    // get_app_dir automatically to match the OS preference.
    let subdirectory = format!("{}/{}", &version.to_string(), &filename);
    get_app_dir(AppDataType::UserData, &APP_INFO, &subdirectory).unwrap()
}

pub fn scan_files(project: &Project, _build_settings: &BuildSettings) {
    // Check for main.lua in directory root
    let main_lua_file = PathBuf::new().join(&project.directory).join("main.lua");

    if !main_lua_file.exists() {
        eprintln!("Could not find main.lua in project root.");
        std::process::exit(1);
    }
}

pub fn init(project: &Project, build_settings: &BuildSettings) {
    // Currently does nothing. This step would be where the build process
    // would be halted for some reason (dirty files, etc.).
    scan_files(&project, &build_settings);

    // Ensure release directory exists.
    let release_dir_path = project.get_release_path(&build_settings);

    if !release_dir_path.exists() {
        println!("Creating release directory {}", release_dir_path.display());

        match std::fs::create_dir(&release_dir_path) {
            Ok(_) => {}
            Err(err) => {
                eprintln!("Could not create release directory: '{}'", err);
                std::process::exit(1);
            }
        };
    }
}

//
// LÖVE .love build
//
pub fn create_love(project: &Project, build_settings: &BuildSettings) -> BuildStatistics {
    // Stats
    let start = std::time::Instant::now();

    let method = zip::CompressionMethod::Deflated;

    let src_dir = &project.directory;
    let love_path = project
        .get_release_path(build_settings)
        .join(get_love_file_name(&project));
    let dst_file = love_path.to_str().unwrap();
    println!("Outputting LÖVE as {}", dst_file);

    match collect_zip_directory(src_dir, dst_file, method, &build_settings.ignore_list) {
        Ok(_) => {
            println!("done: {} written to {}", src_dir, dst_file);
        }
        Err(e) => {
            println!("Error: {:?}", e);
        }
    }

    BuildStatistics {
        build_name: String::from("LÖVE"),
        build_time: start.elapsed(),
    }
}

fn should_exclude_file(file_name: String, ignore_list: &Vec<String>) -> bool {
    for exclude_pattern in ignore_list {
        // @Performance @TODO: Could cache regex in a multi-build to
        // avoid recompiling the same patterns
        let re = regex::Regex::new(exclude_pattern).unwrap();
        if re.is_match(file_name.as_str()) {
            return true;
        }
    }

    false
}

fn zip_directory<T>(
    it: &mut Iterator<Item = DirEntry>,
    prefix: &str,
    writer: T,
    method: zip::CompressionMethod,
    ignore_list: &Vec<String>,
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
        let name = path.strip_prefix(Path::new(prefix)).unwrap();

        if path.is_file() && !should_exclude_file(name.to_str().unwrap().to_string(), &ignore_list)
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
    ignore_list: &Vec<String>,
) -> zip::result::ZipResult<()> {
    if !Path::new(src_dir).is_dir() {
        return Err(ZipError::FileNotFound);
    }

    let path = Path::new(dst_file);
    let file = File::create(&path).unwrap();

    let walkdir = WalkDir::new(src_dir.to_string());
    let it = walkdir.into_iter();

    zip_directory(
        &mut it.filter_map(|e| e.ok()),
        src_dir,
        file,
        method,
        ignore_list,
    )?;

    Ok(())
}

impl Project {
    fn get_release_path(&self, build_settings: &BuildSettings) -> PathBuf {
        let mut path = Path::new(self.directory.as_str()).canonicalize().unwrap();
        path.push(build_settings.output_directory.as_str());
        path
    }
}

impl BuildStatistics {
    pub fn display(&self) {
        println!("Built '{}' in {:?}", self.build_name, self.build_time);
    }
}
