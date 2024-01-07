#![warn(clippy::all, clippy::pedantic, clippy::nursery, clippy::cargo)]
#![allow(
    clippy::non_ascii_literal,
    clippy::missing_docs_in_private_items,
    clippy::implicit_return,
    clippy::print_stdout,
    clippy::module_name_repetitions,
    clippy::expect_used
)]
mod types;
use crate::build::get_boon_data_path;
use crate::types::{
    Bitness, BuildSettings, BuildStatistics, LoveVersion, Platform, Project, Target, LOVE_VERSIONS,
};

mod build;
mod download;

use anyhow::{bail, Context, Result};
use config::Config;
use humansize::{file_size_opts, FileSize};
use prettytable::{row, Table};
use remove_dir_all::remove_dir_all;
use std::collections::HashSet;
use std::fs::File;
use std::path::{Path, PathBuf};
use structopt::StructOpt;
use walkdir::WalkDir;

#[derive(StructOpt, Debug)]
#[structopt(
    name = "boon",
    author = "Cameron McHenry",
    about = "boon: LÖVE2D build and deploy tool"
)]
enum BoonOpt {
    #[structopt(about = "Build game for a target platform")]
    Build {
        #[structopt(
            long,
            short,
            help="Specify which target platform to build for",
            possible_values=&Target::variants(),
            default_value="love"
        )]
        target: Target,
        #[structopt(
            long,
            short,
            help = "Specify which target version of LÖVE to build for",
            possible_values=LOVE_VERSIONS,
            default_value="11.5",
        )]
        version: LoveVersion,
        directory: String,
    },
    #[structopt(about = "Remove built packages")]
    Clean,
    #[structopt(about = "Initialize configuration for project")]
    Init,
    #[structopt(about = "Manage multiple LÖVE versions")]
    Love(LoveSubcommand),
}

#[derive(StructOpt, Debug)]
enum LoveSubcommand {
    #[structopt(about = "Download a version of LÖVE")]
    Download {
        #[structopt(possible_values=LOVE_VERSIONS)]
        version: LoveVersion,
    },
    #[structopt(about = "Remove a version of LÖVE")]
    Remove {
        #[structopt(possible_values=LOVE_VERSIONS)]
        version: LoveVersion,
    },
    #[structopt(about = "List installed LÖVE versions")]
    List,
}

const BOON_CONFIG_FILE_NAME: &str = "Boon.toml";
const DEFAULT_CONFIG: &str = include_str!(concat!("../", "Boon.toml"));

fn main() -> Result<()> {
    // load in config from Settings file
    let (settings, build_settings) =
        get_settings().context("Could not load project settings or build settings")?;

    match BoonOpt::from_args() {
        BoonOpt::Init => init().context("Failed to initialize boon configuration file")?,

        BoonOpt::Build {
            target,
            version,
            directory,
        } => build(&settings, &build_settings, target, version, directory)
            .context("Failed to build project")?,
        BoonOpt::Love(subcmd) => {
            match subcmd {
                LoveSubcommand::Download { version } => {
                    love_download(version).context("Failed to download and install LÖVE")?;
                }
                LoveSubcommand::Remove { version } => {
                    love_remove(version).context("Failed to remove LÖVE")?;
                }
                LoveSubcommand::List => {
                    // List installed versions
                    let installed_versions = get_installed_love_versions()
                        .context("Could not get installed LÖVE versions")?;

                    if installed_versions.is_empty() {
                        println!("No LÖVE versions installed.");
                    } else {
                        println!("Installed versions:");
                        for version in installed_versions {
                            println!("* {version}");
                        }
                    }
                }
            }
        }
        BoonOpt::Clean => clean(&build_settings).context("Failed to clean release directory")?,
    }

    Ok(())
}

