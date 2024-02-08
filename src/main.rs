use clap::Parser;
use core::panic;
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    fs::{self, File},
    io::Write,
    path::{Path, PathBuf},
};

#[derive(Parser, Debug)]
#[clap(about = r#"
Activate an environment
"#)]
struct Activate {
    /// Name of the environment to activate. If not provided, any active environment will be deactivated.
    env_name: Option<String>,

    /// If provided, the command to unset the old env variables and load the new env will be sent to std out. This
    /// is useful if you want your current shell to take on the output e.g. `eval "$(activate dev -e)"`. (Linux only)
    #[arg(short, default_value = "false")]
    eval: bool,

    /// If provided, will recursively activate the environment in the current directory and all subdirectories. Ignores files
    /// specified in `.gitignore` and hidden files.
    #[arg(short, default_value = "false")]
    recursive: bool,
}

const ACTIVATE_TOML: &'static str = "activate.toml";
const ACTIVATE_DIR: &'static str = ".activate";
const ACTIVATE_STATE_DIR: &'static str = "state";
const ENV_FILE: &'static str = "env.json";
const LINKS_FILE: &'static str = "links.toml";

fn main() {
    let args: Activate = Activate::parse();

    let is_recursive = args.recursive;
    let selected_env = args.env_name;
    let eval = args.eval;

    let mut output = Vec::<String>::new();
    if is_recursive {
        let (tx, rx) = crossbeam_channel::unbounded::<String>();

        ignore::WalkBuilder::new(".")
            .hidden(true)
            .git_ignore(true)
            .git_global(false)
            .git_exclude(false)
            .parents(true)
            .threads(num_cpus::get())
            .build_parallel()
            .run(|| {
                let tx = tx.clone();
                let selected_env = selected_env.clone();
                Box::new(move |result| {
                    let entry = result.expect("Could not get entry.");
                    let path = entry.path();
                    if path.is_dir() {
                        let activate_file = path.join(ACTIVATE_TOML);
                        if activate_file.exists() {
                            activate(&activate_file, selected_env.clone(), eval)
                                .map(|o| tx.send(o).expect("Could not send output."));
                        }
                    }
                    ignore::WalkState::Continue
                })
            });

        drop(tx);
        for r in rx {
            output.push(r);
        }
    } else {
        let activate_file = Path::new(ACTIVATE_TOML);
        if !activate_file.exists() {
            panic!(
                "No `{}` file found in the current directory.",
                ACTIVATE_TOML
            );
        }
        activate(&activate_file, selected_env, eval).map(|o| output.push(o));
    }

    let output = output.join("\n");
    if !output.is_empty() {
        println!("{}", output);
    }
}

/// Sources parameters and activates the environment. Returns a string to set the environment variables if `eval` is true.
fn activate(activate_file: &Path, selected_env: Option<String>, eval: bool) -> Option<String> {
    let contents = fs::read_to_string(&activate_file)
        .expect(&format!("Could not read `{}` file.", ACTIVATE_TOML));
    let toml: Environments =
        toml::from_str(&contents).expect(&format!("Could not parse `{}`.", ACTIVATE_TOML));

    let current_dir = activate_file.parent().unwrap();
    let activate_dir = current_dir.join(ACTIVATE_DIR);
    let state_dir = activate_dir.join(ACTIVATE_STATE_DIR);
    let env_file = state_dir.join(ENV_FILE);
    let links_file = state_dir.join(LINKS_FILE);

    let new_env: Option<HashMap<String, String>>;
    let old_env_vars;
    if let Some(selected_env) = &selected_env {
        let EnvironmentData { env, links } = toml
            .0
            .expect(&format!("No environments found in `{}`.", ACTIVATE_TOML))
            .remove(selected_env)
            .expect(&format!("'{}' is not a valid environment", &selected_env));

        if state_dir.exists() {
            old_env_vars = decativate_current(&env_file, &links_file, &current_dir);
        } else {
            old_env_vars = None;
            fs::create_dir_all(&state_dir).expect(&format!(
                "Could not create `{}` directory.",
                state_dir.to_string_lossy()
            ));
            create_gitignore_file(&state_dir);
        }

        activate_new(&env, &env_file, &links, &links_file, &current_dir);
        new_env = env;
    } else {
        if state_dir.exists() {
            old_env_vars = decativate_current(&env_file, &links_file, &current_dir);
        } else {
            old_env_vars = None;
        }
        new_env = None;
    }

    if eval {
        let mut output = String::new();
        if let Some(old_env) = old_env_vars {
            if let Some(old_env_vars) = old_env.0 {
                let mut keys: Vec<String> = old_env_vars.into_keys().collect();
                keys.sort();
                for key in keys {
                    output.push_str(&format!("unset {}\n", key));
                }
            }
        }
        if let Some(env) = new_env {
            let mut keys: Vec<&str> = env.keys().map(|k| k.as_str()).collect();
            keys.sort();
            for key in keys {
                if let Some(value) = env.get(key) {
                    output.push_str(&format!("export {}={}\n", key, value));
                }
            }
        }
        return Some(output);
    }
    return None;
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
    current_dir: &Path,
) -> Option<ActiveEnvironmentEnv> {
    let old_env_vars;
    if current_env_file.exists() {
        old_env_vars = Some(remove_env(current_env_file));
    } else {
        old_env_vars = None;
    }
    if current_links_file.exists() {
        remove_links(current_links_file, current_dir);
    }

    old_env_vars
}

