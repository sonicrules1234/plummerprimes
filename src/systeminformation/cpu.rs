use std::collections::HashMap;
use std::fs;
use regex::Regex;
use serde::{Deserialize, Serialize};
const AMD_SPEEDS: &str = r#"
{
    '8346': '1.8',
    '8347': '1.9',
    '8350': '2.0',
    '8354': '2.2',
    '8356|SE': '2.4',
    '8356': '2.3',
    '8360': '2.5',
    '2372': '2.1',
    '2373': '2.1',
    '2374': '2.2',
    '2376': '2.3',
    '2377': '2.3',
    '2378': '2.4',
    '2379': '2.4',
    '2380': '2.5',
    '2381': '2.5',
    '2382': '2.6',
    '2384': '2.7',
    '2386': '2.8',
    '2387': '2.8',
    '2389': '2.9',
    '2393': '3.1',
    '8374': '2.2',
    '8376': '2.3',
    '8378': '2.4',
    '8379': '2.4',
    '8380': '2.5',
    '8381': '2.5',
    '8382': '2.6',
    '8384': '2.7',
    '8386': '2.8',
    '8387': '2.8',
    '8389': '2.9',
    '8393': '3.1',
    '2419EE': '1.8',
    '2423HE': '2.0',
    '2425HE': '2.1',
    '2427': '2.2',
    '2431': '2.4',
    '2435': '2.6',
    '2439SE': '2.8',
    '8425HE': '2.1',
    '8431': '2.4',
    '8435': '2.6',
    '8439SE': '2.8',
    '4122': '2.2',
    '4130': '2.6',
    '4162EE': '1.7',
    '4164EE': '1.8',
    '4170HE': '2.1',
    '4174HE': '2.3',
    '4176HE': '2.4',
    '4180': '2.6',
    '4184': '2.8',
    '6124HE': '1.8',
    '6128HE': '2.0',
    '6132HE': '2.2',
    '6128': '2.0',
    '6134': '2.3',
    '6136': '2.4',
    '6140': '2.6',
    '6164HE': '1.7',
    '6166HE': '1.8',
    '6168': '1.9',
    '6172': '2.1',
    '6174': '2.2',
    '6176': '2.3',
    '6176SE': '2.3',
    '6180SE': '2.5',
    '3250': '2.5',
    '3260': '2.7',
    '3280': '2.4',
    '4226': '2.7',
    '4228': '2.8',
    '4230': '2.9',
    '4234': '3.1',
    '4238': '3.3',
    '4240': '3.4',
    '4256': '1.6',
    '4274': '2.5',
    '4276': '2.6',
    '4280': '2.8',
    '4284': '3.0',
    '6204': '3.3',
    '6212': '2.6',
    '6220': '3.0',
    '6234': '2.4',
    '6238': '2.6',
    '6262HE': '1.6',
    '6272': '2.1',
    '6274': '2.2',
    '6276': '2.3',
    '6278': '2.4',
    '6282SE': '2.6',
    '6284SE': '2.7',
    '6308': '3.5',
    '6320': '2.8',
    '6328': '3.2',
    '6338P': '2.3',
    '6344': '2.6',
    '6348': '2.8',
    '6366': '1.8',
    '6370P': '2.0',
    '6376': '2.3',
    '6378': '2.4',
    '6380': '2.5',
    '6386': '2.8',
    'FX|4100': '3.6',
    'FX|4120': '3.9',
    'FX|4130': '3.8',
    'FX|4150': '3.8',
    'FX|4170': '4.2',
    'FX|6100': '3.3',
    'FX|6120': '3.6',
    'FX|6130': '3.6',
    'FX|6200': '3.8',
    'FX|8100': '2.8',
    'FX|8120': '3.1',
    'FX|8140': '3.2',
    'FX|8150': '3.6',
    'FX|8170': '3.9',
    'FX|4300': '3.8',
    'FX|4320': '4.0',
    'FX|4350': '4.2',
    'FX|6300': '3.5',
    'FX|6350': '3.9',
    'FX|8300': '3.3',
    'FX|8310': '3.4',
    'FX|8320': '3.5',
    'FX|8350': '4.0',
    'FX|8370': '4.0',
    'FX|9370': '4.4',
    'FX|9590': '4.7',
    'FX|8320E': '3.2',
    'FX|8370E': '3.3',
    '1200': '3.1',
    'Pro 1200': '3.1',
    '1300X': '3.5',
    'Pro 1300': '3.5',
    '1400': '3.2',
    '1500X': '3.5',
    'Pro 1500': '3.5',
    '1600': '3.2',
    '1600X': '3.6',
    'Pro 1600': '3.2',
    '1700': '3.0',
    'Pro 1700': '3.0',
    '1700X': '3.4',
    'Pro 1700X': '3.4',
    '1800X': '3.6',
    '1900X': '3.8',
    '1920': '3.2',
    '1920X': '3.5',
    '1950X': '3.4',
    '200GE': '3.2',
    'Pro 200GE': '3.2',
    '220GE': '3.4',
    '240GE': '3.5',
    '3000G': '3.5',
    '300GE': '3.4',
    '3050GE': '3.4',
    '2200G': '3.5',
    'Pro 2200G': '3.5',
    '2200GE': '3.2',
    'Pro 2200GE': '3.2',
    '2400G': '3.6',
    'Pro 2400G': '3.6',
    '2400GE': '3.2',
    'Pro 2400GE': '3.2',
    'Pro 200U': '2.3',
    '300U': '2.4',
    '2200U': '2.5',
    '3200U': '2.6',
    '2300U': '2.0',
    'Pro 2300U': '2.0',
    '2500U': '2.0',
    'Pro 2500U': '2.2',
    '2600H': '3.2',
    '2700U': '2.0',
    'Pro 2700U': '2.2',
    '2800H': '3.3',
    '7351': '2.4',
    '7351P': '2.4',
    '7401': '2.0',
    '7401P': '2.0',
    '7551P': '2.0',
    '7551': '2.0',
    '7251': '2.1',
    '7261': '2.5',
    '7281': '2.1',
    '7301': '2.2',
    '7371': '3.1',
    '7451': '2.3',
    '7501': '2.0',
    '7571': '2.2',
    '7601': '2.2',
    'V1500B': '2.2',
    'V1780B': '3.35',
    'V1202B': '2.3',
    'V1404I': '2.0',
    'V1605B': '2.0',
    'V1756B': '3.25',
    'V1807B': '3.35',
    '3101': '2.1',
    '3151': '2.7',
    '3201': '1.5',
    '3251': '2.5',
    '3255': '2.5',
    '3301': '2.0',
    '3351': '1.9',
    '3401': '1.85',
    '3451': '2.15',
    '1200|AF': '3.1',
    '2300X': '3.5',
    '2500X': '3.6',
    '2600': '3.4',
    '2600E': '3.1',
    '1600|AF': '3.2',
    '2600X': '3.6',
    '2700': '3.2',
    '2700E': '2.8',
    'Pro 2700': '3.2',
    '2700X': '3.7',
    'Pro 2700X': '3.6',
    '2920X': '3.5',
    '2950X': '3.5',
    '2970WX': '3.0',
    '2990WX': '3.0',
    'Pro 300GE': '3.4',
    'Pro 3125GE': '3.4',
    '3150G': '3.5',
    'Pro 3150G': '3.5',
    '3150GE': '3.3',
    'Pro 3150GE': '3.3',
    '3200G': '3.6',
    'Pro 3200G': '3.6',
    '3200GE': '3.3',
    'Pro 3200GE': '3.3',
    '3350G': '3.6',
    'Pro 3350G': '3.6',
    '3350GE': '3.3',
    'Pro 3350GE': '3.3',
    '3400G': '3.7',
    'Pro 3400G': '3.7',
    '3400GE': '3.3',
    'Pro 3400GE': '3.3',
    '3300U': '2.1',
    'PRO 3300U': '2.1',
    '3450U': '2.1',
    '3500U': '2.1',
    'PRO 3500U': '2.1',
    '3500C': '2.1',
    '3550H': '2.1',
    '3580U': '2.1',
    '3700U': '2.3',
    'PRO 3700U': '2.3',
    '3700C': '2.3',
    '3750H': '2.3',
    '3780U': '2.3',
    '3100': '3.6',
    '3300X': '3.8',
    '3500': '3.6',
    '3500X': '3.6',
    '3600': '3.6',
    'Pro 3600': '3.6',
    '3600X': '3.8',
    '3600XT': '3.8',
    'Pro 3700': '3.6',
    '3700X': '3.6',
    '3800X': '3.9',
    '3800XT': '3.9',
    '3900': '3.1',
    'Pro 3900': '3.1',
    '3900X': '3.8',
    '3900XT': '3.8',
    '3950X': '3.5',
    '3960X': '3.8',
    '3970X': '3.7',
    '3990X': '2.9',
    '3945WX': '4.0',
    '3955WX': '3.9',
    '3975WX': '3.5',
    '3995WX': '2.7',
    '4300GE': '3.5',
    'Pro 4300GE': '3.5',
    '4300G': '3.8',
    'Pro 4300G': '3.8',
    '4600GE': '3.3',
    'Pro 4650GE': '3.3',
    '4600G': '3.7',
    'Pro 4650G': '3.7',
    '4700GE': '3.1',
    'Pro 4750GE': '3.1',
    '4700G': '3.6',
    'Pro 4750G': '3.6',
    '4300U': '2.7',
    '4450U': '2.5',
    'Pro 4450U': '2.5',
    '4500U': '2.3',
    '4600U': '2.1',
    'PRO 4650U': '2.1',
    '4680U': '2.1',
    '4600HS': '3.0',
    '4600H': '3.0',
    '4700U': '2.0',
    'PRO 4750U': '1.7',
    '4800U': '1.8',
    '4800HS': '2.9',
    '4800H': '2.9',
    '4900HS': '3.0',
    '4900H': '3.3',
    '5300U': '2.6',
    '5500U': '2.1',
    '5700U': '1.8',
    '7232P': '3.1',
    '7302P': '3.0',
    '7402P': '2.8',
    '7502P': '2.5',
    '7702P': '2.0',
    '7252': '3.1',
    '7262': '3.2',
    '7272': '2.9',
    '7282': '2.8',
    '7302': '3.0',
    '7352': '2.3',
    '7402': '2.8',
    '7452': '2.35',
    '7502': '2.5',
    '7532': '2.4',
    '7542': '2.9',
    '7552': '2.2',
    '7642': '2.3',
    '7662': '2.0',
    '7702': '2.0',
    '7742': '2.25',
    '7H12': '2.6',
    '7F32': '3.7',
    '7F52': '3.5',
    '7F72': '3.2',
    '7763': '2.45',
    '7713': '2.0',
    '7713P': '2.0',
    '7663': '2.0',
    '7643': '2.3',
    '75F3': '2.95',
    '7543': '2.8',
    '7543P': '2.8',
    '7513': '2.6',
    '7453': '2.75',
    '74F3': '3.2',
    '7443': '2.85',
    '7443P': '2.85',
    '7413': '2.65',
    '73F3': '3.5',
    '7343': '3.2',
    '7313': '3.0',
    '7313P': '3.0',
    '72F3': '3.7',
    '5600X': '3.7',
    '5800X': '3.8',
    '5900X': '3.7',
    '5950X': '3.4'
  };"#;
