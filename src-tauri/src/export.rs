use crate::{
    encoder::{self, Encoder},
    media,
    model::{dimensions, Clip, Settings, Source},
    process::{command, render_progress, tool},
    text::TextEngine,
};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{
    collections::BTreeMap,
    fs,
    path::{Path, PathBuf},
    sync::{
        atomic::{AtomicBool, Ordering},
        Mutex,
    },
    time::UNIX_EPOCH,
};

#[derive(Clone, Serialize, Deserialize)]
pub struct ExportSource {
    pub source: Source,
    pub clips: Vec<Clip>,
}
#[derive(Clone, Serialize)]
pub struct Progress {
    pub phase: String,
    pub progress: f64,
    pub message: String,
    pub clip: usize,
    pub total: usize,
}
#[derive(Serialize, Deserialize, Default)]
struct Record {
    fingerprint: String,
    duration: f64,
    #[serde(default)]
    encoder: String,
}

pub fn hash(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}
pub fn source_key(source: &Source) -> String {
    hash(source.path.as_bytes())[..10].into()
}
fn safe_name(name: &str) -> String {
    name.chars()
        .map(|c| {
            if "<>:\"/\\|?*".contains(c) || c.is_control() {
                '_'
            } else {
                c
            }
        })
        .take(90)
        .collect::<String>()
        .trim_end_matches([' ', '.'])
        .into()
}

pub fn atomic_json(path: &Path, value: &impl Serialize) -> Result<(), String> {
    let temp = path.with_extension("json.partial");
    fs::write(
        &temp,
        serde_json::to_vec_pretty(value).map_err(|e| e.to_string())?,
    )
    .map_err(|e| e.to_string())?;
    fs::rename(&temp, path).map_err(|e| e.to_string())
}

fn valid_output(path: &Path, expected: f64, stop: &AtomicBool) -> bool {
    media::probe(&path.to_string_lossy(), stop)
        .map(|s| (s.duration - expected).abs() < 0.25)
        .unwrap_or(false)
}

fn crop_overlay(image: &image::RgbaImage) -> Option<(image::RgbaImage, u32, u32)> {
    let mut bounds = (image.width(), image.height(), 0, 0);
    for (x, y, pixel) in image.enumerate_pixels() {
        if pixel[3] > 0 {
            bounds = (
                bounds.0.min(x),
                bounds.1.min(y),
                bounds.2.max(x),
                bounds.3.max(y),
            );
        }
    }
    if bounds.0 > bounds.2 || bounds.1 > bounds.3 {
        return None;
    }
    Some((
        image::imageops::crop_imm(
            image,
            bounds.0,
            bounds.1,
            bounds.2 - bounds.0 + 1,
            bounds.3 - bounds.1 + 1,
        )
        .to_image(),
        bounds.0,
        bounds.1,
    ))
}

