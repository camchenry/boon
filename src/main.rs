extern crate clap;
extern crate app_dirs;
extern crate walkdir;
extern crate zip;
extern crate reqwest;
extern crate config;
extern crate regex;
extern crate remove_dir_all;
extern crate glob;

mod types;
use crate::types::*;

mod download;
mod build;

use clap::{App, Arg, SubCommand};
use app_dirs::*;
use std::path::{Path, PathBuf};
use std::fs::File;
use walkdir::WalkDir;
use remove_dir_all::*;

const APP_INFO: AppInfo = AppInfo {
    name: "boon",
    author: "boon"
};

const DEFAULT_CONFIG: &str = include_str!("../Boon.toml");

fn main() {
    // @TODO: Get values from local project config
    // load in config from Settings file
    let mut settings = config::Config::new();

    let default_config = config::File::from_str(DEFAULT_CONFIG, config::FileFormat::Toml);

    match settings.merge(default_config) {
        Ok(_) => {},
        _ => {
            eprintln!("Could not set default configuration.");
            std::process::exit(1);
        }
    }

    let mut ignore_list: Vec<String> = settings.get("build.ignore_list").unwrap();

    if Path::new("Boon.toml").exists() {
        // Add in `./Boon.toml`
        match settings.merge(config::File::with_name("Boon")) {
            Ok(_) => {},
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
        ignore_list: ignore_list,
        exclude_default_ignore_list: settings.get("build.exclude_default_ignore_list").unwrap(),
        output_directory: settings.get("build.output_directory").unwrap(),
    };

    let targets = &["love", "windows", "macos"];

    let default_love_version = "11.2";
    let available_love_versions = &[
        "11.2",
        "11.1",
        "11.0",
        "0.10.2",
    ];

    let subcmd_build = SubCommand::with_name("build")
        .about("Build game for a target platform")
        .arg(Arg::from_usage("-t, --target 'Specify which target platform to build for'")
             .required(true)
             .possible_values(targets)
             .default_value("love")
            )
        .arg(Arg::with_name("DIRECTORY")
             .required(true)
             .takes_value(true)
            )
        .arg(Arg::from_usage("-v, --version 'Specify which target version of LÖVE to build for'")
             .default_value(default_love_version)
             .possible_values(available_love_versions)
            );

    let subcmd_love_download = SubCommand::with_name("download")
        .about("Download a version of LÖVE")
        .arg(Arg::with_name("VERSION")
             .required(true)
             .takes_value(true)
             .possible_values(available_love_versions)
            );

    let subcmd_love_remove = SubCommand::with_name("remove")
        .about("Remove a version of LÖVE")
        .arg(Arg::with_name("VERSION")
             .required(true)
             .takes_value(true)
             .possible_values(available_love_versions)
            );

    let subcmd_love = SubCommand::with_name("love")
        .about("Manage multiple LÖVE versions")
        .subcommand(subcmd_love_download)
        .subcommand(subcmd_love_remove);


    let subcmd_init = SubCommand::with_name("init")
        .about("Initialize configuration for project");

    let app_m = App::new("boon")
        .version("0.1.0")
        .author("Cameron McHenry")
        .about("boon: LÖVE2D build and deploy tool")
        .subcommand(subcmd_init)
        .subcommand(subcmd_build)
        .subcommand(subcmd_love)
        .get_matches();

    match app_m.subcommand() {
        ("init", Some(_subcmd)) => {
            if Path::new("Boon.toml").exists() {
                println!("Project already initialized.");
            } else {
                match File::create("Boon.toml") {
                    Ok(_) => {
                        std::fs::write("Boon.toml", DEFAULT_CONFIG).expect("Unable to write config file");
                    },
                    Err(e) => {
                        eprintln!("Error while creating configuration file: {}", e);
                        std::process::exit(1);
                    }
                }
            }
        },
        ("build", Some(subcmd)) => {
            let directory = subcmd.value_of("DIRECTORY");
            let target = subcmd.value_of("target");
            let version: LoveVersion = subcmd.value_of("version")
                .unwrap()
                .parse::<LoveVersion>()
                .unwrap();

            println!("Building target `{}` from directory `{}`", target.unwrap(), directory.unwrap());

            let project = Project {
                title: settings.get_str("project.title").unwrap(),
                package_name: settings.get_str("project.package_name").unwrap(),
                directory: directory.unwrap().to_string(),
                uti: settings.get_str("project.uti").unwrap(),

                authors: settings.get_str("project.authors").unwrap(),
                description: settings.get_str("project.description").unwrap(),
                email: settings.get_str("project.email").unwrap(),
                url: settings.get_str("project.url").unwrap(),
                version: settings.get_str("project.version").unwrap(),
            };

            build::build_init(&project, &build_settings);

            let mut stats_list = Vec::new();

            match target {
                Some("love") => {
                    let stats = build::build_love(&project, &build_settings);
                    stats_list.push(stats);
                }
                Some("windows") => {
                    let stats = build::build_love(&project, &build_settings);
                    stats_list.push(stats);
                    let stats = build::build_windows(&project, &build_settings, &version, &Bitness::X86);
                    stats_list.push(stats);
                    let stats = build::build_windows(&project, &build_settings, &version, &Bitness::X64);
                    stats_list.push(stats);
                }
                Some("macos") => {
                    let stats = build::build_love(&project, &build_settings);
                    stats_list.push(stats);
                    let stats = build::build_macos(&project, &build_settings, &version, &Bitness::X64);
                    stats_list.push(stats);
                }
                _ => {}
            }

            // Display build report
            println!();
            for stats in stats_list {
                stats.display();
            }
        },
        ("love", Some(subcmd)) => {
            match subcmd.subcommand() {
                ("download", Some(love_subcmd)) => {
                    let version: LoveVersion = love_subcmd.value_of("VERSION")
                        .unwrap()
                        .parse::<LoveVersion>()
                        .unwrap();

                    download::download_love(&version, &Platform::Windows, &Bitness::X86);
                    download::download_love(&version, &Platform::Windows, &Bitness::X64);
                    download::download_love(&version, &Platform::MacOs, &Bitness::X64);

                    println!("\nLÖVE {} is now available for building.", version.to_string())
                },
                ("remove", Some(love_subcmd)) => {
                    let version = love_subcmd.value_of("VERSION")
                        .unwrap()
                        .parse::<LoveVersion>()
                        .unwrap()
                        .to_string();

                    let installed_versions = get_installed_love_versions();

                    if installed_versions.contains(&version) {
                        let output_file_path = app_dir(AppDataType::UserData, &APP_INFO, "/").unwrap();
                        let path = PathBuf::new()
                            .join(output_file_path)
                            .join(&version);
                        match remove_dir_all(&path) {
                            Ok(_) => {
                                println!("Removed LÖVE version {}.", version);
                            },
                            Err(err) => {
                                eprintln!("Could not remove {}: {}", path.display(), err);
                            }
                        };
                    } else {
                        println!("Version '{}' not installed", version);
                    }
                },
                _ => {
                    // List installed versions
                    let installed_versions = get_installed_love_versions();

                    println!("Installed versions:");
                    for version in installed_versions {
                        println!("* {}", version);
                    }
                }
            }
        },
        _ => {
            println!("No command supplied.");
            println!("{}", app_m.usage());
        },
    }
}

fn get_installed_love_versions() -> Vec<String> {
    let mut installed_versions: Vec<String> = Vec::new();
    let output_file_path = app_dir(AppDataType::UserData, &APP_INFO, "/").unwrap();
    let walker = WalkDir::new(output_file_path)
        .max_depth(1)
        .into_iter();
    for entry in walker {
        let entry = entry.unwrap();
        if entry.depth() == 1 {
            let file_name = entry.file_name().to_str().expect("Could not parse file name to str");

            // Exclude directories that do not parse to a love
            // version, just in case some bogus directories
            // got in there somehow.
            match file_name.parse::<LoveVersion>() {
                Ok(version) => installed_versions.push(version.to_string()),
                Err(_) => {},
            }
        }
    }

    installed_versions
}
