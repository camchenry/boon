extern crate clap;
extern crate app_dirs;
extern crate walkdir;
extern crate zip;
extern crate reqwest;
extern crate config;
extern crate regex;

mod types;
use types::*;

mod download;
mod build;

use clap::{App, Arg, SubCommand};
use app_dirs::*;

const APP_INFO: AppInfo = AppInfo {
    name: "love-kit",
    author: "love-kit"
};

fn main() {
    let targets = &["love", "windows", "macos"];

    // @TODO: Get values from local project config
    // load in config from Settings file
    let mut settings = config::Config::default();
    settings
        // Add in `./Settings.toml`
        .merge(config::File::with_name("Settings")).unwrap();

    let build_settings = BuildSettings {
        debug_halt: settings.get("build.debug_halt").unwrap(),
        ignore_list: settings.get("build.ignore_list").unwrap()
    };

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
             .default_value("11.1")
             .possible_values(&["11.1", "0.10.2"])
            );

    let subcmd_download = SubCommand::with_name("download")
        .about("Download a version of LÖVE")
        .arg(Arg::with_name("VERSION")
             .required(true)
             .takes_value(true)
             .possible_values(&["11.1", "0.10.2"])
            );

    let app_m = App::new("love-kit")
        .version("1.0")
        .author("Cameron McHenry")
        .about("LÖVE2D Kit build/deploy tool")
        .subcommand(subcmd_build)
        .subcommand(subcmd_download)
        .get_matches();

    match app_m.subcommand() {
        ("build", Some(subcmd)) => {
            let directory = subcmd.value_of("DIRECTORY");
            let target = subcmd.value_of("target");
            let version: LoveVersion = subcmd.value_of("version")
                .unwrap()
                .parse::<LoveVersion>()
                .unwrap();

            println!("Building target `{}` from directory `{}`", target.unwrap(), directory.unwrap());

            let project = Project {
                title: settings.get_str("project.title").unwrap_or("My Game".to_owned()),
                package_name: settings.get_str("project.package_name").unwrap_or("my_game".to_owned()),
                directory: directory.unwrap().to_string(),
                uti: settings.get_str("project.uti").unwrap_or("com.company.mygame".to_owned()),

                authors: settings.get_str("project.authors").unwrap_or("Developer Name".to_owned()),
                description: settings.get_str("project.description").unwrap_or("Your description here.".to_owned()),
                email: settings.get_str("project.email").unwrap_or("email@example.com".to_owned()),
                url: settings.get_str("project.url").unwrap_or("http://www.example.com/".to_owned()),
                version: settings.get_str("version").unwrap_or("v1.0".to_owned()),
            };

            build::build_init(&project, &build_settings);

            match target {
                Some("love") => {
                    build::build_love(&project, &build_settings)
                }
                Some("windows") => {
                    build::build_love(&project, &build_settings);
                    build::build_windows(&project, &version, &Bitness::X86);
                    build::build_windows(&project, &version, &Bitness::X64);
                }
                Some("macos") => {
                    build::build_love(&project, &build_settings);
                    build::build_macos(&project, &version, &Bitness::X64);
                }
                _ => {}
            }
        },
        ("download", Some(subcmd)) => {
            let version: LoveVersion = subcmd.value_of("VERSION")
                .unwrap()
                .parse::<LoveVersion>()
                .unwrap();

            download::download_love(&version, &Platform::Windows, &Bitness::X86);
            download::download_love(&version, &Platform::Windows, &Bitness::X64);
            download::download_love(&version, &Platform::MacOs, &Bitness::X64);

            println!("\nLÖVE {} is now available for building.", version.to_string())
        },
        _ => {
            println!("No command supplied.");
            println!("{}", app_m.usage());
        },
    }
}
