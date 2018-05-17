extern crate std;

use std::str::FromStr;

/// Represents an operating system or other platform/environment.
#[derive(Debug)]
pub enum Platform {
    Windows,
    MacOs,
}

/// Represents a CPU architecture
#[derive(Debug)]
pub enum Bitness {
    X86, // 32 bit
    X64, // 64 bit
}

/// Represents a specific version of LÖVE2D
#[derive(Debug)]
pub enum LoveVersion {
    V11_1,
    V0_10_2,
}

/// File info about remote download
pub struct LoveVersionFileInfo<'a> {
    pub version: &'a LoveVersion,
    pub platform: &'a ::Platform,
    pub bitness: &'a ::Bitness,
    pub filename: &'a str,
    pub url: &'a str,
}

impl FromStr for LoveVersion {
    type Err = ();
    fn from_str(s: &str) -> Result<Self, Self::Err>{
        match s {
            "11.1" => Ok(LoveVersion::V11_1),
            "11.1.0" => Ok(LoveVersion::V11_1),
            "0.10.2" => Ok(LoveVersion::V0_10_2),
            _ => Err(()),
        }
    }
}

pub struct BuildSettings<> {
    pub debug_halt: bool,
    pub ignore_list: Vec<String>,
}

pub struct ProjectSettings<'a> {
    pub authors: &'a str,
    pub description: &'a str,
    pub email: &'a str,
    pub package_name: &'a str,
    pub project_title: &'a str,
    pub project_url: &'a str,
    pub project_uti: &'a str,
    pub project_version: &'a str,
}
