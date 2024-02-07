use clap::Parser;
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    env,
    fs::{self, File},
    io::{BufReader, Read, Write},
    iter::Map,
    path::{Path, PathBuf},
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
const LINKS_FILE: &'static str = "links.toml";

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
    let EnvironmentData { env, links } = toml
        .0
        .expect(&format!("No environments found in `{}`.", ACTIVATE_TOML))
        .remove(&selected_env)
        .expect(&format!("'{}' is not a valid environment", &selected_env));

    let activate_current_dir = Path::new(ACTIVATE_DIR);
    let state_dir = activate_current_dir.join(ACTIVATE_STATE_DIR);
    let env_file = state_dir.join(ENV_FILE);
    let links_file = state_dir.join(LINKS_FILE);

    let old_env_vars;
    if state_dir.exists() {
        old_env_vars = decativate_current(&env_file, &links_file);
    } else {
        old_env_vars = None;
        fs::create_dir_all(&state_dir).expect(&format!(
            "Could not create `{}` directory.",
            state_dir.to_string_lossy()
        ));
        create_gitignore_file(&state_dir);
    }

    activate_new(&env, &env_file, &links, &links_file);

    if args.eval {
        let mut output = String::new();
        if let Some(old_env) = old_env_vars {
            if let Some(old_env_vars) = old_env.0 {
                for (key, _) in old_env_vars {
                    output.push_str(&format!("unset {}\n", key));
                }
            }
        }
        if let Some(env) = env {
            for (key, value) in env {
                output.push_str(&format!("export {}={}\n", key, value));
            }
        }
        print!("{}", output);
    }
}

#[derive(Debug, Deserialize)]
struct Environments(Option<HashMap<String, EnvironmentData>>);

#[derive(Debug, Serialize, Deserialize)]
struct EnvironmentData {
    env: Option<HashMap<String, String>>,
    links: Option<HashMap<String, String>>,
}

#[derive(Debug, Serialize, Deserialize)]
struct ActiveEnvironmentEnv(Option<HashMap<String, String>>);

#[derive(Debug, Serialize, Deserialize)]
struct ActiveEnvironmentLinks(Option<HashMap<String, String>>);

fn decativate_current(
    current_env_file: &Path,
    current_links_file: &Path,
) -> Option<ActiveEnvironmentEnv> {
    let old_env_vars;
    if current_env_file.exists() {
        old_env_vars = Some(remove_env(current_env_file));
    } else {
        old_env_vars = None;
    }
    if current_links_file.exists() {
        remove_links(current_links_file);
    }

    old_env_vars
}

fn activate_new(
    env: &Option<HashMap<String, String>>,
    env_file: &Path,
    links: &Option<HashMap<String, String>>,
    links_file: &Path,
) {
    if let Some(env) = env {
        add_env(env, env_file);
    }
    if let Some(links) = links {
        add_links(links, links_file);
    }
}

//************************************************************************//

fn add_env(env_vars: &HashMap<String, String>, env_file: &Path) {
    let mut env_file = File::options()
        .create(true)
        .append(true)
        .open(&env_file)
        .expect(&format!("Could not create `{}` file.", ENV_FILE));
    env_file
        .write(
            serde_json::to_string(&ActiveEnvironmentEnv(Some(env_vars.clone())))
                .expect("Could not serialize environment variables")
                .as_bytes(),
        )
        .expect(&format!("Could not write to `{}` file.", ENV_FILE));
}

fn remove_env(current_env_file: &Path) -> ActiveEnvironmentEnv {
    let env_string = fs::read_to_string(&current_env_file)
        .expect(&format!("Could not read `{}` file.", ENV_FILE));
    let old_env_vars_result = serde_json::from_str::<ActiveEnvironmentEnv>(&env_string);
    let old_env_vars = match old_env_vars_result {
        Ok(ok) => ok,
        Err(err) => {
            if err.is_eof() {
                ActiveEnvironmentEnv(None)
            } else {
                panic!("Could not parse `{}` file. Error was: {}", ENV_FILE, err)
            }
        }
    };

    fs::remove_file(&current_env_file).expect(&format!(
        "Could not remove `{}` file. Environemnt is still active.",
        ENV_FILE
    ));

    old_env_vars
}

//************************************************************************//

fn create_gitignore_file(state_dir: &Path) {
    fs::write(&state_dir.join(".gitignore"), "*").expect("Could not create `.gitignore` file.");
}

//************************************************************************//

fn add_links(links: &HashMap<String, String>, current_links_file: &Path) {
    let mut links_file = File::options()
        .create(true)
        .append(true)
        .open(current_links_file)
        .expect(&format!("Could not open `{}` file.", LINKS_FILE));
    for (key, value) in links {
        let source = Path::new(&key);
        if !source.exists() {
            panic!("The link `{}` does not exist.", source.to_string_lossy());
        }
        if source.starts_with("./") || source.starts_with("../") {
            panic!("The source `{}` should not start with `./` or `../`. The source is relative to the `.activate` directory.", source.to_string_lossy());
        }
        let target = Path::new(&value);
        if target.exists() {
            panic!("The link `{}` already exists.", target.to_string_lossy());
        }
        if target.starts_with("./") || target.starts_with("../") {
            panic!("The target `{}` should not start with `./` or `../`. The target is relative to the `.activate` directory.", target.to_string_lossy());
        }
        links_file
            .write_all(&format!("\"{}\"=\"{}\"\n", key, value).as_bytes())
            .expect(&format!("Could not write to `{}` file.", LINKS_FILE));
        let metadata =
            fs::symlink_metadata(&key).expect(&format!("Could not get metadata for `{}`.", &key));
        let depth = target
            .components()
            .skip(1)
            .fold(PathBuf::new(), |p, _| p.join(".."));
        let link_path = depth.join(source);
        if metadata.is_dir() {
            #[cfg(windows)]
            std::os::windows::fs::symlink_any(link_path, target).expect(&format!(
                "Could not create link from `{}` to `{}`.",
                &key, &value
            ));
        } else {
            #[cfg(windows)]
            std::os::windows::fs::symlink_file(link_path, target).expect(&format!(
                "Could not create link from `{}` to `{}`.",
                &key, &value
            ));
        }
        #[cfg(unix)]
        std::os::unix::fs::symlink(link_path, target).expect(&format!(
            "Could not create link from `{}` to `{}`.",
            &key, &value
        ));
    }
}

fn remove_links(current_links_file: &Path) {
    let links_string = fs::read_to_string(&current_links_file)
        .expect(&format!("Could not read `{}` file.", LINKS_FILE));
    let links = toml::from_str::<ActiveEnvironmentLinks>(&links_string)
        .expect(&format!("Could not parse `{}` file.", LINKS_FILE));
    if let Some(links) = links.0 {
        for (_key, value) in links {
            let target = Path::new(&value);
            if target.exists() {
                fs::remove_file(&target).expect(&format!(
                    "Could not remove link `{}`. The link is still active.",
                    target.to_string_lossy()
                ));
            }
        }
    }

    fs::remove_file(&current_links_file).expect(&format!(
        "Could not remove `{}` file. Links are still active.",
        LINKS_FILE
    ));
}