/// Activates the new environment.
fn activate_new(
    env: &Option<HashMap<String, String>>,
    env_file: &Path,
    links: &Option<HashMap<String, String>>,
    links_file: &Path,
    current_dir: &Path,
) {
    if let Some(env) = env {
        add_env(env, env_file);
    }
    if let Some(links) = links {
        add_links(links, links_file, current_dir);
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

fn add_links(
    links: &HashMap<String, String>,
    current_links_file: &Path,
    current_dir: &Path,
) {
    let mut links_file = File::options()
        .create(true)
        .append(true)
        .open(current_links_file)
        .expect(&format!("Could not open `{}` file.", LINKS_FILE));
    for (key, value) in links {
        let source = Path::new(&key);
        if source.starts_with("./") || source.starts_with("../") {
            panic!("The source `{}` should not start with `./` or `../`. The source is relative to the `.activate` directory.", source.to_string_lossy());
        }
        let mut source = current_dir.join(source);
        if source.starts_with("./") {
            source = source.strip_prefix("./").unwrap().to_path_buf();
        }
        if !source.exists() {
            panic!("The source `{}` does not exist.", source.to_string_lossy());
        }
        let target = Path::new(&value);
        if target.starts_with("./") || target.starts_with("../") {
            panic!("The target `{}` should not start with `./` or `../`. The target is relative to the `.activate` directory.", target.to_string_lossy());
        }
        let mut target = current_dir.join(target);
        if target.starts_with("./") {
            target = target.strip_prefix("./").unwrap().to_path_buf();
        }
        if target.exists() {
            panic!("The target `{}` already exists.", target.to_string_lossy());
        }
        if target.is_symlink() {
            panic!("The link `{}` already exists.", target.to_string_lossy());
        }
        links_file
            .write_all(&format!("\"{}\"=\"{}\"\n", key, value).as_bytes())
            .expect(&format!(
                "Could not write to `{}` file. In directory `{}`.",
                LINKS_FILE,
                current_dir.to_string_lossy()
            ));
        let depth_adjustment = PathBuf::from(value)
            .components()
            .skip(1)
            .fold(PathBuf::new(), |p, _| p.join(".."));
        let link_path = depth_adjustment.join(key);
        #[cfg(windows)]
        {
            let metadata = fs::symlink_metadata(&key)
                .expect(&format!("Could not get metadata for `{}`.", &key));
            if metadata.is_dir() {
                std::os::windows::fs::symlink_any(link_path, target).expect(&format!(
                    "Could not create link from `{}` to `{}`, in directory `{}`.",
                    &key,
                    &value,
                    current_active_dir.to_string_lossy()
                ));
            } else {
                std::os::windows::fs::symlink_file(link_path, target).expect(&format!(
                    "Could not create link from `{}` to `{}`, in directory `{}`.",
                    &key,
                    &value,
                    current_active_dir.to_string_lossy()
                ));
            }
        }
        #[cfg(unix)]
        std::os::unix::fs::symlink(link_path, target).expect(&format!(
            "Could not create link from `{}` to `{}`, in directory `{}`.",
            &key,
            &value,
            current_dir.to_string_lossy()
        ));
    }
}

fn remove_links(current_links_file: &Path, current_dir: &Path) {
    let links_string = fs::read_to_string(&current_links_file)
        .expect(&format!("Could not read `{}` file.", LINKS_FILE));
    let links = toml::from_str::<ActiveEnvironmentLinks>(&links_string)
        .expect(&format!("Could not parse `{}` file.", LINKS_FILE));
    if let Some(links) = links.0 {
        for (_key, value) in links {
            let target = current_dir.join(&value);
            if target.exists() {
                if target.is_symlink() {
                    fs::remove_file(&target).expect(&format!(
                        "Could not remove link `{}`.",
                        target.to_string_lossy()
                    ));
                } else {
                    panic!("The existing link `{}` is not a symlink. Therefore it will not be removed.", target.to_string_lossy());
                }
            }
        }
    }

    fs::remove_file(&current_links_file).expect(&format!(
        "Could not remove `{}` file. Links are still active.",
        LINKS_FILE
    ));
}
