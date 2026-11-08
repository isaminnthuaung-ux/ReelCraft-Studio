use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct SystemInfo {
    pub cpu: String,
    pub gpus: Vec<String>,
    pub ram_bytes: Option<u64>,
    pub os: String,
}

#[cfg(windows)]
pub fn read() -> SystemInfo {
    use serde::Deserialize;
    use std::sync::atomic::AtomicBool;
    #[derive(Default, Deserialize)]
    #[serde(default)]
    struct RegistryInfo {
        cpu: String,
        gpus: Vec<String>,
        os: String,
    }
    // Read inventory rather than WMI, which can be disabled by local policy.
    // This fixed script takes no user input and runs without a console window.
    let script = r#"
$ErrorActionPreference='Stop'
[Console]::OutputEncoding=[System.Text.UTF8Encoding]::new($false)
$cpu=(Get-ItemProperty -LiteralPath 'HKLM:\HARDWARE\DESCRIPTION\System\CentralProcessor\0').ProcessorNameString
$version=Get-ItemProperty -LiteralPath 'HKLM:\SOFTWARE\Microsoft\Windows NT\CurrentVersion'
$name=$version.ProductName
if ([int]$version.CurrentBuildNumber -ge 22000 -and $name -like 'Windows 10*') { $name=$name -replace '^Windows 10','Windows 11' }
$os=(@($name,$version.DisplayVersion,('Build '+$version.CurrentBuildNumber)) | Where-Object { $_ }) -join ' '
$gpus=@(Get-ChildItem -LiteralPath 'HKLM:\SYSTEM\CurrentControlSet\Control\Video' -ErrorAction SilentlyContinue | ForEach-Object {
    $adapter=Get-ItemProperty -LiteralPath ($_.PSPath+'\0000') -ErrorAction SilentlyContinue
    if ($adapter.DriverDesc) { $adapter.DriverDesc }
} | Where-Object { $_ -notmatch 'Remote|Mirage|Basic Render|RDP' } | Sort-Object -Unique)
@{cpu=$cpu.Trim();gpus=$gpus;os=$os} | ConvertTo-Json -Compress
"#;
    let registry = crate::process::capture(
        crate::process::command("powershell.exe").args([
            "-NoProfile",
            "-NonInteractive",
            "-Command",
            script,
        ]),
        &AtomicBool::new(false),
    )
    .ok()
    .and_then(|bytes| serde_json::from_slice::<RegistryInfo>(&bytes).ok())
    .unwrap_or_default();
    SystemInfo {
        cpu: if registry.cpu.is_empty() {
            std::env::var("PROCESSOR_IDENTIFIER").unwrap_or_else(|_| "Unavailable".into())
        } else {
            registry.cpu
        },
        gpus: registry.gpus,
        ram_bytes: physical_memory(),
        os: if registry.os.is_empty() {
            format!("Windows ({})", std::env::consts::ARCH)
        } else {
            registry.os
        },
    }
}

#[cfg(windows)]
fn physical_memory() -> Option<u64> {
    #[repr(C)]
    #[derive(Default)]
    struct MemoryStatus {
        length: u32,
        load: u32,
        total_physical: u64,
        available_physical: u64,
        total_page: u64,
        available_page: u64,
        total_virtual: u64,
        available_virtual: u64,
        available_extended: u64,
    }
    #[link(name = "kernel32")]
    unsafe extern "system" {
        fn GlobalMemoryStatusEx(status: *mut MemoryStatus) -> i32;
    }
    let mut status = MemoryStatus {
        length: std::mem::size_of::<MemoryStatus>() as u32,
        ..Default::default()
    };
    // SAFETY: the Win32 ABI layout matches MEMORYSTATUSEX; the initialized
    // buffer remains valid and exclusively borrowed for the duration of the call.
    if unsafe { GlobalMemoryStatusEx(&mut status) } != 0 {
        Some(status.total_physical)
    } else {
        None
    }
}

#[cfg(not(windows))]
pub fn read() -> SystemInfo {
    let cpu = std::fs::read_to_string("/proc/cpuinfo")
        .ok()
        .and_then(|s| {
            s.lines().find_map(|line| {
                line.strip_prefix("model name")
                    .and_then(|v| v.split_once(':'))
                    .map(|(_, v)| v.trim().to_string())
            })
        })
        .unwrap_or_else(|| "Unavailable".into());
    let ram_bytes = std::fs::read_to_string("/proc/meminfo").ok().and_then(|s| {
        s.lines().find_map(|line| {
            line.strip_prefix("MemTotal:")
                .and_then(|v| v.split_whitespace().next())
                .and_then(|v| v.parse::<u64>().ok())
                .map(|kb| kb * 1024)
        })
    });
    SystemInfo {
        cpu,
        gpus: Vec::new(),
        ram_bytes,
        os: format!("{} ({})", std::env::consts::OS, std::env::consts::ARCH),
    }
}

#[cfg(all(test, windows))]
mod tests {
    #[test]
    fn physical_ram_is_available_without_wmi() {
        assert!(super::physical_memory().unwrap() > 0);
    }
    #[test]
    fn desktop_inventory_is_serializable() {
        let info = super::read();
        assert!(!info.cpu.is_empty());
        assert!(!info.os.is_empty());
        assert!(info.ram_bytes.unwrap() > 0);
        println!("{}", serde_json::to_string(&info).unwrap());
    }
}
