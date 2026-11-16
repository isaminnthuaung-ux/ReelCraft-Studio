//! Video frames remain on CUDA through scaling, compositing and NVENC.
//! Static artwork is prepared once per source; Unicode text stays in TextEngine.
use crate::{
    media,
    model::{Settings, Source, WallpaperMode},
    process::{capture, command, tool},
};
use std::{
    fs,
    path::{Path, PathBuf},
    sync::atomic::AtomicBool,
};

pub struct Background(pub PathBuf);
pub fn available(stop: &AtomicBool) -> bool {
    capture(
        command(&tool("ffmpeg")).args(["-hide_banner", "-filters"]),
        stop,
    )
    .map(|bytes| {
        let text = String::from_utf8_lossy(&bytes);
        ["scale_cuda", "overlay_cuda", "hwupload_cuda", "pad_cuda"]
            .iter()
            .all(|name| text.split_whitespace().any(|word| word == *name))
    })
    .unwrap_or(false)
}
/// Reuse driver-compiled kernels across clips, including portable installations.
pub fn configure_cuda(cmd: &mut std::process::Command) {
    let cache = std::env::temp_dir().join("reelcraft-cuda-cache");
    if fs::create_dir_all(&cache).is_ok() {
        cmd.env("CUDA_CACHE_PATH", cache);
    }
}
impl Drop for Background {
    fn drop(&mut self) {
        let _ = fs::remove_file(&self.0);
    }
}

pub fn static_background(source: &Source, settings: &Settings) -> bool {
    let (w, h) = crate::model::dimensions(source, &settings.ratio);
    let matching_ratio =
        (source.width as f64 / source.height as f64 - w as f64 / h as f64).abs() < 0.01;
    settings.fill != "blur" || ((settings.scale - 1.0).abs() < 0.0001 && matching_ratio)
}

pub fn prepare(
    source: &Source,
    settings: &Settings,
    w: u32,
    h: u32,
    path: &Path,
    stop: &AtomicBool,
) -> Result<Background, String> {
    // Reuse the exact CPU framing recipe for static artwork, including split-wallpaper
    // crop/pan/zoom. A black stand-in occupies the opaque video's final footprint.
    let mut cmd = command(&tool("ffmpeg"));
    cmd.args([
        "-v",
        "error",
        "-f",
        "lavfi",
        "-i",
        &format!(
            "color=c=black:s={}x{}:r=1",
            source.width / 2 * 2,
            source.height / 2 * 2
        ),
    ]);
    for wallpaper in settings.wallpaper_paths() {
        cmd.arg("-i").arg(wallpaper);
    }
    let bytes = capture(
        cmd.args([
            "-filter_complex",
            &media::framing_filter(source, w, h, settings),
            "-map",
            "[frame]",
            "-frames:v",
            "1",
            "-c:v",
            "png",
            "-f",
            "image2pipe",
            "pipe:1",
        ]),
        stop,
    )?;
    fs::write(path, bytes).map_err(|e| e.to_string())?;
    Ok(Background(path.into()))
}

