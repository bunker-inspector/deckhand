pub struct Config<'a> {
    pub baseline_duration: u64,
    pub polling_interval: u64,
    pub polling_duration: u64,
    pub log_directory: String,
    pub standard_deviation_threshold: u64,
    pub logger_list: Vec<&'a str>
}
