use std::{
    io::{BufRead, BufReader, Read},
    path::PathBuf,
    process::{Command, Stdio},
    sync::atomic::{AtomicBool, Ordering},
    thread,
    time::Duration,
};

pub fn command(program: &str) -> Command {
    let mut cmd = Command::new(program);
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        cmd.creation_flags(0x08000000);
    }
    cmd
}

/// Read FFmpeg's progress pipe while supervising cancellation. Progress is
/// streamed to the caller instead of held until the entire part finishes.
pub fn render_progress(
    cmd: &mut Command,
    stop: &AtomicBool,
    mut notify: impl FnMut(f64, &str),
) -> Result<(), String> {
    if stop.load(Ordering::Relaxed) {
        return Err("Stopped. Completed parts are kept for resume.".into());
    }
    let mut child = cmd
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| e.to_string())?;
    let stdout = child.stdout.take().unwrap();
    let mut stderr = child.stderr.take().unwrap();
    let (sender, receiver) = std::sync::mpsc::channel();
    let reader = thread::spawn(move || {
        for line in BufReader::new(stdout).lines().map_while(Result::ok) {
            if sender.send(line).is_err() {
                break;
            }
        }
    });
    let errors = thread::spawn(move || {
        let mut bytes = Vec::new();
        let _ = stderr.read_to_end(&mut bytes);
        bytes
    });
    let mut seconds = 0.0;
    let mut speed = String::new();
    let mut drain = || {
        for line in receiver.try_iter() {
            if let Some(value) = line.strip_prefix("out_time_us=") {
                if let Ok(value) = value.parse::<f64>() {
                    if value.is_finite() {
                        seconds = value.max(0.0) / 1_000_000.0;
                    }
                }
            } else if let Some(value) = line.strip_prefix("speed=") {
                speed = value.trim().to_string();
            } else if line.starts_with("progress=") {
                notify(seconds, &speed);
            }
        }
    };
    let status = loop {
        drain();
        if stop.load(Ordering::Relaxed) {
            let _ = child.kill();
        }
        match child.try_wait() {
            Ok(Some(status)) => break status,
            Ok(None) => thread::sleep(Duration::from_millis(80)),
            Err(error) => {
                let _ = child.kill();
                let _ = child.wait();
                let _ = reader.join();
                let _ = errors.join();
                return Err(error.to_string());
            }
        }
    };
    let _ = reader.join();
    drain();
    let bytes = errors.join().unwrap_or_default();
    if stop.load(Ordering::Relaxed) {
        return Err("Stopped. Completed parts are kept for resume.".into());
    }
    if !status.success() {
        return Err(String::from_utf8_lossy(&bytes)
            .chars()
            .rev()
            .take(1800)
            .collect::<String>()
            .chars()
            .rev()
            .collect());
    }
    Ok(())
}

pub fn tool(name: &str) -> String {
    if let Ok(exe) = std::env::current_exe() {
        if let Some(root) = exe.parent() {
            let path = root.join("bin").join(format!("{name}.exe"));
            if path.is_file() {
                return path.to_string_lossy().into();
            }
        }
    }
    let chocolatey = PathBuf::from(format!(
        "C:/ProgramData/chocolatey/lib/ffmpeg/tools/ffmpeg/bin/{name}.exe"
    ));
    if chocolatey.is_file() {
        return chocolatey.to_string_lossy().into();
    }
    name.into()
}

pub fn capture(cmd: &mut Command, stop: &AtomicBool) -> Result<Vec<u8>, String> {
    if stop.load(Ordering::Relaxed) {
        return Err("Stopped. Completed parts are kept for resume.".into());
    }
    let mut child=cmd.stdin(Stdio::null()).stdout(Stdio::piped()).stderr(Stdio::piped()).spawn().map_err(|e|format!("Could not start media tool: {e}. Install FFmpeg or place it in the app's bin folder."))?;
    let mut out = child.stdout.take().unwrap();
    let mut err = child.stderr.take().unwrap();
    let reader = thread::spawn(move || {
        let mut b = Vec::new();
        let _ = out.read_to_end(&mut b);
        b
    });
    let errors = thread::spawn(move || {
        let mut b = Vec::new();
        let _ = err.read_to_end(&mut b);
        b
    });
    let status = loop {
        if stop.load(Ordering::Relaxed) {
            let _ = child.kill();
        }
        match child.try_wait() {
            Ok(Some(s)) => break s,
            Ok(None) => thread::sleep(Duration::from_millis(80)),
            Err(e) => {
                let _ = child.kill();
                let _ = child.wait();
                return Err(e.to_string());
            }
        }
    };
    let output = reader.join().unwrap_or_default();
    let error = String::from_utf8_lossy(&errors.join().unwrap_or_default()).into_owned();
    if stop.load(Ordering::Relaxed) {
        return Err("Stopped. Completed parts are kept for resume.".into());
    }
    if !status.success() {
        return Err(error
            .chars()
            .rev()
            .take(1800)
            .collect::<String>()
            .chars()
            .rev()
            .collect());
    }
    Ok(output)
}

pub fn open_folder(path: &str) -> Result<(), String> {
    let p = PathBuf::from(path);
    if !p.is_dir() {
        return Err("The output folder does not exist yet.".into());
    }
    command("explorer.exe")
        .arg(p)
        .spawn()
        .map_err(|e| e.to_string())?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn streaming_render_reports_progress_and_stops() {
        let stop = std::sync::Arc::new(AtomicBool::new(false));
        let flag = stop.clone();
        let mut updates = Vec::new();
        let timer = thread::spawn(move || {
            thread::sleep(Duration::from_millis(450));
            flag.store(true, Ordering::SeqCst);
        });
        let result = render_progress(
            command(&tool("ffmpeg")).args([
                "-v",
                "error",
                "-re",
                "-f",
                "lavfi",
                "-i",
                "sine=duration=10",
                "-progress",
                "pipe:1",
                "-stats_period",
                "0.1",
                "-f",
                "null",
                "-",
            ]),
            &stop,
            |time, _| updates.push(time),
        );
        timer.join().unwrap();
        assert!(result.unwrap_err().starts_with("Stopped"));
        assert!(updates.iter().any(|s| *s > 0.0));
    }
    #[test]
    fn cancel_interrupts_active_analysis() {
        let stop = std::sync::Arc::new(AtomicBool::new(false));
        let flag = stop.clone();
        let timer = thread::spawn(move || {
            thread::sleep(Duration::from_millis(250));
            flag.store(true, Ordering::SeqCst);
        });
        let now = std::time::Instant::now();
        let result = capture(
            command(&tool("ffmpeg")).args([
                "-v",
                "error",
                "-re",
                "-f",
                "lavfi",
                "-i",
                "sine=duration=20",
                "-f",
                "null",
                "-",
            ]),
            &stop,
        );
        timer.join().unwrap();
        assert!(result.unwrap_err().starts_with("Stopped"));
        assert!(now.elapsed() < Duration::from_secs(4));
    }
}
