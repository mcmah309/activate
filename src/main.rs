use clap::Parser;
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    fmt::Debug,
    fs::{self, File},
    io::Write,
    path::{Path, PathBuf},
};

#[derive(Parser, Debug)]
#[clap(about = r#"
A tool to manage environment-specific configurations. Simplifying working across various settings like Development, Testing, Production, etc.
"#)]
struct ActivateArgs {
    /// Name of the environment to activate. If not provided, any active environment will be deactivated.
    env_name: Option<String>,

    /// If provided, the command to unset the old env variables and load the new env will not be sent to std out.
    #[arg(short, default_value = "false")]
    silent: bool,

    /// If provided, will activate the environment in the current directory and all subdirectories. Ignores files
    /// specified in `.gitignore` and hidden files.
    #[arg(short, default_value = "false")]
    descendants: bool,

    /// Name of the configmap to create.
    #[arg(long, default_value = "activate")]
    configmap_name: String,
}

const ACTIVATE_TOML: &'static str = "activate.toml";
const ACTIVATE_DIR: &'static str = ".activate";
const ACTIVATE_STATE_DIR: &'static str = "state";
const ENV_FILE: &'static str = "env.json";
const ALL_ENV_FILE: &'static str = ".env";
const ALL_CONFIGMAP_FILE: &'static str = "configMap";
const LINKS_FILE: &'static str = "links.toml";

fn main() {
    let args: ActivateArgs = ActivateArgs::parse();

    let ActivateArgs {
        env_name: selected_env,
        silent,
        descendants,
        configmap_name,
    } = args;

    let mut envs = Vec::<NewAndOldEnv>::new();
    if descendants {
        let (tx, rx) = crossbeam_channel::unbounded::<NewAndOldEnv>();

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
                    let entry = result.exit("Could not get entry.");
                    let path = entry.path();
                    if path.is_dir() {
                        let activate_file = path.join(ACTIVATE_TOML);
                        if activate_file.exists() {
                            tx.send(activate(&activate_file, selected_env.clone()))
                                .exit("Could not send output.");
                        }
                    }
                    ignore::WalkState::Continue
                })
            });

        drop(tx);
        for r in rx {
            envs.push(r);
        }
    } else {
        let activate_file = Path::new(ACTIVATE_TOML);
        if !activate_file.exists() {
            exit(&format!(
                "No `{}` file found in the current directory.",
                ACTIVATE_TOML
            ));
        }
        envs.push(activate(&activate_file, selected_env));
    }
    let env = envs.into_iter().fold(
        NewAndOldEnv {
            old_env: HashMap::new(),
            new_env: HashMap::new(),
        },
        |mut acc, env| {
            acc.old_env.extend(env.old_env);
            acc.new_env.extend(env.new_env);
            acc
        },
    );
    let env_file_data = env.new_env.iter().fold(String::new(), |mut s, (k, v)| {
        s.push_str(&format!("{}={}\n", k, v));
        s
    });
    fs::write(Path::new(ACTIVATE_DIR).join(ALL_ENV_FILE), env_file_data)
        .exit(format!("Could not write to `{}` file.", ALL_ENV_FILE).as_str());
    let configmap_file_data = env.new_env.iter().fold(
        format!(
            r#"apiVersion: v1
kind: ConfigMap
metadata:
  name: {}
data:
"#,
            configmap_name
        ),
        |mut s, (k, v)| {
            s.push_str(&format!("  {}: \"{}\"\n", k, v));
            s
        },
    );
    fs::write(
        Path::new(ACTIVATE_DIR).join(ALL_CONFIGMAP_FILE),
        configmap_file_data,
    )
    .exit("Could not write to `configmap` file.");

    // eval output
    if !silent {
        let mut output = Vec::<String>::new();
        let mut keys: Vec<&String> = env.old_env.keys().collect();
        keys.sort();
        for key in keys {
            output.push(format!("unset {}", key));
        }
        let mut keys: Vec<&String> = env.new_env.keys().collect();
        keys.sort();
        for key in keys {
            if let Some(value) = env.new_env.get(key) {
                output.push(format!("export {}={}", key, value));
            }
        }
        if !output.is_empty() {
            let output = output.join("\n");
            println!("{}", output);
        }
    }
}

