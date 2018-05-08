extern crate clap;

use clap::{App, Arg, SubCommand};

fn main() {
    let targets = &["love"];

    let app_m = App::new("love-kit")
        .version("1.0")
        .author("Cameron McHenry")
        .about("LÖVE2D Kit build/deploy tool")
        .subcommand(SubCommand::with_name("build")
                    .about("Build game for a target platform (win32, macos, ...)")
                    .arg(Arg::with_name("TARGET")
                         .required(true)
                         .possible_values(targets)
                    ))
        .get_matches();

    match app_m.subcommand() {
        ("build", Some(sub_m)) => {
            let target = sub_m.value_of("TARGET");
            match target {
                Some("love") => {
                    println!("Building target `{}`", target.unwrap());
                    build_love();
                }
                _ => {}
            }
        },

        // No command used
        _                      => {
            println!("{}", app_m.usage());
        },
    }
}

fn build_love() {
    println!("Now building love build!");
}
