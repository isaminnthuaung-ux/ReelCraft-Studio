use crate::model::EncodingProfile;
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
    pub fn args(self, profile: EncodingProfile) -> Vec<&'static str> {
        let (preset, cq) = match profile {
            EncodingProfile::Fastest => ("p1", "18"),
            EncodingProfile::Fast => ("p3", "18"),
            EncodingProfile::Balanced => ("p5", "21"),
            EncodingProfile::Quality => ("p7", "18"),
        };
        match self {
            Self::Nvidia => vec![
                "-c:v",
                "h264_nvenc",
                "-preset",
                preset,
                "-tune",
                "hq",
                "-rc",
                "vbr",
                "-cq",
                cq,
                "-b:v",
                "0",
                "-multipass",
                "disabled",
                "-rc-lookahead",
                "0",
                "-profile:v",
                "high",
                "-pix_fmt",
                "yuv420p",
            ],
            Self::Cpu => vec!["-c:v", "libx264", "-preset", "veryfast", "-crf", "20"],
        }
    }
}

/// A listed encoder can still fail because of drivers or unavailable hardware.
/// Encode two small frames to check that NVENC actually initializes.
pub fn select(stop: &AtomicBool, profile: EncodingProfile) -> Result<Encoder, String> {
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
            .args(Encoder::Nvidia.args(profile))
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
    fn profiles_reach_nvenc_and_keep_cpu_fallback_consistent() {
        for (profile, preset, cq) in [
            (EncodingProfile::Fastest, "p1", "18"),
            (EncodingProfile::Fast, "p3", "18"),
            (EncodingProfile::Balanced, "p5", "21"),
            (EncodingProfile::Quality, "p7", "18"),
        ] {
            let args = Encoder::Nvidia.args(profile);
            assert!(args.windows(2).any(|p| p == ["-preset", preset]));
            assert!(args.windows(2).any(|p| p == ["-cq", cq]));
            assert_eq!(
                Encoder::Cpu.args(profile),
                Encoder::Cpu.args(EncodingProfile::Fastest)
            );
        }
    }

    #[test]
    fn cpu_fallback_is_bounded_and_never_retries_stop() {
        assert_eq!(fallback(Encoder::Nvidia, true, false), Some(Encoder::Cpu));
        assert_eq!(fallback(Encoder::Cpu, true, false), None);
        assert_eq!(fallback(Encoder::Nvidia, true, true), None);
    }
}
