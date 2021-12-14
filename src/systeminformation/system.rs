use std::fs;
use serde::{Deserialize, Serialize};
#[derive(Clone, Deserialize, Serialize, Debug)]
pub struct SystemInfo {
    manufacturer: String,
    model: String,
    version: String,
    sku: String,
    r#virtual: bool,
}
impl SystemInfo {
    pub fn new() -> Self {
        let mut manufacturer = fs::read_to_string("/sys/devices/virtual/dmi/id/sys_vendor").unwrap_or("-".to_string()).trim().replace("Default string", "");
        let mut model = fs::read_to_string("/sys/devices/virtual/dmi/id/product_name").unwrap_or("-".to_string()).trim().replace("Default string", "").to_string();
        let mut version = fs::read_to_string("/sys/devices/virtual/dmi/id/product_version").unwrap_or("-".to_string()).trim().replace("Default string", "").to_string();
        let mut sku = "-".to_string();
        if let Ok(sys_info) = smbioslib::table_load_from_device() {
            let base_info: Vec<smbioslib::SMBiosSystemInformation> = sys_info.collect();
            let info = base_info.get(0).unwrap();
            if let Some(man) = info.manufacturer() {
                manufacturer = man.clone();
            }
            if let Some(m) = info.product_name() {
                model = m.clone();
            }
            if let Some(v) = info.version() {
                version = v.clone();
            }
            if let Some(s) = info.sku_number() {
                sku = s.clone();
            }
        }
        let model_clone = model.clone().to_lowercase();
        let model_str: &str = model_clone.as_str();
        Self {
            manufacturer: manufacturer,
            model: model,
            version: version,
            sku: sku,
            r#virtual: vec!["virtualbox", "kvm", "virtual machine", "bochs"].contains(&model_str) || model_str.starts_with("vmware") || model_str.starts_with("droplet"),
        }
    }
}