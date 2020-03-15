#![warn(
    clippy::all,
    clippy::restriction,
    clippy::pedantic,
    clippy::nursery,
    clippy::cargo
)]
#![allow(
    clippy::non_ascii_literal,
    clippy::missing_docs_in_private_items,
    clippy::implicit_return,
    clippy::print_stdout,
    clippy::module_name_repetitions,
    clippy::result_expect_used
)]
mod types;
use crate::types::*;

mod build;
mod download;

use anyhow::{Context, Result};
use app_dirs::*;
use clap::{crate_version, App, Arg, ArgMatches, SubCommand};
use config::Config;
use humansize::{file_size_opts, FileSize};
use prettytable::{cell, row, Table};
use remove_dir_all::*;
use std::fs::File;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

const APP_INFO: AppInfo = AppInfo {
    name: "boon",
    author: "boon",
};

const BOON_CONFIG_FILE_NAME: &str = "Boon.toml";
const DEFAULT_CONFIG: &str = include_str!(concat!("../", "Boon.toml"));
const LOVE_VERSIONS: &[&str] = &["11.3", "11.2", "11.1", "11.0", "0.10.2"];
const DEFAULT_LOVE_VERSION: &str = "11.3"; // Update with each new version
const BUILD_TARGETS: &[&str] = &["love", "windows", "macos", "all"];

fn main() -> Result<()> {
    // load in config from Settings file
    let (settings, build_settings) =
        get_settings().context("Could not load project settings or build settings")?;

    let subcmd_build = SubCommand::with_name("build")
        .about("Build game for a target platform")
        .arg(
            Arg::from_usage("-t, --target 'Specify which target platform to build for'")
                .required(true)
                .possible_values(BUILD_TARGETS)
                .default_value("love"),
        )
        .arg(Arg::with_name("DIRECTORY").required(true).takes_value(true))
        .arg(
            Arg::from_usage("-v, --version 'Specify which target version of LÖVE to build for'")
                .default_value(DEFAULT_LOVE_VERSION)
                .possible_values(LOVE_VERSIONS),
        );

    let subcmd_love_download = SubCommand::with_name("download")
        .about("Download a version of LÖVE")
        .arg(
            Arg::with_name("VERSION")
                .required(true)
                .takes_value(true)
                .possible_values(LOVE_VERSIONS),
        );

    let subcmd_love_remove = SubCommand::with_name("remove")
        .about("Remove a version of LÖVE")
        .arg(
            Arg::with_name("VERSION")
                .required(true)
                .takes_value(true)
                .possible_values(LOVE_VERSIONS),
        );

    let subcmd_love = SubCommand::with_name("love")
        .about("Manage multiple LÖVE versions")
        .subcommand(subcmd_love_download)
        .subcommand(subcmd_love_remove);

    let subcmd_init = SubCommand::with_name("init").about("Initialize configuration for project");

    let subcmd_clean = SubCommand::with_name("clean").about("Remove built packages");

    let app_m = App::new("boon")
        .version(crate_version!())
        .author("Cameron McHenry")
        .about("boon: LÖVE2D build and deploy tool")
        .subcommand(subcmd_init)
        .subcommand(subcmd_build)
        .subcommand(subcmd_love)
        .subcommand(subcmd_clean)
        .get_matches();

    match app_m.subcommand() {
        ("init", _) => init().context("Failed to initialize boon configuration file")?,
        ("build", Some(subcmd)) => {
            build(&settings, &build_settings, subcmd).context("Failed to build project")?
        }
        ("love", Some(subcmd)) => {
            match subcmd.subcommand() {
                ("download", Some(love_subcmd)) => {
                    love_download(love_subcmd).context("Failed to download and install LÖVE")?
                }
                ("remove", Some(love_subcmd)) => {
                    love_remove(love_subcmd).context("Failed to remove LÖVE")?
                }
                _ => {
                    // List installed versions
                    let installed_versions = get_installed_love_versions()
                        .context("Could not get installed LÖVE versions")?;

                    println!("Installed versions:");
                    for version in installed_versions {
                        println!("* {}", version);
                    }
                }
            }
        }
        ("clean", Some(_subcmd)) => {
            clean(&build_settings).context("Failed to clean release directory")?
        }
        _ => {
            println!("No command supplied.");
            println!("{}", app_m.usage());
        }
    }

    Ok(())
}

