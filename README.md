# activate

A sane way to manage your environments.

## Problem Statement
Configuration should not require additional code to build or load envinments.
Yet code in Dev, QA, Prod, etc. need different configurations to run. 
The solution often used is loading environment variables or property files at build or run time.
This leads to a few problems:
- Setting up environments may take some additional impartive configuration.
- Switching between environments is tedious.
- Developers have to maintain custom impementations and build scripts.
- No good solution exists for switching entire mono-repos between environments.

## Solution
`activate` solves all these problems and more.
- create a `active.toml` file and declaratively define your environments.
- load and unloading an environment is as easy as `activate <name>` and deactivate with just `activate`.
- No custom build scripts necessary, have managed per environment files and environment variables.
- switch an entire mono-repo with `activate <name> -r`, all directories containing `activate.toml` are switched.

## Example Use Cases

1. You have assets, data files, executables, or program files that should be used in different environments like Dev, QA, etc. 
```toml
[dev.links]
"path/to/dev/data" = "app/data"

[qa.links]
"path/to/test/data" = "app/data"
```
`app/data` be a symlink to the file or directory of the active environment.

2. You want to load an environment variables
```toml
[dev.env]
HOST = 0.0.0.0
PORT = 3000

[dev.env]
HOST = 178.32.44.2
PORT = 443
```
To load into your current shell run (this will also unload any activate environment).
```bash
eval "$(activate -e <name>)"`
```
Alternatively you can load the active env file yourself or from an application, located at `.activate/state/env.json`.
3. You are using a mono-repo and you want to load to do any command recursively with any `activate.toml` files in sub-directories
```bash
activate -r <name>
```

## activate.toml Schema
```
[<ENV_NAME>.env]
<ENV_VAR_NAME> = <ENV_VAR_VALUE>

[<ENV_NAME>.links]
"<SOURCE_PATH_FROM_ROOT>" = "<TARGET_PATH_FROM_ROOT>"
```

## ROADMAP
- Allow activate multiple active environments at the same time
- Allow specifying environment entering and leaving scripts