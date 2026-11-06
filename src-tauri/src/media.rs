use crate::{
    model::{Settings, Source},
    process::{capture, command, tool},
};
use base64::{engine::general_purpose::STANDARD, Engine};
use serde_json::Value;
use std::{path::Path, sync::atomic::AtomicBool};

pub fn probe(path: &str, stop: &AtomicBool) -> Result<Source, String> {
    if !Path::new(path).is_file() {
        return Err(format!("File not found: {path}"));
    }
    let bytes = capture(
        command(&tool("ffprobe")).args([
            "-v",
            "error",
            "-show_streams",
            "-show_format",
            "-of",
            "json",
            path,
        ]),
        stop,
    )?;
    let data: Value = serde_json::from_slice(&bytes).map_err(|e| e.to_string())?;
    let streams = data["streams"].as_array().ok_or("No media streams found")?;
    let video = streams
        .iter()
        .find(|s| s["codec_type"] == "video" && s["disposition"]["attached_pic"] != 1)
        .ok_or("No video stream found")?;
    let read_num = |v: &Value| {
        v.as_str()
            .and_then(|s| s.parse::<f64>().ok())
            .or(v.as_f64())
            .filter(|d| d.is_finite() && *d > 0.0)
    };
    let duration = read_num(&video["duration"])
        .or_else(|| read_num(&data["format"]["duration"]))
        .ok_or("Cannot read video duration")?;
    let width = video["width"].as_u64().unwrap_or(0) as u32;
    let height = video["height"].as_u64().unwrap_or(0) as u32;
    if width < 2 || height < 2 {
        return Err("Invalid video dimensions".into());
    }
    Ok(Source {
        path: std::fs::canonicalize(path)
            .map_err(|e| e.to_string())?
            .to_string_lossy()
            .into(),
        name: Path::new(path)
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .into(),
        duration,
        width,
        height,
        has_audio: streams.iter().any(|s| s["codec_type"] == "audio"),
    })
}

pub fn levels(source: &Source, stop: &AtomicBool) -> Result<Option<Vec<f64>>, String> {
    if !source.has_audio {
        return Ok(None);
    }
    let bytes=capture(command(&tool("ffmpeg")).args(["-hide_banner","-v","error","-i",&source.path,"-map","0:a:0","-vn","-af","aresample=8000,aformat=channel_layouts=mono,asetnsamples=n=400:p=1,astats=metadata=1:reset=1,ametadata=print:key=lavfi.astats.Overall.RMS_level:file=-","-f","null","-"]),stop)?;
    let values = String::from_utf8_lossy(&bytes)
        .lines()
        .filter_map(|l| l.strip_prefix("lavfi.astats.Overall.RMS_level="))
        .filter_map(|s| s.parse::<f64>().ok())
        .map(|n| if n.is_finite() { n.max(-100.0) } else { -100.0 })
        .collect::<Vec<_>>();
    if values.is_empty() {
        return Err("Audio analysis produced no level measurements.".into());
    }
    Ok(Some(values))
}

pub fn filter(width: u32, height: u32, fill: &str) -> String {
    if fill == "blur" && width >= 32 && height >= 32 {
        // Blur one-sixteenth of the output pixels, like the original splitter.
        let bw = (width / 8 * 2).max(2);
        let bh = (height / 8 * 2).max(2);
        let radius = 14_u32.min(bw.min(bh) / 2 - 1).max(1);
        let chroma = 7_u32.min(bw.min(bh) / 4 - 1).max(1);
        format!("[0:v:0]split=2[back][front];[back]scale={bw}:{bh}:force_original_aspect_ratio=increase,crop={bw}:{bh},boxblur=luma_radius={radius}:luma_power=2:chroma_radius={chroma}:chroma_power=1,scale={width}:{height},eq=brightness=-0.06:saturation=0.85[bg];[front]scale={width}:{height}:force_original_aspect_ratio=decrease[fg];[bg][fg]overlay=(W-w)/2:(H-h)/2,setsar=1[frame]")
    } else if fill == "crop" {
        format!("[0:v:0]scale={width}:{height}:force_original_aspect_ratio=increase,crop={width}:{height},setsar=1[frame]")
    } else {
        format!("[0:v:0]scale={width}:{height}:force_original_aspect_ratio=decrease,pad={width}:{height}:(ow-iw)/2:(oh-ih)/2:color=black,setsar=1[frame]")
    }
}

pub fn frame_filter(source: &Source, width: u32, height: u32, fill: &str) -> String {
    if (source.width as f64 / source.height as f64 - width as f64 / height as f64).abs() < 0.01 {
        // Avoid split/blur/overlay entirely when footage already fits the frame.
        return filter(width, height, "crop");
    }
    filter(width, height, fill)
}

pub fn preview(
    source: &Source,
    settings: &Settings,
    time: f64,
    stop: &AtomicBool,
) -> Result<String, String> {
    let (mut w, mut h) = crate::model::dimensions(source, &settings.ratio);
    let scale = (960.0 / w as f64).min(640.0 / h as f64).min(1.0);
    w = (w as f64 * scale) as u32 / 2 * 2;
    h = (h as f64 * scale) as u32 / 2 * 2;
    let graph = frame_filter(
        source,
        w.max(2),
        h.max(2),
        if settings.ratio == "Original" {
            "black"
        } else {
            &settings.fill
        },
    );
    let bytes = capture(
        command(&tool("ffmpeg")).args([
            "-v",
            "error",
            "-ss",
            &time
                .clamp(0.0, (source.duration - 0.05).max(0.0))
                .to_string(),
            "-i",
            &source.path,
            "-filter_complex",
            &graph,
            "-map",
            "[frame]",
            "-frames:v",
            "1",
            "-f",
            "image2pipe",
            "-c:v",
            "png",
            "pipe:1",
        ]),
        stop,
    )?;
    if bytes.is_empty() {
        return Err("No preview frame returned.".into());
    }
    Ok(format!("data:image/png;base64,{}", STANDARD.encode(bytes)))
}

pub fn available() -> Vec<(String, bool)> {
    ["ffmpeg", "ffprobe"]
        .iter()
        .map(|name| {
            (
                name.to_string(),
                command(&tool(name))
                    .arg("-version")
                    .output()
                    .map(|p| p.status.success())
                    .unwrap_or(false),
            )
        })
        .collect()
}

#[cfg(test)]
mod optimization_tests {
    use super::*;
    #[test]
    fn blur_is_small_and_matching_ratios_skip_it() {
        let source = Source {
            path: String::new(),
            name: String::new(),
            width: 1920,
            height: 1080,
            duration: 10.0,
            has_audio: false,
        };
        assert!(!frame_filter(&source, 1920, 1080, "blur").contains("boxblur"));
        let graph = frame_filter(&source, 1080, 1920, "blur");
        assert!(graph.contains("scale=270:480"));
        assert!(graph.contains("boxblur"));
    }
}
