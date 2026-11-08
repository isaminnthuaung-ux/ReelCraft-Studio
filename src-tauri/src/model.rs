use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Default, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum EncodingProfile {
    #[default]
    Fastest,
    Fast,
    Balanced,
    Quality,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Source {
    pub path: String,
    pub name: String,
    pub duration: f64,
    pub width: u32,
    pub height: u32,
    pub has_audio: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Clip {
    pub n: usize,
    pub start: f64,
    pub end: f64,
    pub cut: String,
    #[serde(default = "yes")]
    pub enabled: bool,
}
fn yes() -> bool {
    true
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TextLayer {
    pub enabled: bool,
    pub text: String,
    pub font: String,
    pub size: f32,
    pub x: f32,
    pub y: f32,
    pub color: String,
    pub opacity: f32,
    pub boxed: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct WallpaperLayer {
    pub path: String,
    pub zoom: f64,
    pub x: f64,
    pub y: f64,
}
impl Default for WallpaperLayer {
    fn default() -> Self {
        Self {
            path: String::new(),
            zoom: 1.0,
            x: 0.5,
            y: 0.5,
        }
    }
}
#[derive(Clone, Copy, Debug, Default, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum WallpaperMode {
    #[default]
    Auto,
    Split,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct WallpaperLayout {
    pub mode: WallpaperMode,
    pub top: WallpaperLayer,
    pub bottom: WallpaperLayer,
    pub video_y: f64,
}
impl Default for WallpaperLayout {
    fn default() -> Self {
        Self {
            mode: WallpaperMode::Auto,
            top: Default::default(),
            bottom: Default::default(),
            video_y: 0.5,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Settings {
    pub seconds: f64,
    pub smart: bool,
    pub extra: f64,
    pub ratio: String,
    pub fill: String,
    #[serde(default = "default_scale")]
    pub scale: f64,
    #[serde(default)]
    pub wallpaper: String,
    #[serde(default)]
    pub wallpaper_layout: WallpaperLayout,
    #[serde(default)]
    pub encoding_profile: EncodingProfile,
    pub text: TextLayer,
    pub label: TextLayer,
}

fn default_scale() -> f64 {
    1.0
}

impl Settings {
    pub fn wallpaper_paths(&self) -> Vec<&str> {
        if self.fill != "wallpaper" {
            return Vec::new();
        }
        if self.wallpaper_layout.mode == WallpaperMode::Auto {
            return vec![&self.wallpaper];
        }
        [&self.wallpaper_layout.top, &self.wallpaper_layout.bottom]
            .iter()
            .map(|layer| {
                if layer.path.is_empty() {
                    self.wallpaper.as_str()
                } else {
                    layer.path.as_str()
                }
            })
            .collect()
    }
    pub fn validate(&self) -> Result<(), String> {
        if !self.wallpaper_layout.video_y.is_finite()
            || !(0.0..=1.0).contains(&self.wallpaper_layout.video_y)
        {
            return Err("Video vertical position must be between 0% and 100%.".into());
        }
        for layer in [&self.wallpaper_layout.top, &self.wallpaper_layout.bottom] {
            if !layer.zoom.is_finite()
                || !(1.0..=3.0).contains(&layer.zoom)
                || !layer.x.is_finite()
                || !(0.0..=1.0).contains(&layer.x)
                || !layer.y.is_finite()
                || !(0.0..=1.0).contains(&layer.y)
            {
                return Err("Wallpaper zoom or position is outside its supported range.".into());
            }
        }
        if !self.scale.is_finite() || !(0.25..=2.0).contains(&self.scale) {
            return Err("Video scale must be between 25% and 200%.".into());
        }
        if !self.seconds.is_finite() || !(5.0..=36000.0).contains(&self.seconds) {
            return Err("Part duration must be between 5 seconds and 10 hours.".into());
        }
        if !self.extra.is_finite() || !(0.0..=120.0).contains(&self.extra) {
            return Err("Pause extension must be between 0 and 120 seconds.".into());
        }
        if !["Original", "9:16", "16:9", "1:1"].contains(&self.ratio.as_str())
            || !["blur", "black", "crop", "wallpaper"].contains(&self.fill.as_str())
        {
            return Err("Unsupported frame settings.".into());
        }
        if self
            .wallpaper_paths()
            .iter()
            .any(|path| !std::path::Path::new(path).is_file())
        {
            return Err(
                "Choose a background wallpaper image again; the saved image is unavailable.".into(),
            );
        }
        for l in [&self.text, &self.label] {
            if !l.size.is_finite()
                || !(12.0..=200.0).contains(&l.size)
                || !l.x.is_finite()
                || !l.y.is_finite()
                || !(0.0..=1.0).contains(&l.x)
                || !(0.0..=1.0).contains(&l.y)
                || !l.opacity.is_finite()
                || !(0.0..=1.0).contains(&l.opacity)
            {
                return Err("Invalid text position, size or opacity.".into());
            }
            if l.text.chars().count() > 4000 {
                return Err("A text layer is limited to 4,000 characters.".into());
            }
            if l.color.len() != 7
                || !l.color.starts_with('#')
                || !l.color[1..].bytes().all(|b| b.is_ascii_hexdigit())
            {
                return Err("Text color must be #RRGGBB.".into());
            }
        }
        Ok(())
    }
}

pub fn dimensions(source: &Source, ratio: &str) -> (u32, u32) {
    match ratio {
        "9:16" => (1080, 1920),
        "16:9" => (1920, 1080),
        "1:1" => (1080, 1080),
        _ => (
            (source.width / 2 * 2).max(2),
            (source.height / 2 * 2).max(2),
        ),
    }
}

pub fn label_text(text: &str, n: usize, total: usize) -> String {
    let mm = |s: String| {
        s.chars()
            .map(|c| {
                c.to_digit(10)
                    .map(|d| char::from_u32(0x1040 + d).unwrap())
                    .unwrap_or(c)
            })
            .collect::<String>()
    };
    let total = if text.contains("{mm}") {
        mm(total.to_string())
    } else {
        total.to_string()
    };
    text.replace("{n}", &n.to_string())
        .replace("{mm}", &mm(n.to_string()))
        .replace("{total}", &total)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn myanmar_part_labels_use_consistent_digits() {
        assert_eq!(label_text("အပိုင်း {mm}/{total}", 2, 12), "အပိုင်း ၂/၁၂");
        assert_eq!(label_text("Part {n}/{total}", 2, 12), "Part 2/12");
    }
}
