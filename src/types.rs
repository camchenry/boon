extern crate std;
extern crate config;

use std::str::FromStr;

pub struct Project<'a> {
    pub title: String,      // Ex: "My Super Awesome Game"
    pub app_name: String,   // Ex: "super_game"
    pub directory: String,
    pub uti: String,        // Uniform Type Identifier, e.g. "org.love2d.love"
    pub settings: &'a config::Config,
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
    V11_1,
    V0_10_2,
}

/// File info about remote download
pub struct LoveVersionFileInfo<'a>{
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

impl ToString for LoveVersion {
    fn to_string(&self) -> String {
        use types::LoveVersion::*;

        match self {
            &V11_1 => "11.1",
            &V0_10_2 => "0.10.2",
        }.to_string()
    }
}
