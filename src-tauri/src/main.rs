#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
mod encoder;
mod export;
mod media;
mod model;
mod planner;
mod process;
mod text;
use model::{Clip, Settings, Source};
use serde::Serialize;
use std::{
    path::PathBuf,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc, Mutex,
    },
};
use tauri::{Emitter, Manager, State};

struct Studio {
    busy: AtomicBool,
    stop: AtomicBool,
    text: Mutex<text::TextEngine>,
    data: PathBuf,
}
struct Job(Arc<Studio>);
impl Drop for Job {
    fn drop(&mut self) {
        self.0.busy.store(false, Ordering::SeqCst);
    }
}
fn acquire(studio: Arc<Studio>) -> Result<Job, String> {
    if studio.busy.swap(true, Ordering::SeqCst) {
        return Err("Wait for the current analysis or export to finish.".into());
    }
    studio.stop.store(false, Ordering::SeqCst);
    Ok(Job(studio))
}

#[derive(Serialize)]
struct Boot {
    fonts: Vec<String>,
    tools: Vec<(String, bool)>,
    settings: Option<serde_json::Value>,
}

#[tauri::command]
fn bootstrap(studio: State<Arc<Studio>>) -> Result<Boot, String> {
    Ok(Boot {
        fonts: studio.text.lock().map_err(|e| e.to_string())?.families(),
        tools: media::available(),
        settings: std::fs::read(studio.data.join("settings.json"))
            .ok()
            .and_then(|b| serde_json::from_slice(&b).ok()),
    })
}

#[tauri::command]
async fn add_sources(
    paths: Vec<String>,
    studio: State<'_, Arc<Studio>>,
) -> Result<Vec<Source>, String> {
    let job = acquire(studio.inner().clone())?;
    tauri::async_runtime::spawn_blocking(move || {
        let mut files = Vec::new();
        for path in paths {
            let p = PathBuf::from(path);
            if p.is_dir() {
                let mut children = std::fs::read_dir(p)
                    .map_err(|e| e.to_string())?
                    .filter_map(Result::ok)
                    .map(|p| p.path())
                    .filter(|p| {
                        p.is_file()
                            && p.extension().is_some_and(|e| {
                                ["mp4", "mkv", "mov", "webm", "avi", "m4v"]
                                    .contains(&e.to_string_lossy().to_lowercase().as_str())
                            })
                    })
                    .collect::<Vec<_>>();
                children.sort();
                files.extend(children);
            } else {
                files.push(p);
            }
        }
        files.sort();
        files.dedup();
        files
            .into_iter()
            .map(|p| media::probe(&p.to_string_lossy(), &job.0.stop))
            .collect()
    })
    .await
    .map_err(|e| e.to_string())?
}

#[tauri::command]
async fn analyze_source(
    source: Source,
    settings: Settings,
    studio: State<'_, Arc<Studio>>,
) -> Result<Vec<Clip>, String> {
    settings.validate()?;
    let job = acquire(studio.inner().clone())?;
    tauri::async_runtime::spawn_blocking(move || {
        let source = media::probe(&source.path, &job.0.stop)?;
        let levels = if settings.smart {
            media::levels(&source, &job.0.stop)?
        } else {
            None
        };
        planner::plan(
            source.duration,
            settings.seconds,
            settings.extra,
            levels.as_deref(),
        )
    })
    .await
    .map_err(|e| e.to_string())?
}

#[tauri::command]
async fn preview_frame(source: Source, settings: Settings, time: f64) -> Result<String, String> {
    settings.validate()?;
    if !time.is_finite() {
        return Err("Invalid preview time".into());
    }
    tauri::async_runtime::spawn_blocking(move || {
        media::preview(&source, &settings, time, &AtomicBool::new(false))
    })
    .await
    .map_err(|e| e.to_string())?
}

