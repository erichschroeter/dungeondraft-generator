use clap::{value_parser, Arg};
use config::{Config, Environment, File};
use directories::UserDirs;
use log::{debug, error, info, trace, warn, LevelFilter};
use serde::Deserialize;
use std::path::PathBuf;

#[derive(Debug, Deserialize)]
struct Settings {
    verbose: String,
    config_path: PathBuf,
}

impl Default for Settings {
    fn default() -> Self {
        Settings {
            verbose: "info".to_string(),
            config_path: default_config_path(),
        }
    }
}

impl std::fmt::Display for Settings {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl From<Config> for Settings {
    fn from(value: Config) -> Self {
        let mut cfg = Settings::default();
        if let Ok(o) = value.get_string("verbose") {
            cfg.verbose = o;
        }
        if let Ok(o) = value.get_string("config") {
            cfg.config_path = PathBuf::new().join(o);
        }
        cfg
    }
}

fn default_config_path() -> PathBuf {
    let user_dirs = UserDirs::new().unwrap();
    let mut path = PathBuf::from(user_dirs.home_dir());
    path.push("config/fixme/default.json");
    path
}

fn setup_logging(verbose: &str) {
    env_logger::builder()
        .filter(None, verbose.parse().unwrap_or(LevelFilter::Info))
        .init();
}

fn main() {
    const ABOUT: &str = "An example CLI program using the following crates:

  - clap
  - config
  - env_logger
  - directories
  - serde";
    let matches = clap::Command::new("fixme")
        .version("v1.0.0")
        .author("Erich Schroeter <erich.schroeter@gmail.com>")
        .about(ABOUT)
        .long_about(format!(
            "{}

Argument values are processed in the following order, using the last processed value:

  1. config file (e.g. $HOME/config/fixme/default.json)
  2. environment variable (e.g. FIXME_config=<path>)
  3. explicit argument (e.g. --config <path>)",
            ABOUT
        ))
        .arg(
            Arg::new("config")
                .short('c')
                .long("config")
                .value_name("FILE")
                .help(format!(
                    "Sets a custom config file [default: {}]",
                    Settings::default().config_path.display().to_string()
                ))
                .value_parser(value_parser!(PathBuf)),
        )
        .arg(
            Arg::new("verbose")
                .short('v')
                .long("verbose")
                .value_name("VERBOSE")
                .help(format!(
                    "Sets the verbosity log level [default: {}]",
                    Settings::default().verbose
                ))
                .long_help("Choices: [error, warn, info, debug, trace]"),
        )
        .get_matches();

    let settings = Config::builder()
        .add_source(
            File::with_name(&Settings::default().config_path.display().to_string()).required(false),
        )
        .add_source(Environment::with_prefix("FIXME"))
        .build()
        .unwrap();

    let mut settings: Settings = settings.try_into().unwrap();

    if let Some(o) = matches.get_one::<String>("verbose") {
        settings.verbose = o.to_owned();
    }

    if let Some(o) = matches.get_one::<PathBuf>("config") {
        settings.config_path = o.to_owned();
    }

    setup_logging(&settings.verbose);

    error!("testing");
    warn!("testing");
    info!("{}", settings);
    debug!("testing");
    trace!("testing");
}
