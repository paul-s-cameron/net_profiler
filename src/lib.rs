use std::{
    fmt::Display, net::{IpAddr, Ipv4Addr}, process::{Command, Stdio}
};

pub type Result<T> = core::result::Result<T, Error>;
pub type Error = Box<dyn std::error::Error>; // For early dev.

#[derive(serde::Deserialize, serde::Serialize)]
#[derive(Default, Debug, Clone, PartialEq, Eq, Hash)]
#[serde(default)]
pub struct NetworkProfile {
    pub name: String,
    pub ips: Vec<IP>,
    pub gateways: Vec<String>,
    pub dns: DNS,
    pub mac: Option<MAC>,
}

#[derive(serde::Deserialize, serde::Serialize)]
#[derive(Default, Debug, Clone, PartialEq, Eq, Hash)]
pub struct IP {
    pub address: String,
    pub subnet: String,
}

#[derive(serde::Deserialize, serde::Serialize)]
#[derive(Default, Debug, Clone, PartialEq, Eq, Hash)]
pub enum DNS {
    #[default]
    DHCP,
    Quad9,
    Google,
    Cloudflare,
    OpenDNS,
    Custom {
        primary: String,
        secondary: String,
    },
}

#[derive(serde::Deserialize, serde::Serialize)]
#[derive(Default, Debug, Clone, PartialEq, Eq, Hash)]
pub struct MAC {
    address: String,
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

impl From<(&'static str, &'static str)> for DNS {
    fn from(value: (&'static str, &'static str)) -> Self {
        Self::Custom {
            primary: value.0.into(),
            secondary: value.0.into(),
        }
    }
}

impl Display for DNS {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", match self {
            DNS::DHCP => "DHCP",
            DNS::Quad9 => "Quad9",
            DNS::Google => "Google",
            DNS::Cloudflare => "Cloudflare",
            DNS::OpenDNS => "OpenDNS",
            DNS::Custom { primary: _, secondary: _ } => "Custom"
        })
    }
}

impl DNS {
    pub const QUAD9: (&str, &str) = ("9.9.9.9", "149.112.112.112");
    pub const GOOGLE: (&str, &str) = ("8.8.8.8", "8.8.4.4");
    pub const CLOUDFLARE: (&str, &str) = ("1.1.1.2", "1.0.0.2");
    pub const OPENDNS: (&str, &str) = ("208.67.222.222", "208.67.220.220");
    
    pub fn addresses(&self) -> Option<(String, String)> {
        match &self {
            DNS::DHCP => None,
            DNS::Quad9 => Some((DNS::QUAD9.0.into(),DNS::QUAD9.1.into())),
            DNS::Google => Some((DNS::GOOGLE.0.into(),DNS::GOOGLE.1.into())),
            DNS::Cloudflare => Some((DNS::CLOUDFLARE.0.into(),DNS::CLOUDFLARE.1.into())),
            DNS::OpenDNS => Some((DNS::OPENDNS.0.into(),DNS::OPENDNS.1.into())),
            DNS::Custom { primary, secondary } => Some((primary.into(), secondary.into()))
        }
    }

    pub fn primary(&self) -> Option<String> {
        match &self {
            DNS::DHCP => None,
            DNS::Quad9 => Some(DNS::QUAD9.0.into()),
            DNS::Google => Some(DNS::GOOGLE.0.into()),
            DNS::Cloudflare => Some(DNS::CLOUDFLARE.0.into()),
            DNS::OpenDNS => Some(DNS::OPENDNS.0.into()),
            DNS::Custom { primary, secondary: _ } => Some(primary.into())
        }
    }

    pub fn secondary(&self) -> Option<String> {
        match &self {
            DNS::DHCP => None,
            DNS::Quad9 => Some(DNS::QUAD9.1.into()),
            DNS::Google => Some(DNS::GOOGLE.1.into()),
            DNS::Cloudflare => Some(DNS::CLOUDFLARE.1.into()),
            DNS::OpenDNS => Some(DNS::OPENDNS.1.into()),
            DNS::Custom { primary: _, secondary } => Some(secondary.into())
        }
    }
}

impl From<(&'static str, &'static str)> for IP {
    fn from(value: (&'static str, &'static str)) -> Self {
        Self { address: value.0.to_string(), subnet: value.1.to_string() }
    }
}

