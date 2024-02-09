# activate

A sane way to manage your environments.

## Problem Statement
Configuration should not require additional code to build or load environments.
Yet code in Dev, QA, Prod, etc. need different configurations to run. 
The solution often used is loading environment variables or property files at build or run time.
This by itself has a few drawbacks:
- Setting up environments may take some additional impartive configuration.
- Switching between environments is tedious.
- Developers may have to maintain custom impementations and build scripts.
- No good solution exists for switching entire mono-repos between environments.

## Solution
`activate` solves all these problems and more.
- Create a `active.toml` file and declaratively define your environments.
- Loading and unloading an environment is as easy as `activate <name>` and deactivate with just `activate`.
- No custom build scripts necessary, have per environment managed files/directories and environment variables.
- Switch an entire mono-repo with `activate -r <name>`, all directories containing `activate.toml` are switched to `<name>`.

## Example Use Cases

1. You have assets, data files, executables, or program files that should be used in different environments like Dev, QA, etc. 
    ```toml
    [dev.links]
    "path/to/dev/data" = "app/data"

    [qa.links]
    "path/to/qa/data" = "app/data"
    ```
    `app/data` is symlinked to the file or directory of the active environment.

2. You want to load an environment variables
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
    Alternatively you can load the active `.env` file yourself or from an application, located at `.activate/.env`.
    This can also be particularly useful for dev containers. Just add `"runArgs": ["--env-file",".activate/.env"]` to your
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
"<SOURCE_PATH_FROM_ROOT>" = "<TARGET_PATH_FROM_ROOT>"
```

## ROADMAP
- Allow activating multiple environments at the same time
- Allow specifying environment entering and leaving scripts
- Add defualt environment and shell hook