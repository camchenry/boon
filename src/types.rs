#![allow(clippy::use_debug)]
use clap::arg_enum;
use enum_primitive_derive::Primitive;
use num_traits::FromPrimitive;
use std::collections::HashSet;
use std::fmt::{Display, Formatter};
use std::str::FromStr;

#[derive(Debug, Clone)]
pub struct BuildSettings {
    pub output_directory: String,
    pub ignore_list: HashSet<String>,
    pub exclude_default_ignore_list: bool,
    pub targets: Vec<Target>,
}

#[derive(Debug, Clone)]
pub struct Project {
    pub title: String,        // Ex: "My Super Awesome Game"
    pub package_name: String, // Ex: "super_game"
    pub directory: String,
    pub uti: String, // Uniform Type Identifier, e.g. "org.love2d.love"

    pub authors: String,
    pub description: String,
    pub email: String,
    pub url: String,
    pub version: String,
}

/// Represents an operating system or other platform/environment.
#[derive(Debug, Copy, Clone)]
pub enum Platform {
    Windows,
    MacOs,
}

/// Represents a CPU architecture
#[derive(Debug, Copy, Clone)]
pub enum Bitness {
    X86, // 32 bit
    X64, // 64 bit
}

const LOVE_VERSIONS: [&str; 5] = ["11.3", "11.2", "11.1", "11.0", "0.10.2"];
/// Represents a specific version of LÖVE2D
#[derive(Copy, Clone, Debug, Primitive)]
pub enum LoveVersion {
    V11_3 = 0,
    V11_2 = 1,
    V11_1 = 2,
    V11_0 = 3,
    V0_10_2 = 4,
}

/// File info about remote download
pub struct LoveDownloadLocation {
    pub filename: String,
    pub url: String,
}

#[derive(Debug, Clone)]
/// Stats about the build duration, size, etc.
pub struct BuildStatistics {
    /// Name of the build, e.g. Windows, macOS, etc.
    pub name: String,
    /// File name of the build output
    pub file_name: String,
    /// Time it took to build
    pub time: std::time::Duration,
    /// The size of the final build in bytes
    pub size: u64,
}

impl FromStr for LoveVersion {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        LOVE_VERSIONS
            .iter()
            .enumerate()
            .find(|(_, v)| s == **v)
            .map(|(i, _)| Self::from_usize(i))
            .flatten()
            .ok_or(format!("{} is not a valid love version.", s))
    }
}

impl LoveVersion {
    pub const fn variants() -> [&'static str; 5] {
        LOVE_VERSIONS
    }
}

impl Display for LoveVersion {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", LOVE_VERSIONS[*self as usize])
    }
}

impl Display for Bitness {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        use crate::types::Bitness::{X64, X86};

        let str = match self {
            X86 => "x86",
            X64 => "x64",
        };
        write!(f, "{}", str)
    }
}

impl Display for Platform {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        use crate::types::Platform::{MacOs, Windows};

        let str = match self {
            Windows => "Windows",
            MacOs => "macOS",
        };
        write!(f, "{}", str)
    }
}

impl Display for BuildSettings {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{{\n\
            \toutput_directory: {}\n\
            \texclude_default_ignore_list: {}\n\
            \tignore_list: {:?}\n\
            }}",
            self.output_directory, self.exclude_default_ignore_list, self.ignore_list
        )
    }
}

arg_enum! {
    #[derive(Debug, Copy, Clone, PartialEq)]
    #[allow(non_camel_case_types)]
    pub enum Target {
        love,
        windows,
        macos,
        all,
    }
}
