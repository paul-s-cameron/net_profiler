use std::{path::PathBuf, process::Command, net::Ipv4Addr};

#[derive(serde::Deserialize, serde::Serialize)]
#[derive(Default, Debug, Clone, PartialEq, Eq, Hash)]
#[serde(default)]
pub struct NetworkProfile {
    pub name: String,
    pub adapter: String,
    pub ip: String,
    pub subnet: String,
    pub gateway: String,
    pub dns_provider: DNSProvider,
    pub primary_dns: String,
    pub secondary_dns: String,
}

impl NetworkProfile {
    pub fn load(&self) {
        // Check if adapter is blank
        if self.adapter.is_empty() {
            return;
        }

        // Set the windows adapters values to the profile values
        let adapter = self.adapter.clone();

        let ip_address: &String = &self.ip;
        let subnet: &String = &self.subnet;
        let gateway: &String = &self.gateway;
        let dns_servers: Vec<&str> = match self.dns_provider {
            DNSProvider::Quad9 => vec!["9.9.9.9","149.112.112.112"],
            DNSProvider::Google => vec!["8.8.8.8","8.8.4.4"],
            DNSProvider::Cloudflare => vec!["1.1.1.2","1.0.0.2"],
            DNSProvider::OpenDNS => vec!["208.67.222.222","208.67.220.220"],
            DNSProvider::Custom => vec![self.primary_dns.as_str(), self.secondary_dns.as_str()],
            _ => vec!["",""],
        };

        // Set IP subnet and gateway
        let output = Command::new("powershell")
            .arg("-Command")
            .arg(format!(
                "netsh interface ip set address \"{}\" static {} {} {}",
                adapter, ip_address, subnet, gateway
            ))
            .output()
            .expect("Failed to set DNS servers");

        // Set DNS servers
        if let DNSProvider::None = self.dns_provider { return }
        let _output = Command::new("powershell")
            .arg("-Command")
            .arg(format!(
                "netsh interface ip set dns \"{}\" static {} primary validate=no; netsh interface ip add dns \"{}\" {} validate=no",
                adapter, dns_servers[0], adapter, dns_servers[1]
            ))
            .output()
            .expect("Failed to set DNS servers");
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
    None,
    Quad9,
    Google,
    Cloudflare,
    OpenDNS,
    Custom,
}