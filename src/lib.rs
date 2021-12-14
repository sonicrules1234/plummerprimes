//mod dockerreader;
mod systeminformation;
use docker_api::image::BuildOpts;
use std::io::Read;
use std::path::PathBuf;
use std::io::Write;
//use chrono::Utc;
//use serde_json::json;
use std::collections::HashMap;
//use std::time::Duration;
//use std::io::BufReader;
//use std::io::BufRead;
//use std::process::{Command, Stdio};
use std::sync::mpsc::{TryRecvError, channel, Receiver};
use serde::{Deserialize, Serialize};
use regex::Regex;
use std::fs;
use structopt::StructOpt;
//use futures_util::future::ready;
use futures_util::stream::StreamExt;
//use bollard::{Docker, image::BuildImageOptions, models::HostConfig, container::{CreateContainerOptions, Config, LogsOptions, LogOutput}};
use docker_api::{Docker, api::{ContainerCreateOpts, image::models::ImageBuildChunk}};
/// Builds and runs benchmarks from https://youtu.be/D3h62rgewZM
#[derive(StructOpt, Debug, Clone)]
#[structopt(name = "plummerprimes")]
struct PlummerPrimesConfig {
    // /// Implementation directory
    // #[structopt(short, long, parse(from_os_str))]
    // directory: PathBuf,

    /// Output formatter
    #[structopt(short, long, default_value="table")]
    formatter: String,

    /// Read/Write save from/to given file
    #[structopt(long="save-file", parse(from_os_str), default_value="save.db")]
    save_file: PathBuf,

    /// Write report file(s) to given file
    #[structopt(short, long="report-dir", parse(from_os_str), default_value="./")]
    report_dir: PathBuf,

    /// Report file base name
    #[structopt(short="b", long="report-base", default_value="report")]
    report_base: String,

    /// Run with seccomp:unconfined (native performance for interpreted languages)
    #[structopt(short, long)]
    unconfined: bool,

    /// Debug
    #[structopt(long)]
    debug: bool,

    /// Only output report
    #[structopt(long="only-output-report")]
    only_output: bool,

    /// List formats to output to
    #[structopt(short, long="list-formatters")]
    list_formatters: bool,

    /// Build and run specified solution name (Does not save to the save file) [example: primerust_solution_1]
    #[structopt(short, long)]
    solution: Option<String>,

    /// Make json output compatible with the primes project's nodejs output
    #[structopt(long)]
    compat: bool,
}


#[derive(Serialize, Deserialize, Clone, Debug)]
struct RegexOutput {
    label: String,
    passes: usize,
    duration: String,
    threads: usize,
    extra: HashMap<String, String>,
}

impl RegexOutput {
    fn new(reg: Regex, extra_reg: Regex, input: &str) -> RegexOutput {
        let caps = reg.captures(input).unwrap();
        let mut extra: HashMap<String, String> = HashMap::new();
        if let Some(x) = caps.name("extra") {
            if extra_reg.is_match(x.as_str()) {
                for caps_extra in extra_reg.captures_iter(x.as_str()) {
                    extra.insert(caps_extra.name("key").unwrap().as_str().to_string(), caps_extra.name("value").unwrap().as_str().to_string());
                }
            }
        }
        Self {
           label: caps.name("label").unwrap().as_str().to_string(),
           passes: caps.name("passes").unwrap().as_str().parse::<usize>().unwrap(),
           duration: caps.name("duration").unwrap().as_str().to_string(),
           threads: caps.name("threads").unwrap().as_str().parse::<usize>().unwrap(),
           extra: extra,
        }
    }
}

#[derive(Clone, Deserialize, Serialize, Debug)]
struct RunOutput {
    label: String,
    passes: usize,
    duration: String,
    threads: usize,
    pps: f64,
    extra: HashMap<String, String>,
}

#[derive(Clone, Deserialize, Serialize, Debug)]
struct SaveOutput {
    extra_data: crate::systeminformation::ExtraData,
    single: Vec<HashMap<String, Option<String>>>,
    multi: Vec<HashMap<String, Option<String>>>,
    broken_solutions: Vec<String>,
}