pub fn load_profile(profile: &NetworkProfile) {
    // // Check if adapter is blank
    // if self.adapter.is_empty() {
    //     return;
    // }

    // let adapter = self.adapter.as_str();
    // let dns_provider: Option<DNS> = match self.dns_provider {
    //     DNSProvider::Quad9 => Some(("9.9.9.9","149.112.112.112").into()),
    //     DNSProvider::Google => Some(("8.8.8.8","8.8.4.4").into()),
    //     DNSProvider::Cloudflare => Some(("1.1.1.2","1.0.0.2").into()),
    //     DNSProvider::OpenDNS => Some(("208.67.222.222","208.67.220.220").into()),
    //     DNSProvider::Custom => Some(self.custom_dns.clone()),
    //     _ => None,
    // };

    // // Set IP subnet and gateway
    // if let Some(first_address) = self.ips.first() {
    //     let gateway = self.gateways.first().map(|x| x.as_str());
    //     set_ip_addr(adapter, &first_address.address, &first_address.subnet, gateway);
    //     for ip in self.ips.iter().skip(1) {
    //         add_ip_addr(adapter, &ip.address, &ip.subnet);
    //     }
    // }

    // if self.gateways.len() > 1 {
    //     for (i, gateway) in self.gateways.iter().skip(1).enumerate() {
    //         add_gateway(adapter, gateway, i+1);
    //     }
    // }

    // // Set DNS servers
    // if let Some(dns) = dns_provider {
    //     let output = Command::new("powershell")
    //         .arg("-Command")
    //         .arg(format!(
    //             "netsh interface ip set dns \"{}\" static {} primary validate=no; netsh interface ip add dns \"{}\" {} validate=no",
    //             adapter, dns.primary, adapter, dns.secondary
    //         ))
    //         .output()
    //         .expect("Failed to set DNS servers");
    // } else {
    //     let output = Command::new("powershell")
    //         .arg("-Command")
    //         .arg(format!(
    //             "netsh interface ip set dnsservers \"{}\" source=dhcp",
    //             adapter,
    //         ))
    //         .output()
    //         .expect("Failed to set DNS servers");
    // }

    // // Set Mac Address
    // if !self.mac_address.is_empty() {
    //     if self.validate_mac_address(&self.mac_address) {
    //         self.set_mac_address();
    //     } else {
    //         eprintln!("Invalid MAC address: {}", self.mac_address);
    //     }
    // }
}

/// Sets the primary static IP address for a network adapter.
/// This **must** be called only once per adapter.
pub fn set_ip_addr(
    adapter: &str,
    ip_address: &str,
    subnet: &str,
    gateway: Option<&str>
) -> Result<()> {
    #[cfg(target_os = "windows")]
    {
        let gateway_arg = gateway.unwrap_or("none"); // Use "none" if no gateway is provided

        let output = Command::new("netsh")
            .args([
                "interface", "ip", "set", "address",
                adapter, "static", ip_address, subnet, gateway_arg,
            ])
            .stdout(Stdio::inherit()) // Print command output to console
            .stderr(Stdio::piped())   // Capture stderr for error handling
            .output();

        match output {
            Ok(output) if output.status.success() => {
                println!(
                    "Successfully set primary IP address: {} on {} (Gateway: {})",
                    ip_address, adapter, gateway_arg
                );
            }
            Ok(output) => {
                eprintln!(
                    "Error setting primary IP address: {}",
                    String::from_utf8_lossy(&output.stderr)
                );
            }
            Err(err) => {
                eprintln!("Failed to execute netsh command: {}", err);
            }
        }
    }
    #[cfg(target_os = "linux")]
    {
        //TODO: Implement Linux command to set IP address
    }

    Ok(())
}


/// Adds an additional static IP address to a network adapter.
/// This can be called multiple times.
pub fn add_ip_addr(
    adapter: &str,
    ip_address: &str,
    subnet: &str
) -> Result<()> {
    #[cfg(target_os = "windows")]
    {
        let output = Command::new("netsh")
            .args([
                "interface", "ip", "add", "address",
                adapter, ip_address, subnet,
            ])
            .stdout(Stdio::inherit()) // Print command output to console
            .stderr(Stdio::piped())   // Capture stderr for error handling
            .output();

        match output {
            Ok(output) if output.status.success() => {
                println!("Successfully added IP address: {} on {}", ip_address, adapter);
            }
            Ok(output) => {
                eprintln!(
                    "Error adding IP address: {}",
                    String::from_utf8_lossy(&output.stderr)
                );
            }
            Err(err) => {
                eprintln!("Failed to execute netsh command: {}", err);
            }
        }
    }
    #[cfg(target_os = "linux")]
    {
        //TODO: Implement Linux command to add IP address
    }

    Ok(())
}

/// Adds an additional gateway to a network adapter with a specified metric.
/// Lower metric values have higher priority.
pub fn add_gateway(
    adapter: &str,
    gateway: &str,
    metric: usize
) -> Result<()> {
    #[cfg(target_os = "windows")]
    {
        let metric = metric.to_string();
        let output = Command::new("netsh")
            .args([
                "interface", "ip", "set", "route",
                "0.0.0.0", "0.0.0.0", adapter, gateway, metric.as_str(),
            ])
            .stdout(Stdio::inherit())
            .stderr(Stdio::piped())  
            .output();

        match output {
            Ok(output) if output.status.success() => {
                println!("Successfully added gateway: {} with metric {} on {}", gateway, metric, adapter);
            }
            Ok(output) => {
                eprintln!(
                    "Error adding gateway: {}",
                    String::from_utf8_lossy(&output.stderr)
                );
            }
            Err(err) => {
                eprintln!("Failed to execute netsh command: {}", err);
            }
        }
    }
    #[cfg(target_os = "linux")]
    {
        //TODO: Implement Linux command to add gateway
    }

    Ok(())
}

pub fn check_valid_ipv4(ip_address: &str) -> bool {
    match ip_address.parse::<Ipv4Addr>() {
        Ok(_) => true,
        Err(e) => false
    }
}