/// Initializes the project settings and build settings.
// @TODO: Get values from local project config
fn get_settings() -> Result<(Config, BuildSettings)> {
    let mut settings = config::Config::new();
    let default_config = config::File::from_str(DEFAULT_CONFIG, config::FileFormat::Toml);
    settings.merge(default_config).context(format!(
        "Could not set default configuration `{BOON_CONFIG_FILE_NAME}`"
    ))?;

    let mut ignore_list: HashSet<String> = settings.get("build.ignore_list").unwrap();
    if Path::new(BOON_CONFIG_FILE_NAME).exists() {
        // Add in `./Boon.toml`
        settings
            .merge(config::File::with_name(BOON_CONFIG_FILE_NAME))
            .context(format!(
                "Error while reading config file `{BOON_CONFIG_FILE_NAME}`."
            ))?;

        let project_ignore_list: HashSet<String> = settings.get("build.ignore_list").unwrap();

        if settings.get("build.exclude_default_ignore_list").unwrap() {
            ignore_list = project_ignore_list;
        } else {
            ignore_list.extend(project_ignore_list);
        }
    }

    let hash_targets: HashSet<String> = settings.get("build.targets").unwrap();
    let mut targets: Vec<Target> = Vec::new();
    for target in &hash_targets {
        targets.push(match target.as_str() {
            "love" => Target::love,
            "windows" => Target::windows,
            "macos" => Target::macos,
            "all" => Target::all,
            _ => bail!("{} is not a valid build target.", target),
        });
    }

    let build_settings = BuildSettings {
        ignore_list,
        exclude_default_ignore_list: settings.get("build.exclude_default_ignore_list")?,
        output_directory: settings.get("build.output_directory")?,
        targets,
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
fn love_remove(version: LoveVersion) -> Result<()> {
    let version = version.to_string();
    let installed_versions =
        get_installed_love_versions().context("Could not get installed LÖVE versions")?;

    if installed_versions.contains(&version) {
        let output_file_path = get_boon_data_path()?;
        let path = PathBuf::new().join(output_file_path).join(&version);
        remove_dir_all(&path).with_context(|| {
            format!(
                "Could not remove installed version of LÖVE {} at path `{}`",
                version,
                path.display()
            )
        })?;
        println!("Removed LÖVE version {version}.");
    } else {
        println!("LÖVE version '{version}' is not installed.");
    }

    Ok(())
}

/// `boon love download` subcommand
fn love_download(version: LoveVersion) -> Result<()> {
    download::download_love(version, Platform::Windows, Bitness::X86).context(format!(
        "Could not download LÖVE {version} for Windows (32-bit)"
    ))?;
    download::download_love(version, Platform::Windows, Bitness::X64).context(format!(
        "Could not download LÖVE {version} for Windows (64-bit)"
    ))?;
    download::download_love(version, Platform::MacOs, Bitness::X64)
        .context(format!("Could not download LÖVE {version} for macOS"))?;

    println!("\nLÖVE {version} is now available for building.");

    Ok(())
}

/// `boon init` command
fn init() -> Result<()> {
    if Path::new(BOON_CONFIG_FILE_NAME).exists() {
        println!("Project already initialized.");
    } else {
        File::create(BOON_CONFIG_FILE_NAME).context(format!(
            "Failed to create config file `{BOON_CONFIG_FILE_NAME}`."
        ))?;
        std::fs::write(BOON_CONFIG_FILE_NAME, DEFAULT_CONFIG).context(format!(
            "Failed to write default configuration to `{BOON_CONFIG_FILE_NAME}`."
        ))?;
    }

    Ok(())
}

/// `boon build` command
fn build(
    settings: &Config,
    build_settings: &BuildSettings,
    target: Target,
    version: LoveVersion,
    directory: String,
) -> Result<()> {
    let mut targets = &build_settings.targets;
    let cmd_target = vec![target];
    if target != Target::love {
        targets = &cmd_target;
    }

    if targets.contains(&Target::all) {
        println!("Building all targets from directory `{directory}`");
    } else {
        println!("Building targets `{targets:?}` from directory `{directory}`");
    }

    let project = Project {
        title: settings
            .get_str("project.title")
            .context("Could not get project title")?,
        package_name: settings
            .get_str("project.package_name")
            .context("Could not get project package name")?,
        directory,
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
        format!("Failed to initialize the build process using build settings: {build_settings}")
    })?;

    let mut stats_list = Vec::new();

    build_love(build_settings, &project, &mut stats_list)?;

    if targets.contains(&Target::windows) || targets.contains(&Target::all) {
        build_windows(build_settings, version, &project, &mut stats_list)?;
    }

    if targets.contains(&Target::macos) || targets.contains(&Target::all) {
        build_macos(build_settings, version, &project, &mut stats_list)?;
    }

    // Display build report
    display_build_report(stats_list);

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

fn display_build_report(build_stats: Vec<BuildStatistics>) {
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
}

fn get_installed_love_versions() -> Result<Vec<String>> {
    let mut installed_versions: Vec<String> = Vec::new();
    let output_file_path = get_boon_data_path()?;
    let walker = WalkDir::new(output_file_path).max_depth(1).into_iter();
    for entry in walker {
        let entry = entry.expect("Could not get DirEntry");
        if entry.depth() == 1 {
            let file_name = entry
                .file_name()
                .to_str()
                .with_context(|| format!("Could not parse file name `{entry:?}` to str"))?;

            // Exclude directories that do not parse to a love
            // version, just in case some bogus directories
            // got in there somehow.
            if let Ok(version) = file_name.parse::<LoveVersion>() {
                installed_versions.push(version.to_string());
            }
        }
    }

    Ok(installed_versions)
}
