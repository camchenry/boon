extern crate std;
extern crate zip;
extern crate config;

use types::*;

use APP_INFO;
use app_dirs::*;

use ::{Platform, Bitness};

use std::io::prelude::*;
use std::iter::Iterator;
use std::io::{Write, Seek};

use std::path::{Path, PathBuf};
use std::fs::File;

use zip::result::ZipError;
use zip::write::FileOptions;
use walkdir::{WalkDir, DirEntry};

use self::config::ConfigError;
use self::config::Config;

const METHOD_DEFLATED: Option<zip::CompressionMethod> = Some(zip::CompressionMethod::Deflated);

static mut IS_LOVE_BUILT: bool = false;

// @TODO: Return an Option instead
pub fn get_love_filename<'a>(version: &LoveVersion, platform: &Platform, bitness: &Bitness) -> &'a str {
    match (version, platform, bitness) {
        (&LoveVersion::V11_1,   &Platform::Windows, &Bitness::X64) => "love-11.1.0-win64",
        (&LoveVersion::V11_1,   &Platform::Windows, &Bitness::X86) => "love-11.1.0-win32",
        (&LoveVersion::V11_1,   &Platform::MacOs,   &Bitness::X64) => "love-11.1.0-macos",
        (&LoveVersion::V0_10_2, &Platform::Windows, &Bitness::X64) => "love-0.10.2-win64",
        (&LoveVersion::V0_10_2, &Platform::Windows, &Bitness::X86) => "love-0.10.2-win32",
        (&LoveVersion::V0_10_2, &Platform::MacOs,   &Bitness::X64) => "love-0.10.2-macos",
        _ => ""
    }
}

pub fn get_output_filename<'a>(name: String, platform: &Platform, bitness: &Bitness) -> String {
    match (platform, bitness) {
        (&Platform::Windows, &Bitness::X64) => format!("{}-win64.exe", name),
        (&Platform::Windows, &Bitness::X86) => format!("{}-win32.exe", name),
        (&Platform::MacOs,   &Bitness::X64) => format!("{}-macos.app", name),
        _ => {
            panic!("Unsupported platform {:?}-{:?}");
        }
    }
}

// TODO: check CONFIG to see if DEBUG set to true should halt building process
pub fn scan_files(directory: String, settings: &Config) {
    let globals_file = format!("{}{}", directory, "/globals.lua");
    println!("Looking for globals.lua at: {}", globals_file);

    let mut f = File::open(globals_file).expect("file not found");

    let mut contents = String::new();
    f.read_to_string(&mut contents)
        .expect("something went wrong reading the file");

    if (contents.find("RELEASE = false") != None && contents.find("DEBUG = not RELEASE") != None) || contents.find("DEBUG = true") != None {
        println!("!!!WARNING!!! Debug is ENABLED!");
        if settings.get_bool("debug_halt").unwrap() {
            panic!("DEBUG set to false. If you want to build anyway, modify debug_halt in Settings.");
        }
    } else if (contents.find("RELEASE = true") != None && contents.find("DEBUG = not RELEASE") != None) || contents.find("DEBUG = false") != None {
        println!("You can rest easy. Debug is DISABLED.")
    } else {
        println!("It is uncertain what DEBUG is set to. Make sure to verify it on your own.")
    }
}

pub fn build_love(directory: String, settings: &Config) {
    let method = METHOD_DEFLATED;

    let src_dir = &directory;
    let dst_file: &str = "test.love";

    match collect_zip_directory(src_dir, dst_file, method.unwrap(), settings) {
        Ok(_) => {
            println!("done: {} written to {}", src_dir, dst_file);
        },
        Err(e) => {
            println!("Error: {:?}", e);
        }
    }

    unsafe {
        IS_LOVE_BUILT = true;
    }
}

pub fn build_windows(directory: String, version: &LoveVersion, bitness: &Bitness) {
    unsafe {
        if !IS_LOVE_BUILT {
            println!("Error: Cannot build for windows because .love not built.");
        }
    }

    let filename = get_love_filename(version, &Platform::Windows, bitness);

    let app_dir_path = get_app_dir(AppDataType::UserData, &APP_INFO, "").unwrap();

    let mut love_exe_path = PathBuf::new();
    love_exe_path.push(&format!("{}/{}/love.exe", &app_dir_path.display(), &filename));
    if !love_exe_path.exists() {
        println!("\nlove.exe not found at '{:?}'\nYou may need to download LÖVE first: `love-kit download <version>`\n\tRecommended version: 11.1", love_exe_path);
        panic!();
    }

    let output_file_name = get_output_filename(String::from("game"), &Platform::Windows, bitness);
    let output_path = Path::new(output_file_name.as_str());

    println!("Copying love from {}", love_exe_path.display());

    let mut output_file = match File::create(&output_path) {
        Ok(file) => file,
        Err(why) => {
            panic!("Unable to create file `{}`: {}", output_path.display(), why);
        }
    };

    let paths = &[
        love_exe_path.as_path(),
        Path::new("./test.love"),
    ];

    let mut buffer = Vec::new();
    for path in paths {
        if path.is_file() {
            let mut file = match File::open(path) {
                Ok(file) => file,
                Err(why) => panic!("Could not open file: {}", why),
            };

            match file.read_to_end(&mut buffer) {
                Ok(_) => {},
                Err(why) => panic!("Could not read file: {}", why),
            };

            match output_file.write_all(&*buffer) {
                Ok(_) => {},
                Err(why) => panic!("Could not write output file: {}", why),
            };

            buffer.clear();
        }
    }
}

fn should_exclude_file(file_name: String, settings: &Config) -> bool {
    let ignores_list: Vec<String> = settings.get("ignore_list").unwrap();
    
    for exclude_name in ignores_list {
        if file_name.find(&exclude_name) != None {
            return true;
        }
    }

    return false
}

fn zip_directory<T>(it: &mut Iterator<Item=DirEntry>, prefix: &str, writer: T, method: zip::CompressionMethod, settings: &Config)
              -> zip::result::ZipResult<()>
    where T: Write+Seek
{
    let mut zip = zip::ZipWriter::new(writer);
    let options = FileOptions::default()
        .compression_method(method)
        .unix_permissions(0o644);

    let mut buffer = Vec::new();
    for entry in it {
        let path = entry.path();
        let name = path.strip_prefix(Path::new(prefix))
            .unwrap()
            .to_str()
            .unwrap();

        if path.is_file() && !should_exclude_file(name.to_string(), settings) {
            println!("adding {:?} ...", name);
            //println!("adding as {:?} ...", path, name);
            zip.start_file(name, options)?;
            let mut f = File::open(path)?;

            f.read_to_end(&mut buffer)?;
            zip.write_all(&*buffer)?;
            buffer.clear();
        }
    }
    zip.finish()?;
    Result::Ok(())
}

fn collect_zip_directory(src_dir: &str, dst_file: &str, method: zip::CompressionMethod, settings: &Config) -> zip::result::ZipResult<()> {
    if !Path::new(src_dir).is_dir() {
        return Err(ZipError::FileNotFound);
    }

    let path = Path::new(dst_file);
    let file = File::create(&path).unwrap();

    let walkdir = WalkDir::new(src_dir.to_string());
    let it = walkdir.into_iter();

    zip_directory(&mut it.filter_map(|e| e.ok()), src_dir, file, method, settings)?;

    Ok(())
}
