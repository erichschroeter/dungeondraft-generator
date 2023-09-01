use clap::{value_parser, Arg};
use config::{Config, Environment, File};
use directories::UserDirs;
use log::{debug, error, info, trace, warn, LevelFilter};
use opencv::prelude::*;
use opencv::imgcodecs::imread;
use opencv::imgproc;
use opencv::core;
use opencv::types::VectorOfMat;
use serde::Deserialize;
use std::{path::{PathBuf, Path}, io};

fn create_backup(path: &Path) -> Result<bool, Box<dyn std::error::Error>> {
    let backup_path = get_backup_path(path);

    if backup_path.exists() {
        return Ok(false); // Backup already exists
    }

    std::fs::copy(path, &backup_path)?;
    Ok(true)
}

fn get_backup_path(origional_path: &Path) -> PathBuf {
    let mut backup_path = origional_path.to_path_buf();
    backup_path.set_extension("dungeondraft_map.bak");
    backup_path
}

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

fn main() -> Result<(), Box<dyn std::error::Error>> {
    const ABOUT: &str = "A program to generate DungeonDraft maps.";
    let matches = clap::Command::new("fixme")
        .version("v0.1.0")
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
        .arg(
            Arg::new("mapfile")
                .value_name("FILE")
                .help("A .dungeondraft_map file")
                .value_parser(value_parser!(PathBuf))
        )
        .arg(
            Arg::new("image")
                .short('i')
                .long("image")
                .value_name("FILE")
                .help("A .dungeondraft_map file")
                .value_parser(value_parser!(PathBuf))
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

    // if let Some(o) = matches.get_one::<PathBuf>("mapfile") {
    //     debug!("Reading {}", o.display());
    //     let file = std::fs::File::open(o)?;
    //     let reader = io::BufReader::new(file);
    //     let data: serde_json::Value = serde_json::from_reader(reader)?;
    //     create_backup(o).unwrap();
    //     debug!("{:?}", data);
    // }
    if let Some(o) = matches.get_one::<PathBuf>("image") {
        debug!("Analyzing {}", o.display());
        // let image = Reader::open(o).unwrap().decode().unwrap().to_rgb8();
        let image = imread(o.as_os_str().to_str().unwrap(), opencv::imgcodecs::ImreadModes::IMREAD_COLOR as i32)?;

        // Convert the image to grayscale
        let mut gray_image = Mat::default();
        imgproc::cvt_color(&image, &mut gray_image, imgproc::COLOR_BGR2GRAY, 0)?;

        // Apply edge detection (e.g. using the Canny algorithm)
        let mut edges = Mat::default();
        imgproc::canny(&gray_image, &mut edges, 50.0, 150.0, 3, false)?;

        // Find contours in the edge-detected image
        let mut contours = VectorOfMat::new();
        let mut hierarchy = Mat::default();
        imgproc::find_contours_with_hierarchy(
            &mut edges,
            &mut contours,
            &mut hierarchy,
            imgproc::RETR_EXTERNAL,
            imgproc::CHAIN_APPROX_SIMPLE,
            core::Point::new(0, 0),
        )?;

        // Iterate over detected contours and print their coords and dimensions
        for contour in contours.iter() {
            let area = imgproc::contour_area(&contour, false)?;
            if area > 100.0 {
                let bounding_rect = imgproc::bounding_rect(&contour)?;
                println!(
                    "Shape detected at ({}, {}) with width: {} and height {}",
                    bounding_rect.x,
                    bounding_rect.y,
                    bounding_rect.width,
                    bounding_rect.height,
                );
            }
        }
    }
    // DONE read .dungeondraft_map file
    // TODO read .png/.jpg/etc file
    // TODO insert/update/add attributes
    // DONE write .dungeondraft_map.bak if not already exist
    // TODO write .dungeondraft_map

    Ok(())
}
