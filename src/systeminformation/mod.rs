use serde::{Deserialize, Serialize};
use chrono::Utc;
use docker_api::Docker;
pub mod cpu;
pub mod system;
pub mod os;
pub fn option_to_string<T>(val: Option<T>) -> String where T :std::fmt::Display {
    if val.is_some() {
        return val.unwrap().to_string();
    } else {
        return String::new();
    }
}

pub fn obs_to_os(val: Option<&String>) -> Option<String> {
    let mut output = None;
    if val.is_some() {
        output = Some(val.unwrap().to_string());
    }
    output
}

#[derive(Clone, Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
struct DockerInfo {
    kernel_version: String,
    operating_system: String,
    os_version: String,
    os_type: String,
    architecture: String,
    ncpu: u64,
    mem_total: u64,
    server_version: String
}


#[derive(Clone, Deserialize, Serialize, Debug)]
pub struct ExtraData {
    pub version: String,
    pub metadata: MetaData,
    pub machine: Machine,
}

#[derive(Clone, Deserialize, Serialize, Debug)]
pub struct MetaData {
    date: i64,
}

#[derive(Clone, Deserialize, Serialize, Debug)]
pub struct Machine {
    cpu: crate::systeminformation::cpu::CPUInfo,
    system: crate::systeminformation::system::SystemInfo,
    os: crate::systeminformation::os::OSInfo,
    docker: DockerInfo,
}


impl ExtraData {
    pub async fn new(nodejs_arch: String) -> Self {
        let version = "1".to_string();
        let metadata = MetaData {
            date: Utc::now().naive_utc().timestamp(),
        };
        let docker = Docker::new("unix:///var/run/docker.sock").unwrap();
        let info = docker.info().await.unwrap();
        let machine = Machine { 
            cpu: crate::systeminformation::cpu::CPUInfoData::new().into(),
            system: crate::systeminformation::system::SystemInfo::new(),
            os: crate::systeminformation::os::OSInfo::new(nodejs_arch),
            docker: DockerInfo {
                kernel_version: info.kernel_version,
                operating_system: info.operating_system,
                architecture: info.architecture,
                mem_total: info.mem_total,
                ncpu: info.n_cpu,
                os_type: info.os_type,
                os_version: info.os_version,
                server_version: info.server_version,
            }
        };
        Self {
            version: version,
            metadata: metadata,
            machine: machine,
        }
    }
}