#[derive(Clone, Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct CPUInfo {
    manufacturer: String,
    brand: String,
    vendor: String,
    family: String,
    model: String,
    stepping: String,
    revision: String,
    voltage: String,
    speed: f64,
    speed_min: f64,
    speed_max: f64,
    governor: String,
    cores: usize,
    physical_cores: usize,
    processors: usize,
    socket: String,
    flags: String,
    virtualization: bool,
    cache: HashMap<String, usize>,
}

pub struct CPUInfoData {
    manufacturer: Option<String>,
    brand: Option<String>,
    vendor: Option<String>,
    family: Option<String>,
    model: Option<String>,
    stepping: Option<String>,
    revision: Option<String>,
    voltage: Option<String>,
    speed: Option<f64>,
    speed_min: Option<f64>,
    speed_max: Option<f64>,
    governor: Option<String>,
    cores: Option<usize>,
    physical_cores: Option<usize>,
    processors: Option<usize>,
    socket: Option<String>,
    flags: Vec<String>,
    virtualization: Option<bool>,
    cache: HashMap<String, usize>,
}



impl From<CPUInfoData> for CPUInfo {
    fn from(val: CPUInfoData) -> CPUInfo {
        CPUInfo {
            manufacturer: super::option_to_string(val.manufacturer),
            brand: super::option_to_string(val.brand),
            vendor: super::option_to_string(val.vendor),
            family: super::option_to_string(val.family),
            model: super::option_to_string(val.model),
            stepping: super::option_to_string(val.stepping),
            revision: match val.revision {
                Some(x) => x,
                None => String::new(),
            },
            voltage: match val.voltage {
                Some(x) => x,
                None => String::new(),
            },
            speed: f64::trunc(val.speed.unwrap() * 100.0) / 100.0,
            speed_min: f64::trunc(val.speed_min.unwrap() * 100.0) / 100.0,
            speed_max: f64::trunc(val.speed_max.unwrap() * 100.0) / 100.0,
            governor: super::option_to_string(val.governor),
            cores: val.cores.unwrap(),
            physical_cores: val.physical_cores.unwrap(),
            processors: val.processors.unwrap(),
            socket: match val.socket {
                Some(x) => x,
                None => String::new(),
            },
            flags: val.flags.join(" "),
            virtualization: val.virtualization.unwrap(),
            cache: val.cache,
        }
    }
}

