extern crate zip;
extern crate std;
use crate::types::*;

use app_dirs::*;
use crate::APP_INFO;

use reqwest;
use crate::{Platform, Bitness};

use std::io::Write;
use std::fs::File;

pub fn download_love(version: &LoveVersion, platform: &Platform, bitness: &Bitness) {
    let file_info = match (version, platform, bitness) {
        (&LoveVersion::V11_2, &Platform::Windows, &Bitness::X64) => LoveVersionFileInfo{
            version, platform, bitness,
            filename: "love-11.2-win64.zip",
            url: "https://bitbucket.org/rude/love/downloads/love-11.2-win64.zip",
        },
        (&LoveVersion::V11_2, &Platform::Windows, &Bitness::X86) => LoveVersionFileInfo{
            version, platform, bitness,
            filename: "love-11.2-win32.zip",
            url: "https://bitbucket.org/rude/love/downloads/love-11.2-win32.zip",
        },
        (&LoveVersion::V11_2, &Platform::MacOs, &Bitness::X64) => LoveVersionFileInfo{
            version, platform, bitness,
            filename: "love-11.2-macos.zip",
            url: "https://bitbucket.org/rude/love/downloads/love-11.2-macos.zip",
        },

        (&LoveVersion::V11_1, &Platform::Windows, &Bitness::X64) => LoveVersionFileInfo{
            version, platform, bitness,
            filename: "love-11.1-win64.zip",
            url: "https://bitbucket.org/rude/love/downloads/love-11.1-win64.zip",
        },
        (&LoveVersion::V11_1, &Platform::Windows, &Bitness::X86) => LoveVersionFileInfo{
            version, platform, bitness,
            filename: "love-11.1-win32.zip",
            url: "https://bitbucket.org/rude/love/downloads/love-11.1-win32.zip",
        },
        (&LoveVersion::V11_1, &Platform::MacOs, &Bitness::X64) => LoveVersionFileInfo{
            version, platform, bitness,
            filename: "love-11.1-macos.zip",
            url: "https://bitbucket.org/rude/love/downloads/love-11.1-macos.zip",
        },

        (&LoveVersion::V11_0, &Platform::Windows, &Bitness::X64) => LoveVersionFileInfo{
            version, platform, bitness,
            filename: "love-11.0.0-win64.zip",
            url: "https://bitbucket.org/rude/love/downloads/love-11.0.0-win64.zip",
        },
        (&LoveVersion::V11_0, &Platform::Windows, &Bitness::X86) => LoveVersionFileInfo{
            version, platform, bitness,
            filename: "love-11.0.0-win32.zip",
            url: "https://bitbucket.org/rude/love/downloads/love-11.0.0-win32.zip",
        },
        (&LoveVersion::V11_0, &Platform::MacOs, &Bitness::X64) => LoveVersionFileInfo{
            version, platform, bitness,
            filename: "love-11.0.0-macos.zip",
            url: "https://bitbucket.org/rude/love/downloads/love-11.0.0-macos.zip",
        },

        (&LoveVersion::V0_10_2, &Platform::Windows, &Bitness::X64) => LoveVersionFileInfo{
            version, platform, bitness,
            filename: "love-0.10.2-win64.zip",
            url: "https://bitbucket.org/rude/love/downloads/love-0.10.2-win64.zip",
        },
        (&LoveVersion::V0_10_2, &Platform::Windows, &Bitness::X86) => LoveVersionFileInfo{
            version, platform, bitness,
            filename: "love-0.10.2-win32.zip",
            url: "https://bitbucket.org/rude/love/downloads/love-0.10.2-win32.zip",
        },
        (&LoveVersion::V0_10_2, &Platform::MacOs, &Bitness::X64) => LoveVersionFileInfo{
            version, platform, bitness,
            filename: "love-0.10.2-macosx-x64.zip",
            url: "https://bitbucket.org/rude/love/downloads/love-0.10.2-macosx-x64.zip",
        },
        _ => {
            panic!("Unsupported platform {:?}-{:?} for version {:?}", bitness, platform, version);
        }
    };

    let mut output_file_path = app_dir(AppDataType::UserData, &APP_INFO, version.to_string().as_str()).unwrap();
    output_file_path.push(file_info.filename);

    let zip_exists: bool = output_file_path.exists();

    // @TODO: Add integrity checking with hash
    if zip_exists {
        println!("File already exists: {:?}", output_file_path);
    } else {
        println!("Downloading '{}'", file_info.url);

        let mut resp = match reqwest::get(file_info.url) {
            Ok(res) => {
                match reqwest::get(res.url().as_str()) {
                    Ok(res) => res,
                    Err(why) => {
                        panic!("Could not fetch '{}': {}", file_info.url, why);
                    }
                }
            }
            Err(why) => {
                panic!("Could not fetch '{}': {}", file_info.url, why);
            }
        };

        let file = match File::create(&output_file_path) {
            Ok(file) => file,
            Err(why) => {
                panic!("Unable to create file '{}': {}", output_file_path.display(), why);
            }
        };

        let mut writer = std::io::BufWriter::new(&file);
        &resp.copy_to(&mut writer);
        match writer.flush() {
            Ok(_) => {}
            Err(why) => {
                panic!("Could not write file '{}': {}", output_file_path.display(), why);
            }
        }
    }

    println!("Extracting '{}'", output_file_path.display());
    {
        let file = match File::open(&output_file_path) {
            Ok(file) => file,
            Err(why) => {
                panic!("Unable to open file '{}': {}", output_file_path.display(), why);
            }
        };

        let mut archive = match zip::ZipArchive::new(&file) {
            Ok(archive) => archive,
            Err(why) => panic!("{}", why),
        };

        for i in 0..archive.len() {
            let mut file = archive.by_index(i).unwrap();
            let mut outpath = output_file_path.clone();
            outpath.pop();
            outpath.push(file.sanitized_name());

            if (&*file.name()).ends_with('/') {
                //println!("File {} extracted to \"{}\"", i, outpath.as_path().display());
                std::fs::create_dir_all(&outpath).unwrap();
            } else {
                //println!("File {} extracted to \"{}\" ({} bytes)", i, outpath.as_path().display(), file.size());
                if let Some(p) = outpath.parent() {
                    if !p.exists() {
                        std::fs::create_dir_all(&p).unwrap();
                    }
                }
                let mut outfile = std::fs::File::create(&outpath).unwrap();
                std::io::copy(&mut file, &mut outfile).unwrap();
            }

            // Get and Set permissions
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;

                if let Some(mode) = file.unix_mode() {
                    std::fs::set_permissions(&outpath, std::fs::Permissions::from_mode(mode)).unwrap();
                }
            }
        }

    }
}
