# activate

[![crates.io](https://img.shields.io/crates/v/activate)](https://crates.io/crates/activate)
[![License: MIT](https://img.shields.io/badge/license-MIT-purple.svg)](https://opensource.org/licenses/MIT)
[![Build Status](https://github.com/mcmah309/activate/actions/workflows/rust.yml/badge.svg)](https://github.com/mcmah309/activate/actions)

A sane way to manage environment-specific configurations. Simplify the way you work and build across various settings like Development, Testing, Production, and more.

## Problem Statement
Code in different environments such as Dev, QA, Prod, etc. may need various configurations to run. 
The solution often used is loading environment variables or property files at build or run time.
This by itself has a few drawbacks:
- Setting up environments may take some additional imperative configuration or worse user setup.
- Switching between environments is tedious.
- Developers may have to maintain custom implementations and build scripts.
- No good solution exists for switching entire mono-repos between environments.

## Solution
`activate` solves all these problems.
- Create an `active.toml` file and declaratively define your environments.
- Loading and unloading an environment is as easy as `activate <name>` and deactivate with just `activate`.
- No custom build scripts necessary, have per environment managed files/directories and environment variables.
- Switch an entire mono-repo with `activate -r <name>`, all directories containing `activate.toml` are switched to `<name>`.

## Example Use Cases

1. You have assets, data files, executables, or program files that should be used in different environments like Dev, QA, etc. e.g.
    ```toml
    [dev.links]
    "app/data" = "path/to/dev/data"

    [qa.links]
    "app/data" = "path/to/qa/data"
    ```
    `app/data` is created and symlinked to the file or directory of the active environment.

2. You want different environment variables in each environment e.g.
    ```toml
    [dev.env]
    HOST = "localhost"
    PORT = 3000

    [qa.env]
    HOST = "178.32.44.2"
    PORT = 443
    ```
    To load into your current shell run (this will also unload any activate environment).
    ```bash
    eval "$(activate -e <name>)"`
    ```
    Consider adding the following to `~/.bashrc` as a shortcut
    ```bash
    a() {
        eval "$(activate -e "$@")";
    }
    ```
    To easily load and unload environments. e.g.
    ```bash
    a dev
    ```
    Alternatively you can load the active `.env` file yourself or from an application, located at `.activate/.env`.
    This can also be useful for dev containers. Just add `"runArgs": ["--env-file",".activate/.env"]` to your
    `.devcontainer/devcontainer.json` file.

3. You are using a mono-repo and want to switch everything to a certain environment. Run:
    ```bash
    activate -r <name>
    ```
    any directory/subdirecory (respecting `.gitignore`) with an `activate.toml` file is switched to `<name>`

## activate.toml Schema
```
[<ENV_NAME>.env]
<ENV_VAR_NAME> = <ENV_VAR_VALUE>

[<ENV_NAME>.links]
"<LINK_PATH_FROM_ROOT>" = "<SOURCE_PATH_FROM_ROOT>"
```

## Install

```bash
cargo install activate
```

## ROADMAP
- Allow activating multiple environments at the same time
- Allow specifying environment entering and leaving scripts
- Add defualt environment and shell hook