/// Initializes the project settings and build settings.
// @TODO: Get values from local project config
fn get_settings() -> Result<(Config, BuildSettings)> {
    let mut settings = config::Config::new();
    let default_config = config::File::from_str(DEFAULT_CONFIG, config::FileFormat::Toml);
    settings.merge(default_config).context(format!(
        "Could not set default configuration `{}`",
        BOON_CONFIG_FILE_NAME
    ))?;

    let mut ignore_list: Vec<String> = settings.get("build.ignore_list").unwrap();
    if Path::new(BOON_CONFIG_FILE_NAME).exists() {
        // Add in `./Boon.toml`
        settings
            .merge(config::File::with_name(BOON_CONFIG_FILE_NAME))
            .context(format!(
                "Error while reading config file `{}`.",
                BOON_CONFIG_FILE_NAME
            ))?;

        let mut project_ignore_list: Vec<String> = settings.get("build.ignore_list").unwrap();

        if settings.get("build.exclude_default_ignore_list").unwrap() {
            ignore_list = project_ignore_list;
        } else {
            ignore_list.append(&mut project_ignore_list);
        }
    }

    let build_settings = BuildSettings {
        ignore_list,
        exclude_default_ignore_list: settings.get("build.exclude_default_ignore_list")?,
        output_directory: settings.get("build.output_directory")?,
    };

    Ok((settings, build_settings))
}

/// `boon clean` command
fn clean(build_settings: &BuildSettings) -> Result<()> {
    // @TODO: Get top-level directory from git?
    let directory = ".";
    let mut release_dir_path = Path::new(directory)
        .canonicalize()
        .context("Could not get canonical directory path")?;
    release_dir_path.push(build_settings.output_directory.as_str());

    if release_dir_path.exists() {
        println!("Cleaning {}", release_dir_path.display());
        remove_dir_all(&release_dir_path).with_context(|| {
            format!(
                "Could not clean release directory `{}`",
                release_dir_path.display()
            )
        })?;
        println!(
            "Release directory `{}` cleaned.",
            release_dir_path.display()
        );
    } else {
        println!(
            "Could not find expected release directory at `{}`",
            release_dir_path.display()
        );
    }

    Ok(())
}

/// `boon love remove` subcommand
fn love_remove(love_subcmd: &ArgMatches) -> Result<()> {
    let version = love_subcmd
        .value_of("VERSION")
        .context("Could not parse version string")?
        .parse::<LoveVersion>()
        .expect("Could not parse LoveVersion")
        .to_string();

    let installed_versions =
        get_installed_love_versions().context("Could not get installed LÖVE versions")?;

    if installed_versions.contains(&version) {
        let output_file_path = app_dir(AppDataType::UserData, &APP_INFO, "/")
            .context("Could not get app user data path")?;
        let path = PathBuf::new().join(output_file_path).join(&version);
        remove_dir_all(&path).with_context(|| {
            format!(
                "Could not remove installed version of LÖVE {} at path `{}`",
                version,
                path.display()
            )
        })?;
        println!("Removed LÖVE version {}.", version);
    } else {
        println!("LÖVE version '{}' is not installed.", version);
    }

    Ok(())
}

/// `boon love download` subcommand
fn love_download(love_subcmd: &ArgMatches) -> Result<()> {
    let version = love_subcmd
        .value_of("VERSION")
        .context("Could not parse version string")?
        .parse::<LoveVersion>()
        .expect("Could not parse LoveVersion");

    download::download_love(version, Platform::Windows, Bitness::X86).context(format!(
        "Could not download LÖVE {} for Windows (32-bit)",
        version.to_string()
    ))?;
    download::download_love(version, Platform::Windows, Bitness::X64).context(format!(
        "Could not download LÖVE {} for Windows (64-bit)",
        version.to_string()
    ))?;
    download::download_love(version, Platform::MacOs, Bitness::X64).context(format!(
        "Could not download LÖVE {} for macOS",
        version.to_string()
    ))?;

    println!(
        "\nLÖVE {} is now available for building.",
        version.to_string()
    );

    Ok(())
}

/// `boon init` command
fn init() -> Result<()> {
    if Path::new(BOON_CONFIG_FILE_NAME).exists() {
        println!("Project already initialized.");
    } else {
        File::create(BOON_CONFIG_FILE_NAME).context(format!(
            "Failed to create config file `{}`.",
            BOON_CONFIG_FILE_NAME
        ))?;
        std::fs::write(BOON_CONFIG_FILE_NAME, DEFAULT_CONFIG).context(format!(
            "Failed to write default configuration to `{}`.",
            BOON_CONFIG_FILE_NAME
        ))?;
    }

    Ok(())
}