pub fn graph(
    source: &Source,
    settings: &Settings,
    w: u32,
    h: u32,
    decode: bool,
    overlay: Option<(u32, u32)>,
) -> String {
    let sx = w as f64 / source.width as f64;
    let sy = h as f64 / source.height as f64;
    let fit = if settings.fill == "crop" {
        sx.max(sy)
    } else {
        sx.min(sy)
    };
    let sw = ((source.width as f64 * fit * settings.scale / 2.0).round() as u32 * 2).max(2);
    let sh = ((source.height as f64 * fit * settings.scale / 2.0).round() as u32 * 2).max(2);
    let y = if settings.fill == "wallpaper"
        && settings.wallpaper_layout.mode == WallpaperMode::Split
    {
        format!(
            "{}",
            (((h as f64 - sh as f64) * settings.wallpaper_layout.video_y / 2.0).round() as i64) * 2
        )
    } else {
        format!("{}", ((h as i64 - sh as i64) / 2) & !1)
    };
    let x = ((w as i64 - sw as i64) / 2) & !1;
    let upload = if decode {
        ""
    } else {
        "format=yuv420p,hwupload_cuda,"
    };
    let mut graph = format!("[0:v:0]{upload}scale_cuda=format=yuv420p,split=2[clock][front];[clock]scale_cuda={w}:{h}[base];[1:v]format=yuv420p,hwupload_cuda[wall];[base][wall]overlay_cuda=0:0:eof_action=repeat[bg];[front]scale_cuda={sw}:{sh}[fg];[bg][fg]overlay_cuda=x={x}:y={y}:eof_action=repeat[frame]");
    if let Some((x, y)) = overlay {
        graph.push_str(&format!(";[2:v]format=yuva420p,hwupload_cuda[text];[frame][text]overlay_cuda={x}:{y}:eof_action=repeat,crop={w}:{h}:0:0,pad_cuda={w}:{h}:0:0,setsar=1[out]"));
    } else {
        graph.push_str(&format!(
            ";[frame]crop={w}:{h}:0:0,pad_cuda={w}:{h}:0:0,setsar=1[out]"
        ));
    }
    // overlay_cuda can allocate 32-pixel-aligned backing frames when it makes
    // inputs writable. Crop the visible area, then use CUDA padding to copy the visible area so NVENC
    // receives the requested frame dimensions rather than the allocation size.
    graph
}

