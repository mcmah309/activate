use clap::Parser;
use serde::Deserialize;
use std::{
    collections::HashMap, fs::{self, File}, io::{BufReader, Read}, iter::Map, path::Path
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

fn main() {
    let activate_file = Path::new("activate.toml");
    if !activate_file.exists() {
        panic!("No `activate.toml` file found in the current directory.");
    }
    let file = File::open(activate_file).expect("Could not open `activate.toml` file.");
    let mut buf_reader = BufReader::new(file);
    let mut contents = String::new();
    buf_reader
        .read_to_string(&mut contents)
        .expect("Could not read `activate.toml` file.");
    
    let toml: Value = toml::from_str(&contents).expect("Could not parse `activate.toml`.");

    let environments_map: &toml::map::Map<String, Value> = toml
        .as_table()
        .expect("Root of `activate.toml` must be a table.");

    let mut environments: Vec<Environment> = Vec::new();
    for (env_name, env_data) in environments_map.into_iter() {
        environments.push(Environment {
            name: env_name.to_string(),
            data: env_data.to_owned().try_into().expect("The table data does not follow the expect schema"),
        });
    }

    let args: Activate = Activate::parse();

    decativate_old_env();
    add_env_vars();
}

#[derive(Debug, Deserialize)]
struct Environment {
    name: String,
    data: EnvironmentData,
}

#[derive(Debug, Deserialize)]
struct EnvironmentData {
    env: HashMap<String,String>,
}


#[derive(Debug, Deserialize)]
struct EnvVar {
    name: String,
    value: String,
}

fn decativate_old_env() {
    // ...
}

fn add_env_vars() {
    // ...
}
