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

mod types;
use crate::types::*;

mod build;
mod download;

use app_dirs::*;
use clap::{App, Arg, SubCommand};
use remove_dir_all::*;
use std::fs::File;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

const APP_INFO: AppInfo = AppInfo {
    name: "boon",
    author: "boon",
};

const DEFAULT_CONFIG: &str = include_str!("../Boon.toml");

fn main() {
    // @TODO: Get values from local project config
    // load in config from Settings file
    let mut settings = config::Config::new();

    let default_config = config::File::from_str(DEFAULT_CONFIG, config::FileFormat::Toml);

    if settings.merge(default_config).is_err() {
        eprintln!("Could not set default configuration.");
        std::process::exit(1);
    }

    let mut ignore_list: Vec<String> = settings.get("build.ignore_list").unwrap();

    if Path::new("Boon.toml").exists() {
        // Add in `./Boon.toml`
        match settings.merge(config::File::with_name("Boon")) {
            Ok(_) => {}
            Err(e) => {
                eprintln!("Error reading config file: {}", e);
                std::process::exit(1);
            }
        };

        let mut project_ignore_list: Vec<String> = settings.get("build.ignore_list").unwrap();

        if settings.get("build.exclude_default_ignore_list").unwrap() {
            ignore_list = project_ignore_list;
        } else {
            ignore_list.append(&mut project_ignore_list);
        }
    }

    let build_settings = BuildSettings {
        ignore_list,
        exclude_default_ignore_list: settings.get("build.exclude_default_ignore_list").unwrap(),
        output_directory: settings.get("build.output_directory").unwrap(),
    };

    let targets = &["love", "windows", "macos"];

    let default_love_version = "11.2";
    let available_love_versions = &["11.2", "11.1", "11.0", "0.10.2"];

    let subcmd_build = SubCommand::with_name("build")
        .about("Build game for a target platform")
        .arg(
            Arg::from_usage("-t, --target 'Specify which target platform to build for'")
                .required(true)
                .possible_values(targets)
                .default_value("love"),
        )
        .arg(Arg::with_name("DIRECTORY").required(true).takes_value(true))
        .arg(
            Arg::from_usage("-v, --version 'Specify which target version of LÖVE to build for'")
                .default_value(default_love_version)
                .possible_values(available_love_versions),
        );

    let subcmd_love_download = SubCommand::with_name("download")
        .about("Download a version of LÖVE")
        .arg(
            Arg::with_name("VERSION")
                .required(true)
                .takes_value(true)
                .possible_values(available_love_versions),
        );

    let subcmd_love_remove = SubCommand::with_name("remove")
        .about("Remove a version of LÖVE")
        .arg(
            Arg::with_name("VERSION")
                .required(true)
                .takes_value(true)
                .possible_values(available_love_versions),
        );

    let subcmd_love = SubCommand::with_name("love")
        .about("Manage multiple LÖVE versions")
        .subcommand(subcmd_love_download)
        .subcommand(subcmd_love_remove);

    let subcmd_init = SubCommand::with_name("init").about("Initialize configuration for project");

    let subcmd_clean = SubCommand::with_name("clean").about("Remove built packages");

    let app_m = App::new("boon")
        .version("0.1.0")
        .author("Cameron McHenry")
        .about("boon: LÖVE2D build and deploy tool")
        .subcommand(subcmd_init)
        .subcommand(subcmd_build)
        .subcommand(subcmd_love)
        .subcommand(subcmd_clean)
        .get_matches();

    match app_m.subcommand() {
        ("init", Some(_subcmd)) => {
            if Path::new("Boon.toml").exists() {
                println!("Project already initialized.");
            } else {
                match File::create("Boon.toml") {
                    Ok(_) => {
                        std::fs::write("Boon.toml", DEFAULT_CONFIG)
                            .expect("Unable to write config file");
                    }
                    Err(e) => {
                        eprintln!("Error while creating configuration file: {}", e);
                        std::process::exit(1);
                    }
                }
            }
        }
        ("build", Some(subcmd)) => {
            let directory = subcmd
                .value_of("DIRECTORY")
                .expect("Could not parse directory from command");
            let target = subcmd
                .value_of("target")
                .expect("Could not parse target from command");
            let version = subcmd
                .value_of("version")
                .expect("Could not parse version string")
                .parse::<LoveVersion>()
                .expect("Could not parse LoveVersion");

            println!(
                "Building target `{}` from directory `{}`",
                target, directory
            );

            let project = Project {
                title: settings
                    .get_str("project.title")
                    .expect("Could not get project title"),
                package_name: settings
                    .get_str("project.package_name")
                    .expect("Could not get project package name"),
                directory: directory.to_string(),
                uti: settings
                    .get_str("project.uti")
                    .expect("Could not get project UTI"),
                authors: settings
                    .get_str("project.authors")
                    .expect("Could not get project authors"),
                description: settings
                    .get_str("project.description")
                    .expect("Could not get project description"),
                email: settings
                    .get_str("project.email")
                    .expect("Could not get project email"),
                url: settings
                    .get_str("project.url")
                    .expect("Could not get project URL"),
                version: settings
                    .get_str("project.version")
                    .expect("Could not get project version"),
            };

            build::build_init(&project, &build_settings);

            let mut stats_list = Vec::new();

            match target {
                "love" => {
                    let stats = build::build_love(&project, &build_settings);
                    stats_list.push(stats);
                }
                "windows" => {
                    let stats = build::build_love(&project, &build_settings);
                    stats_list.push(stats);
                    let stats =
                        build::build_windows(&project, &build_settings, &version, &Bitness::X86);
                    stats_list.push(stats);
                    let stats =
                        build::build_windows(&project, &build_settings, &version, &Bitness::X64);
                    stats_list.push(stats);
                }
                "macos" => {
                    let stats = build::build_love(&project, &build_settings);
                    stats_list.push(stats);
                    let stats =
                        build::build_macos(&project, &build_settings, &version, &Bitness::X64);
                    stats_list.push(stats);
                }
                _ => {}
            }

            // Display build report
            println!();
            for stats in stats_list {
                stats.display();
            }
        }
        ("love", Some(subcmd)) => {
            match subcmd.subcommand() {
                ("download", Some(love_subcmd)) => {
                    let version = love_subcmd
                        .value_of("VERSION")
                        .expect("Could not parse version string")
                        .parse::<LoveVersion>()
                        .expect("Could not parse LoveVersion");

                    download::download_love(&version, &Platform::Windows, &Bitness::X86);
                    download::download_love(&version, &Platform::Windows, &Bitness::X64);
                    download::download_love(&version, &Platform::MacOs, &Bitness::X64);

                    println!(
                        "\nLÖVE {} is now available for building.",
                        version.to_string()
                    )
                }
                ("remove", Some(love_subcmd)) => {
                    let version = love_subcmd
                        .value_of("VERSION")
                        .expect("Could not parse version string")
                        .parse::<LoveVersion>()
                        .expect("Could not parse LoveVersion")
                        .to_string();

                    let installed_versions = get_installed_love_versions();

                    if installed_versions.contains(&version) {
                        let output_file_path = app_dir(AppDataType::UserData, &APP_INFO, "/")
                            .expect("Could not get app directory path");
                        let path = PathBuf::new().join(output_file_path).join(&version);
                        match remove_dir_all(&path) {
                            Ok(_) => {
                                println!("Removed LÖVE version {}.", version);
                            }
                            Err(err) => {
                                eprintln!("Could not remove {}: {}", path.display(), err);
                            }
                        };
                    } else {
                        println!("Version '{}' not installed", version);
                    }
                }
                _ => {
                    // List installed versions
                    let installed_versions = get_installed_love_versions();

                    println!("Installed versions:");
                    for version in installed_versions {
                        println!("* {}", version);
                    }
                }
            }
        }
        ("clean", Some(_subcmd)) => {
            // @TODO: Get top-level directory from git?
            let directory = ".";
            let mut release_dir_path = Path::new(directory)
                .canonicalize()
                .expect("Could not get canonical directory path");
            release_dir_path.push(build_settings.output_directory.as_str());

            if release_dir_path.exists() {
                println!("Cleaning {}", release_dir_path.display());

                match remove_dir_all(&release_dir_path) {
                    Ok(_) => {
                        println!();
                    }
                    Err(err) => {
                        eprintln!("Could not clean {}: {}", release_dir_path.display(), err);
                    }
                };
            }

            println!("Release directory cleaned.");
        }
        _ => {
            println!("No command supplied.");
            println!("{}", app_m.usage());
        }
    }
}

fn get_installed_love_versions() -> Vec<String> {
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
                .expect("Could not parse file name to str");

            // Exclude directories that do not parse to a love
            // version, just in case some bogus directories
            // got in there somehow.
            if let Ok(version) = file_name.parse::<LoveVersion>() {
                installed_versions.push(version.to_string())
            }
        }
    }

    installed_versions
}
