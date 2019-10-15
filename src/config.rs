use crate::{request::Request, TimerUpdater};

use clap::{App, AppSettings, Arg, ArgMatches, SubCommand};

use std::{env, path::Path};

pub enum Config {
    Daemon(TimerUpdater),
    Remote(Request),
}

impl Config {
    pub fn new() -> Self {
        let matches = Self::new_app().get_matches();
        match matches.subcommand() {
            ("daemon", Some(sub_m)) => {
                Self::Daemon(TimerUpdater::new(sub_m.value_of("UPDATER").unwrap().into()))
            }
            ("remote", Some(sub_m)) => Self::Remote(Request::from_matches(&sub_m)),
            _ => unreachable!(),
        }
    }

    fn new_app<'a, 'b>() -> App<'a, 'b> {
        App::new("rimer")
            .setting(AppSettings::SubcommandRequired)
            .max_term_width(80)
            .version("1.0")
            .author("xFA25E")
            .about(include_str!("../help/rimer.txt"))
            .subcommand(
                SubCommand::with_name("daemon")
                    .about("Starts main background process")
                    .arg(
                        Arg::with_name("UPDATER")
                            .empty_values(false)
                            .help(include_str!("../help/rimer_daemon_updater.txt"))
                            .next_line_help(true)
                            .required(true)
                            .validator(validate_program)
                            .value_name("UPDATER"),
                    ),
            )
            .subcommand(
                SubCommand::with_name("remote")
                    .about(include_str!("../help/rimer_remote.txt"))
                    .arg(
                        Arg::with_name("COMMAND")
                            .empty_values(false)
                            .help("Remote command")
                            .possible_values(&[
                                "stopwatch",
                                "countdown",
                                "pause",
                                "halt",
                                "resume",
                                "report",
                                "quit",
                            ])
                            .required(true)
                            .requires_ifs(&[
                                ("countdown", "NAME"),
                                ("countdown", "DURATION"),
                                ("countdown", "STEP"),
                                ("stopwatch", "NAME"),
                                ("stopwatch", "STEP"),
                                ("pause", "NAME"),
                                ("halt", "NAME"),
                                ("resume", "NAME"),
                            ])
                            .value_name("COMMAND"),
                    )
                    .arg(
                        Arg::with_name("NAME")
                            .empty_values(false)
                            .help("Timer name")
                            .value_name("NAME"),
                    )
                    .arg(
                        Arg::with_name("DURATION")
                            .empty_values(false)
                            .help("Seconds to run")
                            .long("duration")
                            .short("d")
                            .takes_value(true)
                            .validator(validate_positive_number)
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
                            .validator(validate_positive_number)
                            .value_name("STEP"),
                    ),
            )
    }
}

impl Request {
    pub fn from_matches(matches: &ArgMatches) -> Self {
        let get_value = move |s| matches.value_of(s).unwrap();
        let get_num_value = |s| get_value(s).parse::<u64>().unwrap();
        let get_name = || get_value("NAME");

        match get_value("COMMAND") {
            "countdown" => {
                let dur = get_num_value("DURATION");
                let step = get_num_value("STEP");
                Self::Countdown {
                    name: get_name().to_string(),
                    dur,
                    step,
                }
            }
            "stopwatch" => {
                let step = get_num_value("STEP");
                Self::Stopwatch {
                    name: get_name().to_string(),
                    step,
                }
            }
            "pause" => Self::Pause(get_name().to_string()),
            "halt" => Self::Halt(get_name().to_string()),
            "resume" => Self::Resume(get_name().to_string()),
            "report" => Self::Report,
            "quit" => Self::Quit,
            _ => unreachable!(),
        }
    }
}

type ClapResult = Result<(), String>;

fn validate_positive_number(s: String) -> ClapResult {
    match s.parse::<u64>() {
        Ok(num) => {
            if num == 0 {
                Err("count can't be zero".into())
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
                return Ok(());
            }
        }
    }
    Err("can't find given updater command".into())
}
