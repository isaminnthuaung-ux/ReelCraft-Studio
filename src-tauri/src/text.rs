use crate::model::{label_text, Settings};
use base64::{engine::general_purpose::STANDARD, Engine};
use cosmic_text::{Align, Attrs, Buffer, Color, Family, FontSystem, Metrics, Shaping, SwashCache};
use image::{ImageEncoder, Rgba, RgbaImage};
use serde::Serialize;
use std::{collections::BTreeSet, path::Path};

pub struct TextEngine {
    pub fonts: FontSystem,
    cache: SwashCache,
}

#[derive(Serialize)]
pub struct Overlay {
    pub image: String,
    pub warnings: Vec<String>,
}

impl TextEngine {
    pub fn new(folder: &Path) -> Self {
        let mut fonts = FontSystem::new();
        fonts.db_mut().load_fonts_dir(folder);
        Self {
            fonts,
            cache: SwashCache::new(),
        }
    }
    pub fn families(&self) -> Vec<String> {
        self.fonts
            .db()
            .faces()
            .flat_map(|f| f.families.iter().map(|p| p.0.clone()))
            .collect::<BTreeSet<_>>()
            .into_iter()
            .collect()
    }
    pub fn render(
        &mut self,
        settings: &Settings,
        w: u32,
        h: u32,
        n: usize,
        total: usize,
    ) -> Result<(RgbaImage, Vec<String>), String> {
        if w > 8192 || h > 8192 || w < 2 || h < 2 {
            return Err("Frame dimensions are outside supported limits (2–8192 pixels).".into());
        }
        let mut canvas = RgbaImage::new(w, h);
        let mut warnings = Vec::new();
        for layer in [&settings.text, &settings.label] {
            if !layer.enabled || layer.text.trim().is_empty() {
                continue;
            }
            if !self.families().contains(&layer.font) {
                return Err(format!(
                    "Font '{}' is unavailable. Select or import it first.",
                    layer.font
                ));
            }
            let text = label_text(&layer.text, n, total);
            let width = (w as f32 * 0.84).max(20.0);
            let mut buffer =
                Buffer::new(&mut self.fonts, Metrics::new(layer.size, layer.size * 1.5));
            buffer.set_size(&mut self.fonts, Some(width), None);
            buffer.set_text(
                &mut self.fonts,
                &text,
                &Attrs::new().family(Family::Name(&layer.font)),
                Shaping::Advanced,
            );
            for line in &mut buffer.lines {
                line.set_align(Some(Align::Center));
            }
            buffer.shape_until_scroll(&mut self.fonts, false);
            let mut fallback = false;
            for run in buffer.layout_runs() {
                for g in run.glyphs {
                    if g.glyph_id == 0 {
                        warnings.push("Some characters have no glyph in your installed fonts. Import a font covering this script.".into());
                    }
                    if let Some(face) = self.fonts.db().face(g.font_id) {
                        if !face.families.iter().any(|p| p.0 == layer.font) {
                            fallback = true;
                        }
                    }
                }
            }
            if fallback {
                warnings.push(format!(
                    "{} uses fallback fonts for some characters.",
                    layer.font
                ));
            }
            let height = buffer
                .layout_runs()
                .map(|r| r.line_top + r.line_height)
                .fold(0_f32, f32::max)
                .max(layer.size * 1.5);
            let ox = layer.x * w as f32 - width / 2.0;
            let oy = layer.y * h as f32 - height / 2.0;
            let rgb = u32::from_str_radix(&layer.color[1..], 16).map_err(|e| e.to_string())?;
            let color = Color::rgba(
                (rgb >> 16) as u8,
                (rgb >> 8) as u8,
                rgb as u8,
                (layer.opacity * 255.0) as u8,
            );
            let mut ink = RgbaImage::new(w, h);
            let mut bounds = (w as i32, h as i32, 0, 0);
            buffer.draw(&mut self.fonts, &mut self.cache, color, |x, y, _, _, c| {
                let px = x + ox.round() as i32;
                let py = y + oy.round() as i32;
                if px >= 0 && py >= 0 && px < w as i32 && py < h as i32 {
                    let (r, g, b, a) = c.as_rgba_tuple();
                    image::Pixel::blend(
                        ink.get_pixel_mut(px as u32, py as u32),
                        &Rgba([r, g, b, a]),
                    );
                    bounds = (
                        bounds.0.min(px),
                        bounds.1.min(py),
                        bounds.2.max(px),
                        bounds.3.max(py),
                    );
                }
            });
            if layer.boxed && bounds.0 <= bounds.2 {
                let pad = (layer.size * 0.4) as i32;
                for y in (bounds.1 - pad).max(0)..(bounds.3 + pad + 1).min(h as i32) {
                    for x in (bounds.0 - pad).max(0)..(bounds.2 + pad + 1).min(w as i32) {
                        image::Pixel::blend(
                            canvas.get_pixel_mut(x as u32, y as u32),
                            &Rgba([6, 10, 16, (layer.opacity * 175.0) as u8]),
                        );
                    }
                }
            }
            image::imageops::overlay(&mut canvas, &ink, 0, 0);
            if ox < 0.0 || oy < 0.0 || ox + width > w as f32 || oy + height > h as f32 {
                warnings.push(
                    "A text layer extends past the frame. Move it inward or reduce its size."
                        .into(),
                );
            }
        }
        warnings.sort();
        warnings.dedup();
        Ok((canvas, warnings))
    }
}

pub fn data_image(image: &RgbaImage) -> Result<String, String> {
    let mut bytes = Vec::new();
    image::codecs::png::PngEncoder::new(&mut bytes)
        .write_image(
            image.as_raw(),
            image.width(),
            image.height(),
            image::ExtendedColorType::Rgba8,
        )
        .map_err(|e| e.to_string())?;
    Ok(format!("data:image/png;base64,{}", STANDARD.encode(bytes)))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn unicode_overlay_is_real_pixels() {
        let mut engine = TextEngine::new(Path::new("."));
        let family = engine
            .families()
            .into_iter()
            .find(|f| f == "Myanmar Text")
            .or_else(|| engine.families().into_iter().next())
            .expect("system font");
        let layer = crate::model::TextLayer {
            enabled: true,
            text: "မြန်မာစာ John ၁၂၃".into(),
            font: family,
            size: 48.0,
            x: 0.5,
            y: 0.5,
            color: "#ffffff".into(),
            opacity: 1.0,
            boxed: true,
        };
        let mut settings = Settings {
            encoding_profile: Default::default(),
            wallpaper: String::new(),
            wallpaper_layout: Default::default(),
            scale: 1.0,
            seconds: 60.0,
            smart: false,
            extra: 0.0,
            ratio: "16:9".into(),
            fill: "black".into(),
            text: layer.clone(),
            label: layer,
        };
        settings.label.enabled = false;
        let (pixels, _) = engine.render(&settings, 640, 360, 1, 3).unwrap();
        assert!(pixels.pixels().any(|p| p[3] > 0));
        assert!(data_image(&pixels)
            .unwrap()
            .starts_with("data:image/png;base64,"));
    }
}
