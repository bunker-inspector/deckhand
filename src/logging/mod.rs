mod print;

use crate::config::Config;
use crate::polling::PollResult;

use print::PrintLogger;

trait Logger {
    fn log(&self, pollresult: &PollResult);
}

pub struct LoggerGroup {
    loggers: Vec<Box<dyn Logger>>,
}

impl<'a> LoggerGroup {
    pub fn new(config: &'a Config) -> LoggerGroup {
        let mut loggers = vec![];
        for logger_name in config.logger_list.iter() {
            let logger = match *logger_name {
                "print" => PrintLogger::new(),
                _ => panic!("Invalid logger name input!"),
            };
            loggers.push(Box::new(logger) as Box<dyn Logger>);
        }

        LoggerGroup { loggers }
    }

    pub fn log(&self, result: &PollResult) {
        self.loggers.iter().for_each(|l| l.log(result))
    }
}
