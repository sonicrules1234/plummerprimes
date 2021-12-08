use std::io::Read;
use std::path::PathBuf;
use std::io::Write;
use std::collections::HashMap;
//use std::time::Duration;
use std::io::BufReader;
use std::io::BufRead;
use std::process::{Command, Stdio};
use std::sync::mpsc::{TryRecvError, channel, Receiver};
use serde::{Deserialize, Serialize};
use regex::Regex;
use std::fs;
use structopt::StructOpt;

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

    /// Write save to given file
    #[structopt(short, long="save-file", parse(from_os_str), default_value="save.db")]
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

}

#[derive(Deserialize, Clone, Debug)]
struct RegexOutput {
    label: String,
    passes: usize,
    duration: String,
    threads: usize,
    extra: String,
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

impl From<RegexOutput> for RunOutput {
    fn from(val: RegexOutput) -> RunOutput {
        let mut extra: HashMap<String, String> = HashMap::new();
        for key_val in val.extra.split(",") {
            let (key, value) = key_val.split_once("=").unwrap();
            extra.insert(key.to_string(), value.to_string());
        }
        RunOutput {
            label: val.clone().label,
            passes: val.clone().passes,
            duration: val.clone().duration,
            threads: val.threads,
            pps: (val.passes as f64 / val.duration.parse::<f64>().unwrap()),
            extra: extra,
        }
    }
}
pub fn run_benchmarks(rawregex: &str, target: &str) {
    let opts = PlummerPrimesConfig::from_args();
    //let input_file = opts.clone().save_file;
    //let formatter = opts.clone().formatter;
    //let output_file = opts.clone().report_file;
    if opts.clone().list_formatters {
        println!("List of formatters: table, html, csv");
        return;
    }
    if opts.only_output {
        output_report(opts.clone().formatter, opts.clone().save_file, opts.clone().report_base, opts.clone().report_dir);
        return;
    }
    let arch = target.split_once("-").unwrap().0;
    let arch_file: &str;
    let mut broken_solutions: Vec<String> = Vec::new();
    if arch == "x86_64" {
        arch_file = "arch-amd64";
    } else if arch == "aarch64" {
        arch_file = "arch-arm64";
    } else {
        panic!("Unsupported architecture!");
    }
    let reg = Regex::new(rawregex).unwrap();
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
                let indiv_sols = get_solutions_from(reg.clone(), name.clone(), &rx, opts.clone(), solution_path.clone());
                for sol in indiv_sols.clone() {
                    benchmarks.push(LangOutput {language: lang.clone(), output: sol, solution_name: name.clone()});
                    if opts.debug {
                        println!("{}", benchmarks.clone().len());
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
        println!("Multi: {:?}", save_output_multi);
        println!("Single: {:?}", save_output_single);
        println!("Broken Solutions: {:?}", broken_solutions);
    }
    let save_output = SaveOutput {
        single: save_output_single,
        multi: save_output_multi,
        broken_solutions: broken_solutions,
    };
    let mut f = fs::File::create(opts.clone().save_file).unwrap();
    f.write_all(&rmp_serde::encode::to_vec(&save_output).unwrap()).unwrap();
    println!("Saved data to {}", opts.clone().save_file.to_str().unwrap());
    //output_report(opts.clone().formatter, opts.clone().save_file, opts.clone().report_base, );
    output_report(opts.clone().formatter, opts.clone().save_file, opts.clone().report_base, opts.clone().report_dir);
    //println!("Saved report to {}", opts.clone().report_file.to_str().unwrap());
}
fn get_solutions_from(reg: Regex, name: String, rx: &Receiver<String>, opts: PlummerPrimesConfig, directory: PathBuf) -> Vec<RunOutput> {
    let mut base_run_args = "run --rm ".to_string();
    if opts.unconfined {
        base_run_args.push_str("--security-opt seccomp=unconfined ")
    }
    base_run_args.push_str(name.as_str());
    let mut skip_this = false;
    {
        println!("Building docker container for {}...", name.clone());
        let mut child = Command::new("docker").arg("build").arg("-t").arg(name.as_str()).arg(directory)
        .stdin(Stdio::inherit())
        .stdout(Stdio::null())
        .spawn().unwrap();
        let exit_status = child.wait().unwrap();
        if !exit_status.success() {
            skip_this = true;
        }
    }
    let mut outputs: Vec<RunOutput> = Vec::new();
    if skip_this {
        println!("Error building docker container...\nSkipping run...");
        return outputs;
    }
    println!("Running {}...", name);
    let mut cmd = Command::new("docker");
    let mut c = cmd.args(base_run_args.split(" "));
    if !opts.debug {
        c = c.stderr(Stdio::null());
    }
    let mut child = c//.arg("run").arg("--rm").arg(name.as_str())
        .stdout(Stdio::piped())
        .spawn().unwrap();
    let mut reader = BufReader::new(child.stdout.take().unwrap());
    let mut status = match rx.try_recv() {
        Ok(rx) => rx,
        Err(TryRecvError::Empty) => "keepgoing".to_string(),
        Err(TryRecvError::Disconnected) => "disconnected".to_string(), 
    };
    //let mut buff_status = "keepgoing";
    let mut line = String::new();
    let mut buff_status = match reader.read_line(&mut line) {
        Ok(0) => "done",
        Ok(_) => "keepgoing",
        Err(_) => "error",
    };
    while status.as_str() == "keepgoing" && buff_status == "keepgoing" {
        line = line.trim().to_string();
        if opts.debug {
            println!("'{}'", line);
        }
        if reg.clone().is_match(line.as_str()) {
            let reg_out: RegexOutput = de_regex::from_str_regex(line.as_str(), reg.clone()).unwrap();
            let run_out: RunOutput = reg_out.into();
            outputs.push(run_out);
        } else if opts.debug {
            println!("Not a match");
        }
        status = match rx.try_recv() {
            Ok(rx) => rx,
            Err(TryRecvError::Empty) => "keepgoing".to_string(),
            Err(TryRecvError::Disconnected) => "disconnected".to_string(), 
        };
        if status.as_str() != "keepgoing" {
            break;
        }
        line = String::new();
        buff_status = match reader.read_line(&mut line) {
            Ok(0) => "done",
            Ok(_) => "keepgoing",
            Err(_) => "error",
        };
    }
    if status.as_str() != "keepgoing" {
        nix::sys::signal::kill(
            nix::unistd::Pid::from_raw(child.id() as i32), 
            nix::sys::signal::Signal::SIGINT
        ).expect("cannot kill process");
        child.wait().unwrap();
        std::process::exit(1);
    }
    //assert_eq!(true, outputs.len() > 0 || vec!["primeawk_solution_1", "primecomal_solution_1", "primecsharp_solution_2"].into_iter().map(|a| a.to_string()).collect::<Vec<String>>().contains(&name));
    outputs
}
fn output_report(formatter: String, save_file: PathBuf, base_name: String, report_dir: PathBuf) {
    let mut f = fs::File::open(save_file).unwrap();
    let mut buf: Vec<u8> = Vec::new();
    f.read_to_end(&mut buf).unwrap();
    let save_output: SaveOutput = rmp_serde::decode::from_slice(&buf).unwrap();
    let mut alpha_single_keys = save_output.single[0].keys().map(|b| b.to_string()).collect::<Vec<String>>();
    let mut alpha_multi_keys = save_output.multi[0].keys().map(|b| b.to_string()).collect::<Vec<String>>();
    alpha_single_keys.sort();
    alpha_multi_keys.sort();
    let mut table_single = table_print::Table::new(alpha_single_keys.clone());
    let mut table_multi = table_print::Table::new(alpha_multi_keys.clone());

    //let mut vec_single: Vec<Vec<String>> = Vec::new();
    //let mut vec_multi: Vec<Vec<String>> = Vec::new();
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
        //println!("{}", this_vec.join(" "));
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
        let mut html_data = "<html>\n<head>\n<title>PlummerPrimes Output</title>\n</head>\n<body>\n<p>Single Threaded</p>\n".to_string();
        html_data.push_str(table_single.get_html_element().as_str());
        html_data.push_str("\n<p>Multi Threaded</p>\n");
        html_data.push_str(table_multi.get_html_element().as_str());
        let output_path = report_dir.join(format!("{}.html", base_name).as_str());
        let mut o = fs::File::create(output_path.clone()).unwrap();
        o.write_all(html_data.as_bytes()).unwrap();
        println!("Saved report to {}", output_path.to_str().unwrap());
    } else if formatter.as_str() == "csv" {
        let output_path_single = report_dir.join(format!("{}_single.csv", base_name).as_str());
        let output_path_multi = report_dir.join(format!("{}_multi.csv", base_name).as_str());
        let mut o = fs::File::create(output_path_single.clone()).unwrap();
        o.write_all(table_single.get_csv().as_bytes()).unwrap();
        drop(o);
        println!("Saved report to {}", output_path_single.to_str().unwrap());
        let mut o = fs::File::create(output_path_multi.clone()).unwrap();
        println!("Saved report to {}", output_path_multi.to_str().unwrap());
        o.write_all(table_multi.get_csv().as_bytes()).unwrap();
    } else if formatter.as_str() == "table" {
        let width = termsize::get().unwrap().cols;
        println!("Single-threaded: ");
        match table_single.get_pretty(width.into()) {
            Ok(s) => {
                println!("{}", s);
            },
            Err(_) => {
                println!("Terminal not wide enough to display table.\nTry using --formatter with a different formatter than table.\nYou can use the --list-formatters option to get a list.");
            }
        }
        println!("Multi-threaded: ");
        match table_multi.get_pretty(width.into()) {
            Ok(s) => {
                println!("{}", s);
            },
            Err(_) => {
                println!("Terminal not wide enough to display table.\nTry using --formatter with a different formatter than table.\nYou can use the --list-formatters option to get a list.");
            }
        }
        //println!("{}", table_single)
    }
    /*
    if formatter == "html" {
        let mut html_data = "<html>\n<head><title>plummerprimes output</title></head>\n<body>\n<p>Single threaded</p>\n<table>\n<tr>\n<th>".to_string();
        let mut alpha_single_keys = save_output.single[0].keys().map(|b| b.to_string()).collect::<Vec<String>>();
        let mut alpha_multi_keys = save_output.multi[0].keys().map(|b| b.to_string()).collect::<Vec<String>>();
        alpha_single_keys.sort();
        alpha_multi_keys.sort();
        html_data.push_str(alpha_single_keys.clone().join("</th>\n<th>").as_str());
        html_data.push_str("</th>\n</tr>\n<tr>\n");
        for out in save_output.single {
            for key in alpha_single_keys.clone() {
                let mut value = String::new();
                let val = out.get(&key).unwrap();
                if val.is_some() {
                    value = val.as_ref().unwrap().clone();
                } 
                html_data.push_str(format!("<td>{}</td>", value).as_str());
                html_data.push_str("\n");
            }
            html_data.push_str("</tr>\n");
        }
        html_data.push_str("</table>\n<p>Multi threaded</p>\n<table>\n<tr>\n<th>");
        html_data.push_str(alpha_multi_keys.clone().join("</th>\n<th>").as_str());
        html_data.push_str("</th>\n</tr>\n<tr>\n");
        for out in save_output.multi {
            for key in alpha_multi_keys.clone() {
                let mut value = String::new();
                let val = out.get(&key).unwrap();
                if val.is_some() {
                    value = val.as_ref().unwrap().clone();
                } 
                html_data.push_str(format!("<td>{}</td>", value).as_str());
                html_data.push_str("\n");
            }
            html_data.push_str("</tr>\n");
        }
        html_data.push_str("</table>\n</body>\n</html>");
        let mut o = fs::File::create(output_file).unwrap();
        o.write_all(html_data.as_bytes()).unwrap();
    }
    */

}
