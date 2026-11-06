use serde::{Deserialize, Serialize};

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
pub struct Settings {
    pub seconds: f64,
    pub smart: bool,
    pub extra: f64,
    pub ratio: String,
    pub fill: String,
    pub text: TextLayer,
    pub label: TextLayer,
}

impl Settings {
    pub fn validate(&self) -> Result<(), String> {
        if !self.seconds.is_finite() || !(5.0..=36000.0).contains(&self.seconds) {
            return Err("Part duration must be between 5 seconds and 10 hours.".into());
        }
        if !self.extra.is_finite() || !(0.0..=120.0).contains(&self.extra) {
            return Err("Pause extension must be between 0 and 120 seconds.".into());
        }
        if !["Original", "9:16", "16:9", "1:1"].contains(&self.ratio.as_str())
            || !["blur", "black", "crop"].contains(&self.fill.as_str())
        {
            return Err("Unsupported frame settings.".into());
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