#[derive(Clone, Deserialize, Serialize, Debug)]
struct LangOutput {
    language: String,
    output: RunOutput,
    solution_name: String,
}

#[derive(Clone, Deserialize, Serialize, Debug)]
struct CompatOutput {
    implementation: String,
    solution: String,
    label: String,
    passes: usize,
    duration: f64,
    threads: usize,
    tags: HashMap<String, String>,
}

//impl From<SaveOutput> for CompatOutput {
//    fn from(val: SaveOutput) -> CompatOutput {
//        let implementation = 
//    }
//} 

impl From<RegexOutput> for RunOutput {
    fn from(val: RegexOutput) -> RunOutput {
        RunOutput {
            label: val.clone().label,
            passes: val.clone().passes,
            duration: val.clone().duration,
            threads: val.threads,
            pps: (val.passes as f64 / val.duration.parse::<f64>().unwrap()),
            extra: val.extra,
        }
    }
}


#[derive(Clone, Deserialize, Serialize, Debug)]
struct JSONCompatOutput {
    version: String,
    metadata: crate::systeminformation::MetaData,
    machine: crate::systeminformation::Machine,
    results: Vec<CompatOutput>,
}

impl From<SaveOutput> for JSONCompatOutput {
    fn from(val: SaveOutput) -> JSONCompatOutput {
        let extra_data = val.clone().extra_data;
        let mut all_results = val.clone().single;
        all_results.append(&mut val.multi.clone());
        let mut new_results: Vec<CompatOutput> = Vec::new();
        for old_result in all_results {
            let mut tags: HashMap<String, String> = HashMap::new();
            let mut mut_result = old_result.clone();
            let implementation = mut_result["language"].clone().unwrap().to_lowercase();
            mut_result.remove("language");
            let solution_name = mut_result["solution_name"].clone().unwrap();
            let solution = solution_name.split("_").last().unwrap();
            mut_result.remove("solution_name");
            let label = mut_result["label"].clone().unwrap();
            mut_result.remove("label");
            let threads = mut_result["threads"].clone().unwrap().parse::<usize>().unwrap();
            mut_result.remove("threads");
            let duration = mut_result["duration"].clone().unwrap().parse::<f64>().unwrap();
            mut_result.remove("duration");
            let passes = mut_result["passes"].clone().unwrap().parse::<usize>().unwrap();
            mut_result.remove("passes");
            mut_result.remove("passes/second");
            for tag_name in mut_result.clone().keys() {
                let res = mut_result[tag_name].clone();
                if res.is_none() {
                    mut_result.remove(tag_name);
                } else {
                    tags.insert(tag_name.to_string(), res.unwrap().to_string());
                }
            }
            new_results.push(CompatOutput {
                duration: duration,
                implementation: implementation.to_string(),
                label: label.to_string(),
                passes: passes,
                solution: solution.to_string(),
                threads: threads,
                tags: tags,
            });
        }
        JSONCompatOutput {
            version: extra_data.version,
            metadata: extra_data.metadata,
            machine: extra_data.machine,
            results: new_results,
        }
    }
}

