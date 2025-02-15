use std::process::{Command, Stdio};

/// Sets the primary static IP address for a network adapter.
/// This **must** be called only once per adapter.
pub fn set_ip_addr(adapter: &str, ip_address: &str, subnet: &str, gateway: Option<&str>) {
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


/// Adds an additional static IP address to a network adapter.
/// This can be called multiple times.
pub fn add_ip_addr(adapter: &str, ip_address: &str, subnet: &str) {
    let output = Command::new("netsh")
        .args([
            "interface", "ip", "add", "address",
            adapter, ip_address, subnet,
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
        }
        Err(err) => {
            eprintln!("Failed to execute netsh command: {}", err);
        }
    }
}

/// Adds an additional gateway to a network adapter with a specified metric.
/// Lower metric values have higher priority.
pub fn add_gateway(adapter: &str, gateway: &str, metric: usize) {
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
