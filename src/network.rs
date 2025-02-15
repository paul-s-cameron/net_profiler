use std::{path::PathBuf, process::Command, net::Ipv4Addr};
use registry::{RegKey, Hive, Security};

use crate::utils;

#[derive(serde::Deserialize, serde::Serialize)]
#[derive(Default, Debug, Clone, PartialEq, Eq, Hash)]
#[serde(default)]
pub struct NetworkProfile {
    pub name: String,
    pub adapter: String,
    pub ips: Vec<IP>,
    pub gateways: Vec<String>,
    pub dns_provider: DNSProvider,
    pub custom_dns: DNS,
    pub mac_address: String,
}

impl NetworkProfile {
    pub fn load(&self) {
        // Check if adapter is blank
        if self.adapter.is_empty() {
            return;
        }

        let adapter = self.adapter.as_str();
        let dns_provider: Option<DNS> = match self.dns_provider {
            DNSProvider::Quad9 => Some(("9.9.9.9","149.112.112.112").into()),
            DNSProvider::Google => Some(("8.8.8.8","8.8.4.4").into()),
            DNSProvider::Cloudflare => Some(("1.1.1.2","1.0.0.2").into()),
            DNSProvider::OpenDNS => Some(("208.67.222.222","208.67.220.220").into()),
            DNSProvider::Custom => Some(self.custom_dns.clone()),
            _ => None,
        };

        // Set IP subnet and gateway
        if let Some(first_address) = self.ips.first() {
            let gateway = self.gateways.first().map(|x| x.as_str());
            utils::set_ip_addr(adapter, &first_address.address, &first_address.subnet, gateway);
            for ip in self.ips.iter().skip(1) {
                utils::add_ip_addr(adapter, &ip.address, &ip.subnet);
            }
        }

        if self.gateways.len() > 1 {
            for (i, gateway) in self.gateways.iter().skip(1).enumerate() {
                utils::add_gateway(adapter, gateway, i+1);
            }
        }

        // Set DNS servers
        if let Some(dns) = dns_provider {
            let output = Command::new("powershell")
                .arg("-Command")
                .arg(format!(
                    "netsh interface ip set dns \"{}\" static {} primary validate=no; netsh interface ip add dns \"{}\" {} validate=no",
                    adapter, dns.primary, adapter, dns.secondary
                ))
                .output()
                .expect("Failed to set DNS servers");
        } else {
            let output = Command::new("powershell")
                .arg("-Command")
                .arg(format!(
                    "netsh interface ip set dnsservers \"{}\" source=dhcp",
                    adapter,
                ))
                .output()
                .expect("Failed to set DNS servers");
        }

        // Set Mac Address
        if !self.mac_address.is_empty() {
            if self.validate_mac_address(&self.mac_address) {
                self.set_mac_address();
            } else {
                eprintln!("Invalid MAC address: {}", self.mac_address);
            }
        }
    }

    /// Validate the MAC address format (e.g., "XX-XX-XX-XX-XX-XX")
    fn validate_mac_address(&self, mac: &str) -> bool {
        let mac_regex = regex::Regex::new(r"^([A-Fa-f0-9]{2}[:-]){5}[A-Fa-f0-9]{2}$").unwrap();
        mac_regex.is_match(mac)
    }

    /// Set the MAC address using the registry
    fn set_mac_address(&self) {
        // Open the registry key for the network adapter class
        let class_key_path = r"SYSTEM\CurrentControlSet\Control\Class\{4d36e972-e325-11ce-bfc1-08002be10318}";
        let hklm = Hive::LocalMachine;
        let class_key = hklm.open(class_key_path, Security::AllAccess).unwrap();

        // Find the subkey corresponding to the adapter
        for i in 0..50 {
            let subkey_name = format!("{:04}", i);
            if let Ok(adapter_key) = class_key.open(&subkey_name, Security::AllAccess) {
                if let Ok(value) = adapter_key.value::<String>("DriverDesc".to_string()) {
                    match value {
                        registry::Data::String(value) => {
                            if value.to_string_lossy() == self.adapter {
                                let address = registry::Data::String(
                                    utfx::U16CString::from_str(&self.adapter)
                                        .expect("Failed to convert MAC address to U16CString")
                                );

                                // Set the "NetworkAddress" value
                                adapter_key
                                    .set_value("NetworkAddress", &address)
                                    .expect("Failed to set NetworkAddress registry value");
                                
                                println!("Successfully set MAC address for adapter: {}", self.adapter);
                                return;
                            }
                        }
                        _ => {}
                    }
                }
            }
        }

        eprintln!("Adapter not found in registry for setting MAC address.");
    }
}

impl From<serde_json::Value> for NetworkProfile {
    fn from(value: serde_json::Value) -> Self {
        serde_json::from_value(value).unwrap_or_default()
    }
}

impl Into<serde_json::Value> for NetworkProfile {
    fn into(self) -> serde_json::Value {
        serde_json::to_value(&self).unwrap_or_default()
    }
}

#[derive(serde::Deserialize, serde::Serialize)]
#[derive(Default, Debug, Clone, PartialEq, Eq, Hash)]
pub enum DNSProvider {
    #[default]
    DHCP,
    Quad9,
    Google,
    Cloudflare,
    OpenDNS,
    Custom,
}

#[derive(serde::Deserialize, serde::Serialize)]
#[derive(Default, Debug, Clone, PartialEq, Eq, Hash)]
pub struct DNS {
    pub primary: String,
    pub secondary: String,
}

impl From<(&'static str, &'static str)> for DNS {
    fn from(value: (&'static str, &'static str)) -> Self {
        Self { primary: value.0.to_string(), secondary: value.1.to_string() }
    }
}

#[derive(serde::Deserialize, serde::Serialize)]
#[derive(Default, Debug, Clone, PartialEq, Eq, Hash)]
pub struct IP {
    pub address: String,
    pub subnet: String,
}

impl From<(&'static str, &'static str)> for IP {
    fn from(value: (&'static str, &'static str)) -> Self {
        Self { address: value.0.to_string(), subnet: value.1.to_string() }
    }
}