pub async fn run_benchmarks(raw_regex: &str, extra_regex: &str, target: &str) {
    //let mut sys_info = smbioslib::table_load_from_device().unwrap();
    //let base_info: Vec<smbioslib::SMBiosProcessorInformation> = sys_info.collect();
    //for info in base_info {
    //    println!("{:?}", info.processor_);
    //}
    //return;
    let arch = target.split_once("-").unwrap().0;
    let arch_file: &str;
    let nodejs_arch: String;
    if arch == "x86_64" {
        arch_file = "arch-amd64";
        nodejs_arch = "x64".to_string();
    } else if arch == "aarch64" {
        arch_file = "arch-arm64";
        nodejs_arch = "arm64".to_string();
    } else {
        panic!("Unsupported architecture!");
    }
    let extra_data = crate::systeminformation::ExtraData::new(nodejs_arch).await;
    //println!("{}", serde_json::to_string_pretty(&extra_data).unwrap());
    //return;
    let opts = PlummerPrimesConfig::from_args();
    //let input_file = opts.clone().save_file;
    //let formatter = opts.clone().formatter;
    //let output_file = opts.clone().report_file;
    if opts.clone().list_formatters {
        println!("List of formatters: table, html, csv, json");
        return;
    }
    if opts.only_output {
        let mut f = fs::File::open(opts.clone().save_file).unwrap();
        let mut buf: Vec<u8> = Vec::new();
        f.read_to_end(&mut buf).unwrap();
        output_report(opts.clone().formatter, buf, opts.clone().report_base, opts.clone().report_dir, opts.clone().compat);
        return;
    }
    //let arch_file: &str;
    let mut broken_solutions: Vec<String> = Vec::new();
    
    let reg = Regex::new(raw_regex).unwrap();
    let extra_reg = Regex::new(extra_regex).unwrap();
    let dir_root = std::env::current_dir().unwrap();
    //let dir_root = cur_dir.parent().unwrap();
    let (tx, rx) = channel::<String>();
    ctrlc::set_handler(move || tx.send("ctrlc".to_string()).expect("Could not send signal on channel."))
        .expect("Error setting Ctrl-C handler");
    //let testregex = Regex::new(rawregex).unwrap();
    let mut benchmarks: Vec<LangOutput> = Vec::new();
    let languages = fs::read_dir(dir_root.clone()).unwrap().filter_map(|x| {
        let dir_entry = x.unwrap();
        if dir_entry.path().is_dir() {
            let name: String = dir_entry.file_name().into_string().unwrap();
            if name.starts_with("Prime") {
                return Some(name.clone().split_once("Prime").unwrap().1.to_string());
            }
        }
        None
    });
    let mut status = match rx.try_recv() {
        Ok(rx) => rx,
        Err(TryRecvError::Empty) => "keepgoing".to_string(),
        Err(TryRecvError::Disconnected) => "disconnected".to_string(), 
    };
    for lang in languages {
        if status == "keepgoing" {
            let local_dir = dir_root.clone().join(format!("Prime{}", lang));
            let num_solutions = fs::read_dir(local_dir.clone()).unwrap().filter(|x| x.as_ref().unwrap().file_name().into_string().unwrap().starts_with("solution_")).count();
            for solution_num in 1..(num_solutions + 1) {
                let name = format!("prime{}_solution_{}", lang.to_lowercase(), solution_num);
                if let Some(solution_name) = opts.clone().solution {
                    if name.clone() != solution_name {
                        continue;
                    }
                }
                let solution_path = local_dir.clone().join(format!("solution_{}", solution_num).as_str());
                if fs::read_dir(solution_path.clone()).unwrap().filter(|x| x.as_ref().unwrap().file_name().into_string().unwrap().starts_with("arch-")).count() > 0 {
                    if !solution_path.clone().join(arch_file).exists() {
                        eprintln!("Skipping {} because of unsupported architecture", name.clone());
                        continue;
                    }
                }
                if solution_path.join("build-no").exists() {
                    eprintln!("Skipping {} because build is disabled", name.clone());
                    continue;
                }
                let indiv_sols = get_solutions_from(reg.clone(), extra_reg.clone(), name.clone(), &rx, opts.clone(), solution_path.clone()).await;
                for sol in indiv_sols.clone() {
                    benchmarks.push(LangOutput {language: lang.clone(), output: sol, solution_name: name.clone()});
                    if opts.debug {
                        eprintln!("{}", benchmarks.clone().len());
                    }
                }
                if indiv_sols.clone().len() == 0 {
                    broken_solutions.push(name);
                }
            }
        } else {
            break;
        }
        status = match rx.try_recv() {
            Ok(rx) => rx,
            Err(TryRecvError::Empty) => "keepgoing".to_string(),
            Err(TryRecvError::Disconnected) => "disconnected".to_string(), 
        };
    }
    benchmarks.sort_by_key(|x| x.output.passes);
    benchmarks.reverse();
    let mut save_output_multi: Vec<HashMap<String, Option<String>>> = Vec::new();
    let mut save_output_single: Vec<HashMap<String, Option<String>>> = Vec::new();
    let mut keys_single: Vec<String> = Vec::new();//vec!["Language", "Label", "Duration", "Threads"].into_iter().map(|x| x.to_string()).collect();
    let mut keys_multi = keys_single.clone();
    for benchmark in benchmarks.clone() {
        if benchmark.output.threads > 1 {
            for extra_key in benchmark.output.extra.keys() {
                if !keys_multi.contains(extra_key) {
                    keys_multi.push(extra_key.clone());
                }
            }
        } else {
            for extra_key in benchmark.output.extra.keys() {
                if !keys_single.contains(extra_key) {
                    keys_single.push(extra_key.clone());
                }
            }
        }
    }
    for benchmark in benchmarks.clone() {
        let mut output: HashMap<String, Option<String>> = HashMap::new();
        output.insert("language".into(), Some(benchmark.language));
        output.insert("duration".into(), Some(benchmark.output.duration));
        output.insert("label".into(), Some(benchmark.output.label));
        output.insert("threads".into(), Some(benchmark.output.threads.to_string()));
        output.insert("passes".into(), Some(benchmark.output.passes.to_string()));
        output.insert("solution_name".into(), Some(benchmark.solution_name.to_string()));
        output.insert("passes/second".into(), Some(benchmark.output.pps.to_string()));
        if benchmark.output.threads > 1 {
            for key in keys_multi.clone() {
                let val: Option<String>;
                if benchmark.output.extra.contains_key(&key) {
                    val = Some(benchmark.output.extra.get(&key).unwrap().to_string());
                } else {
                    val = None;
                }
                output.insert(key, val);
            }
            save_output_multi.push(output);
        } else {
            for key in keys_single.clone() {
                let val: Option<String>;
                if benchmark.output.extra.contains_key(&key) {
                    val = Some(benchmark.output.extra.get(&key).unwrap().to_string());
                } else {
                    val = None;
                }
                output.insert(key, val);
            }
            save_output_single.push(output);
        }
    }
    if opts.debug {
        eprintln!("Multi: {:?}", save_output_multi);
        eprintln!("Single: {:?}", save_output_single);
        eprintln!("Broken Solutions: {:?}", broken_solutions);
    }
    let save_output = SaveOutput {
        extra_data: extra_data,
        single: save_output_single,
        multi: save_output_multi,
        broken_solutions: broken_solutions,
    };
    if opts.clone().solution.is_none() {
        let mut f = fs::File::create(opts.clone().save_file).unwrap();
        f.write_all(&rmp_serde::encode::to_vec(&save_output).unwrap()).unwrap();
        println!("Saved data to {}", opts.clone().save_file.to_str().unwrap());
    }
    output_report(opts.clone().formatter, rmp_serde::encode::to_vec(&save_output).unwrap(), opts.clone().report_base, opts.clone().report_dir, opts.clone().compat);
}
async fn process_docker_output(message: &[u8], buffer: String, outs: Vec<RunOutput>, rx: &Receiver<String>, reg: Regex, extra_reg: Regex,  opts: PlummerPrimesConfig) -> (String, Vec<RunOutput>, String) {
    let mut buf = buffer.clone();
    let mut outputs = outs.clone();
    let mut status = String::new();
    let mut last = String::new();  
    buf.push_str(String::from_utf8_lossy(message).into_owned().as_str());
    let split: Vec<&str> = buf.split("\n").collect();
    let num_items = split.len();
    for (num, b) in split.clone().into_iter().enumerate() {
        last = b.to_string();
        status = match rx.try_recv() {
            Ok(rx) => rx,
            Err(TryRecvError::Empty) => "keepgoing".to_string(),
            Err(TryRecvError::Disconnected) => "disconnected".to_string(), 
        };
        if status.as_str() != "keepgoing" {
            std::process::exit(1);
        }
        if num == num_items - 1 && last == String::new() {
            break;
        }
        if opts.debug {
            eprintln!("{}", last);
        }
        if reg.clone().is_match(last.as_str()) {
            if opts.debug {
                eprintln!("Matched");
            }
            let reg_out = RegexOutput::new(reg.clone(), extra_reg.clone(), last.as_str());
            let run_out: RunOutput = reg_out.into();
            outputs.push(run_out);
        } else if opts.debug  {
            eprintln!("Not a match");
        }
    }

    (last, outputs, status)
}
async fn get_solutions_from(reg: Regex, extra_reg: Regex, name: String, rx: &Receiver<String>, opts: PlummerPrimesConfig, directory: PathBuf) -> Vec<RunOutput> {
    let mut status = match rx.try_recv() {
        Ok(rx) => rx,
        Err(TryRecvError::Empty) => "keepgoing".to_string(),
        Err(TryRecvError::Disconnected) => "disconnected".to_string(), 
    };
    let mut skip_this = false;
    println!("Building docker container for {}...", name.clone());
    let docker = Docker::new("unix:///var/run/docker.sock").unwrap();
    let options = BuildOpts::builder(directory).tag(name.clone()).build();
    let mut stream = docker.images().build(&options);
    while let Some(build_result) = stream.next().await {
        status = match rx.try_recv() {
            Ok(rx) => rx,
            Err(TryRecvError::Empty) => "keepgoing".to_string(),
            Err(TryRecvError::Disconnected) => "disconnected".to_string(), 
        };
        if status.as_str() != "keepgoing" {
            std::process::exit(1);
        }
        match build_result {
            Ok(output) => {
                if opts.debug {
                    eprintln!("[BUILD] {:?}", output);
                }
                if let ImageBuildChunk::Update { stream } = output.clone() {
                    if stream.starts_with(format!("Successfully tagged {}", name.clone()).as_str()) {
                        break;
                    }
                }
                if let ImageBuildChunk::Error { error, error_detail } = output.clone() {
                    eprintln!("Error: {:?} {:?}", error, error_detail);
                }
            },
            Err(e) => {
                eprintln!("Error: {}", e);
                skip_this = true;
                break;
            },
        }
    }
    let mut outputs: Vec<RunOutput> = Vec::new();
    if skip_this {
        eprintln!("Error building docker container...\nSkipping run...");
        return outputs;
    }
    println!("Running {}...", name);

    let mut container_opts = ContainerCreateOpts::builder(name.as_str()).name(format!("{}_container", name).as_str()).auto_remove(true);
    if opts.clone().unconfined {
        container_opts = container_opts.security_options(vec!["seccomp=unconfined".to_string()])
    }
    let container = docker.containers().create(&container_opts.build()).await.unwrap();
    let start_result = container.start().await;
    if let Err(e) = start_result {
        if let docker_api::errors::Error::Fault { code: _code, message } = e {
            eprintln!("Error: {}", message);
        } else {
            eprintln!("Error: {:?}", e);
        }
        return outputs;
    }
    let tty = container.attach().await.unwrap();
    let mut stream = tty.split().0;
    let mut buf = String::new();
    while let Some(msg) = stream.next().await {
        if status.as_str() != "keepgoing" {
            break
        }
        let message = msg.unwrap();
        let x = process_docker_output(message.as_ref(), buf, outputs, rx, reg.clone(), extra_reg.clone(), opts.clone()).await;
        buf = x.0.clone();
        outputs = x.1.clone();
        status = x.2.clone();
    }
    //assert_eq!(true, outputs.len() > 0 || vec!["primeawk_solution_1", "primecomal_solution_1", "primecsharp_solution_2"].into_iter().map(|a| a.to_string()).collect::<Vec<String>>().contains(&name));
    outputs
}
fn output_report(formatter: String, buf: Vec<u8>, base_name: String, report_dir: PathBuf, compat: bool) {
    //let mut f = fs::File::open(save_file).unwrap();
    //let mut buf: Vec<u8> = Vec::new();
    //f.read_to_end(&mut buf).unwrap();
    let save_output: SaveOutput = rmp_serde::decode::from_slice(&buf).unwrap();
    let save_output_clone = save_output.clone();
    let mut alpha_single_keys: Vec<String> = Vec::new();
    let mut alpha_multi_keys: Vec<String> = Vec::new();
    let mut output_multi = false;
    let mut output_single = false;
    if save_output.single.len() > 0 {
        alpha_single_keys = save_output.single[0].keys().map(|b| b.to_string()).collect::<Vec<String>>();
        output_single = true;
    }
    if save_output.multi.len() > 0 {
        alpha_multi_keys = save_output.multi[0].keys().map(|b| b.to_string()).collect::<Vec<String>>();
        output_multi = true;
    }
    alpha_single_keys.sort();
    alpha_multi_keys.sort();
    let mut table_single = table_print::Table::new(alpha_single_keys.clone());
    let mut table_multi = table_print::Table::new(alpha_multi_keys.clone());
    for out in save_output.single {
        let mut this_vec: Vec<String> = Vec::new();
        for key in alpha_single_keys.clone() {
            let mut value = String::new();
            let val = out.get(&key).unwrap();
            if val.is_some() {
                value = val.as_ref().unwrap().clone();
            }
            this_vec.push(value);
        }
        table_single.insert_row(this_vec);
    }
    for out in save_output.multi {
        let mut this_vec: Vec<String> = Vec::new();
        for key in alpha_multi_keys.clone() {
            let mut value = String::new();
            let val = out.get(&key).unwrap();
            if val.is_some() {
                value = val.as_ref().unwrap().clone();
            }
            this_vec.push(value);
        }
        table_multi.insert_row(this_vec);
    }
    if formatter.as_str() == "html" {
        let mut html_data = "<html>\n<head>\n<title>PlummerPrimes Output</title>\n</head>\n<body>\n".to_string();
        if output_single {
            html_data.push_str("<p>Single Threaded</p>\n");
            html_data.push_str(table_single.get_html_element().as_str());
        }
        if output_multi {
            html_data.push_str("\n<p>Multi Threaded</p>\n");
            html_data.push_str(table_multi.get_html_element().as_str());
        }
        html_data.push_str("\n</body>\n</html>");
        let output_path = report_dir.join(format!("{}.html", base_name).as_str());
        let mut o = fs::File::create(output_path.clone()).unwrap();
        o.write_all(html_data.as_bytes()).unwrap();
        println!("Saved report to {}", output_path.to_str().unwrap());
    } else if formatter.as_str() == "csv" {
        let output_path_single = report_dir.join(format!("{}_single.csv", base_name).as_str());
        let output_path_multi = report_dir.join(format!("{}_multi.csv", base_name).as_str());
        if output_single {
            let mut o = fs::File::create(output_path_single.clone()).unwrap();
            o.write_all(table_single.get_csv().as_bytes()).unwrap();
            println!("Saved report to {}", output_path_single.to_str().unwrap());
        }
        if output_multi {
            let mut o = fs::File::create(output_path_multi.clone()).unwrap();
            println!("Saved report to {}", output_path_multi.to_str().unwrap());
            o.write_all(table_multi.get_csv().as_bytes()).unwrap();
        }
    } else if formatter.as_str() == "table" {
        let width = termsize::get().unwrap().cols;
        
        if output_single {
            println!("Single-threaded: ");
            match table_single.get_pretty(width.into()) {
                Ok(s) => {
                    println!("{}", s);
                },
                Err(_) => {
                    println!("Terminal not wide enough to display table.\nTry using --formatter with a different formatter than table.\nYou can use the --list-formatters option to get a list.");
                }
            }
        }
        if output_multi {
            println!("Multi-threaded: ");
            match table_multi.get_pretty(width.into()) {
                Ok(s) => {
                    println!("{}", s);
                },
                Err(_) => {
                    println!("Terminal not wide enough to display table.\nTry using --formatter with a different formatter than table.\nYou can use the --list-formatters option to get a list.");
                }
            }
        }
    } else if formatter.as_str() == "json" {
        let json_data: String;
        let mut output_path = report_dir.clone();
        if compat {
            output_path = output_path.join(format!("{}_compat.json", base_name).as_str());
            let compat_json: JSONCompatOutput = save_output_clone.into();
            json_data = serde_json::to_string_pretty(&compat_json).unwrap();
        } else {
            output_path = output_path.join(format!("{}.json", base_name).as_str());
            json_data = serde_json::to_string_pretty(&save_output_clone).unwrap();
        }
        let mut o = fs::File::create(output_path.clone()).unwrap();
        o.write_all(json_data.as_bytes()).unwrap();
        println!("Saved report to {}", output_path.to_str().unwrap());
    }
}

