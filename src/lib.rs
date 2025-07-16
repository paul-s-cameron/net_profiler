use std::{
    fmt::Display, net::Ipv4Addr, process::{Command, Stdio}
};

pub type Result<T> = core::result::Result<T, Error>;
pub type Error = Box<dyn std::error::Error>; // For early dev.

/// Check if the application needs to be relaunched with elevated privileges
/// Returns true if the application should continue running, false if it was relaunched
pub fn check_and_relaunch_elevated() -> Result<bool> {
    // Check if we're already running as root
    let is_root = unsafe { libc::getuid() == 0 };
    
    if is_root {
        // Already running as root, continue normal execution
        Ok(true)
    } else {
        // Not root, need to relaunch with elevated privileges
        println!("Network configuration requires elevated privileges. Relaunching with pkexec...");
        
        // Get the current executable path
        let current_exe = std::env::current_exe()
            .map_err(|e| format!("Failed to get current executable path: {}", e))?;
        
        // Get current arguments
        let args: Vec<String> = std::env::args().skip(1).collect();
        
        // Try pkexec first
        let pkexec_result = Command::new("pkexec")
            .arg(&current_exe)
            .args(&args)
            .status();
            
        match pkexec_result {
            Ok(status) => {
                // pkexec succeeded, exit with the same status code
                std::process::exit(status.code().unwrap_or(0));
            }
            Err(_) => {
                // Fallback to sudo
                println!("pkexec not available, trying sudo...");
                let sudo_result = Command::new("sudo")
                    .arg(&current_exe)
                    .args(&args)
                    .status();
                    
                match sudo_result {
                    Ok(status) => {
                        std::process::exit(status.code().unwrap_or(0));
                    }
                    Err(e) => {
                        return Err(format!("Failed to elevate privileges: {}", e).into());
                    }
                }
            }
        }
    }
}


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
    None,
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
            secondary: value.1.into(), // Fixed: was value.0.into()
        }
    }
}

