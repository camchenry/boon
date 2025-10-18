#![warn(clippy::all, clippy::pedantic, clippy::nursery, clippy::cargo)]
#![allow(
    clippy::non_ascii_literal,
    clippy::missing_docs_in_private_items,
    clippy::implicit_return,
    clippy::print_stdout,
    clippy::expect_used
)]
pub mod macos;
pub mod windows;

use crate::types::{Bitness, BuildSettings, BuildStatistics, LoveVersion, Platform, Project};

use directories::ProjectDirs;

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
        (LoveVersion::V11_5, Platform::Windows, Bitness::X64) => "love-11.5-win64",
        (LoveVersion::V11_5, Platform::Windows, Bitness::X86) => "love-11.5-win32",

        (LoveVersion::V11_4, Platform::Windows, Bitness::X64) => "love-11.4-win64",
        (LoveVersion::V11_4, Platform::Windows, Bitness::X86) => "love-11.4-win32",

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
    format!("{}.love", project.title.clone())
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

pub fn get_boon_data_path() -> Result<PathBuf> {
    ProjectDirs::from("", "", "boon").map_or_else(
        || Err(anyhow::anyhow!("Could not get app data directory")),
        |project_dirs| Ok(project_dirs.data_local_dir().to_path_buf()),
    )
}

/// Get a platform-specific path to the app cache directory where LÖVE is stored.
pub fn get_love_version_path(
    version: LoveVersion,
    platform: Platform,
    bitness: Bitness,
) -> Result<PathBuf> {
    let filename = get_love_version_file_name(version, platform, bitness);
    let boon_path = get_boon_data_path()
        .with_context(|| format!("Could not get version directory for LÖVE version {version}"))?;
    Ok(boon_path.join(version.to_string()).join(filename))
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
    println!("Outputting LÖVE as {dst_file}");

    collect_zip_directory(src_dir, dst_file, method, &build_settings.ignore_list)
        .with_context(|| format!("Error while zipping files from `{src_dir}` to `{dst_file}`"))??;

    let build_metadata = std::fs::metadata(dst_file)
        .with_context(|| format!("Failed to read file metadata for '{dst_file}'"))?;

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

use std::collections::HashSet as StdHashSet;
use std::fs::{self, Metadata};
use std::os::unix::fs::FileTypeExt;
use std::os::unix::fs::PermissionsExt;

fn zip_directory<T>(
    it: &mut dyn Iterator<Item = DirEntry>,
    prefix: &str,
    mut writer: T,
    method: zip::CompressionMethod,
    ignore_list: &HashSet<String>,
) -> zip::result::ZipResult<()>
where
    T: Write + Seek,
{
    let mut zip = zip::ZipWriter::new(&mut writer);
    let mut buffer = Vec::new();

    let mut visited_dirs: StdHashSet<PathBuf> = StdHashSet::new();
    let mut zipped_paths: StdHashSet<PathBuf> = StdHashSet::new();

    fn add_path_to_zip<T: Write + Seek>(
        zip: &mut zip::ZipWriter<&mut T>,
        path: &Path,
        name: &Path,
        method: zip::CompressionMethod,
        ignore_list: &HashSet<String>,
        buffer: &mut Vec<u8>,
        visited_dirs: &mut StdHashSet<PathBuf>,
        zipped_paths: &mut StdHashSet<PathBuf>,
    ) -> zip::result::ZipResult<()> {
        let meta = fs::symlink_metadata(path)?;
        let file_type = meta.file_type();
        let name_str = name.to_str().expect("Could not do string conversion");
        if should_exclude_file(name_str, ignore_list) {
            println!("Excluding {}", name.display());
            return Ok(());
        }

        if file_type.is_symlink() {
            let target = fs::read_link(path)?;
            let target_path = if target.is_absolute() {
                target
            } else {
                path.parent().unwrap_or(Path::new("")).join(target)
            };
            let canon = fs::canonicalize(&target_path)?;
            if !zipped_paths.insert(canon.clone()) {
                // Already zipped this file/dir
                return Ok(());
            }
            let target_meta = fs::metadata(&target_path)?;
            if target_meta.is_dir() {
                // Avoid cycles
                if !visited_dirs.insert(canon.clone()) {
                    println!("Skipping already visited symlinked dir: {}", canon.display());
                    return Ok(());
                }
                // Add directory entry
                let dir_name = name_str.trim_end_matches('/').to_owned() + "/";
                zip.add_directory(dir_name.clone(), FileOptions::default().compression_method(method).unix_permissions(0o777))?;
                for entry in WalkDir::new(&target_path) {
                    let entry = entry.map_err(|e| zip::result::ZipError::Io(e.into()))?;
                    let entry_path = entry.path();
                    let entry_name = name.join(entry_path.strip_prefix(&target_path).unwrap());
                    add_path_to_zip(zip, entry_path, &entry_name, method, ignore_list, buffer, visited_dirs, zipped_paths)?;
                }
            } else {
                // Symlink to file
                zip.start_file(name_str, FileOptions::default().compression_method(method).unix_permissions(0o755))?;
                let mut f = File::open(&target_path)?;
                f.read_to_end(buffer)?;
                zip.write_all(buffer)?;
                buffer.clear();
            }
        } else if file_type.is_dir() {
            let canon = fs::canonicalize(path)?;
            if !zipped_paths.insert(canon.clone()) {
                // Already zipped this dir
                return Ok(());
            }
            // Avoid cycles
            if !visited_dirs.insert(canon.clone()) {
                println!("Skipping already visited dir: {}", canon.display());
                return Ok(());
            }
            // Add directory entry
            let dir_name = name_str.trim_end_matches('/').to_owned() + "/";
            zip.add_directory(dir_name.clone(), FileOptions::default().compression_method(method).unix_permissions(0o777))?;
            for entry in fs::read_dir(path).map_err(zip::result::ZipError::Io)? {
                let entry = entry.map_err(zip::result::ZipError::Io)?;
                let entry_path = entry.path();
                let entry_name = name.join(entry.file_name());
                add_path_to_zip(zip, &entry_path, &entry_name, method, ignore_list, buffer, visited_dirs, zipped_paths)?;
            }
        } else if file_type.is_file() {
            let canon = fs::canonicalize(path)?;
            if !zipped_paths.insert(canon.clone()) {
                // Already zipped this file
                return Ok(());
            }
            zip.start_file(name_str, FileOptions::default().compression_method(method).unix_permissions(0o666))?;
            let mut f = File::open(path)?;
            f.read_to_end(buffer)?;
            zip.write_all(buffer)?;
            buffer.clear();
        }
        Ok(())
    }

    for entry in it {
        let path = entry.path();
        let name = path.strip_prefix(Path::new(prefix)).expect("Could not get path suffix");
        add_path_to_zip(&mut zip, &path, name, method, ignore_list, &mut buffer, &mut visited_dirs, &mut zipped_paths)?;
    }
    zip.finish()?;
    Ok(())
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
    let file = File::create(path)
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
