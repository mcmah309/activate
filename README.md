# activate

[![crates.io](https://img.shields.io/crates/v/activate)](https://crates.io/crates/activate)
[![License: MIT](https://img.shields.io/badge/license-MIT-purple.svg)](https://opensource.org/licenses/MIT)
[![Build Status](https://github.com/mcmah309/activate/actions/workflows/rust.yml/badge.svg)](https://github.com/mcmah309/activate/actions)

A tool to manage environment-specific configurations. Simplifying working across various settings like Development, Testing, Production, etc.

## Motivation
### Problem Statement
Code in different environments such as Dev, QA, Prod, etc. may need various configurations to run. 
The solution often used is loading environment variables or property files at build or run time.
This by itself has a few drawbacks:
- Setting up environments may take some additional imperative configuration, or worse, manual developer setup.
- Switching between environments is tedious.
- Developers may have to maintain custom implementations and build scripts.
- No good solution exists for switching entire monorepos between environments. Instead, a master config is often used.

### Solution
`activate` solves all these problems.
- An `active.toml` file declaratively defines environments.
- Loading and unloading an environment is as easy as a single command.
- No custom build scripts necessary. Each environment has managed files/directories and environment variables.
- The config can be distributed throughout a repo and everything switched with a single command.

## More Details

### Files and Directories
Different environments like Dev, QA, etc. may have assets, data files, executables, or program files that should be used
in each. All of these can be switched over at once. e.g.
```toml
[dev.links]
"app/data" = "path/to/dev/data"

[qa.links]
"app/data" = "path/to/qa/data"
```
The result of the above is `app/data` is created and symlinked to the file or directory of the active environment.

### Env Variables
Often each environment has specific environment variables. This can be easily defined.
e.g.
```toml
[dev.env]
HOST = "localhost"
PORT = 3000

[qa.env]
HOST = "178.32.44.2"
PORT = 443
```
To load an environment into the current shell, and unload any activate environment, run
```bash
eval "$(activate <name>)"`
```
Consider adding the following to `~/.bashrc` as a shortcut
```bash
a() {
    eval "$(activate "$@")";
}
```
Then environments can be easily loaded with an even shorter command
```bash
a dev
```

### Monorepo
`activate.toml` files can be distributed across a codebase, where each application has its own
`activate.toml` file. From the root of the repo everything can be switched together with the `-d`
flag. e.g.
```bash
activate -d <name>
```
Any directory/subdirectory (respecting `.gitignore`) with an `activate.toml` file is switched to `<name>`.

## `activate.toml` Schema
```
[<ENV_NAME>.env]
<ENV_VAR_NAME> = <ENV_VAR_VALUE>

[<ENV_NAME>.links]
"<LINK_PATH_FROM_ROOT>" = "<SOURCE_PATH_FROM_ROOT>"
```

## Install

## Debian - Ubuntu, Linux Mint, Pop!_OS, etc.

```bash
release_ver=<INSERT_CURRENT_VERSION> # e.g. release_ver='v0.2.7'
deb_file="activate_$(echo $release_ver | sed 's/^v//')-1_amd64.deb"
curl -LO https://github.com/mcmah309/activate/releases/download/$release_ver/$deb_file
dpkg -i "$deb_file"
```

## Cargo
```bash
cargo install activate
```