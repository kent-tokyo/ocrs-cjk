use std::process;

use crate::models::cache_dir;

enum Status {
    Ok,
    Info,
    Error,
}

struct Check {
    label: &'static str,
    status: Status,
    detail: String,
}

impl Check {
    fn ok(label: &'static str, detail: impl Into<String>) -> Self {
        Self { label, status: Status::Ok, detail: detail.into() }
    }
    fn info(label: &'static str, detail: impl Into<String>) -> Self {
        Self { label, status: Status::Info, detail: detail.into() }
    }
    fn error(label: &'static str, detail: impl Into<String>) -> Self {
        Self { label, status: Status::Error, detail: detail.into() }
    }
}

pub fn run_doctor() {
    let mut checks: Vec<Check> = Vec::new();

    // 1. Cache directory
    match cache_dir() {
        Ok(dir) => checks.push(Check::ok("Cache dir", dir.display().to_string())),
        Err(e) => checks.push(Check::error("Cache dir", e.to_string())),
    }

    // 2-5. Model files in cache
    let cache = cache_dir().ok();
    let model_check = |filename: &'static str, label: &'static str, hint: &str| -> Check {
        match &cache {
            Some(dir) if dir.join(filename).exists() => {
                Check::ok(label, format!("{} (cached)", filename))
            }
            Some(_) => Check::info(label, format!("not cached  ({})", hint)),
            None => Check::error(label, "cache dir unavailable".to_string()),
        }
    };

    checks.push(model_check(
        "text-detection.rten",
        "Latin detection model",
        "auto-downloads on first OCR",
    ));
    checks.push(model_check(
        "text-recognition.rten",
        "Latin recognition model",
        "auto-downloads on first OCR",
    ));
    checks.push(model_check(
        "PP-OCRv5_server_det_infer.onnx",
        "CJK detection model",
        "run: ocrs --lang ja <image>",
    ));
    checks.push(model_check(
        "PP-OCRv5_server_rec_infer.onnx",
        "CJK recognition model",
        "run: ocrs --lang ja <image>",
    ));

    // 6. ONNX feature
    if cfg!(feature = "onnx") {
        checks.push(Check::ok("ONNX support", "enabled"));
    } else {
        checks.push(Check::error(
            "ONNX support",
            "disabled — rebuild with `--features onnx` to use CJK models",
        ));
    }

    // 7. PDF support (lopdf is native-only)
    if cfg!(not(target_arch = "wasm32")) {
        checks.push(Check::ok("PDF support", "enabled"));
    } else {
        checks.push(Check::info("PDF support", "not available on WASM"));
    }

    // 8. CJK alphabet (always embedded)
    let alpha_len = crate::CJK_ALPHABET.chars().count();
    checks.push(Check::ok(
        "CJK alphabet",
        format!("embedded ({} chars)", alpha_len),
    ));

    // --- print ---
    let sep = "─".repeat(48);
    eprintln!("ocrs doctor");
    eprintln!("{}", sep);

    let label_width = checks.iter().map(|c| c.label.len()).max().unwrap_or(0);
    let mut errors = 0usize;
    for c in &checks {
        let (icon, _) = match c.status {
            Status::Ok => ("✓", false),
            Status::Info => ("○", false),
            Status::Error => {
                errors += 1;
                ("✗", true)
            }
        };
        eprintln!(
            "{}  {:<width$}  {}",
            icon,
            c.label,
            c.detail,
            width = label_width
        );
    }

    eprintln!("{}", sep);
    if errors == 0 {
        eprintln!("All checks passed.");
    } else {
        eprintln!("{} error(s) found.", errors);
        process::exit(1);
    }
}
