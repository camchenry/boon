use std::fmt::{Display, Formatter};
use std::str::FromStr;

#[derive(Debug, Clone)]
pub struct BuildSettings {
    pub output_directory: String,
    pub ignore_list: Vec<String>,
    pub exclude_default_ignore_list: bool,
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

/// Represents a specific version of LÖVE2D
#[derive(Debug, Copy, Clone)]
pub enum LoveVersion {
    V11_3,
    V11_2,
    V11_1,
    V11_0,
    V0_10_2,
}

/// File info about remote download
pub struct LoveDownloadLocation {
    pub filename: String,
    pub url: String,
}

pub struct BuildStatistics {
    pub build_name: String,
    pub build_time: std::time::Duration,
}

impl FromStr for LoveVersion {
    type Err = ();
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "11.3" => Ok(LoveVersion::V11_3),
            "11.2" => Ok(LoveVersion::V11_2),
            "11.1" => Ok(LoveVersion::V11_1),
            "11.0" => Ok(LoveVersion::V11_0),
            "11.0.0" => Ok(LoveVersion::V11_0),
            "0.10.2" => Ok(LoveVersion::V0_10_2),
            _ => Err(()),
        }
    }
}

impl Display for LoveVersion {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        use crate::types::LoveVersion::*;

        let str = match self {
            &V11_3 => "11.3",
            &V11_2 => "11.2",
            &V11_1 => "11.1",
            &V11_0 => "11.0",
            &V0_10_2 => "0.10.2",
        };
        write!(f, "{}", str)
    }
}

impl Display for Bitness {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        use crate::types::Bitness::*;

        let str = match self {
            &X86 => "x86",
            &X64 => "x64",
        };
        write!(f, "{}", str)
    }
}

impl Display for Platform {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        use crate::types::Platform::*;

        let str = match self {
            &Windows => "Windows",
            &MacOs => "macOS",
        };
        write!(f, "{}", str)
    }
}
