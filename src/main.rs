#[macro_use]
extern crate clap;
extern crate sysinfo;
mod config;
mod logging;
mod polling;

use clap::App;
use config::Config;
use logging::LoggerGroup;
use polling::{compare, get_baseline, read};
use std::{thread, time};
use sysinfo::{System, SystemExt};

fn main() {
    let yaml = load_yaml!("./../cli.yml");
    let matches = App::from_yaml(yaml).get_matches();

    let config = Config {
        baseline_duration: matches
            .value_of("baseline-duration")
            .unwrap_or("60")
            .parse()
            .unwrap(),
        polling_interval: matches
            .value_of("polling-interval")
            .unwrap_or("10")
            .parse()
            .unwrap(),
        polling_duration: matches
            .value_of("polling-duration")
            .unwrap_or("600")
            .parse()
            .unwrap(),
        standard_deviation_threshold: matches
            .value_of("standard-deviation-threshold")
            .unwrap_or("3")
            .parse()
            .unwrap(),
        log_directory: matches
            .value_of("log-directory")
            .unwrap_or("~/.deckhand/")
            .to_string(),
        logger_list: match matches.values_of("logger") {
            Some(loggers) => loggers.collect(),
            None => vec![],
        },
    };

    start(config);
}

pub fn start(config: Config) {
    let mut sys = System::new();

    let baseline = get_baseline(&mut sys, config.baseline_duration);
    let polling_interval = time::Duration::new(config.polling_interval, 0);
    let loggers = LoggerGroup::new(&config);

    loop {
        thread::sleep(polling_interval);

        let pt = read(&mut sys);
        let result = compare(&pt, &baseline, &config);

        loggers.log(&result);
    }
}
