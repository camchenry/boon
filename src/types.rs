extern crate std;

use std::str::FromStr;

pub struct BuildSettings<> {
    pub ignore_list: Vec<String>,
}

pub struct Project {
    pub title: String,          // Ex: "My Super Awesome Game"
    pub package_name: String,   // Ex: "super_game"
    pub directory: String,
    pub uti: String,            // Uniform Type Identifier, e.g. "org.love2d.love"

    pub authors: String,
    pub description: String,
    pub email: String,
    pub url: String,
    pub version: String,
}

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
    V11_2,
    V11_1,
    V11_0,
    V0_10_2,
}

/// File info about remote download
pub struct LoveVersionFileInfo<'a> {
    pub version: &'a LoveVersion,
    pub platform: &'a crate::Platform,
    pub bitness: &'a crate::Bitness,
    pub filename: &'a str,
    pub url: &'a str,
}

impl FromStr for LoveVersion {
    type Err = ();
    fn from_str(s: &str) -> Result<Self, Self::Err>{
        match s {
            "11.2" => Ok(LoveVersion::V11_2),
            "11.1" => Ok(LoveVersion::V11_1),
            "11.0" => Ok(LoveVersion::V11_0),
            "11.0.0" => Ok(LoveVersion::V11_0),
            "0.10.2" => Ok(LoveVersion::V0_10_2),
            _ => Err(()),
        }
    }
}

impl ToString for LoveVersion {
    fn to_string(&self) -> String {
        use crate::types::LoveVersion::*;

        match self {
            &V11_2 => "11.2",
            &V11_1 => "11.1",
            &V11_0 => "11.0",
            &V0_10_2 => "0.10.2",
        }.to_string()
    }
}
