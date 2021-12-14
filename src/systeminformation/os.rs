use std::collections::HashMap;
use std::fs;
use serde::{Deserialize, Serialize};
use regex::Regex;
#[derive(Clone, Deserialize, Serialize, Debug)]
pub struct OSInfo {
    platform: String,
    distro: String,
    release: String,
    codename: String,
    kernel: String,
    arch: String,
    codepage: String,
    logofile: String,
    build: String,
    servicepack: String,
    uefi: bool,
}

impl OSInfo {
    #[cfg(target_os = "linux")]
    pub fn new(arch: String) -> Self {
        let files_regex = Regex::new(r"^(.+)\-release$").unwrap();
        let files = fs::read_dir("/etc").unwrap().filter(|x| {
            //println!("{:?}", x.as_ref().unwrap().file_name().into_string().unwrap().as_str());
            files_regex.is_match(x.as_ref().unwrap().file_name().into_string().unwrap().as_str())
        });
        let mut output = String::new();
        for file_path_result in files {
            let file_path = file_path_result.unwrap().path();
            output.push_str(fs::read_to_string(file_path).unwrap_or(String::new()).as_str());
        }
        output.push_str(fs::read_to_string("/usr/lib/os-release").unwrap_or(String::new()).as_str());
        output = output.replace("\r", "").trim().to_string();
        let mut out_hashmap: HashMap<String, String> = HashMap::new();
        for line in output.split("\n") {
            if !line.contains("=") {
                continue;
            }
            let parts = line.split_once("=").unwrap();
            out_hashmap.insert(parts.0.trim().to_uppercase().to_string(), parts.1.trim().to_string());
        }
        let mut code_name = out_hashmap.get("DISTRIB_CODENAME").unwrap_or(&"".to_string()).replace('"', "").to_string();
        if code_name == String::new() {
            code_name = out_hashmap.get("VERSION_CODENAME").unwrap_or(&"".to_string()).replace('"', "").to_string()
        }
        let mut distro = out_hashmap.get("DISTRIB_ID").unwrap_or(&"".to_string()).replace('"', "").to_string();
        if distro == String::new() {
            distro = out_hashmap.get("NAME").unwrap_or(&"unknown".to_string()).replace('"', "").to_string()
        }
        let mut release = out_hashmap.get("VERSION").unwrap_or(&"".to_string()).replace('"', "").to_string();
        if release == String::new() {
            release = out_hashmap.get("DISTRIB_RELEASE").unwrap_or(&"".to_string()).replace('"', "").to_string()
        }
        if release == String::new() {
            release = out_hashmap.get("VERSION_ID").unwrap_or(&"unknown".to_string()).replace('"', "").to_string()
        }
        let version = fs::read_to_string("/proc/version").unwrap().split_whitespace().collect::<Vec<&str>>()[2].to_string();
        let code_page = "UTF-8".to_string();
        
        Self {
            platform: "linux".to_string(),
            distro: distro,
            release: release,
            codename: code_name,
            kernel: version,
            arch: arch,
            codepage: code_page,
            logofile: out_hashmap.get("ID").unwrap_or(out_hashmap.get("LOGO").unwrap_or(&"".to_string())).replace('"', "").to_string(),
            build: out_hashmap.get("BUILD_ID").unwrap_or(&"".to_string()).replace('"', "").to_string(),
            servicepack: String::new(),
            uefi: std::path::Path::new("/sys/firmware/efi").exists(),
        }
    }
}