#[tauri::command]
async fn render_overlay(
    settings: Settings,
    width: u32,
    height: u32,
    n: usize,
    total: usize,
    studio: State<'_, Arc<Studio>>,
) -> Result<text::Overlay, String> {
    settings.validate()?;
    let studio = studio.inner().clone();
    tauri::async_runtime::spawn_blocking(move || {
        let (image, warnings) = studio
            .text
            .lock()
            .map_err(|e| e.to_string())?
            .render(&settings, width, height, n, total)?;
        Ok(text::Overlay {
            image: text::data_image(&image)?,
            warnings,
        })
    })
    .await
    .map_err(|e| e.to_string())?
}

#[tauri::command]
async fn import_font(path: String, studio: State<'_, Arc<Studio>>) -> Result<Vec<String>, String> {
    let studio = studio.inner().clone();
    tauri::async_runtime::spawn_blocking(move || {
        let path = PathBuf::from(path);
        let extension = path
            .extension()
            .unwrap_or_default()
            .to_string_lossy()
            .to_lowercase();
        if !["ttf", "otf", "ttc"].contains(&extension.as_str()) {
            return Err("Choose a TTF, OTF or TTC font.".into());
        }
        let bytes = std::fs::read(&path).map_err(|e| e.to_string())?;
        if bytes.len() > 32 * 1024 * 1024 {
            return Err("Font file exceeds 32 MB.".into());
        }
        let dest =
            studio
                .data
                .join("fonts")
                .join(format!("{}.{}", export::hash(&bytes), extension));
        std::fs::write(&dest, bytes).map_err(|e| e.to_string())?;
        let mut engine = studio.text.lock().map_err(|e| e.to_string())?;
        let count = engine.fonts.db().len();
        engine
            .fonts
            .db_mut()
            .load_font_file(&dest)
            .map_err(|e| e.to_string())?;
        if engine.fonts.db().len() == count {
            return Err("This font has no supported faces.".into());
        }
        Ok(engine.families())
    })
    .await
    .map_err(|e| e.to_string())?
}

#[tauri::command]
async fn export_clips(
    sources: Vec<export::ExportSource>,
    settings: Settings,
    output: String,
    studio: State<'_, Arc<Studio>>,
    app: tauri::AppHandle,
) -> Result<Vec<String>, String> {
    let job = acquire(studio.inner().clone())?;
    tauri::async_runtime::spawn_blocking(move || {
        export::run(
            sources,
            settings,
            &output,
            &job.0.text,
            &job.0.stop,
            |event| {
                let _ = app.emit("studio-progress", event);
            },
        )
    })
    .await
    .map_err(|e| e.to_string())?
}
#[tauri::command]
fn cancel_job(studio: State<Arc<Studio>>) {
    studio.stop.store(true, Ordering::SeqCst);
}
#[tauri::command]
fn open_output(path: String) -> Result<(), String> {
    process::open_folder(&path)
}
#[tauri::command]
fn save_settings(settings: Settings, studio: State<Arc<Studio>>) -> Result<(), String> {
    settings.validate()?;
    export::atomic_json(&studio.data.join("settings.json"), &settings)
}

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| {
            let data = app.path().app_data_dir()?;
            std::fs::create_dir_all(data.join("fonts"))?;
            let text = Mutex::new(text::TextEngine::new(&data.join("fonts")));
            app.manage(Arc::new(Studio {
                busy: AtomicBool::new(false),
                stop: AtomicBool::new(false),
                text,
                data,
            }));
            Ok(())
        })
        .on_window_event(|window, event| {
            if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                let state = window.state::<Arc<Studio>>();
                if state.busy.load(Ordering::SeqCst) {
                    state.stop.store(true, Ordering::SeqCst);
                    api.prevent_close();
                    let _ = window.emit(
                        "studio-progress",
                        export::Progress {
                            phase: "stopping".into(),
                            progress: 0.0,
                            message:
                                "Stopping media work safely. Close the window again after it stops."
                                    .into(),
                            clip: 0,
                            total: 0,
                        },
                    );
                }
            }
        })
        .invoke_handler(tauri::generate_handler![
            bootstrap,
            add_sources,
            analyze_source,
            preview_frame,
            render_overlay,
            import_font,
            export_clips,
            cancel_job,
            open_output,
            save_settings
        ])
        .run(tauri::generate_context!())
        .expect("Could not start ReelCraft Studio");
}
