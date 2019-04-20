# boon

boon is a build and deploy tool for LÖVE. It makes it easy to package your game for multiple platforms, similar to [love-release](https://github.com/MisterDA/love-release). It is a single executable with no other dependecies, so once it's installed, you're ready to go. It can be used across multiple projects and makes automated building a breeze.

[![Build Status](https://dev.azure.com/cameronamchenry/boon/_apis/build/status/camchenry.boon%20(1)?branchName=master)](https://dev.azure.com/cameronamchenry/boon/_build/latest?definitionId=4&branchName=master)

Licensed under the MIT License.

# Features
* Package your game for multiple platforms. Supported platforms:
    * Native (.love)
    * Windows (.exe)
    * macOS (.app)
    * Linux (coming soon)
* Deploy to multiple destinations (coming soon)
* No external dependencies
* Easy to install

# Getting started

## Installation

### Download prebuilt binaries (recommended)

boon has prebuilt binaries on the GitHub Releases page. Download the zip file, then extract the executable onto your PATH.

If you're a **Windows** user, download the `x86_64-pc-windows-msvc` file.

If you're a **macOS** user, download the `x86_64-apple-darwin` file.

If you're a **Linux** user, download the `x86_64-unknown-linux-gnu` file.

## Usage

### Initialization
To start using boon with your project, it is recommended to first initialize it. This will create a `Boon.toml` file that will let you configure the settings for your project.

```bash
$ boon init
```

If you don't initialize boon, you can still build your project normally, but the default configuration will be used to build it instead. You can initialize it later, or create a `Boon.toml` file yourself.

### Downloading LÖVE

In order to build your project, you first need to download the versionof LÖVE that you are using for it.

```bash
# Will download LÖVE 11.2 for building
$ boon love download 11.2
```

### Building your project

Finally, to build your project just run `boon build` followed by where you want to run it. Usually, you just want to run it on the current directory, `.`.

```bash
$ boon build .
```

Without a target specified, this will build a `.love` file and put it in the `release` directory. This is shorthand for `boon build <dir> --target love`

#### Building for Windows

To build a Windows application:

```bash
$ boon build . --target windows
```

#### Building for macOS

To build a macOS application:

```bash
$ boon build . --target macos
```

### Building for a different version of LÖVE

If you would like to build for a LÖVE version other than the default, you can specify it using the `--version` flag.

```bash
$ boon build . --version 0.10.2
```

## Compiling from source

boon is written in Rust, so you will need to install [Rust](https://www.rust-lang.org/) in order to compile it.

To build boon:
```bash
git clone git@github.com:camchenry/boon.git
cd boon
cargo build --release
./target/release/boon --version
boon 0.1.0
```
