use crate::logging::Logger;
use crate::polling::PollResult;

pub struct PrintLogger {}

impl PrintLogger {
    pub fn new() -> PrintLogger {
        PrintLogger{}
    }
}

impl Logger for PrintLogger {
    fn log(&self, poll_result: &PollResult) {
        println!("{:?}", poll_result);
    }
}