pub fn run(
    sources: Vec<ExportSource>,
    settings: Settings,
    output: &str,
    text: &Mutex<TextEngine>,
    stop: &AtomicBool,
    notify: impl Fn(Progress),
) -> Result<Vec<String>, String> {
    settings.validate()?;
    if sources.is_empty() {
        return Err("Add a video before exporting.".into());
    }
    let root = PathBuf::from(output);
    if !root.is_dir() {
        return Err("Choose an existing output folder.".into());
    }
    let total = sources
        .iter()
        .map(|s| s.clips.iter().filter(|c| c.enabled).count())
        .sum::<usize>();
    if total == 0 {
        return Err("Select at least one part to export.".into());
    }
    let total_seconds = sources
        .iter()
        .flat_map(|entry| &entry.clips)
        .filter(|c| c.enabled)
        .map(|c| c.end - c.start)
        .sum::<f64>()
        .max(0.1);
    let mut finished_seconds = 0.0;
    let mut selected_encoder: Option<Encoder> = None;
    let mut completed = 0;
    let mut files = Vec::new();
    let settings_hash = hash(&serde_json::to_vec(&settings).map_err(|e| e.to_string())?);
    for entry in sources {
        if stop.load(Ordering::Relaxed) {
            return Err("Stopped. Completed parts are kept for resume.".into());
        }
        let source = media::probe(&entry.source.path, stop)?;
        let stat = fs::metadata(&source.path).map_err(|e| e.to_string())?;
        let identity = format!(
            "{}:{}:{}",
            source.path,
            stat.len(),
            stat.modified()
                .map_err(|e| e.to_string())?
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_nanos()
        );
        let stem = Path::new(&source.name)
            .file_stem()
            .unwrap_or_default()
            .to_string_lossy();
        let folder = root.join(format!("{}_{}", safe_name(&stem), source_key(&source)));
        fs::create_dir_all(&folder).map_err(|e| e.to_string())?;
        let state_path = folder.join(".long-to-shot.json");
        let mut records: BTreeMap<String, Record> = fs::read(&state_path)
            .ok()
            .and_then(|b| serde_json::from_slice(&b).ok())
            .unwrap_or_default();
        let (w, h) = dimensions(&source, &settings.ratio);
        let mut last = 0.0;
        for clip in &entry.clips {
            if !clip.start.is_finite()
                || !clip.end.is_finite()
                || clip.start < last - 0.001
                || clip.end <= clip.start
                || clip.end > source.duration + 0.05
            {
                return Err(
                    "Clip boundaries overlap or exceed the source video. Analyze it again.".into(),
                );
            }
            last = clip.end;
        }
        for clip in entry.clips.iter().filter(|c| c.enabled) {
            if stop.load(Ordering::Relaxed) {
                return Err("Stopped. Completed parts are kept for resume.".into());
            }
            let (overlay, warnings) = text.lock().map_err(|e| e.to_string())?.render(
                &settings,
                w,
                h,
                clip.n,
                entry.clips.len(),
            )?;
            if warnings.iter().any(|w| w.contains("no glyph")) {
                return Err(warnings.join(" "));
            }
            let legacy_fp = hash(
                format!(
                    "studio-v1:{identity}:{settings_hash}:{}:{}:{}:{}",
                    clip.n,
                    clip.start,
                    clip.end,
                    hash(overlay.as_raw())
                )
                .as_bytes(),
            );
            let fp =
                hash(format!("render-v2-auto-nvenc-quarter-blur-cropped:{legacy_fp}").as_bytes());
            let name = format!("{}_part_{:02}_{}.mp4", safe_name(&stem), clip.n, &fp[..8]);
            let out = folder.join(&name);
            let duration = clip.end - clip.start;
            let legacy_name = format!(
                "{}_part_{:02}_{}.mp4",
                safe_name(&stem),
                clip.n,
                &legacy_fp[..8]
            );
            let reusable = [(&name, &fp), (&legacy_name, &legacy_fp)]
                .into_iter()
                .find_map(|(name, identity)| {
                    let file = folder.join(name);
                    if records
                        .get(name)
                        .is_some_and(|r| &r.fingerprint == identity)
                        && valid_output(&file, duration, stop)
                    {
                        Some(file)
                    } else {
                        None
                    }
                });
            if let Some(file) = reusable {
                completed += 1;
                finished_seconds += duration;
                notify(Progress {
                    phase: "export".into(),
                    progress: finished_seconds / total_seconds,
                    message: format!("Reused {} - part {}", source.name, clip.n),
                    clip: completed,
                    total,
                });
                files.push(file.to_string_lossy().into());
                continue;
            }
            if out.exists() {
                return Err(format!(
                    "An unverified output already exists: {}. Move it before retrying.",
                    out.display()
                ));
            }
            let mut codec = match selected_encoder {
                Some(codec) => codec,
                None => {
                    let codec = encoder::select(stop)?;
                    selected_encoder = Some(codec);
                    codec
                }
            };
            notify(Progress {
                phase: "export".into(),
                progress: finished_seconds / total_seconds,
                message: format!(
                    "{} - rendering {} - part {}",
                    codec.name(),
                    source.name,
                    clip.n
                ),
                clip: completed + 1,
                total,
            });
            let overlay_path = folder.join(format!(".overlay-{}.png", &fp[..12]));
            let cropped = crop_overlay(&overlay);
            let mut graph = media::frame_filter(
                &source,
                w,
                h,
                if settings.ratio == "Original" {
                    "black"
                } else {
                    &settings.fill
                },
            );
            if let Some((pixels, x, y)) = &cropped {
                pixels.save(&overlay_path).map_err(|e| e.to_string())?;
                graph.push_str(&format!(
                    ";[frame][1:v]overlay={x}:{y}:eof_action=repeat,format=yuv420p[out]"
                ));
            } else {
                graph.push_str(";[frame]format=yuv420p[out]");
            }
            let temp = folder.join(format!(".{}.partial.mp4", &fp[..16]));
            let high_water = std::cell::Cell::new(0_f64);
            let render = |codec: Encoder| {
                let mut cmd = command(&tool("ffmpeg"));
                cmd.args([
                    "-y",
                    "-v",
                    "error",
                    "-ss",
                    &clip.start.to_string(),
                    "-t",
                    &duration.to_string(),
                    "-i",
                    &source.path,
                ]);
                if cropped.is_some() {
                    cmd.arg("-i").arg(&overlay_path);
                }
                cmd.args(["-filter_complex", &graph, "-map", "[out]", "-map", "0:a:0?"])
                    .args(codec.args())
                    .args([
                        "-c:a",
                        "aac",
                        "-b:a",
                        "192k",
                        "-movflags",
                        "+faststart",
                        "-progress",
                        "pipe:1",
                        "-stats_period",
                        "0.5",
                        "-nostats",
                    ])
                    .arg(&temp);
                render_progress(&mut cmd, stop, |seconds, speed| {
                    let fraction = (seconds / duration).clamp(0.0, 0.99);
                    high_water.set(high_water.get().max(fraction));
                    notify(Progress {
                        phase: "rendering".into(),
                        progress: (finished_seconds + duration * high_water.get()) / total_seconds,
                        message: format!(
                            "{} - part {}/{} - {:.0}%{}",
                            codec.name(),
                            completed + 1,
                            total,
                            high_water.get() * 100.0,
                            if speed.is_empty() {
                                String::new()
                            } else {
                                format!(" - {speed}")
                            }
                        ),
                        clip: completed + 1,
                        total,
                    });
                })
            };
            let mut result = render(codec);
            if let Some(fallback) =
                encoder::fallback(codec, result.is_err(), stop.load(Ordering::Relaxed))
            {
                notify(Progress {
                    phase: "encoder".into(),
                    progress: (finished_seconds + duration * high_water.get()) / total_seconds,
                    message: format!(
                        "NVIDIA encoding failed; retrying this part on CPU. {}",
                        result.as_ref().unwrap_err()
                    ),
                    clip: completed + 1,
                    total,
                });
                codec = fallback;
                selected_encoder = Some(codec);
                result = render(codec);
            }
            let _ = fs::remove_file(&overlay_path);
            if let Err(e) = result {
                let _ = fs::remove_file(&temp);
                return Err(e);
            }
            if !valid_output(&temp, duration, stop) {
                let _ = fs::remove_file(&temp);
                return Err("Output validation failed; completed parts are safe.".into());
            }
            fs::rename(&temp, &out).map_err(|e| e.to_string())?;
            records.insert(
                name,
                Record {
                    fingerprint: fp,
                    duration,
                    encoder: codec.name().into(),
                },
            );
            atomic_json(&state_path, &records)?;
            files.push(out.to_string_lossy().into());
            completed += 1;
            finished_seconds += duration;
            notify(Progress {
                phase: "export".into(),
                progress: finished_seconds / total_seconds,
                message: format!("Saved part {} of {}", completed, total),
                clip: completed,
                total,
            });
        }
    }
    Ok(files)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::process::capture;
    #[test]
    fn transparent_overlays_are_skipped_and_visible_pixels_keep_their_position() {
        let mut pixels = image::RgbaImage::new(100, 100);
        assert!(crop_overlay(&pixels).is_none());
        pixels.put_pixel(40, 60, image::Rgba([255, 255, 255, 200]));
        pixels.put_pixel(50, 70, image::Rgba([255, 255, 255, 255]));
        let (cropped, x, y) = crop_overlay(&pixels).unwrap();
        assert_eq!((cropped.width(), cropped.height(), x, y), (11, 11, 40, 60));
    }
    #[test]
    fn same_names_have_different_folders() {
        let s = |path: &str| Source {
            path: path.into(),
            name: "movie.mp4".into(),
            duration: 60.0,
            width: 640,
            height: 360,
            has_audio: false,
        };
        assert_ne!(
            source_key(&s("C:/a/movie.mp4")),
            source_key(&s("C:/b/movie.mp4"))
        );
    }

    #[test]
    fn real_video_unicode_export_and_resume() {
        let dir = tempfile::tempdir().unwrap();
        let input = dir.path().join("မြန်မာ sample.mp4");
        let stop = AtomicBool::new(false);
        capture(
            command(&tool("ffmpeg")).args([
                "-v",
                "error",
                "-f",
                "lavfi",
                "-i",
                "testsrc2=s=320x180:r=30:d=12",
                "-f",
                "lavfi",
                "-i",
                "sine=frequency=440:duration=12",
                "-c:v",
                "libx264",
                "-preset",
                "ultrafast",
                "-c:a",
                "aac",
                "-shortest",
                &input.to_string_lossy(),
            ]),
            &stop,
        )
        .unwrap();
        let source = media::probe(&input.to_string_lossy(), &stop).unwrap();
        assert!(source.has_audio);
        let levels = media::levels(&source, &stop).unwrap().unwrap();
        assert!(levels.len() >= 230);
        let engine = TextEngine::new(dir.path());
        let font = engine
            .families()
            .into_iter()
            .find(|f| f == "Myanmar Text")
            .expect("Myanmar Text is installed on this Windows test machine");
        let layer = crate::model::TextLayer {
            enabled: true,
            text: "မြန်မာစာ John".into(),
            font,
            size: 24.0,
            x: 0.5,
            y: 0.65,
            color: "#ffffff".into(),
            opacity: 1.0,
            boxed: true,
        };
        let mut settings = Settings {
            seconds: 5.0,
            smart: false,
            extra: 0.0,
            ratio: "Original".into(),
            fill: "black".into(),
            text: layer.clone(),
            label: layer,
        };
        settings.label.enabled = false;
        let frame = media::preview(&source, &settings, 1.0, &stop).unwrap();
        assert!(frame.starts_with("data:image/png;base64,"));
        let clips = crate::planner::plan(source.duration, 5.0, 0.0, None).unwrap();
        let entries = vec![ExportSource { source, clips }];
        let text = Mutex::new(engine);
        let events = std::cell::RefCell::new(Vec::new());
        let files = run(
            entries.clone(),
            settings.clone(),
            &dir.path().to_string_lossy(),
            &text,
            &stop,
            |p| {
                events.borrow_mut().push(p);
            },
        )
        .unwrap();
        assert_eq!(files.len(), 3);
        let events = events.into_inner();
        assert!(events.iter().any(|p| p.phase == "rendering"));
        assert!(events.windows(2).all(|w| w[1].progress >= w[0].progress));
        assert_eq!(events.last().unwrap().progress, 1.0);
        println!(
            "Export encoder: {}",
            events
                .iter()
                .find(|p| p.message.contains("rendering"))
                .unwrap()
                .message
        );
        let total = files
            .iter()
            .map(|p| media::probe(p, &stop).unwrap().duration)
            .sum::<f64>();
        assert!((total - 12.0).abs() < 0.15);
        let mtime = fs::metadata(&files[0]).unwrap().modified().unwrap();
        let again = run(
            entries.clone(),
            settings.clone(),
            &dir.path().to_string_lossy(),
            &text,
            &stop,
            |_| {},
        )
        .unwrap();
        assert_eq!(files, again);
        assert_eq!(mtime, fs::metadata(&files[0]).unwrap().modified().unwrap());

        // Simulate a completed part written by the earlier CPU-only release.
        // Keep its original fingerprint/record format (without encoder metadata).
        let source = &entries[0].source;
        let first = &entries[0].clips[0];
        let stat = fs::metadata(&source.path).unwrap();
        let identity = format!(
            "{}:{}:{}",
            source.path,
            stat.len(),
            stat.modified()
                .unwrap()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        );
        let (w, h) = dimensions(source, &settings.ratio);
        let (pixels, _) = text
            .lock()
            .unwrap()
            .render(&settings, w, h, first.n, entries[0].clips.len())
            .unwrap();
        let settings_key = hash(&serde_json::to_vec(&settings).unwrap());
        let old_key = hash(
            format!(
                "studio-v1:{identity}:{settings_key}:{}:{}:{}:{}",
                first.n,
                first.start,
                first.end,
                hash(pixels.as_raw())
            )
            .as_bytes(),
        );
        let folder = Path::new(&files[0]).parent().unwrap();
        let stem = Path::new(&source.name)
            .file_stem()
            .unwrap()
            .to_string_lossy();
        let old_name = format!(
            "{}_part_{:02}_{}.mp4",
            safe_name(&stem),
            first.n,
            &old_key[..8]
        );
        let old_path = folder.join(&old_name);
        fs::copy(&files[0], &old_path).unwrap();
        let state_file = folder.join(".long-to-shot.json");
        let mut state: serde_json::Value =
            serde_json::from_slice(&fs::read(&state_file).unwrap()).unwrap();
        state
            .as_object_mut()
            .unwrap()
            .remove(Path::new(&files[0]).file_name().unwrap().to_str().unwrap());
        state[&old_name] =
            serde_json::json!({"fingerprint":old_key,"duration":first.end-first.start});
        atomic_json(&state_file, &state).unwrap();
        let events = std::cell::RefCell::new(Vec::new());
        let resumed = run(
            entries,
            settings,
            &dir.path().to_string_lossy(),
            &text,
            &stop,
            |p| {
                events.borrow_mut().push(p);
            },
        )
        .unwrap();
        assert_eq!(resumed[0], old_path.to_string_lossy());
        assert!(events
            .borrow()
            .iter()
            .all(|p| p.message.starts_with("Reused")));
    }

    #[test]
    fn portrait_blur_export_without_audio() {
        let dir = tempfile::tempdir().unwrap();
        let input = dir.path().join("silent.mp4");
        let stop = AtomicBool::new(false);
        capture(
            command(&tool("ffmpeg")).args([
                "-v",
                "error",
                "-f",
                "lavfi",
                "-i",
                "testsrc2=s=320x180:r=30:d=2",
                "-c:v",
                "libx264",
                "-preset",
                "ultrafast",
                &input.to_string_lossy(),
            ]),
            &stop,
        )
        .unwrap();
        let source = media::probe(&input.to_string_lossy(), &stop).unwrap();
        assert!(!source.has_audio);
        assert!(media::levels(&source, &stop).unwrap().is_none());
        let layer = crate::model::TextLayer {
            enabled: false,
            text: String::new(),
            font: String::new(),
            size: 24.0,
            x: 0.5,
            y: 0.5,
            color: "#ffffff".into(),
            opacity: 1.0,
            boxed: false,
        };
        let settings = Settings {
            seconds: 5.0,
            smart: false,
            extra: 0.0,
            ratio: "9:16".into(),
            fill: "blur".into(),
            text: layer.clone(),
            label: layer,
        };
        let clips = crate::planner::plan(source.duration, 5.0, 0.0, None).unwrap();
        let text = Mutex::new(TextEngine::new(dir.path()));
        let files = run(
            vec![ExportSource { source, clips }],
            settings,
            &dir.path().to_string_lossy(),
            &text,
            &stop,
            |_| {},
        )
        .unwrap();
        let exported = media::probe(&files[0], &stop).unwrap();
        assert_eq!((exported.width, exported.height), (1080, 1920));
        assert!(!exported.has_audio);
    }
}
