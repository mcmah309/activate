
## Features
- Create an `env.json` file applications can load at build or runtime
- Set  and unset environment variables for the enviornmnet in the current shell `eval "$(activate <name> -e)"
- Link and unlink files and directories for each environment
- Recusively activate all `activate.toml` environments with the same name `activate <name> -r`

## ROADMAP
- Allow activate multiple environments at the same time
- Allow specifying environment entering and leaving scripts