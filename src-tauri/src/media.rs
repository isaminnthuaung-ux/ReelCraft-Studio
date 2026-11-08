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
    } else if fill == "wallpaper" {
        // The source video supplies the output timestamps, not the still image.
        // Repeat the wallpaper's final frame without looping an infinite input.
        format!("[0:v:0]split=2[front][clock];[clock]scale=2:2,scale={width}:{height},setsar=1[base];[1:v:0]scale={width}:{height}:force_original_aspect_ratio=increase,crop={width}:{height},setsar=1[wall];[base][wall]overlay=eof_action=repeat[bg];[front]scale={width}:{height}:force_original_aspect_ratio=decrease,setsar=1[fg];[bg][fg]overlay=(W-w)/2:(H-h)/2:eof_action=repeat,setsar=1[frame]")
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

pub fn scaled_frame_filter(
    source: &Source,
    width: u32,
    height: u32,
    fill: &str,
    scale: f64,
) -> String {
    if (scale - 1.0).abs() < 0.0001 {
        return frame_filter(source, width, height, fill);
    }
    let sx = width as f64 / source.width as f64;
    let sy = height as f64 / source.height as f64;
    let fit = if fill == "crop" {
        sx.max(sy)
    } else {
        sx.min(sy)
    };
    let sw = ((source.width as f64 * fit * scale / 2.0).round() as u32 * 2).max(2);
    let sh = ((source.height as f64 * fit * scale / 2.0).round() as u32 * 2).max(2);
    let background = if fill == "wallpaper" {
        format!("[back]scale=2:2,scale={width}:{height},setsar=1[base];[1:v:0]scale={width}:{height}:force_original_aspect_ratio=increase,crop={width}:{height},setsar=1[wall];[base][wall]overlay=eof_action=repeat[bg]")
    } else if fill == "blur" && width >= 32 && height >= 32 {
        let bw = (width / 8 * 2).max(2);
        let bh = (height / 8 * 2).max(2);
        let radius = 14_u32.min(bw.min(bh) / 2 - 1).max(1);
        let chroma = 7_u32.min(bw.min(bh) / 4 - 1).max(1);
        format!("[back]scale={bw}:{bh}:force_original_aspect_ratio=increase,crop={bw}:{bh},boxblur=luma_radius={radius}:luma_power=2:chroma_radius={chroma}:chroma_power=1,scale={width}:{height},eq=brightness=-0.06:saturation=0.85,setsar=1[bg]")
    } else {
        format!("[back]scale=2:2,scale={width}:{height},drawbox=color=black:t=fill,setsar=1[bg]")
    };
    format!("[0:v:0]split=2[front][back];{background};[front]scale={sw}:{sh},setsar=1[fg];[bg][fg]overlay=(W-w)/2:(H-h)/2:eof_action=repeat,setsar=1[frame]")
}

pub fn framing_filter(source: &Source, width: u32, height: u32, settings: &Settings) -> String {
    use crate::model::WallpaperMode;
    if settings.fill != "wallpaper" || settings.wallpaper_layout.mode != WallpaperMode::Split {
        return scaled_frame_filter(source, width, height, &settings.fill, settings.scale);
    }
    let fit = (width as f64 / source.width as f64).min(height as f64 / source.height as f64);
    let sw = ((source.width as f64 * fit * settings.scale / 2.0).round() as u32 * 2).max(2);
    let sh = ((source.height as f64 * fit * settings.scale / 2.0).round() as u32 * 2).max(2);
    let video_y = (((height as f64 - sh as f64) * settings.wallpaper_layout.video_y / 2.0).round()
        as i64)
        * 2;
    let top_height = video_y.clamp(0, height as i64) as u32;
    let bottom_y = (video_y + sh as i64).clamp(0, height as i64) as u32;
    let bottom_height = height - bottom_y;
    let mut graph = format!("[0:v:0]split=2[front][clock];[clock]scale=2:2,scale={width}:{height},drawbox=color=black:t=fill,setsar=1[base0]");
    let mut base = "base0".to_string();
    for (index, region_height, y, layer) in [
        (1, top_height, 0, &settings.wallpaper_layout.top),
        (
            2,
            bottom_height,
            bottom_y,
            &settings.wallpaper_layout.bottom,
        ),
    ] {
        if region_height < 2 {
            continue;
        }
        let zoom = layer.zoom;
        let x = layer.x;
        let pan_y = layer.y;
        graph.push_str(&format!(";[{index}:v:0]scale={width}:{region_height}:force_original_aspect_ratio=increase:force_divisible_by=2,scale=w='trunc(iw*{zoom}/2)*2':h='trunc(ih*{zoom}/2)*2',crop={width}:{region_height}:x='(iw-ow)*{x}':y='(ih-oh)*{pan_y}',setsar=1[wall{index}];[{base}][wall{index}]overlay=0:{y}:eof_action=repeat[base{index}]"));
        base = format!("base{index}");
    }
    graph.push_str(&format!(";[front]scale={sw}:{sh},setsar=1[fg];[{base}][fg]overlay=(W-w)/2:{video_y}:eof_action=repeat,setsar=1[frame]"));
    graph
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
    let graph = framing_filter(source, w.max(2), h.max(2), settings);
    let mut cmd = command(&tool("ffmpeg"));
    cmd.args([
        "-v",
        "error",
        "-ss",
        &time
            .clamp(0.0, (source.duration - 0.05).max(0.0))
            .to_string(),
        "-i",
        &source.path,
    ]);
    for path in settings.wallpaper_paths() {
        cmd.arg("-i").arg(path);
    }
    let bytes = capture(
        cmd.args([
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

pub fn import_wallpaper(path: &str, data: &Path, stop: &AtomicBool) -> Result<String, String> {
    let source = std::fs::canonicalize(path).map_err(|e| e.to_string())?;
    let extension = source
        .extension()
        .unwrap_or_default()
        .to_string_lossy()
        .to_lowercase();
    if !["png", "jpg", "jpeg", "webp", "bmp"].contains(&extension.as_str()) {
        return Err("Choose a PNG, JPG, WebP or BMP wallpaper.".into());
    }
    if std::fs::metadata(&source).map_err(|e| e.to_string())?.len() > 32 * 1024 * 1024 {
        return Err("Wallpaper image exceeds 32 MB.".into());
    }
    let bytes = capture(
        command(&tool("ffmpeg")).args([
            "-v",
            "error",
            "-i",
            &source.to_string_lossy(),
            "-frames:v",
            "1",
            "-vf",
            "scale=w='min(4096,iw)':h='min(4096,ih)':force_original_aspect_ratio=decrease",
            "-f",
            "image2pipe",
            "-c:v",
            "png",
            "pipe:1",
        ]),
        stop,
    )?;
    image::load_from_memory(&bytes).map_err(|e| format!("Cannot read wallpaper: {e}"))?;
    let folder = data.join("wallpapers");
    std::fs::create_dir_all(&folder).map_err(|e| e.to_string())?;
    let dest = folder.join(format!("{}.png", crate::export::hash(&bytes)));
    std::fs::write(&dest, bytes).map_err(|e| e.to_string())?;
    Ok(dest.to_string_lossy().into())
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
    fn split_wallpaper_preview_and_export_keep_regions_separate() {
        use crate::model::{
            Clip, EncodingProfile, TextLayer, WallpaperLayer, WallpaperLayout, WallpaperMode,
        };
        use std::sync::Mutex;
        let dir = tempfile::tempdir().unwrap();
        let stop = AtomicBool::new(false);
        let input = dir.path().join("green.mp4");
        capture(
            command(&tool("ffmpeg")).args([
                "-v",
                "error",
                "-f",
                "lavfi",
                "-i",
                "color=c=green:s=320x180:r=60:d=1",
                "-c:v",
                "libx264",
                "-preset",
                "ultrafast",
                &input.to_string_lossy(),
            ]),
            &stop,
        )
        .unwrap();
        let top = dir.path().join("red.png");
        let bottom = dir.path().join("blue.png");
        image::RgbaImage::from_pixel(500, 300, image::Rgba([255, 0, 0, 255]))
            .save(&top)
            .unwrap();
        image::RgbaImage::from_pixel(300, 500, image::Rgba([0, 0, 255, 255]))
            .save(&bottom)
            .unwrap();
        let source = probe(&input.to_string_lossy(), &stop).unwrap();
        let layer = TextLayer {
            enabled: true,
            text: "Split wallpaper".into(),
            font: "Segoe UI".into(),
            size: 24.0,
            x: 0.5,
            y: 0.1,
            color: "#ffffff".into(),
            opacity: 1.0,
            boxed: true,
        };
        let settings = Settings {
            seconds: 5.0,
            smart: false,
            extra: 0.0,
            ratio: "9:16".into(),
            fill: "wallpaper".into(),
            scale: 0.75,
            wallpaper: top.to_string_lossy().into(),
            encoding_profile: EncodingProfile::Fastest,
            wallpaper_layout: WallpaperLayout {
                mode: WallpaperMode::Split,
                video_y: 0.4,
                top: WallpaperLayer {
                    path: top.to_string_lossy().into(),
                    zoom: 1.8,
                    x: 0.2,
                    y: 0.7,
                },
                bottom: WallpaperLayer {
                    path: bottom.to_string_lossy().into(),
                    zoom: 1.5,
                    x: 0.8,
                    y: 0.3,
                },
            },
            text: layer.clone(),
            label: TextLayer {
                enabled: false,
                ..layer
            },
        };
        settings.validate().unwrap();
        let check_pixels = |bytes: &[u8]| {
            let frame = image::load_from_memory(bytes).unwrap().to_rgb8();
            let top = frame.get_pixel(5, 5);
            assert!(top[0] > 200 && top[2] < 40);
            let bottom = frame.get_pixel(5, frame.height() - 6);
            assert!(bottom[2] > 200 && bottom[0] < 40);
            let middle = frame.get_pixel(frame.width() / 2, frame.height() / 2);
            assert!(middle[1] > 70 && middle[0] < 40);
        };
        let frame = preview(&source, &settings, 0.0, &stop).unwrap();
        check_pixels(&STANDARD.decode(frame.split_once(',').unwrap().1).unwrap());
        // A patterned top image proves panning changes the chosen crop only.
        let pattern = dir.path().join("pattern.png");
        image::RgbaImage::from_fn(500, 300, |x, _| {
            if x < 250 {
                image::Rgba([255, 0, 0, 255])
            } else {
                image::Rgba([255, 0, 255, 255])
            }
        })
        .save(&pattern)
        .unwrap();
        let mut panned = settings.clone();
        panned.wallpaper_layout.top.path = pattern.to_string_lossy().into();
        let mut samples = Vec::new();
        for x in [0.0, 1.0] {
            panned.wallpaper_layout.top.x = x;
            let frame = preview(&source, &panned, 0.0, &stop).unwrap();
            let pixels = image::load_from_memory(
                &STANDARD.decode(frame.split_once(',').unwrap().1).unwrap(),
            )
            .unwrap()
            .to_rgb8();
            samples.push(*pixels.get_pixel(pixels.width() / 2, 50));
            let bottom = pixels.get_pixel(5, pixels.height() - 6);
            assert!(bottom[2] > 200 && bottom[0] < 40);
        }
        assert!(
            samples[0][2] < 40 && samples[1][2] > 200,
            "Panning must select different image content"
        );
        // Regions of zero height and video larger than the frame must still render.
        for y in [0.0, 1.0] {
            let mut edge = settings.clone();
            edge.wallpaper_layout.video_y = y;
            assert!(preview(&source, &edge, 0.0, &stop).is_ok());
        }
        let mut no_gap = settings.clone();
        no_gap.ratio = "Original".into();
        no_gap.scale = 2.0;
        assert!(preview(&source, &no_gap, 0.0, &stop).is_ok());
        let text = Mutex::new(crate::text::TextEngine::new(dir.path()));
        let files = crate::export::run(
            vec![crate::export::ExportSource {
                source,
                clips: vec![Clip {
                    n: 1,
                    start: 0.0,
                    end: 1.0,
                    cut: "end".into(),
                    enabled: true,
                }],
            }],
            settings,
            &dir.path().to_string_lossy(),
            &text,
            &stop,
            |_| {},
        )
        .unwrap();
        let exported = probe(&files[0], &stop).unwrap();
        assert_eq!((exported.width, exported.height), (1080, 1920));
        assert!((exported.duration - 1.0).abs() < 0.1);
        let image = capture(
            command(&tool("ffmpeg")).args([
                "-v",
                "error",
                "-i",
                &files[0],
                "-frames:v",
                "1",
                "-vf",
                "scale=360:640",
                "-f",
                "image2pipe",
                "-c:v",
                "png",
                "pipe:1",
            ]),
            &stop,
        )
        .unwrap();
        check_pixels(&image);
        let fps = capture(
            command(&tool("ffprobe")).args([
                "-v",
                "error",
                "-select_streams",
                "v:0",
                "-show_entries",
                "stream=r_frame_rate",
                "-of",
                "csv=p=0",
                &files[0],
            ]),
            &stop,
        )
        .unwrap();
        assert_eq!(String::from_utf8_lossy(&fps).trim(), "60/1");
    }

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
