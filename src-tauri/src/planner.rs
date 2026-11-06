use crate::model::Clip;

const STEP: f64 = 0.05;

pub fn plan(
    duration: f64,
    seconds: f64,
    extra: f64,
    levels: Option<&[f64]>,
) -> Result<Vec<Clip>, String> {
    if !duration.is_finite()
        || duration <= 0.0
        || !seconds.is_finite()
        || seconds < 5.0
        || !extra.is_finite()
        || extra < 0.0
    {
        return Err("Invalid split duration.".into());
    }
    let mut result = Vec::new();
    let mut start = 0.0;
    while start + seconds < duration - 0.05 {
        let target = start + seconds;
        let (cut, reason) = levels
            .filter(|l| !l.is_empty())
            .map(|l| quiet(l, target, (target + extra).min(duration)))
            .unwrap_or((target, "exact"));
        let cut = cut.clamp(target, (target + extra).min(duration));
        if duration - cut < 20_f64.min(seconds / 4.0) {
            break;
        }
        result.push(Clip {
            n: result.len() + 1,
            start,
            end: cut,
            cut: reason.into(),
            enabled: true,
        });
        start = cut;
    }
    result.push(Clip {
        n: result.len() + 1,
        start,
        end: duration,
        cut: "end".into(),
        enabled: true,
    });
    Ok(result)
}

fn percentile(values: &[f64], fraction: f64) -> f64 {
    let mut values = values.to_vec();
    values.sort_by(f64::total_cmp);
    values[((values.len() - 1) as f64 * fraction).round() as usize]
}

fn quiet(levels: &[f64], start: f64, end: f64) -> (f64, &'static str) {
    let a = (start / STEP).floor() as usize;
    let b = ((end / STEP).ceil() as usize).min(levels.len());
    if a >= b {
        return (start, "exact");
    }
    let window = &levels[a..b];
    let floor = percentile(window, 0.1);
    let speech = percentile(levels, 0.9);
    if speech < -65.0 {
        return (start, "silence");
    }
    if speech - floor < 8.0 {
        return (start, "exact");
    }
    // Never let the adaptive threshold rise close to normal speech.
    let threshold = (speech - 18.0).max(floor + 4.0).min(speech - 8.0);
    let mut best = (0, 0);
    let mut i = 0;
    while i < window.len() {
        if window[i] > threshold {
            i += 1;
            continue;
        }
        let first = i;
        while i < window.len() && window[i] <= threshold {
            i += 1;
        }
        if i - first > best.1 - best.0 {
            best = (first, i);
        }
    }
    if best.1 - best.0 >= 2 {
        return (
            ((a as f64 + (best.0 + best.1) as f64 / 2.0) * STEP).clamp(start, end),
            if best.1 - best.0 >= 6 {
                "pause"
            } else {
                "breath"
            },
        );
    }
    let index = window
        .iter()
        .enumerate()
        .min_by(|a, b| a.1.total_cmp(b.1))
        .map(|p| p.0)
        .unwrap_or(0);
    (((a + index) as f64 * STEP).clamp(start, end), "quietest")
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn coverage() {
        let p = plan(400.0, 180.0, 20.0, None).unwrap();
        assert_eq!(p.len(), 3);
        assert_eq!(p[1].start, p[0].end);
        assert_eq!(p.last().unwrap().end, 400.0);
    }
    #[test]
    fn constant_audio_is_not_a_pause() {
        let l = vec![-20.0; 8000];
        assert_eq!(quiet(&l, 180.0, 200.0), (180.0, "exact"));
    }
    #[test]
    fn pause_stays_inside_window() {
        let mut l = vec![-20.0; 8000];
        for i in 3620..3664 {
            l[i] = -55.0;
        }
        for i in 3996..4060 {
            l[i] = -55.0;
        }
        let (cut, _) = quiet(&l, 180.0, 200.0);
        assert!((180.0..=200.0).contains(&cut));
    }
    #[test]
    fn silence_uses_target() {
        assert_eq!(quiet(&vec![-100.0; 8000], 180.0, 200.0), (180.0, "silence"));
    }
    #[test]
    fn invalid_inputs() {
        assert!(plan(f64::NAN, 10.0, 0.0, None).is_err());
        assert!(plan(10.0, 0.0, 0.0, None).is_err());
    }
}