struct NewAndOldEnv {
    old_env: HashMap<String, String>,
    new_env: HashMap<String, String>,
}

/// Sources parameters and activates the environment. Returns a strings to set the environment variables if `eval` is true.
fn activate(activate_file: &Path, selected_env: Option<String>) -> NewAndOldEnv {
    let contents = fs::read_to_string(&activate_file)
        .exit(&format!("Could not read `{}` file.", ACTIVATE_TOML));
    let toml: Environments =
        toml::from_str(&contents).exit(&format!("Could not parse `{}`.", ACTIVATE_TOML));

    let current_dir = activate_file.parent().unwrap();
    let activate_dir = current_dir.join(ACTIVATE_DIR);
    let state_dir = activate_dir.join(ACTIVATE_STATE_DIR);
    let env_file = state_dir.join(ENV_FILE);
    let links_file = state_dir.join(LINKS_FILE);

    let new_env: Option<HashMap<String, String>>;
    let old_active_env;
    if let Some(selected_env) = &selected_env {
        let EnvironmentData { env, links } = toml
            .0
            .exit(&format!("No environments found in `{}`.", ACTIVATE_TOML))
            .remove(selected_env)
            .exit(&format!("'{}' is not a valid environment", &selected_env));

        if state_dir.exists() {
            old_active_env = decativate_current(&env_file, &links_file, &current_dir);
        } else {
            old_active_env = None;
            fs::create_dir_all(&state_dir).exit(&format!(
                "Could not create `{}` directory.",
                state_dir.to_string_lossy()
            ));
            create_gitignore_file(&activate_dir);
        }

        activate_new(&env, &env_file, &links, &links_file, &current_dir);
        new_env = env;
    } else {
        if state_dir.exists() {
            old_active_env = decativate_current(&env_file, &links_file, &current_dir);
        } else {
            old_active_env = None;
        }
        new_env = None;
    }

    NewAndOldEnv {
        old_env: old_active_env.map(|e| e.0).flatten().unwrap_or_default(),
        new_env: new_env.unwrap_or_default(),
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
        .exit(&format!("Could not create `{}` file.", ENV_FILE));
    env_file
        .write(
            serde_json::to_string(&ActiveEnvironmentEnv(Some(env_vars.clone())))
                .exit("Could not serialize environment variables")
                .as_bytes(),
        )
        .exit(&format!("Could not write to `{}` file.", ENV_FILE));
}

fn remove_env(current_env_file: &Path) -> ActiveEnvironmentEnv {
    let env_string =
        fs::read_to_string(&current_env_file).exit(&format!("Could not read `{}` file.", ENV_FILE));
    let old_env_vars_result = serde_json::from_str::<ActiveEnvironmentEnv>(&env_string);
    let old_env_vars = match old_env_vars_result {
        Ok(ok) => ok,
        Err(err) => {
            if err.is_eof() {
                ActiveEnvironmentEnv(None)
            } else {
                exit(&format!(
                    "Could not parse `{}` file. Error was: {}",
                    ENV_FILE, err
                ));
            }
        }
    };

    fs::remove_file(&current_env_file).exit(&format!(
        "Could not remove `{}` file. Environemnt is still active.",
        ENV_FILE
    ));

    old_env_vars
}

//************************************************************************//

fn create_gitignore_file(activate_dir: &Path) {
    fs::write(
        &activate_dir.join(".gitignore"),
        &format!("state/\n{}\n{}", ALL_ENV_FILE, ALL_CONFIGMAP_FILE),
    )
    .exit("Could not create `.gitignore` file.");
}

//************************************************************************//

fn add_links(links: &HashMap<String, String>, current_links_file: &Path, current_dir: &Path) {
    let mut links_file = File::options()
        .create(true)
        .append(true)
        .open(current_links_file)
        .exit(&format!("Could not open `{}` file.", LINKS_FILE));
    for (key, value) in links {
        let source = Path::new(&value);
        if source.starts_with("./") || source.starts_with("../") {
            exit(&format!("The source `{}` should not start with `./` or `../`. The source is relative to the `activate.toml` directory and below.", source.to_string_lossy()));
        }
        let mut source = current_dir.join(source);
        if source.starts_with("./") {
            source = source.strip_prefix("./").unwrap().to_path_buf();
        }
        if !source.exists()
            && !source
                .components()
                .any(|c| c.as_os_str().to_str().unwrap() == ACTIVATE_DIR)
        {
            exit(&format!(
                "The source `{}` does not exist.",
                source.to_string_lossy()
            ));
        }
        let target = Path::new(&key);
        if target.starts_with("./") || target.starts_with("../") {
            exit(&format!("The target `{}` should not start with `./` or `../`. The target is relative to the `activate.toml` directory and below.", target.to_string_lossy()));
        }
        let mut target = current_dir.join(target);
        if target.starts_with("./") {
            target = target.strip_prefix("./").unwrap().to_path_buf();
        }
        if target.exists() {
            exit(&format!(
                "The target `{}` already exists.",
                target.to_string_lossy()
            ));
        }
        if target.is_symlink() {
            exit(&format!(
                "The link `{}` already exists.",
                target.to_string_lossy()
            ));
        }
        links_file
            .write_all(&format!("\"{}\"=\"{}\"\n", key, value).as_bytes())
            .exit(&format!(
                "Could not write to `{}` file. In directory `{}`.",
                LINKS_FILE,
                current_dir.to_string_lossy()
            ));
        let depth_adjustment = PathBuf::from(key)
            .components()
            .skip(1)
            .fold(PathBuf::new(), |p, _| p.join(".."));
        let link_path = depth_adjustment.join(value);
        // #[cfg(windows)]
        // {
        //     let metadata = fs::symlink_metadata(&value)
        //         .exit(&format!("Could not get metadata for `{}`.", &key));
        //     if metadata.is_dir() {
        //         std::os::windows::fs::symlink_any(link_path, target).exit(&format!(
        //             "Could not link entity `{}` to `{}`, in directory `{}`.",
        //             &key,
        //             &value,
        //             current_dir.to_string_lossy()
        //         ));
        //     } else {
        //         std::os::windows::fs::symlink_file(link_path, target).exit(&format!(
        //             "Could not link entity `{}` to `{}`, in directory `{}`.",
        //             &key,
        //             &value,
        //             current_dir.to_string_lossy()
        //         ));
        //     }
        // }
        #[cfg(unix)]
        std::os::unix::fs::symlink(link_path, target).exit(&format!(
            "Could not link entity `{}` to `{}`, in directory `{}`.",
            &key,
            &value,
            current_dir.to_string_lossy()
        ));
    }
}

fn remove_links(current_links_file: &Path, current_dir: &Path) {
    let links_string = fs::read_to_string(&current_links_file)
        .exit(&format!("Could not read `{}` file.", LINKS_FILE));
    let links = toml::from_str::<ActiveEnvironmentLinks>(&links_string)
        .exit(&format!("Could not parse `{}` file.", LINKS_FILE));
    if let Some(links) = links.0 {
        for (key, _value) in links {
            let target = current_dir.join(&key);
            if target.exists() {
                if target.is_symlink() {
                    fs::remove_file(&target).exit(&format!(
                        "Could not remove link `{}`.",
                        target.to_string_lossy()
                    ));
                } else {
                    exit(&format!("The existing link `{}` is not a symlink. Therefore it will not be removed.", target.to_string_lossy()));
                }
            }
        }
    }

    fs::remove_file(&current_links_file).exit(&format!(
        "Could not remove `{}` file. Links are still active.",
        LINKS_FILE
    ));
}

//************************************************************************//

fn exit(message: &str) -> ! {
    exit_handler(message, std::backtrace::Backtrace::force_capture());
}

fn exit_handler<E: Debug>(message: &str, error: E) -> ! {
    eprintln!("Error: {}", message);
    #[cfg(debug_assertions)]
    {
        eprintln!("{:?}", error);
    }
    std::process::exit(1);
}

trait Exit<T> {
    fn exit(self, exit_message: &str) -> T;
}

impl<T, U: Debug> Exit<T> for Result<T, U> {
    fn exit(self, exit_message: &str) -> T {
        match self {
            Ok(ok) => ok,
            Err(err) => exit_handler(exit_message, err),
        }
    }
}

impl<T> Exit<T> for Option<T> {
    fn exit(self, exit_message: &str) -> T {
        match self {
            Some(v) => v,
            None => exit_handler(exit_message, std::backtrace::Backtrace::force_capture()),
        }
    }
}
