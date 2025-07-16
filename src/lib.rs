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

pub fn load_profile(profile: &NetworkProfile, adapter: &str) {
    if let Some(first_address) = profile.ips.first() {
        let gateway = profile.gateways.first().map(|x| x.as_str());
        set_ip_addr(adapter, &first_address.address, &first_address.subnet, gateway);
        for ip in profile.ips.iter().skip(1) {
            add_ip_addr(adapter, &ip.address, &ip.subnet);
        }
    }

    if profile.gateways.len() > 1 {
        for (i, gateway) in profile.gateways.iter().skip(1).enumerate() {
            add_gateway(adapter, gateway, i+1);
        }
    }

    set_dns(adapter, &profile.dns);

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
                return Err(String::from_utf8_lossy(&output.stderr).into());
            }
            Err(e) => {
                eprintln!("Failed to execute netsh command: {}", e);
                return Err(e.into());
            }
        }
    }
    #[cfg(target_os = "linux")]
    {
        let output = Command::new("ip")
            .args([
                "addr", "flush", "dev", adapter, 
            ])
            .stdout(Stdio::inherit())
            .stderr(Stdio::piped())
            .output();

        if let Err(e) = output {
            eprintln!("Failed to flush IP addresses on {}: {}", adapter, e);
            return Err(e.into());
        }

        let output = Command::new("ip")
            .args([
                "addr", "add", format!("{}/{}", ip_address, subnet).as_str(),
                "dev", adapter,
            ])
            .stdout(Stdio::inherit())
            .stderr(Stdio::piped())
            .output();

        match output {
            Ok(output) if output.status.success() => {
                println!(
                    "Successfully set primary IP address: {} on {} (Gateway: {})",
                    ip_address, adapter, gateway.unwrap_or("none")
                );
            }
            Ok(output) => {
                eprintln!(
                    "Error setting primary IP address: {}",
                    String::from_utf8_lossy(&output.stderr)
                );
                return Err(String::from_utf8_lossy(&output.stderr).into());
            }
            Err(e) => {
                eprintln!("Failed to execute ip command: {}", e);
                return Err(e.into());
            }
        }
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
                return Err(String::from_utf8_lossy(&output.stderr).into());
            }
            Err(e) => {
                eprintln!("Failed to execute netsh command: {}", e);
                return Err(e.into());
            }
        }
    }
    #[cfg(target_os = "linux")]
    {
        let output = Command::new("ip")
            .args([
                "addr", "add", format!("{}/{}", ip_address, subnet).as_str(),
                "dev", adapter,
            ])
            .stdout(Stdio::inherit())
            .stderr(Stdio::piped())
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
                return Err(String::from_utf8_lossy(&output.stderr).into());
            }
            Err(e) => {
                eprintln!("Failed to execute ip command: {}", e);
                return Err(e.into());
            }
        }
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
                log::info!("Successfully added gateway: {} with metric {} on {}", gateway, metric, adapter);
            }
            Ok(output) => {
                log::error!(
                    "Error adding gateway: {}",
                    String::from_utf8_lossy(&output.stderr)
                );
                return Err(String::from_utf8_lossy(&output.stderr).into());
            }
            Err(e) => {
                log::error!("Failed to execute netsh command: {}", e);
                return Err(e.into());
            }
        }
    }
    #[cfg(target_os = "linux")]
    {
        let output = Command::new("ip")
            .args([
                "route", "add", "default", "via", gateway, "dev", adapter, "metric", &metric.to_string(),
            ])
            .stdout(Stdio::inherit())
            .stderr(Stdio::piped())
            .output();

        match output {
            Ok(output) if output.status.success() => {
                log::info!("Successfully added gateway: {} with metric {} on {}", gateway, metric, adapter);
            }
            Ok(output) => {
                log::error!(
                    "Error adding gateway: {}",
                    String::from_utf8_lossy(&output.stderr)
                );
                return Err(String::from_utf8_lossy(&output.stderr).into());
            }
            Err(e) => {
                log::error!("Failed to execute ip command: {}", e);
                return Err(e.into());
            }
        }
    }

    Ok(())
}

pub fn set_dns(
    adapter: &str,
    dns: &DNS
) -> Result<()> {
    #[cfg(target_os = "windows")]
    {
        match dns {
            DNS::DHCP => {
                match Command::new("powershell")
                    .arg("-Command")
                    .arg(format!(
                        "netsh interface ip set dnsservers \"{}\" source=dhcp",
                        adapter,
                    ))
                    .output() {
                        Err(e) => {
                            log::error!("{}", e);
                            return Err(e.into());
                        }
                        Ok(_) => {}
                    }
            }
            _ => {
                match Command::new("powershell")
                    .arg("-Command")
                    .arg(format!(
                        "netsh interface ip set dns \"{}\" static {} primary validate=no; netsh interface ip add dns \"{}\" {} validate=no",
                        adapter, dns.primary().unwrap(), adapter, dns.secondary().unwrap()
                    ))
                    .output() {
                        Err(e) => {
                            log::error!("{}", e);
                            return Err(e.into());
                        }
                        Ok(_) => {}
                    }
            }
        }
    }
    #[cfg(target_os = "linux")]
    {
        match dns {
            DNS::DHCP => {
                match Command::new("nmcli")
                    .args(["con", "modify", adapter, "ipv4.method", "auto"])
                    .output() {
                        Err(e) => {
                            log::error!("{}", e);
                            return Err(e.into());
                        }
                        Ok(_) => {}
                    }
            }
            _ => {
                if let Some((primary, secondary)) = dns.addresses() {
                    match Command::new("nmcli")
                        .args([
                            "con", "modify", adapter,
                            "ipv4.dns", &primary,
                            "ipv4.dns-search", &secondary,
                            "ipv4.method", "manual",
                        ])
                        .output() {
                            Err(e) => {
                                log::error!("{}", e);
                                return Err(e.into());
                            }
                            Ok(_) => {}
                        }
                }
            }
        }
    }

    Ok(())
}

pub fn check_valid_ipv4(ip_address: &str) -> bool {
    match ip_address.parse::<Ipv4Addr>() {
        Ok(_) => true,
        Err(e) => false
    }
}