#[derive(PartialEq, Clone)]
enum CacheType {
    Data(usize),
    Instruction(usize),
    Unified(usize),
} 

impl CPUInfoData {
    pub fn new() -> Self {
        let mut socket = None;
        let mut voltage = None;
        if let Ok(sys_info) = smbioslib::table_load_from_device() {
            let base_info: Vec<smbioslib::SMBiosProcessorInformation> = sys_info.collect();
            socket = base_info[0].socket_designation();
            if let Some(voltage_info) =  base_info[0].voltage() {
                if let smbioslib::ProcessorVoltage::CurrentVolts(volts) = voltage_info {
                    voltage = Some(volts.to_string());
                } 
            }
        }
        let data = fs::read_to_string("/proc/cpuinfo").unwrap().trim().to_string();
        let cpu_data: Vec<String> = data.replace("\r", "").split("\n\n").map(|x| x.to_string()).collect();
        let num_cpus = cpu_data.len();
        let cpu_hashmap_vec = Self::parse_cpuinfo_data(cpu_data);
        let cpu_hashmap = cpu_hashmap_vec[0].clone();
        let brand_manufacturer = match cpu_hashmap.get("model name") {
            Some(model_name) => {
                let t = model_name.replace("(R)", "®").replace("(TM)", "™").replace("CPU", "");
                let words = t.split_whitespace().collect::<Vec<&str>>();
                (Some(words[1..].join(" ").to_string()), Some(words[0].to_string()))
            },
            None => (None, None),
        };
        let brand = brand_manufacturer.0;
        let manufacturer = brand_manufacturer.1;
        let vendor = super::obs_to_os(cpu_hashmap.get("vendor_id"));
        let family = super::obs_to_os(cpu_hashmap.get("cpu family"));
        let model = super::obs_to_os(cpu_hashmap.get("model"));
        let stepping = super::obs_to_os(cpu_hashmap.get("stepping"));
        let revision = super::obs_to_os(cpu_hashmap.get("cpu revision"));
        let flags: Vec<String> = cpu_hashmap.get("flags").unwrap().split(" ").map(|x| x.to_string()).collect();
        let virtualization = Some(flags.clone().contains(&"vmx".to_string()) || flags.clone().contains(&"svm".to_string()));
        let mut speed = match brand.clone() {
            Some(b) => {
                let mut out = Some(cpu_hashmap_vec.clone().into_iter().map(|x| x.get("cpu mhz").unwrap_or(&"0.0".to_string()).parse::<f64>().unwrap() / 1000.0).sum::<f64>() / num_cpus as f64);
                if let Some(parts) = b.split_once("@") {
                    if let Ok(speed) = parts.1.parse::<f64>() {
                        out = Some(speed)
                    } else if manufacturer.as_ref().unwrap() == "AMD" {
                        let s = serde_json::from_str::<serde_json::Value>(AMD_SPEEDS).unwrap();
                        let obj = s.as_object().unwrap();
                        for cpu_model in obj.keys() {
                            let mut found: usize = 0;
                            let cpu_split = cpu_model.split("|");
                            for cpu_part in cpu_split.clone() {
                                if b.contains(cpu_part) {
                                    found += 1;
                                }
                            }
                            if found == cpu_split.count() {
                                out = Some(obj.get(cpu_model).unwrap().as_str().unwrap().parse::<f64>().unwrap())
                            }
                        }
                    }
                }
                out
            },
            None => {
                Some(cpu_hashmap_vec.clone().into_iter().map(|x| x.get("cpu mhz").unwrap_or(&"0.0".to_string()).parse::<f64>().unwrap() / 1000.0).sum::<f64>() / num_cpus as f64)
            },
        };
        if speed == Some(0.0) {
            speed = None;
        }
        let max_speed: Option<f64>;
        let min_speed: Option<f64>;
        if cpu_hashmap.contains_key("cpu mhz") {
            let speeds = cpu_hashmap_vec.clone().into_iter().map(|x| {
                //println!("{:?}", x.clone());
                ordered_float::OrderedFloat(x.get("cpu mhz").unwrap().parse::<f64>().unwrap() / 1000.0)
            });
            max_speed = Some(speeds.clone().max().unwrap().into_inner());
            min_speed = Some(speeds.min().unwrap().into_inner());
        } else {
            max_speed = None;
            min_speed = None;
        }
        let physical_cores: Option<usize>;
        if cpu_hashmap.contains_key("cpu cores") {
            physical_cores = Some(cpu_hashmap.get("cpu cores").unwrap().parse::<usize>().unwrap());
        } else {
            physical_cores = None;
        }
        let processors: Option<usize>;
        if cpu_hashmap.contains_key("physical id") {
            processors = Some(Self::count_physical_ids(cpu_hashmap_vec));
        } else {
            processors = None;
        }
        Self {
            manufacturer: manufacturer,
            brand: brand,
            vendor: vendor,
            family: family,
            model: model,
            stepping: stepping,
            revision: revision,
            voltage: voltage,
            speed: speed,
            speed_min: min_speed,
            speed_max: max_speed,
            governor: Some(fs::read_to_string("/sys/devices/system/cpu/cpu0/cpufreq/scaling_governor").unwrap().trim().to_string()),
            cores: Some(num_cpus),
            physical_cores: physical_cores,
            processors: processors,
            socket: socket,
            flags: flags,
            virtualization: virtualization,
            cache: Self::get_cache_hashmap(),
        
        }
    }
    fn count_physical_ids(data: Vec<HashMap<String, String>>) -> usize {
        let mut physical_core_ids: Vec<String> = Vec::new(); 
        for cpu_info in data {
            let physical_core_id = cpu_info.get("physical id").unwrap();
            if !physical_core_ids.contains(physical_core_id) {
                physical_core_ids.push(physical_core_id.to_string())
            }
        }
        physical_core_ids.len()
    }
    fn parse_cpuinfo_data(data: Vec<String>) -> Vec<HashMap<String, String>> {
        let mut cpu_info_data: Vec<HashMap<String, String>> = Vec::new();
        for cpu_data in data {
            if cpu_data.trim() == "" {
                continue;
            }
            let lines = cpu_data.split("\n");
            let mut cpu_info: HashMap<String, String> = HashMap::new();
            for line in lines {
                if line.trim() == "" {
                    continue;
                }
                let parts = line.split_once(":").unwrap();
                //println!("Parts: '{:?}'", parts.0.trim().to_lowercase().to_string());
                cpu_info.insert(parts.0.trim().to_lowercase().to_string(), parts.1.trim().to_string());
            }
            cpu_info_data.push(cpu_info);
        }
        cpu_info_data
    }
    fn get_cache_hashmap() -> HashMap<String, usize> {
        let mut cache_usize: HashMap<String, usize> = HashMap::new();
        cache_usize.insert("l1d".to_string(), 0);
        cache_usize.insert("l1i".to_string(), 0);
        cache_usize.insert("l2".to_string(), 0);
        cache_usize.insert("l3".to_string(), 0);
        let mut cache_levels: Vec<(CacheType, u8)> = Vec::new();
        let index_num_regex = Regex::new(r"^index(\d+)$").unwrap();
        let cpu_num_regex = Regex::new(r"^cpu(\d+)$").unwrap();
        let cpu_dirs = fs::read_dir("/sys/devices/system/cpu").unwrap().filter(|x| {
            //println!("{:?}", x.as_ref().unwrap().file_name().into_string().unwrap().as_str());
            cpu_num_regex.is_match(x.as_ref().unwrap().file_name().into_string().unwrap().as_str())
        });
        for cpu_dir_result in cpu_dirs {
            let cpu_dir = cpu_dir_result.unwrap().path();
            let indexes = fs::read_dir(cpu_dir.join("cache")).unwrap().filter(|y| {
                //println!("{:?}", y.as_ref().unwrap().file_name().into_string().unwrap().as_str());
                index_num_regex.is_match(y.as_ref().unwrap().file_name().into_string().unwrap().as_str())
            });
            for index_path_result in indexes {
                let index_path = index_path_result.unwrap().path();
                let index_type = fs::read_to_string(&index_path.join("type")).unwrap().trim().to_string();
                let index_level = fs::read_to_string(&index_path.join("level")).unwrap().trim().parse::<u8>().unwrap();
                let index_id = fs::read_to_string(&index_path.join("id")).unwrap().trim().parse::<usize>().unwrap();
                let index_size = fs::read_to_string(&index_path.join("size")).unwrap().trim().split_once("K").unwrap().0.parse::<usize>().unwrap() * 1024;
                let cache_type: CacheType;
                if index_type.as_str() == "Unified" {
                    cache_type = CacheType::Unified(index_id);
                } else if index_type.as_str() == "Data" {
                    cache_type = CacheType::Data(index_id);
                } else {
                    cache_type = CacheType::Instruction(index_id);
                }
                if !cache_levels.contains(&(cache_type.clone(), index_level)) {
                    cache_levels.push((cache_type.clone(), index_level));
                    match cache_type {
                        CacheType::Data(_x) => {
                            let mut current_size = cache_usize.get("l1d").unwrap().to_owned();
                            current_size += index_size;
                            cache_usize.insert("l1d".to_string(), current_size);
                        },
                        CacheType::Instruction(_x) => {
                            let mut current_size = cache_usize.get("l1i").unwrap().to_owned();
                            current_size += index_size;
                            cache_usize.insert("l1i".to_string(), current_size);
                        },
                        CacheType::Unified(_x) => {
                            let key: String;
                            if index_level == 2 {
                                key = "l2".to_string();
                            } else {
                                key = "l3".to_string();
                            }
                            let mut current_size = cache_usize.get(&key).unwrap().to_owned();
                            current_size += index_size;
                            cache_usize.insert(key, current_size);
                        },
                    }
                }
            }
        }
        cache_usize
    }
}
