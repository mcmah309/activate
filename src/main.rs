use clap::Parser;
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    env,
    fs::{self, File},
    io::{BufReader, Read, Write},
    iter::Map,
    path::Path,
};
use toml::{de, Value};

#[derive(Parser, Debug)]
#[clap(about = r#"
Activate an environment
"#)]
struct Activate {
    /// Name of the environment to activate
    env_name: String,
}

const ACTIVATE_TOML: &'static str = "activate.toml";
const ACTIVATE_DIR: &'static str = ".activate";
const ENV_FILE: &'static str = "env.json";

fn main() {
    let activate_file = Path::new(ACTIVATE_TOML);
    if !activate_file.exists() {
        panic!(
            "No `{}` file found in the current directory.",
            ACTIVATE_TOML
        );
    }
    let contents = fs::read_to_string(&activate_file)
        .expect(&format!("Could not read `{}` file.", ACTIVATE_TOML));
    let mut toml: Environments =
        toml::from_str(&contents).expect(&format!("Could not parse `{}`.", ACTIVATE_TOML));

    let args: Activate = Activate::parse();
    let selected_env = args.env_name;
    let EnvironmentData { env } = toml
        .0
        .remove(&selected_env)
        .expect(&format!("'{}' is not a valid environment", &selected_env));

    let activate_current_dir = Path::new(ACTIVATE_DIR);
    if activate_current_dir.exists() {
        decativate_old_env(&activate_current_dir);
    } else {
        fs::create_dir(activate_current_dir)
            .expect(&format!("Could not create `{}` directory.", ACTIVATE_DIR));
    }

    let activate_env_file = fs::File::create(activate_current_dir.join(ENV_FILE))
        .expect(&format!("Could not create `{}` file.", ENV_FILE));
    add_env_vars(&env, &activate_env_file);
}

#[derive(Debug, Deserialize)]
struct Environments(HashMap<String, EnvironmentData>);

#[derive(Debug, Serialize, Deserialize)]
struct EnvironmentData {
    env: HashMap<String, String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct ActiveEnvironmentEnv(HashMap<String, String>);

fn decativate_old_env(activate_current_dir: &Path) {
    let activate_env_file = activate_current_dir.join(ENV_FILE);
    if !activate_env_file.exists() {
        return;
    }
    let contents = fs::read_to_string(&activate_env_file)
        .expect(&format!("Could not read `{}` file.", ENV_FILE));

    let current_env: ActiveEnvironmentEnv =
        serde_json::from_str(&contents).expect(&format!("Could not parse `{}`.", ENV_FILE));

    remove_env_vars(&current_env.0);
    fs::remove_file(&activate_env_file).expect(&format!(
        "Could not remove `{}` file, but environment variables were deactivated. Please remove it manually.",
        ENV_FILE
    ));
}

fn remove_env_vars(env_vars: &HashMap<String, String>) {
    for (key, _value) in env_vars {
        env::remove_var(key);
    }
}

fn add_env_vars(env_vars: &HashMap<String, String>, mut activate_current_file: &File) {
    for (key, value) in env_vars {
        env::set_var(key, value);
    }
    activate_current_file
        .write(
            serde_json::to_string(&ActiveEnvironmentEnv(env_vars.clone()))
                .expect("Could not serialize environment variables")
                .as_bytes(),
        )
        .expect(&format!("Could not write to `{}` file.", ENV_FILE));
}
