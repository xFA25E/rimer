use super::request::Request;
use clap::{App, Arg};
use std::{env, path::Path, time::Duration};

pub enum Config {
    Server { command: String },
    Client { request: Request },
}

impl Config {
    pub fn new() -> Self {
        let matches = App::new("rimer")
            .max_term_width(80)
            .version("1.0")
            .author("xFA25E")
            .about(include_str!("../help/rimer.txt"))
            .arg(
                Arg::with_name("COMMAND")
                    .empty_values(false)
                    .help("Command")
                    .possible_values(&[
                        "start", "add", "pause", "resume", "halt", "report",
                        "quit",
                    ])
                    .required(true)
                    .requires_ifs(&[
                        ("start", "UPDATER"),
                        ("add", "NAME"),
                        ("add", "DURATION"),
                        ("pause", "NAME"),
                        ("halt", "NAME"),
                        ("resume", "NAME"),
                    ])
                    .value_name("COMMAND"),
            )
            .arg(
                Arg::with_name("UPDATER")
                    .empty_values(false)
                    .help(include_str!("../help/rimer_updater.txt"))
                    .next_line_help(true)
                    .validator(validate_program)
                    .value_name("UPDATER"),
            )
            .arg(
                Arg::with_name("NAME")
                    .empty_values(false)
                    .help("Timer name")
                    .long("name")
                    .short("n")
                    .takes_value(true)
                    .value_name("NAME"),
            )
            .arg(
                Arg::with_name("DURATION")
                    .empty_values(false)
                    .help("Seconds to run (max value unsigned 64bit integer)")
                    .long("duration")
                    .short("d")
                    .takes_value(true)
                    .validator(validate_duration)
                    .value_name("DURATION"),
            )
            .arg(
                Arg::with_name("STEP")
                    .default_value("10")
                    .empty_values(false)
                    .help("Updater is executed every <STEP> seconds")
                    .long("step")
                    .short("s")
                    .takes_value(true)
                    .validator(validate_duration)
                    .value_name("STEP"),
            )
            .get_matches();

        let value_of = |s| matches.value_of(s).unwrap();
        let num_value_of = |s| value_of(s).parse::<u64>().unwrap();

        match value_of("COMMAND") {
            "start" => Self::Server {
                command: value_of("UPDATER").into(),
            },
            "add" => Self::Client {
                request: Request::Add {
                    name: value_of("NAME").into(),
                    duration: Duration::from_secs(num_value_of("DURATION")),
                    step: Duration::from_secs(num_value_of("STEP")),
                },
            },
            "pause" => Self::Client {
                request: Request::Pause {
                    name: value_of("NAME").into(),
                },
            },
            "halt" => Self::Client {
                request: Request::Halt {
                    name: value_of("NAME").into(),
                },
            },
            "resume" => Self::Client {
                request: Request::Resume {
                    name: value_of("NAME").into(),
                },
            },
            "report" => Self::Client {
                request: Request::Report,
            },
            "quit" => Self::Client {
                request: Request::Quit,
            },
            _ => unreachable!(),
        }
    }
}

type ClapResult = Result<(), String>;

fn validate_duration(s: String) -> ClapResult {
    match s.parse::<u64>() {
        Ok(num) => {
            if num == 0 {
                Err("Duration cannot be zero".into())
            } else {
                Ok(())
            }
        }
        Err(e) => Err(e.to_string()),
    }
}

fn validate_program(p: String) -> ClapResult {
    if Path::new(&p).exists() {
        return Ok(());
    }
    if let Ok(path) = env::var("PATH") {
        for pa in path.split(':') {
            if Path::new(&format!("{}/{}", pa, p)).exists() {
                if is_executable(Path::new(&format!("{}/{}", pa, p))) {
                    return Ok(());
                } else {
                    return Err("Updater is not executable".into());
                }
            }
        }
    }
    Err("Cannot find given updater command".into())
}

fn is_executable(path: &Path) -> bool {
    std::process::Command::new("test")
        .arg("-x")
        .arg(path)
        .status()
        .unwrap()
        .success()
}