impl Display for DNS {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", match self {
            DNS::None => "None",
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
            DNS::None => None,
            DNS::Quad9 => Some((DNS::QUAD9.0.into(),DNS::QUAD9.1.into())),
            DNS::Google => Some((DNS::GOOGLE.0.into(),DNS::GOOGLE.1.into())),
            DNS::Cloudflare => Some((DNS::CLOUDFLARE.0.into(),DNS::CLOUDFLARE.1.into())),
            DNS::OpenDNS => Some((DNS::OPENDNS.0.into(),DNS::OPENDNS.1.into())),
            DNS::Custom { primary, secondary } => Some((primary.into(), secondary.into()))
        }
    }

    pub fn primary(&self) -> Option<String> {
        match &self {
            DNS::None => None,
            DNS::Quad9 => Some(DNS::QUAD9.0.into()),
            DNS::Google => Some(DNS::GOOGLE.0.into()),
            DNS::Cloudflare => Some(DNS::CLOUDFLARE.0.into()),
            DNS::OpenDNS => Some(DNS::OPENDNS.0.into()),
            DNS::Custom { primary, secondary: _ } => Some(primary.into())
        }
    }

    pub fn secondary(&self) -> Option<String> {
        match &self {
            DNS::None => None,
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

pub fn load_profile(profile: &NetworkProfile, adapter: &str) -> Result<()> {
    if let Some(first_address) = profile.ips.first() {
        let gateway = profile.gateways.first().map(|x| x.as_str());
        
        // Set the primary IP address
        if let Err(e) = set_ip_addr(adapter, &first_address.address, &first_address.subnet, gateway) {
            eprintln!("Failed to set primary IP address: {}", e);
            return Err(e);
        }
        
        // Add additional IP addresses
        for ip in profile.ips.iter().skip(1) {
            if let Err(e) = add_ip_addr(adapter, &ip.address, &ip.subnet) {
                eprintln!("Failed to add IP address {}: {}", ip.address, e);
                return Err(e);
            }
        }
    }

    // Add additional gateways
    if profile.gateways.len() > 1 {
        for (i, gateway) in profile.gateways.iter().skip(1).enumerate() {
            if let Err(e) = add_gateway(adapter, gateway, i + 1) {
                eprintln!("Failed to add gateway {}: {}", gateway, e);
                return Err(e);
            }
        }
    }

    // Set DNS configuration
    if let Err(e) = set_dns(adapter, &profile.dns) {
        eprintln!("Failed to set DNS: {}", e);
        return Err(e);
    }

    println!("Successfully loaded profile '{}' on adapter '{}'", profile.name, adapter);
    Ok(())

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
    let normalized_subnet = normalize_subnet_for_os(subnet)?;
    
    #[cfg(target_os = "windows")]
    {
        let gateway_arg = gateway.unwrap_or("none"); // Use "none" if no gateway is provided

        let output = Command::new("netsh")
            .args([
                "interface", "ip", "set", "address",
                adapter, "static", ip_address, &normalized_subnet, gateway_arg,
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
                "addr", "add", format!("{}{}", ip_address, normalized_subnet).as_str(),
                "dev", adapter,
            ])
            .stdout(Stdio::inherit())
            .stderr(Stdio::piped())
            .output();

        match output {
            Ok(output) if output.status.success() => {
                println!(
                    "Successfully set primary IP address: {} on {}",
                    ip_address, adapter
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

        // Set gateway if provided
        if let Some(gateway) = gateway {
            let output = Command::new("ip")
                .args([
                    "route", "add", "default", "via", gateway, "dev", adapter,
                ])
                .stdout(Stdio::inherit())
                .stderr(Stdio::piped())
                .output();

            match output {
                Ok(output) if output.status.success() => {
                    println!(
                        "Successfully set gateway: {} on {}",
                        gateway, adapter
                    );
                }
                Ok(output) => {
                    eprintln!(
                        "Warning: Failed to set gateway: {}",
                        String::from_utf8_lossy(&output.stderr)
                    );
                    // Don't return error for gateway failures
                }
                Err(e) => {
                    eprintln!("Warning: Failed to execute gateway command: {}", e);
                    // Don't return error for gateway failures
                }
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
    let normalized_subnet = normalize_subnet_for_os(subnet)?;
    
    #[cfg(target_os = "windows")]
    {
        let output = Command::new("netsh")
            .args([
                "interface", "ip", "add", "address",
                adapter, ip_address, &normalized_subnet,
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
                "addr", "add", format!("{}{}", ip_address, normalized_subnet).as_str(),
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
                "interface", "ip", "add", "route", // Fixed: was "set route"
                "0.0.0.0/0", gateway, adapter, "metric", metric.as_str(),
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
            DNS::None => {
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
            DNS::None => {
                // Reset DNS to automatic while keeping static IP configuration
                match Command::new("nmcli")
                    .args(["con", "modify", adapter, "ipv4.dns", "", "ipv4.ignore-auto-dns", "no"])
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
                    // Set DNS servers (multiple DNS servers should be comma-separated)
                    let dns_servers = format!("{},{}", primary, secondary);
                    match Command::new("nmcli")
                        .args([
                            "con", "modify", adapter,
                            "ipv4.dns", &dns_servers,
                            "ipv4.ignore-auto-dns", "yes",
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
    ip_address.parse::<Ipv4Addr>().is_ok()
}

pub fn check_valid_subnet(subnet: &str) -> bool {
    // Check if it's a valid subnet mask in dotted decimal notation (e.g., 255.255.255.0)
    if subnet.parse::<Ipv4Addr>().is_ok() {
        // Additional check to see if it's a valid subnet mask
        if let Ok(addr) = subnet.parse::<Ipv4Addr>() {
            let octets = addr.octets();
            // Convert to u32 for easier bit manipulation
            let mask = u32::from_be_bytes(octets);
            
            // A valid subnet mask should have all 1s followed by all 0s
            // Check if (mask & (mask + 1)) == 0, which is true for valid subnet masks
            mask.leading_ones() + mask.trailing_zeros() == 32
        } else {
            false
        }
    } else if subnet.starts_with('/') && subnet.len() > 1 {
        // Check if it's CIDR notation (e.g., /24)
        if let Ok(cidr) = subnet[1..].parse::<u8>() {
            cidr <= 32
        } else {
            false
        }
    } else {
        false
    }
}

/// Converts CIDR notation to dotted decimal notation
/// Example: "/24" -> "255.255.255.0"
pub fn cidr_to_dotted_decimal(cidr: &str) -> Result<String> {
    if let Some(cidr_str) = cidr.strip_prefix('/') {
        if let Ok(prefix_len) = cidr_str.parse::<u8>() {
            if prefix_len <= 32 {
                // Create a mask with 'prefix_len' number of 1s followed by 0s
                let mask = if prefix_len == 0 {
                    0u32
                } else {
                    !((1u32 << (32 - prefix_len)) - 1)
                };
                
                // Convert to IPv4 address
                let addr = Ipv4Addr::from(mask);
                return Ok(addr.to_string());
            }
        }
    }
    Err(format!("Invalid CIDR notation: {}", cidr).into())
}

/// Normalizes subnet format for the target OS
/// Windows: Converts CIDR to dotted decimal
/// Linux: Keeps CIDR as is, converts dotted decimal to CIDR
pub fn normalize_subnet_for_os(subnet: &str) -> Result<String> {
    #[cfg(target_os = "windows")]
    {
        if subnet.starts_with('/') {
            cidr_to_dotted_decimal(subnet)
        } else {
            Ok(subnet.to_string())
        }
    }
    #[cfg(target_os = "linux")]
    {
        if subnet.starts_with('/') {
            Ok(subnet.to_string())
        } else {
            // Convert dotted decimal to CIDR for Linux
            dotted_decimal_to_cidr(subnet)
        }
    }
}

/// Converts dotted decimal notation to CIDR notation
/// Example: "255.255.255.0" -> "/24"
pub fn dotted_decimal_to_cidr(subnet: &str) -> Result<String> {
    if let Ok(addr) = subnet.parse::<Ipv4Addr>() {
        let mask = u32::from_be_bytes(addr.octets());
        let prefix_len = mask.leading_ones();
        
        // Verify it's a valid subnet mask
        if mask.leading_ones() + mask.trailing_zeros() == 32 {
            Ok(format!("/{}", prefix_len))
        } else {
            Err(format!("Invalid subnet mask: {}", subnet).into())
        }
    } else {
        Err(format!("Invalid dotted decimal notation: {}", subnet).into())
    }
}