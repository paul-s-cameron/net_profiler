#[cfg(target_os = "windows")]
use winres;

fn main() {
    let mut res = winres::WindowsResource::new();
    res.set_manifest(r#"
    <assembly xmlns="urn:schemas-microsoft-com:asm.v1" manifestVersion="1.0">
    <trustInfo xmlns="urn:schemas-microsoft-com:asm.v3">
        <security>
            <requestedPrivileges>
                <requestedExecutionLevel level="requireAdministrator" uiAccess="false" />
            </requestedPrivileges>
        </security>
    </trustInfo>
    </assembly>
    "#);

    match res.compile() {
        Ok(_) => println!("cargo:rerun-if-changed=build.rs"),
        Err(e) => eprintln!("Error: {}", e),
    }
}