/// `boon build` command
fn build(settings: &Config, build_settings: &BuildSettings, subcmd: &ArgMatches) -> Result<()> {
    let directory = subcmd
        .value_of("DIRECTORY")
        .context("Could not parse directory from command")?;
    let target = subcmd
        .value_of("target")
        .context("Could not parse target from command")?;
    let version = subcmd
        .value_of("version")
        .context("Could not parse version string")?
        .parse::<LoveVersion>()
        .expect("Could not parse LoveVersion");

    let is_build_all = target == "all";

    if is_build_all {
        println!("Building all targets from directory `{}`", directory);
    } else {
        println!(
            "Building target `{}` from directory `{}`",
            target, directory
        );
    }

    let project = Project {
        title: settings
            .get_str("project.title")
            .context("Could not get project title")?,
        package_name: settings
            .get_str("project.package_name")
            .context("Could not get project package name")?,
        directory: directory.to_string(),
        uti: settings
            .get_str("project.uti")
            .context("Could not get project UTI")?,
        authors: settings
            .get_str("project.authors")
            .context("Could not get project authors")?,
        description: settings
            .get_str("project.description")
            .context("Could not get project description")?,
        email: settings
            .get_str("project.email")
            .context("Could not get project email")?,
        url: settings
            .get_str("project.url")
            .context("Could not get project URL")?,
        version: settings
            .get_str("project.version")
            .context("Could not get project version")?,
    };

    build::init(&project, build_settings).with_context(|| {
        format!(
            "Failed to initialize the build process using build settings: {}",
            build_settings
        )
    })?;

    let mut stats_list = Vec::new();

    if is_build_all {
        build_love(build_settings, &project, &mut stats_list)?;
        build_windows(build_settings, version, &project, &mut stats_list)?;
        build_macos(build_settings, version, &project, &mut stats_list)?;
    } else {
        if target == "love" {
            build_love(build_settings, &project, &mut stats_list)?;
        }
        if target == "windows" {
            build_love(build_settings, &project, &mut stats_list)?;
            build_windows(build_settings, version, &project, &mut stats_list)?;
        }
        if target == "macos" {
            build_love(build_settings, &project, &mut stats_list)?;
            build_macos(build_settings, version, &project, &mut stats_list)?;
        }
    }

    // Display build report
    display_build_report(stats_list).context("Failed to display build report")?;

    Ok(())
}

fn build_macos(
    build_settings: &BuildSettings,
    version: LoveVersion,
    project: &Project,
    stats_list: &mut Vec<BuildStatistics>,
) -> Result<()> {
    stats_list.push(
        build::macos::create_app(project, build_settings, version, Bitness::X64)
            .context("Failed to build for macOS")?,
    );
    Ok(())
}

fn build_windows(
    build_settings: &BuildSettings,
    version: LoveVersion,
    project: &Project,
    stats_list: &mut Vec<BuildStatistics>,
) -> Result<()> {
    stats_list.push(
        build::windows::create_exe(project, build_settings, version, Bitness::X86)
            .context("Failed to build for Windows 64-bit")?,
    );
    stats_list.push(
        build::windows::create_exe(project, build_settings, version, Bitness::X64)
            .context("Failed to build for Windows 32-bit")?,
    );
    Ok(())
}

fn build_love(
    build_settings: &BuildSettings,
    project: &Project,
    stats_list: &mut Vec<BuildStatistics>,
) -> Result<()> {
    stats_list
        .push(build::create_love(project, build_settings).context("Failed to build .love file")?);
    Ok(())
}

fn display_build_report(build_stats: Vec<BuildStatistics>) -> Result<()> {
    let mut build_report_table = Table::new();
    build_report_table.set_format(*prettytable::format::consts::FORMAT_NO_BORDER_LINE_SEPARATOR);
    build_report_table.set_titles(row!["Build", "File", "Time", "Size"]);

    for stats in build_stats {
        let time = if stats.time.as_millis() < 1000 {
            format!("{:6} ms", stats.time.as_millis())
        } else {
            format!("{:6.2}  s", stats.time.as_secs_f64())
        };
        let size = stats
            .size
            .file_size(file_size_opts::CONVENTIONAL)
            .expect("Could not format build file size");
        build_report_table.add_row(row![
            stats.name,
            stats.file_name,
            r->time, // Right aligned
            r->size // Right aligned
        ]);
    }

    println!();
    build_report_table.printstd();
    Ok(())
}

fn get_installed_love_versions() -> Result<Vec<String>> {
    let mut installed_versions: Vec<String> = Vec::new();
    let output_file_path =
        app_dir(AppDataType::UserData, &APP_INFO, "/").expect("Could not get app directory path");
    let walker = WalkDir::new(output_file_path).max_depth(1).into_iter();
    for entry in walker {
        let entry = entry.expect("Could not get DirEntry");
        if entry.depth() == 1 {
            let file_name = entry
                .file_name()
                .to_str()
                .with_context(|| format!("Could not parse file name `{:?}` to str", entry))?;

            // Exclude directories that do not parse to a love
            // version, just in case some bogus directories
            // got in there somehow.
            if let Ok(version) = file_name.parse::<LoveVersion>() {
                installed_versions.push(version.to_string())
            }
        }
    }

    Ok(installed_versions)
}
