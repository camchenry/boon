extern crate clap;
extern crate app_dirs;
extern crate walkdir;
extern crate zip;
extern crate reqwest;
extern crate config;

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
    let targets = &["love", "windows"];

    // load in config from Settings file
    let mut settings = config::Config::default();
    settings
        // Add in `./Settings.toml`
        .merge(config::File::with_name("Settings")).unwrap();

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

            build::scan_files(directory.unwrap().to_string(), &settings);

            match target {
                Some("love") => {
                    build::build_love(directory.unwrap().to_string(), &settings)
                }
                Some("windows") => {
                    build::build_love(directory.unwrap().to_string(),& settings);
                    build::build_windows(directory.unwrap().to_string(), &version, &Bitness::X86);
                    build::build_windows(directory.unwrap().to_string(), &version, &Bitness::X64);
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

            println!("\nLÖVE v{} is now available for building.", subcmd.value_of("VERSION").unwrap());
        },
        _ => {
            println!("No command supplied.");
            println!("{}", app_m.usage());
        },
    }
}
