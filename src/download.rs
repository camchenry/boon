use crate::types::*;

use crate::APP_INFO;
use app_dirs::*;

use crate::{Bitness, Platform};
use reqwest;

use anyhow::{bail, Context, Result};
use std::fs::File;
use std::io::Write;

pub fn download_love(version: LoveVersion, platform: Platform, bitness: Bitness) -> Result<()> {
    let file_info = get_love_download_location(version, platform, bitness).with_context(|| {
        format!(
            "Could not get download location for LÖVE {} on {} {}",
            version, platform, bitness
        )
    })?;

    let mut output_file_path = app_dir(
        AppDataType::UserData,
        &APP_INFO,
        version.to_string().as_str(),
    )
    .with_context(|| {
        format!(
            "Could not get app user data directory path for version `{}`",
            version.to_string()
        )
    })?;
    output_file_path.push(&file_info.filename);

    // @TODO: Add integrity checking with hash
    if output_file_path.exists() {
        println!("File already exists: {}", output_file_path.display());
    } else {
        println!("Downloading '{}'", file_info.url);

        let mut resp = reqwest::blocking::get(&file_info.url)
            .with_context(|| format!("Could not fetch URL `{}`", &file_info.url))?;

        let file = File::create(&output_file_path)
            .with_context(|| format!("Could not create file `{}`", output_file_path.display()))?;

        let mut writer = std::io::BufWriter::new(&file);
        resp.copy_to(&mut writer).with_context(|| {
            format!(
                "Could not copy response from `{}` to file `{}`",
                resp.url(),
                output_file_path.display()
            )
        })?;
        writer
            .flush()
            .with_context(|| format!("Could not write file `{}`", output_file_path.display()))?;
    }

    println!("Extracting '{}'", output_file_path.display());
    {
        let file = File::open(&output_file_path)
            .with_context(|| format!("Could not open file `{}`", output_file_path.display()))?;

        let mut archive = zip::ZipArchive::new(&file).with_context(|| {
            format!(
                "Could not create zip archive `{}`",
                output_file_path.display()
            )
        })?;

        for i in 0..archive.len() {
            let mut file = archive
                .by_index(i)
                .unwrap_or_else(|_| panic!("Could not get archive file by index '{}'", i));
            let mut outpath = output_file_path.clone();
            outpath.pop();
            outpath.push(file.sanitized_name());

            if file.name().ends_with('/') {
                std::fs::create_dir_all(&outpath).expect("Could not create output directory path");
            } else {
                if let Some(p) = outpath.parent() {
                    if !p.exists() {
                        std::fs::create_dir_all(&p)
                            .expect("Could not create output directory path");
                    }
                }
                let mut outfile =
                    std::fs::File::create(&outpath).expect("Could not create output file");
                std::io::copy(&mut file, &mut outfile).expect("Could not copy data to output file");
            }

            // Get and Set permissions
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;

                if let Some(mode) = file.unix_mode() {
                    std::fs::set_permissions(&outpath, std::fs::Permissions::from_mode(mode))
                        .expect("Could not set permissions on file");
                }
            }
        }
    }

    Ok(())
}

fn get_love_download_location(
    version: LoveVersion,
    platform: Platform,
    bitness: Bitness,
) -> Result<LoveDownloadLocation> {
    let release_location = "https://github.com/love2d/love/releases/download";
    let (version_string, release_file_name) = match (version, platform, bitness) {
        (LoveVersion::V11_3, Platform::Windows, Bitness::X64) => ("11.3", "love-11.3-win64.zip"),
        (LoveVersion::V11_3, Platform::Windows, Bitness::X86) => ("11.3", "love-11.3-win32.zip"),
        (LoveVersion::V11_3, Platform::MacOs, Bitness::X64) => ("11.3", "love-11.3-macos.zip"),

        (LoveVersion::V11_2, Platform::Windows, Bitness::X64) => ("11.2", "love-11.2-win64.zip"),
        (LoveVersion::V11_2, Platform::Windows, Bitness::X86) => ("11.2", "love-11.2-win32.zip"),
        (LoveVersion::V11_2, Platform::MacOs, Bitness::X64) => ("11.2", "love-11.2-macos.zip"),

        (LoveVersion::V11_1, Platform::Windows, Bitness::X64) => ("11.1", "love-11.1-win64.zip"),
        (LoveVersion::V11_1, Platform::Windows, Bitness::X86) => ("11.1", "love-11.1-win32.zip"),
        (LoveVersion::V11_1, Platform::MacOs, Bitness::X64) => ("11.1", "love-11.1-macos.zip"),

        (LoveVersion::V11_0, Platform::Windows, Bitness::X64) => ("11.0", "love-11.0.0-win64.zip"),
        (LoveVersion::V11_0, Platform::Windows, Bitness::X86) => ("11.0", "love-11.0.0-win32.zip"),
        (LoveVersion::V11_0, Platform::MacOs, Bitness::X64) => ("11.0", "love-11.0.0-macos.zip"),

        (LoveVersion::V0_10_2, Platform::Windows, Bitness::X64) => ("10.2", "love-0.10.2-win64.zip"),
        (LoveVersion::V0_10_2, Platform::Windows, Bitness::X86) => ("10.2", "love-0.10.2-win32.zip"),
        (LoveVersion::V0_10_2, Platform::MacOs, Bitness::X64) => ("10.2", "love-0.10.2-macosx-x64.zip"),
        _ => {
            bail!(
                "Unsupported platform {}-{} for version {}",
                platform,
                bitness,
                version
            );
        }
    };

    let url = format!("{}/{}/{}", release_location, version_string, release_file_name);
    Ok(LoveDownloadLocation {
        filename: release_file_name.to_string(),
        url,
    })
}
