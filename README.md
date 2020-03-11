# boon

boon is a build and deploy tool for LÖVE. It makes it easy to package your game for multiple platforms, similar to [love-release](https://github.com/MisterDA/love-release). It is a single executable with no other dependecies. It can be used across multiple projects and makes automated building a breeze.

![Rust](https://github.com/camchenry/boon/workflows/Rust/badge.svg)

Licensed under the MIT License.

# Features
* Package your game for multiple platforms. Supported platforms:
    * Native (.love)
    * Windows (.exe)
    * macOS (.app)
    * Linux (coming soon)
* Package your game for multiple versions of LÖVE. Supported versions:
    * 11.x
    * 0.10.2
* No external dependencies
* Deploy to multiple destinations (coming soon)

# Getting started

## Installation

### Download prebuilt binaries (recommended)

boon has prebuilt binaries on the GitHub Releases page. Download the zip file, then extract the executable onto your PATH.

If you're a **Windows** user, download the `boon-windows-amd64.zip` file.

If you're a **macOS** user, download the `boon-macos-amd64.zip` file.

If you're a **Linux** user, download the `boon-linux-amd64.zip` file.

## Usage

### Initialization
To start using boon with your project, it is recommended to first initialize it. This will create a `Boon.toml` file that will let you configure the settings for your project.

```bash
$ boon init
```

If you don't initialize boon, you can still build your project normally, but the default configuration will be used to build it instead. You can initialize it later, or create a `Boon.toml` file yourself.

### Downloading LÖVE

In order to build your project, you first need to download the version of LÖVE that you are using for it.

```bash
# Will download LÖVE 11.3 for building
$ boon love download 11.3
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
