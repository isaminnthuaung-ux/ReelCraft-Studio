use crate::process::{capture, command, tool};
use std::sync::atomic::AtomicBool;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Encoder {
    Nvidia,
    Cpu,
}
impl Encoder {
    pub fn name(self) -> &'static str {
        match self {
            Self::Nvidia => "NVIDIA NVENC",
            Self::Cpu => "CPU H.264",
        }
    }
    pub fn args(self) -> &'static [&'static str] {
        match self {
            Self::Nvidia => &[
                "-c:v",
                "h264_nvenc",
                "-preset",
                "p5",
                "-rc",
                "vbr",
                "-cq",
                "21",
                "-b:v",
                "0",
            ],
            Self::Cpu => &["-c:v", "libx264", "-preset", "veryfast", "-crf", "20"],
        }
    }
}

/// A listed encoder can still fail because of drivers or unavailable hardware.
/// Encode two small frames to check that NVENC actually initializes.
pub fn select(stop: &AtomicBool) -> Result<Encoder, String> {
    let result = capture(
        command(&tool("ffmpeg"))
            .args([
                "-v",
                "error",
                "-f",
                "lavfi",
                "-i",
                "color=s=640x360:r=30",
                "-frames:v",
                "2",
            ])
            .args(Encoder::Nvidia.args())
            .args(["-f", "null", "-"]),
        stop,
    );
    if stop.load(std::sync::atomic::Ordering::Relaxed) {
        return Err("Stopped. Completed parts are kept for resume.".into());
    }
    Ok(if result.is_ok() {
        Encoder::Nvidia
    } else {
        Encoder::Cpu
    })
}

pub fn fallback(encoder: Encoder, failed: bool, cancelled: bool) -> Option<Encoder> {
    if encoder == Encoder::Nvidia && failed && !cancelled {
        Some(Encoder::Cpu)
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn cpu_fallback_is_bounded_and_never_retries_stop() {
        assert_eq!(fallback(Encoder::Nvidia, true, false), Some(Encoder::Cpu));
        assert_eq!(fallback(Encoder::Cpu, true, false), None);
        assert_eq!(fallback(Encoder::Nvidia, true, true), None);
    }
}