/// A failed CUDA path must not prematurely disable a working NVENC encoder.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Mode {
    CudaDecode,
    CudaUpload,
    Hybrid,
    CpuFilters,
}
impl Mode {
    pub fn gpu_filters(self) -> bool {
        matches!(self, Self::CudaDecode | Self::CudaUpload)
    }
    pub fn gpu_decode(self) -> bool {
        matches!(self, Self::CudaDecode | Self::Hybrid)
    }
    pub fn fallback(self) -> Option<Self> {
        match self {
            Self::CudaDecode => Some(Self::CudaUpload),
            Self::CudaUpload | Self::Hybrid => Some(Self::CpuFilters),
            Self::CpuFilters => None,
        }
    }
    pub fn label(self) -> &'static str {
        match self {
            Self::CudaDecode => "GPU decode + GPU filters",
            Self::CudaUpload => "CPU decode + GPU filters",
            Self::Hybrid => "GPU decode + CPU filters",
            Self::CpuFilters => "CPU decode + CPU filters",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{EncodingProfile, TextLayer, WallpaperLayout};

    fn settings() -> Settings {
        let text = TextLayer {
            enabled: false,
            text: String::new(),
            font: "Segoe UI".into(),
            size: 24.0,
            x: 0.5,
            y: 0.5,
            color: "#ffffff".into(),
            opacity: 1.0,
            boxed: false,
        };
        Settings {
            seconds: 5.0,
            smart: false,
            extra: 0.0,
            ratio: "9:16".into(),
            fill: "black".into(),
            scale: 0.75,
            wallpaper: String::new(),
            wallpaper_layout: WallpaperLayout::default(),
            encoding_profile: EncodingProfile::Fastest,
            text: text.clone(),
            label: text,
        }
    }

    #[test]
    fn fallback_finishes_on_cpu_filters_without_disabling_nvenc() {
        assert_eq!(Mode::CudaDecode.fallback(), Some(Mode::CudaUpload));
        assert_eq!(Mode::CudaUpload.fallback(), Some(Mode::CpuFilters));
        assert_eq!(Mode::Hybrid.fallback(), Some(Mode::CpuFilters));
        assert_eq!(Mode::CpuFilters.fallback(), None);
    }

    #[test]
    fn metadata_from_old_projects_remains_readable() {
        let source: Source = serde_json::from_str(r#"{"path":"x.mp4","name":"x","duration":1,"width":320,"height":180,"has_audio":false}"#).unwrap();
        assert!(!source.gpu_decode);
        assert_eq!(source.fps, 0.0);
    }

    #[test]
    fn cuda_upload_preserves_alpha_and_exact_output_dimensions() {
        let stop = AtomicBool::new(false);
        if crate::encoder::select(&stop, EncodingProfile::Fastest).unwrap()
            != crate::encoder::Encoder::Nvidia
            || !available(&stop)
        {
            eprintln!("Skipping CUDA integration test: no working NVIDIA encoder");
            return;
        }
        let dir = tempfile::tempdir().unwrap();
        let input = dir.path().join("green.mp4");
        capture(
            command(&tool("ffmpeg")).args([
                "-v",
                "error",
                "-f",
                "lavfi",
                "-i",
                "color=green:s=320x180:r=24:d=1",
                "-c:v",
                "libx264",
                "-preset",
                "ultrafast",
                &input.to_string_lossy(),
            ]),
            &stop,
        )
        .unwrap();
        let source = media::inspect(&input.to_string_lossy(), &stop).unwrap();
        assert_eq!(source.codec, "h264");
        assert_eq!(source.fps, 24.0);
        let settings = settings();
        let background = prepare(
            &source,
            &settings,
            360,
            640,
            &dir.path().join("background.png"),
            &stop,
        )
        .unwrap();
        let title = dir.path().join("alpha.png");
        image::RgbaImage::from_pixel(120, 40, image::Rgba([255, 255, 255, 128]))
            .save(&title)
            .unwrap();
        for (decode, fill, scale) in [
            (false, "black", 0.75),
            (true, "black", 0.75),
            (true, "crop", 1.0),
        ] {
            if decode && !source.gpu_decode {
                continue;
            }
            let mut settings = settings.clone();
            settings.fill = fill.into();
            settings.scale = scale;
            let output = dir.path().join(format!("cuda-{decode}-{fill}.mp4"));
            let mut cmd = command(&tool("ffmpeg"));
            configure_cuda(&mut cmd);
            cmd.args([
                "-v",
                "error",
                "-init_hw_device",
                "cuda=render:0",
                "-filter_hw_device",
                "render",
            ]);
            if decode {
                cmd.args([
                    "-hwaccel",
                    "cuda",
                    "-hwaccel_output_format",
                    "cuda",
                    "-hwaccel_device",
                    "render",
                ]);
            }
            cmd.arg("-i")
                .arg(&input)
                .arg("-i")
                .arg(&background.0)
                .arg("-i")
                .arg(&title)
                .args([
                    "-filter_complex",
                    &graph(&source, &settings, 360, 640, decode, Some((60, 100))),
                    "-map",
                    "[out]",
                    "-c:v",
                    "h264_nvenc",
                    "-preset",
                    "p1",
                    "-t",
                    "1",
                ])
                .arg(&output);
            capture(&mut cmd, &stop).unwrap();
            let rendered = media::probe(&output.to_string_lossy(), &stop).unwrap();
            assert_eq!((rendered.width, rendered.height), (360, 640));
            assert_eq!(rendered.fps, 24.0);
            assert!((rendered.duration - 1.0).abs() < 0.05);
            let bytes = capture(
                command(&tool("ffmpeg"))
                    .arg("-v")
                    .arg("error")
                    .arg("-i")
                    .arg(&output)
                    .args([
                        "-frames:v",
                        "1",
                        "-f",
                        "image2pipe",
                        "-c:v",
                        "png",
                        "pipe:1",
                    ]),
                &stop,
            )
            .unwrap();
            let image = image::load_from_memory(&bytes).unwrap().to_rgb8();
            let green = image.get_pixel(180, 320);
            assert!(green[1] > 90 && green[0] < 30 && green[2] < 30);
            if fill == "crop" {
                // A video wider than the canvas must crop at a negative x offset.
                for x in [5, 355] {
                    assert!(image.get_pixel(x, 320)[1] > 90);
                }
                continue;
            }
            let black = image.get_pixel(5, 5);
            assert!(black[0] < 15 && black[1] < 15 && black[2] < 15);
            let alpha = image.get_pixel(80, 110);
            assert!(
                alpha[0] > 100 && alpha[0] < 155 && alpha[1] > 100 && alpha[2] > 100,
                "Alpha must blend instead of becoming opaque: {alpha:?}"
            );
            // The final CUDA copy must not rescale the 32-pixel allocation padding
            // into the visible image or shift the video footprint.
            assert!(image.get_pixel(42, 320)[1] < 20);
            assert!(image.get_pixel(46, 320)[1] > 90);
            assert!(image.get_pixel(316, 320)[1] < 20);
            assert!(image.get_pixel(180, 242)[1] < 20);
            assert!(image.get_pixel(180, 246)[1] > 90);
        }
    }
}
