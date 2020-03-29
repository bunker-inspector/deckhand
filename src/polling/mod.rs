extern crate sysinfo;

use crate::config::Config;
use std::collections::{HashMap, LinkedList};
use std::{thread, time};
use sysinfo::{ComponentExt, ProcessExt, System, SystemExt};

#[derive(Debug)]
pub struct ResourceUsage {
    mem: f32,
    cpu: f32,
    swap: f32,
    temps: HashMap<String, f32>,
}

#[derive(Debug)]
pub struct Baseline {
    mem_avg: f32,
    cpu_avg: f32,
    swap_avg: f32,
    mem_std_dev: f32,
    cpu_std_dev: f32,
    swap_std_dev: f32,
    temp_avgs: HashMap<String, f32>,
    temp_std_devs: HashMap<String, f32>,
}

#[derive(Debug)]
pub enum Resource {
    CPU,
    Memory,
    Swap,
    Temp(String),
}

type Anomolies = Vec<Resource>;

#[derive(Debug)]
pub enum PollResult<'a> {
    Normal(&'a ResourceUsage),
    Exceptional(&'a ResourceUsage, Anomolies),
}

pub fn get_baseline(sys: &mut System, poll_secs: u64) -> Baseline {
    let mut resource_pts = LinkedList::new();
    let one_sec = time::Duration::new(1, 0);

    for i in 0..poll_secs {
        eprint!(
            "\rPolling for baseline, {}s of {}s remaining...",
            poll_secs - i,
            poll_secs
        );
        resource_pts.push_back(read(sys));

        thread::sleep(one_sec);
    }
    eprint!("\r");

    let mut cpu_tot = 0.0;
    let mut mem_tot = 0.0;
    let mut swap_tot = 0.0;
    let mut temp_tots = HashMap::new();
    for pt in &resource_pts {
        cpu_tot += pt.cpu;
        mem_tot += pt.mem as f64;
        swap_tot += pt.swap;

        for (label, temp) in pt.temps.iter() {
            let tot = match temp_tots.get(label) {
                Some((temp_tot, ct)) => (temp_tot + temp, ct + 1),
                None => (*temp, 1),
            };
            temp_tots.insert(label, tot);
        }
    }

    let cpu_avg = cpu_tot / (resource_pts.len() as f32);
    let mem_avg = mem_tot / (resource_pts.len() as f64);
    let swap_avg = if swap_tot > 0.0 {
        swap_tot / (resource_pts.len() as f32)
    } else {
        0.0
    };

    let mut temp_avgs = HashMap::new();
    for (label, (temp_tot, ct)) in temp_tots.iter() {
        temp_avgs.insert(label.to_string(), *temp_tot / *ct as f32);
    }

    let mut cpu_dev = 0.0;
    let mut mem_dev = 0.0;
    let mut swap_dev = 0.0;
    let mut cpu_dev_pos = 0;
    let mut mem_dev_pos = 0;
    let mut swap_dev_pos = 0;
    let mut temp_devs = HashMap::new();
    for pt in &resource_pts {
        if pt.cpu > cpu_avg {
            cpu_dev += pt.cpu - cpu_avg;
            cpu_dev_pos += 1;
        }
        if pt.mem as f64 > mem_avg {
            mem_dev += pt.mem as f64 - mem_avg;
            mem_dev_pos += 1;
        }
        if pt.swap > swap_avg {
            swap_dev += pt.swap as f32 - swap_avg;
            swap_dev_pos += 1;
        }
        for (label, temp) in pt.temps.iter() {
            let avg = temp_avgs.get(label).unwrap();
            if temp > avg {
                let curr_dev = temp - avg;
                let temp_dev = match temp_devs.get(label) {
                    Some((dev, ct)) => (dev + curr_dev, ct + 1),
                    None => (curr_dev, 1),
                };
                temp_devs.insert(label, temp_dev);
            }
        }
    }

    let mut temp_std_devs = HashMap::new();
    for (label, (dev, ct)) in temp_devs {
        temp_std_devs.insert(label.to_string(), dev / ct as f32);
    }

    Baseline {
        mem_avg: mem_avg as f32,
        cpu_avg,
        mem_std_dev: mem_dev as f32 / (mem_dev_pos as f32),
        cpu_std_dev: cpu_dev / (cpu_dev_pos as f32),
        swap_avg: swap_avg,
        temp_avgs,
        swap_std_dev: if swap_dev_pos > 0 {
            swap_dev / (swap_dev_pos as f32)
        } else {
            0.0
        },
        temp_std_devs,
    }
}

pub fn read(sys: &mut System) -> ResourceUsage {
    sys.refresh_all();
    sys.refresh_components_list();
    sys.refresh_components();

    let mut temps = HashMap::new();
    for component in sys.get_components() {
        temps.insert(
            component.get_label().to_string(),
            component.get_temperature(),
        );
    }

    let p = sys.get_processes();
    let cpu = p
        .iter()
        .fold(0.0, |acc, (_, process)| acc + process.cpu_usage())
        / p.len() as f32;

    ResourceUsage {
        mem: sys.get_used_memory() as f32 / sys.get_total_memory() as f32,
        cpu,
        swap: sys.get_used_swap() as f32 / sys.get_total_swap() as f32,
        temps,
    }
}

pub fn compare<'a>(
    usage: &'a ResourceUsage,
    baseline: &Baseline,
    config: &Config,
) -> PollResult<'a> {
    let mut anoms = Vec::new();

    if usage.mem - baseline.mem_avg
        > baseline.mem_std_dev * config.standard_deviation_threshold as f32
    {
        anoms.push(Resource::Memory);
    }
    if usage.cpu - baseline.cpu_avg
        > baseline.cpu_std_dev * config.standard_deviation_threshold as f32
    {
        anoms.push(Resource::CPU);
    }
    if usage.swap - baseline.swap_avg
        > baseline.swap_std_dev * config.standard_deviation_threshold as f32
    {
        anoms.push(Resource::Swap);
    }
    for (label, temp) in usage.temps.iter() {
        let avg = baseline.temp_avgs.get(label).unwrap();
        if temp - avg
            > baseline.temp_std_devs.get(label).unwrap()
                * config.standard_deviation_threshold as f32
        {
            anoms.push(Resource::Temp(label.to_string()));
        }
    }

    if anoms.is_empty() {
        PollResult::Normal(usage)
    } else {
        PollResult::Exceptional(usage, anoms)
    }
}
