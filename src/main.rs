use std::{fs, io::Write, sync::Mutex};

use clap::Parser;
use cpu::run_cpu;
use gpu::run_opencl;
use regex::Regex;

mod cpu;
mod gpu;
mod handler;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    /// Disable OpenCL
    #[arg(short = 'c', long, default_value_t = false)]
    disable_opencl: bool,

    /// OpenCL thread count
    #[arg(short, long, default_value_t = 1024 * 1024)]
    threads: usize,

    /// OpenCL local work size
    #[arg(short, long)]
    local_work_size: Option<usize>,

    /// OpenCL global work size
    #[arg(short, long)]
    global_work_size: Option<usize>,

    /// OpenCL platform index
    #[arg(short, long, default_value_t = 0)]
    platform_idx: usize,

    /// OpenCL device index
    #[arg(short, long, default_value_t = 0)]
    device_idx: usize,

    /// Regex pattern to search
    #[arg(short, long)]
    regexes: Option<Vec<String>>,

    /// Log hashrate every N seconds
    #[arg(short = 'i', long, default_value_t = 10)]
    log_interval: u64,

    /// CSV file to write results to
    #[arg(short = 'f', long)]
    csv_file: Option<String>,

    /// Report every match, not only new height records
    #[arg(short = 'a', long, default_value_t = false)]
    all_matches: bool,
}

fn main() {
    let child = std::thread::Builder::new()
        .stack_size(16 * 1024 * 1024)
        .spawn(main_)
        .unwrap();

    child.join().unwrap();
}

fn main_() {
    let args = Args::parse();
    handler::ALL_MATCHES.store(args.all_matches, std::sync::atomic::Ordering::Relaxed);

    let regexes = match args.regexes.clone() {
        Some(r) => r,
        None => vec![String::from("")],
    };
    let compiled_regexes: Vec<_> = regexes.iter().map(|r| Regex::new(r).unwrap()).collect();

    let csv_file = args.csv_file.clone().map(|path| {
        let mut f = fs::OpenOptions::new()
            .append(true)
            .create(true)
            .open(path)
            .unwrap();
        if f.metadata().unwrap().len() == 0 {
            f.write_all("timestamp,address,height,regex,privatekey\n".as_bytes())
                .unwrap();
        }
        Mutex::new(f)
    });

    if !args.disable_opencl {
        run_opencl(&args, &regexes, compiled_regexes, &csv_file)
    } else {
        run_cpu(&args, &regexes, compiled_regexes, &csv_file)
    }
}
