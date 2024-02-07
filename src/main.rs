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

    /// If provided, the command to unset the old env variables and load the new env will be sent to std out. This
    /// is useful if you want your current shell to take on the output e.g. `eval "$(activate dev -e)"`
    #[arg(short, default_value = "false")]
    eval: bool,
}

const ACTIVATE_TOML: &'static str = "activate.toml";
const ACTIVATE_DIR: &'static str = ".activate";
const ACTIVATE_STATE_DIR: &'static str = "state";
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
    let state_dir = activate_current_dir.join(ACTIVATE_STATE_DIR);
    let old_env_vars;
    if state_dir.exists() {
        old_env_vars = decativate_current(&state_dir);
    } else {
        old_env_vars = None;
        fs::create_dir_all(&state_dir).expect(&format!(
            "Could not create `{}` directory.",
            state_dir.to_string_lossy()
        ));
        create_gitignore_file(&state_dir);
    }

    let activate_env_file =
        fs::File::create(activate_current_dir.join(ACTIVATE_STATE_DIR).join(ENV_FILE))
            .expect(&format!("Could not create `{}` file.", ENV_FILE));
    add_env_file(&env, &activate_env_file);
    if args.eval {
        let mut output = String::new();
        if let Some(old_env_vars) = old_env_vars {
            for (key, _) in old_env_vars.0 {
                output.push_str(&format!("unset {}\n", key));
            }
        }
        
        for (key, value) in env {
            output.push_str(&format!("export {}={}\n", key, value));
        }
        print!("{}", output);
    }
}

#[derive(Debug, Deserialize)]
struct Environments(HashMap<String, EnvironmentData>);

#[derive(Debug, Serialize, Deserialize)]
struct EnvironmentData {
    env: HashMap<String, String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct ActiveEnvironmentEnv(HashMap<String, String>);

fn decativate_current(activate_state_dir: &Path) -> Option<ActiveEnvironmentEnv> {
    let activate_env_file = activate_state_dir.join(ENV_FILE);
    if !activate_env_file.exists() {
        return None;
    }

    let env_string = fs::read_to_string(&activate_env_file).expect(&format!(
        "Could not read `{}` file.",
        ENV_FILE
    ));
    let old_env_vars: ActiveEnvironmentEnv = serde_json::from_str(&env_string)
        .expect(&format!("Could not parse `{}` file.", ENV_FILE));

    fs::remove_file(&activate_env_file).expect(&format!(
        "Could not remove `{}` file. Environemnt is still active.",
        ENV_FILE
    ));

    Some(old_env_vars)
}

//************************************************************************//

fn add_env_file(env_vars: &HashMap<String, String>, mut current_env_file: &File) {
    current_env_file
        .write(
            serde_json::to_string(&ActiveEnvironmentEnv(env_vars.clone()))
                .expect("Could not serialize environment variables")
                .as_bytes(),
        )
        .expect(&format!("Could not write to `{}` file.", ENV_FILE));
}


//************************************************************************//

fn create_gitignore_file(state_dir: &Path) {
    fs::write(&state_dir.join(".gitignore"), "*").expect("Could not create `.gitignore` file.");
}

//************************************************************************//

// fn add_links_file(links: &HashMap<String, String>, mut current_links_file: &File) {
//     current_links_file
//         .write(
//             serde_json::to_string(&ActiveEnvironmentEnv(links.clone()))
//                 .expect("Could not serialize links")
//                 .as_bytes(),
//         )
//         .expect(&format!("Could not write to `{}` file.", ENV_FILE));
// }
