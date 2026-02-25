use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::cmp::Ordering;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpPatch {
    pub patch_type: String,
    pub scene_id: Option<String>,
    pub target: String,
    pub properties: Value,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpAsset {
    pub id: String,
    pub path: String,
    pub asset_type: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

fn ensure_project_meta_dir(project_root: &Path) -> Result<PathBuf> {
    let d = project_root.join(".vidra");
    std::fs::create_dir_all(&d).context("failed to create .vidra dir")?;
    Ok(d)
}

fn patches_path(project_root: &Path) -> Result<PathBuf> {
    Ok(ensure_project_meta_dir(project_root)?.join("mcp_patches.json"))
}

fn assets_path(project_root: &Path) -> Result<PathBuf> {
    Ok(ensure_project_meta_dir(project_root)?.join("mcp_assets.json"))
}

fn load_json_vec<T: for<'de> Deserialize<'de>>(path: &Path) -> Result<Vec<T>> {
    if !path.exists() {
        return Ok(Vec::new());
    }
    let raw = std::fs::read_to_string(path)
        .with_context(|| format!("failed to read {}", path.display()))?;
    let v = serde_json::from_str::<Vec<T>>(&raw)
        .with_context(|| format!("failed to parse {}", path.display()))?;
    Ok(v)
}

fn save_json_vec<T: Serialize>(path: &Path, items: &[T]) -> Result<()> {
    let raw = serde_json::to_string_pretty(items).context("failed to serialize JSON list")?;
    std::fs::write(path, raw).with_context(|| format!("failed to write {}", path.display()))?;
    Ok(())
}

pub fn create_project(name: &str, width: u32, height: u32, fps: u32) -> Result<PathBuf> {
    let project_dir = PathBuf::from(name);
    if project_dir.exists() {
        anyhow::bail!("project directory already exists: {}", project_dir.display());
    }

    std::fs::create_dir_all(project_dir.join("assets"))
        .with_context(|| format!("failed to create project dirs for {}", project_dir.display()))?;

    let mut config = vidra_core::VidraConfig::default();
    config.project.name = name.to_string();
    config.project.resolution = format!("{}x{}", width, height);
    config.project.fps = fps;
    config
        .save_to_file(&project_dir.join("vidra.config.toml"))
        .map_err(|e| anyhow::anyhow!("{}", e))?;

    let main = format!(
        "project({}, {}, {}) {{\n    scene(\"main\", 5s) {{\n        layer(\"welcome_text\") {{\n            text(\"Welcome to Vidra!\", font: \"Inter\", size: 96, color: #FFFFFF)\n            position({}, {})\n        }}\n    }}\n}}\n",
        width,
        height,
        fps,
        width / 2,
        height / 2
    );
    std::fs::write(project_dir.join("main.vidra"), main)
        .with_context(|| format!("failed to write main.vidra in {}", project_dir.display()))?;

    Ok(project_dir)
}

pub fn add_scene_to_vidra_file(file: &Path, scene_name: &str, duration_seconds: f64) -> Result<()> {
    let mut src = std::fs::read_to_string(file)
        .with_context(|| format!("failed to read vidra file: {}", file.display()))?;

    let insert_at = src.rfind('}').context("invalid vidra file: could not find project closing brace")?;
    let scene_block = format!(
        "\n    scene(\"{}\", {}s) {{\n        layer(\"{}__placeholder\") {{\n            solid(#111111)\n        }}\n    }}\n",
        scene_name,
        duration_seconds,
        scene_name.replace(' ', "_")
    );
    src.insert_str(insert_at, &scene_block);
    std::fs::write(file, src)
        .with_context(|| format!("failed to write vidra file: {}", file.display()))?;
    Ok(())
}

fn find_layer_block(src: &str, layer_id: &str) -> Option<(usize, usize)> {
    let needle = format!("layer(\"{}\")", layer_id);
    let layer_start = src.find(&needle)?;

    let open_rel = src[layer_start..].find('{')?;
    let open_idx = layer_start + open_rel;

    let mut depth: i32 = 0;
    let bytes = src.as_bytes();
    let mut i = open_idx;
    while i < bytes.len() {
        match bytes[i] as char {
            '{' => depth += 1,
            '}' => {
                depth -= 1;
                if depth == 0 {
                    return Some((open_idx + 1, i));
                }
            }
            _ => {}
        }
        i += 1;
    }
    None
}

fn upsert_property_line(block: &str, prefix: &str, new_line: &str) -> String {
    let mut out = String::new();
    let mut replaced = false;
    for line in block.lines() {
        let trimmed = line.trim_start();
        if !replaced && trimmed.starts_with(prefix) {
            out.push_str(new_line);
            out.push('\n');
            replaced = true;
        } else {
            out.push_str(line);
            out.push('\n');
        }
    }
    if !replaced {
        out.push_str(new_line);
        out.push('\n');
    }
    out
}

fn extract_quoted_after(s: &str, key: &str) -> Option<String> {
    let idx = s.find(key)?;
    let rest = &s[idx + key.len()..];
    let first_quote = rest.find('"')?;
    let rest2 = &rest[first_quote + 1..];
    let second_quote = rest2.find('"')?;
    Some(rest2[..second_quote].to_string())
}

fn extract_token_after(s: &str, key: &str) -> Option<String> {
    let idx = s.find(key)?;
    let rest = &s[idx + key.len()..];
    let trimmed = rest.trim_start();
    let end = trimmed
        .find(|c: char| c == ',' || c == ')')
        .unwrap_or(trimmed.len());
    let token = trimmed[..end].trim();
    if token.is_empty() {
        None
    } else {
        Some(token.to_string())
    }
}

fn update_text_line(line: &str, properties: &Value) -> String {
    let indent_len = line.len().saturating_sub(line.trim_start().len());
    let indent = &line[..indent_len];

    let existing_text = extract_quoted_after(line, "text(");
    let existing_font = extract_quoted_after(line, "font:");
    let existing_size = extract_token_after(line, "size:");
    let existing_color = extract_token_after(line, "color:");

    let text = properties
        .get("text")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string())
        .or(existing_text)
        .unwrap_or_else(|| "Text".to_string());

    let font = properties
        .get("font")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string())
        .or(existing_font);

    let size = properties
        .get("size")
        .or_else(|| properties.get("fontSize"))
        .and_then(num_as_string)
        .or(existing_size);

    let color = properties
        .get("color")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string())
        .or(existing_color);

    let mut args = Vec::new();
    args.push(format!("\"{}\"", text.replace('"', "\\\"")));
    if let Some(f) = font {
        args.push(format!("font: \"{}\"", f.replace('"', "\\\"")));
    }
    if let Some(s) = size {
        args.push(format!("size: {}", s));
    }
    if let Some(c) = color {
        args.push(format!("color: {}", c));
    }

    format!("{}text({})", indent, args.join(", "))
}

fn num_as_string(v: &Value) -> Option<String> {
    if let Some(n) = v.as_i64() {
        return Some(n.to_string());
    }
    if let Some(n) = v.as_u64() {
        return Some(n.to_string());
    }
    v.as_f64().map(|n| {
        if (n.fract()).abs() < f64::EPSILON {
            format!("{}", n as i64)
        } else {
            format!("{}", n)
        }
    })
}

fn num_as_f64(v: &Value) -> Option<f64> {
    if let Some(n) = v.as_f64() {
        return Some(n);
    }
    if let Some(s) = v.as_str() {
        return s.trim().parse::<f64>().ok();
    }
    None
}

fn format_num(n: f64) -> String {
    if (n.fract()).abs() < f64::EPSILON {
        format!("{}", n as i64)
    } else {
        format!("{}", n)
    }
}

fn read_number_alias(anim: &Value, keys: &[&str]) -> Option<f64> {
    for key in keys {
        if let Some(v) = anim.get(*key).and_then(num_as_f64) {
            return Some(v);
        }
    }
    None
}

fn read_loop_alias(anim: &Value) -> Option<String> {
    for key in ["loop", "repeat", "iterations"] {
        if let Some(v) = anim.get(key) {
            match v {
                Value::Bool(true) => return Some("Infinite".to_string()),
                Value::Bool(false) => return None,
                Value::Number(_) => return num_as_string(v),
                Value::String(s) => {
                    let t = s.trim();
                    if t.is_empty() {
                        continue;
                    }
                    if t.eq_ignore_ascii_case("true") || t.eq_ignore_ascii_case("infinite") {
                        return Some("Infinite".to_string());
                    }
                    if t.eq_ignore_ascii_case("false") {
                        return None;
                    }
                    return Some(t.to_string());
                }
                _ => continue,
            }
        }
    }
    None
}

fn extract_numeric_for_property(value: &Value, property: &str) -> Option<String> {
    if let Some(n) = num_as_string(value) {
        return Some(n);
    }

    if let Some(arr) = value.as_array() {
        let first = arr.first().and_then(num_as_f64);
        let second = arr.get(1).and_then(num_as_f64);
        return match property {
            "X" | "ScaleX" => first.map(format_num),
            "Y" | "ScaleY" => second.or(first).map(format_num),
            "Scale" => match (first, second) {
                (Some(a), Some(b)) => Some(format_num((a + b) / 2.0)),
                (Some(a), None) => Some(format_num(a)),
                _ => None,
            },
            _ => None,
        };
    }

    let obj = value.as_object()?;
    let key_aliases: &[&str] = match property {
        "X" => &["x", "posX", "positionX", "translateX"],
        "Y" => &["y", "posY", "positionY", "translateY"],
        "Opacity" => &["opacity", "alpha"],
        "Rotation" => &["rotation", "rotate", "angle"],
        "ScaleX" => &["scaleX", "x", "scale"],
        "ScaleY" => &["scaleY", "y", "scale"],
        "Scale" => &["scale", "uniform"],
        _ => &[],
    };

    for key in key_aliases {
        if let Some(v) = obj.get(*key).and_then(num_as_string) {
            return Some(v);
        }
    }

    if property == "Scale" {
        let sx = obj
            .get("scaleX")
            .or_else(|| obj.get("x"))
            .and_then(num_as_f64);
        let sy = obj
            .get("scaleY")
            .or_else(|| obj.get("y"))
            .and_then(num_as_f64);

        return match (sx, sy) {
            (Some(a), Some(b)) => Some(format_num((a + b) / 2.0)),
            (Some(a), None) => Some(format_num(a)),
            (None, Some(b)) => Some(format_num(b)),
            (None, None) => None,
        };
    }
    None
}

fn read_endpoint_value_alias(anim: &Value, keys: &[&str], property: &str) -> Option<String> {
    for key in keys {
        if let Some(v) = anim.get(*key).and_then(|x| extract_numeric_for_property(x, property)) {
            return Some(v);
        }
    }
    None
}

fn normalize_easing_token(raw: &str) -> Option<String> {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return None;
    }

    let key: String = trimmed
        .chars()
        .filter(|c| *c != '_' && *c != '-' && !c.is_whitespace())
        .flat_map(|c| c.to_lowercase())
        .collect();

    let normalized = match key.as_str() {
        "linear" => "Linear",
        "ease" => "EaseInOut",
        "easein" => "EaseIn",
        "easeout" => "EaseOut",
        "easeinout" => "EaseInOut",
        "stepstart" => "StepStart",
        "stepend" => "StepEnd",
        _ => trimmed,
    };
    Some(normalized.to_string())
}

fn read_easing_alias(anim: &Value) -> Option<String> {
    for key in ["easing", "curve", "timingFunction", "timing"] {
        if let Some(s) = anim.get(key).and_then(|v| v.as_str()).and_then(normalize_easing_token) {
            return Some(s);
        }
    }
    None
}

fn parse_keyframe_time_value(v: &Value, start_n: f64, end_n: f64) -> Option<f64> {
    if let Some(n) = num_as_f64(v) {
        return Some(n);
    }

    let s = v.as_str()?.trim();
    if s.is_empty() {
        return None;
    }

    if let Some(p) = s.strip_suffix('%') {
        let pct = p.trim().parse::<f64>().ok()?;
        return Some(start_n + (end_n - start_n) * (pct / 100.0));
    }

    let lower = s.to_ascii_lowercase();
    if let Some(ms) = lower.strip_suffix("ms") {
        let n = ms.trim().parse::<f64>().ok()?;
        return Some(n / 1000.0);
    }
    if let Some(sec) = lower.strip_suffix('s') {
        let n = sec.trim().parse::<f64>().ok()?;
        return Some(n);
    }

    s.parse::<f64>().ok()
}

fn parse_keyframe_offset_value(v: &Value, start_n: f64, end_n: f64) -> Option<f64> {
    if let Some(n) = v.as_f64() {
        let fraction = if (0.0..=1.0).contains(&n) { n } else { n / 100.0 };
        let clamped = fraction.clamp(0.0, 1.0);
        return Some(start_n + (end_n - start_n) * clamped);
    }

    if let Some(s) = v.as_str() {
        let t = s.trim();
        if let Some(p) = t.strip_suffix('%') {
            if let Ok(n) = p.trim().parse::<f64>() {
                let clamped = (n / 100.0).clamp(0.0, 1.0);
                return Some(start_n + (end_n - start_n) * clamped);
            }
        }
        if let Ok(n) = t.parse::<f64>() {
            let fraction = if (0.0..=1.0).contains(&n) { n } else { n / 100.0 };
            let clamped = fraction.clamp(0.0, 1.0);
            return Some(start_n + (end_n - start_n) * clamped);
        }
    }

    parse_keyframe_time_value(v, start_n, end_n)
}

fn read_keyframes(anim: &Value, start_n: f64, end_n: f64, property: &str) -> Option<Vec<(f64, String, Option<String>)>> {
    let frames = anim
        .get("keyframes")
        .or_else(|| anim.get("frames"))
        .and_then(|v| v.as_array())?;

    let mut tmp: Vec<(Option<f64>, String, Option<String>)> = Vec::new();
    for frame in frames {
        let time = frame
            .get("time")
            .or_else(|| frame.get("t"))
            .or_else(|| frame.get("at"))
            .and_then(|v| parse_keyframe_time_value(v, start_n, end_n))
            .or_else(|| frame.get("offset").and_then(|v| parse_keyframe_offset_value(v, start_n, end_n)))
            .or_else(|| read_number_alias(frame, &["timeMs", "atMs"]).map(|ms| ms / 1000.0));
        let value = read_endpoint_value_alias(frame, &["value", "v", "val", "to"], property)
            .or_else(|| extract_numeric_for_property(frame, property))?;
        let easing = read_easing_alias(frame);
        tmp.push((time, value, easing));
    }

    if tmp.len() < 2 {
        return None;
    }

    let explicit_count = tmp.iter().filter(|(t, _, _)| t.is_some()).count();
    if explicit_count == 0 {
        let len = tmp.len() as f64;
        for (idx, (time, _, _)) in tmp.iter_mut().enumerate() {
            let fraction = if len <= 1.0 { 0.0 } else { (idx as f64) / (len - 1.0) };
            *time = Some(start_n + (end_n - start_n) * fraction);
        }
    } else if explicit_count != tmp.len() {
        let anchor_indices: Vec<usize> = tmp
            .iter()
            .enumerate()
            .filter_map(|(idx, (time, _, _))| if time.is_some() { Some(idx) } else { None })
            .collect();

        if anchor_indices.is_empty() {
            return None;
        }

        let first_anchor = anchor_indices[0];
        let first_time = tmp[first_anchor].0?;
        if first_anchor > 0 {
            let span = first_anchor as f64;
            for idx in 0..=first_anchor {
                let fraction = (idx as f64) / span;
                tmp[idx].0 = Some(start_n + (first_time - start_n) * fraction);
            }
        }

        for pair in anchor_indices.windows(2) {
            let a = pair[0];
            let b = pair[1];
            let ta = tmp[a].0?;
            let tb = tmp[b].0?;
            let span = (b - a) as f64;
            if span <= 0.0 {
                continue;
            }
            for idx in a..=b {
                let fraction = ((idx - a) as f64) / span;
                tmp[idx].0 = Some(ta + (tb - ta) * fraction);
            }
        }

        let last_anchor = *anchor_indices.last()?;
        let last_time = tmp[last_anchor].0?;
        if last_anchor + 1 < tmp.len() {
            let span = (tmp.len() - 1 - last_anchor) as f64;
            for idx in last_anchor..tmp.len() {
                let fraction = ((idx - last_anchor) as f64) / span;
                tmp[idx].0 = Some(last_time + (end_n - last_time) * fraction);
            }
        }
    }

    let mut out: Vec<(f64, String, Option<String>)> = tmp
        .into_iter()
        .filter_map(|(time, value, easing)| time.map(|t| (t, value, easing)))
        .collect();

    out.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(Ordering::Equal));
    Some(out)
}

fn normalize_animation_property(raw: &str) -> Option<String> {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return None;
    }

    let key: String = trimmed
        .chars()
        .filter(|c| *c != '_' && *c != '-' && !c.is_whitespace())
        .flat_map(|c| c.to_lowercase())
        .collect();

    let canonical = match key.as_str() {
        "alpha" | "opacity" => "Opacity",
        "x" | "posx" | "positionx" | "translatex" => "X",
        "y" | "posy" | "positiony" | "translatey" => "Y",
        "scalex" => "ScaleX",
        "scaley" => "ScaleY",
        "scale" => "Scale",
        "rotation" | "rotate" | "angle" => "Rotation",
        _ => trimmed,
    };

    Some(canonical.to_string())
}

fn find_animate_block(block: &str, property: &str) -> Option<(usize, usize)> {
    let needle = format!("animate({})", property);
    let start = block.find(&needle)?;
    let open_rel = block[start..].find('{')?;
    let open_idx = start + open_rel;

    let bytes = block.as_bytes();
    let mut depth: i32 = 0;
    let mut i = open_idx;
    while i < bytes.len() {
        match bytes[i] as char {
            '{' => depth += 1,
            '}' => {
                depth -= 1;
                if depth == 0 {
                    return Some((start, i + 1));
                }
            }
            _ => {}
        }
        i += 1;
    }
    None
}

fn build_simple_animation_block(anim: &Value) -> Option<String> {
    let property = normalize_animation_property(anim.get("property")?.as_str()?)?;
    let from = read_endpoint_value_alias(anim, &["from", "fromValue", "startValue"], &property);
    let to = read_endpoint_value_alias(anim, &["to", "toValue", "endValue"], &property);

    let delay_ms = read_number_alias(anim, &["delayMs", "startDelayMs"]);
    let duration_ms = read_number_alias(anim, &["durationMs", "lengthMs"]);

    let start_n = read_number_alias(anim, &["start", "startTime", "fromTime"]) 
        .or_else(|| read_number_alias(anim, &["delay", "startDelay"]))
        .or_else(|| delay_ms.map(|ms| ms / 1000.0))
        .unwrap_or(0.0);

    let end_n = read_number_alias(anim, &["end", "endTime", "toTime"])
        .or_else(|| read_number_alias(anim, &["duration", "length"]).map(|d| start_n + d))
        .or_else(|| duration_ms.map(|ms| start_n + (ms / 1000.0)))
        .unwrap_or(1.0);

    let keyframes = read_keyframes(anim, start_n, end_n, &property);

    let start = format_num(start_n);
    let end = format_num(end_n);
    let easing = read_easing_alias(anim).unwrap_or_else(|| "EaseInOut".to_string());

    let loop_line = read_loop_alias(anim).map(|s| format!("                loop({})\n", s));

    let mut body = String::new();
    if let Some(loop_line) = loop_line {
        body.push_str(&loop_line);
    }

    if let Some(frames) = keyframes {
        for (idx, (time, value, frame_easing)) in frames.iter().enumerate() {
            let t = format_num(*time);
            if idx == 0 {
                body.push_str(&format!("                {}s -> {}", t, value));
            } else {
                let segment_easing = frame_easing.as_deref().unwrap_or(easing.as_str());
                body.push_str(&format!("\n                {}s -> {} ~ {}", t, value, segment_easing));
            }
        }
    } else {
        let from = from?;
        let to = to?;
        body.push_str(&format!(
            "                {}s -> {}\n                {}s -> {} ~ {}",
            start, from, end, to, easing
        ));
    }

    Some(format!(
        "            animate({}) {{\n{}\n            }}",
        property, body
    ))
}

fn apply_single_animation_block(block: &mut String, anim: &Value) {
    let Some(new_block) = build_simple_animation_block(anim) else {
        return;
    };

    let Some(property) = anim
        .get("property")
        .and_then(|v| v.as_str())
        .and_then(normalize_animation_property)
    else {
        return;
    };

    if let Some((a_start, a_end)) = find_animate_block(block, &property) {
        block.replace_range(a_start..a_end, &new_block);
    } else {
        if !block.ends_with('\n') {
            block.push('\n');
        }
        block.push_str(&new_block);
        block.push('\n');
    }
}

pub fn apply_layer_properties_to_vidra_file(file: &Path, layer_id: &str, properties: &Value) -> Result<bool> {
    let mut src = std::fs::read_to_string(file)
        .with_context(|| format!("failed to read vidra file: {}", file.display()))?;

    let Some((inner_start, inner_end)) = find_layer_block(&src, layer_id) else {
        return Ok(false);
    };

    let original_block = &src[inner_start..inner_end];
    let mut block = original_block.to_string();

    if properties.get("text").is_some()
        || properties.get("font").is_some()
        || properties.get("size").is_some()
        || properties.get("fontSize").is_some()
        || properties.get("color").is_some()
    {
        let mut out = String::new();
        let mut replaced = false;
        for line in block.lines() {
            if !replaced && line.trim_start().starts_with("text(") {
                out.push_str(&update_text_line(line, properties));
                out.push('\n');
                replaced = true;
            } else {
                out.push_str(line);
                out.push('\n');
            }
        }
        if !replaced {
            // Create a new text(...) call with provided fields.
            out.push_str(&update_text_line("            text(\"Text\")", properties));
            out.push('\n');
        }
        block = out;
    }

    // Position can be supplied as x/y or position:[x,y].
    let mut pos_x: Option<String> = properties.get("x").and_then(num_as_string);
    let mut pos_y: Option<String> = properties.get("y").and_then(num_as_string);
    if let Some(arr) = properties.get("position").and_then(|v| v.as_array()) {
        if pos_x.is_none() {
            pos_x = arr.get(0).and_then(num_as_string);
        }
        if pos_y.is_none() {
            pos_y = arr.get(1).and_then(num_as_string);
        }
    }
    if let (Some(x), Some(y)) = (pos_x, pos_y) {
        block = upsert_property_line(&block, "position(", &format!("            position({}, {})", x, y));
    }

    if let Some(opacity) = properties.get("opacity").and_then(num_as_string) {
        block = upsert_property_line(&block, "opacity(", &format!("            opacity({})", opacity));
    }

    if let Some(v) = properties
        .get("cornerRadius")
        .or_else(|| properties.get("corner_radius"))
        .and_then(num_as_string)
    {
        block = upsert_property_line(&block, "cornerRadius(", &format!("            cornerRadius({})", v));
    }

    if let Some(v) = properties
        .get("strokeWidth")
        .or_else(|| properties.get("stroke_width"))
        .and_then(num_as_string)
    {
        block = upsert_property_line(&block, "strokeWidth(", &format!("            strokeWidth({})", v));
    }

    if let Some(fill) = properties
        .get("fill")
        .or_else(|| properties.get("fillColor"))
        .or_else(|| properties.get("fill_color"))
        .and_then(|v| v.as_str())
    {
        block = upsert_property_line(&block, "fill(", &format!("            fill({})", fill));
    }

    if let Some(stroke) = properties
        .get("stroke")
        .or_else(|| properties.get("strokeColor"))
        .or_else(|| properties.get("stroke_color"))
        .and_then(|v| v.as_str())
    {
        block = upsert_property_line(&block, "stroke(", &format!("            stroke({})", stroke));
    }

    // Minimal animation support: properties.animate = object or [objects]
    if let Some(anim) = properties.get("animate") {
        match anim {
            Value::Array(items) => {
                for item in items {
                    apply_single_animation_block(&mut block, item);
                }
            }
            _ => apply_single_animation_block(&mut block, anim),
        }
    }

    if let Some(rotation) = properties.get("rotation").and_then(num_as_string) {
        block = upsert_property_line(&block, "rotation(", &format!("            rotation({})", rotation));
    }

    let scale = properties.get("scale").and_then(num_as_string);
    let scale_x = properties
        .get("scaleX")
        .or_else(|| properties.get("scale_x"))
        .and_then(num_as_string);
    let scale_y = properties
        .get("scaleY")
        .or_else(|| properties.get("scale_y"))
        .and_then(num_as_string);

    if let (Some(sx), Some(sy)) = (scale_x.clone(), scale_y.clone()) {
        block = upsert_property_line(&block, "scale(", &format!("            scale({}, {})", sx, sy));
    } else if let Some(s) = scale {
        block = upsert_property_line(&block, "scale(", &format!("            scale({})", s));
    } else if let Some(sx) = scale_x {
        // If only one axis provided, preserve common DSL usage by writing scalar scale.
        block = upsert_property_line(&block, "scale(", &format!("            scale({})", sx));
    }

    if block == original_block {
        return Ok(false);
    }

    src.replace_range(inner_start..inner_end, &block);
    std::fs::write(file, src)
        .with_context(|| format!("failed to write vidra file: {}", file.display()))?;
    Ok(true)
}

pub fn record_layer_edit(project_root: &Path, scene_id: &str, layer_path: &str, properties: Value) -> Result<PathBuf> {
    let path = patches_path(project_root)?;
    let mut patches: Vec<McpPatch> = load_json_vec(&path)?;
    patches.push(McpPatch {
        patch_type: "edit_layer".to_string(),
        scene_id: Some(scene_id.to_string()),
        target: layer_path.to_string(),
        properties,
        created_at: chrono::Utc::now(),
    });
    save_json_vec(&path, &patches)?;
    Ok(path)
}

pub fn record_style_set(project_root: &Path, target_id: &str, style_props: Value) -> Result<PathBuf> {
    let path = patches_path(project_root)?;
    let mut patches: Vec<McpPatch> = load_json_vec(&path)?;
    patches.push(McpPatch {
        patch_type: "set_style".to_string(),
        scene_id: None,
        target: target_id.to_string(),
        properties: style_props,
        created_at: chrono::Utc::now(),
    });
    save_json_vec(&path, &patches)?;
    Ok(path)
}

pub fn register_asset(project_root: &Path, id: &str, asset_path: &str, asset_type: &str) -> Result<PathBuf> {
    let path = assets_path(project_root)?;
    let mut assets: Vec<McpAsset> = load_json_vec(&path)?;
    assets.retain(|a| a.id != id);
    assets.push(McpAsset {
        id: id.to_string(),
        path: asset_path.to_string(),
        asset_type: asset_type.to_string(),
        created_at: chrono::Utc::now(),
    });
    assets.sort_by(|a, b| a.id.cmp(&b.id));
    save_json_vec(&path, &assets)?;
    Ok(path)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create_project_and_add_scene() {
        let root = std::env::temp_dir().join(format!("vidra_mcp_tools_{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(&root).unwrap();

        let old = std::env::current_dir().unwrap();
        std::env::set_current_dir(&root).unwrap();

        let project = create_project("demo_mcp_tools", 1280, 720, 30).unwrap();
        assert!(project.join("main.vidra").exists());

        add_scene_to_vidra_file(&project.join("main.vidra"), "extra", 3.0).unwrap();
        let s = std::fs::read_to_string(project.join("main.vidra")).unwrap();
        assert!(s.contains("scene(\"extra\", 3s)"));

        std::env::set_current_dir(old).unwrap();
        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn apply_layer_properties_updates_main_vidra() {
        let root = std::env::temp_dir().join(format!("vidra_mcp_edit_{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(&root).unwrap();

        let file = root.join("main.vidra");
        std::fs::write(
            &file,
            r#"project(1920, 1080, 60) {
    scene("main", 5s) {
        layer("title") {
            text("Old")
            position(100, 200)
        }
    }
}
"#,
        )
        .unwrap();

        let props = serde_json::json!({
            "text": "New Title",
            "x": 960,
            "y": 540,
            "opacity": 0.8
        });

        let changed = apply_layer_properties_to_vidra_file(&file, "title", &props).unwrap();
        assert!(changed);
        let out = std::fs::read_to_string(&file).unwrap();
        assert!(out.contains("text(\"New Title\")"));
        assert!(out.contains("position(960, 540)"));
        assert!(out.contains("opacity(0.8)"));

        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn apply_layer_text_style_properties_updates_text_call() {
        let root = std::env::temp_dir().join(format!("vidra_mcp_text_style_{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(&root).unwrap();

        let file = root.join("main.vidra");
        std::fs::write(
            &file,
            r#"project(1920, 1080, 60) {
    scene("main", 5s) {
        layer("title") {
            text("Old", font: "Inter", size: 90, color: #FFFFFF)
        }
    }
}
"#,
        )
        .unwrap();

        let props = serde_json::json!({
            "text": "Headline",
            "font": "Roboto",
            "size": 72,
            "color": "#00FFAA"
        });
        let changed = apply_layer_properties_to_vidra_file(&file, "title", &props).unwrap();
        assert!(changed);

        let out = std::fs::read_to_string(&file).unwrap();
        assert!(out.contains("text(\"Headline\", font: \"Roboto\", size: 72, color: #00FFAA)"));

        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn apply_layer_transform_properties_updates_rotation_and_scale() {
        let root = std::env::temp_dir().join(format!("vidra_mcp_transform_{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(&root).unwrap();

        let file = root.join("main.vidra");
        std::fs::write(
            &file,
            r#"project(1920, 1080, 60) {
    scene("main", 5s) {
        layer("card") {
            position(200, 200)
            rotation(0)
            scale(1)
        }
    }
}
"#,
        )
        .unwrap();

        let props = serde_json::json!({
            "rotation": 15,
            "scaleX": 1.2,
            "scaleY": 0.9
        });

        let changed = apply_layer_properties_to_vidra_file(&file, "card", &props).unwrap();
        assert!(changed);

        let out = std::fs::read_to_string(&file).unwrap();
        assert!(out.contains("rotation(15)"));
        assert!(out.contains("scale(1.2, 0.9)"));

        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn apply_layer_shape_style_properties_updates_shape_lines() {
        let root = std::env::temp_dir().join(format!("vidra_mcp_shape_style_{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(&root).unwrap();

        let file = root.join("main.vidra");
        std::fs::write(
            &file,
            r#"project(1920, 1080, 60) {
    scene("main", 5s) {
        layer("chip") {
            rect(400, 120)
            fill(#222222)
            stroke(#FFFFFF)
            strokeWidth(1)
            cornerRadius(8)
        }
    }
}
"#,
        )
        .unwrap();

        let props = serde_json::json!({
            "fillColor": "#FF0066",
            "strokeColor": "#00E5FF",
            "strokeWidth": 3,
            "cornerRadius": 16
        });

        let changed = apply_layer_properties_to_vidra_file(&file, "chip", &props).unwrap();
        assert!(changed);

        let out = std::fs::read_to_string(&file).unwrap();
        assert!(out.contains("fill(#FF0066)"));
        assert!(out.contains("stroke(#00E5FF)"));
        assert!(out.contains("strokeWidth(3)"));
        assert!(out.contains("cornerRadius(16)"));

        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn apply_layer_animation_properties_inserts_animate_block() {
        let root = std::env::temp_dir().join(format!("vidra_mcp_anim_{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(&root).unwrap();

        let file = root.join("main.vidra");
        std::fs::write(
            &file,
            r#"project(1920, 1080, 60) {
    scene("main", 5s) {
        layer("title") {
            text("Hello")
        }
    }
}
"#,
        )
        .unwrap();

        let props = serde_json::json!({
            "animate": {
                "property": "Opacity",
                "from": 0,
                "to": 1,
                "start": 0,
                "end": 1,
                "easing": "EaseOut"
            }
        });

        let changed = apply_layer_properties_to_vidra_file(&file, "title", &props).unwrap();
        assert!(changed);
        let out = std::fs::read_to_string(&file).unwrap();
        assert!(out.contains("animate(Opacity)"));
        assert!(out.contains("0s -> 0"));
        assert!(out.contains("1s -> 1 ~ EaseOut"));

        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn apply_layer_animation_properties_replaces_existing_block() {
        let root = std::env::temp_dir().join(format!("vidra_mcp_anim_replace_{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(&root).unwrap();

        let file = root.join("main.vidra");
        std::fs::write(
            &file,
            r#"project(1920, 1080, 60) {
    scene("main", 5s) {
        layer("title") {
            text("Hello")
            animate(Opacity) {
                0s -> 0
                1s -> 1 ~ EaseInOut
            }
        }
    }
}
"#,
        )
        .unwrap();

        let props = serde_json::json!({
            "animate": {
                "property": "Opacity",
                "from": 0.2,
                "to": 0.9,
                "start": 0.5,
                "end": 1.5,
                "easing": "Linear"
            }
        });

        let changed = apply_layer_properties_to_vidra_file(&file, "title", &props).unwrap();
        assert!(changed);
        let out = std::fs::read_to_string(&file).unwrap();
        assert_eq!(out.matches("animate(Opacity)").count(), 1);
        assert!(out.contains("0.5s -> 0.2"));
        assert!(out.contains("1.5s -> 0.9 ~ Linear"));

        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn apply_layer_animation_array_inserts_multiple_blocks() {
        let root = std::env::temp_dir().join(format!("vidra_mcp_anim_array_{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(&root).unwrap();

        let file = root.join("main.vidra");
        std::fs::write(
            &file,
            r#"project(1920, 1080, 60) {
    scene("main", 5s) {
        layer("title") {
            text("Hello")
        }
    }
}
"#,
        )
        .unwrap();

        let props = serde_json::json!({
            "animate": [
                {
                    "property": "Opacity",
                    "from": 0,
                    "to": 1,
                    "start": 0,
                    "end": 1,
                    "easing": "EaseOut"
                },
                {
                    "property": "X",
                    "from": 0,
                    "to": 100,
                    "start": 0,
                    "end": 2,
                    "easing": "Linear"
                }
            ]
        });

        let changed = apply_layer_properties_to_vidra_file(&file, "title", &props).unwrap();
        assert!(changed);
        let out = std::fs::read_to_string(&file).unwrap();
        assert!(out.contains("animate(Opacity)"));
        assert!(out.contains("animate(X)"));
        assert!(out.contains("2s -> 100 ~ Linear"));

        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn apply_layer_animation_delay_duration_and_loop() {
        let root = std::env::temp_dir().join(format!("vidra_mcp_anim_timing_{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(&root).unwrap();

        let file = root.join("main.vidra");
        std::fs::write(
            &file,
            r#"project(1920, 1080, 60) {
    scene("main", 5s) {
        layer("title") {
            text("Hello")
        }
    }
}
"#,
        )
        .unwrap();

        let props = serde_json::json!({
            "animate": {
                "property": "Opacity",
                "from": 0,
                "to": 1,
                "delay": 0.25,
                "duration": 1.75,
                "loop": 3,
                "easing": "EaseIn"
            }
        });

        let changed = apply_layer_properties_to_vidra_file(&file, "title", &props).unwrap();
        assert!(changed);

        let out = std::fs::read_to_string(&file).unwrap();
        assert!(out.contains("animate(Opacity)"));
        assert!(out.contains("loop(3)"));
        assert!(out.contains("0.25s -> 0"));
        assert!(out.contains("2s -> 1 ~ EaseIn"));

        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn apply_layer_animation_alias_timing_and_repeat() {
        let root = std::env::temp_dir().join(format!("vidra_mcp_anim_alias_{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(&root).unwrap();

        let file = root.join("main.vidra");
        std::fs::write(
            &file,
            r#"project(1920, 1080, 60) {
    scene("main", 5s) {
        layer("title") {
            text("Hello")
        }
    }
}
"#,
        )
        .unwrap();

        let props = serde_json::json!({
            "animate": {
                "property": "X",
                "from": 100,
                "to": 500,
                "startTime": 1.5,
                "durationMs": 750,
                "repeat": true,
                "easing": "Linear"
            }
        });

        let changed = apply_layer_properties_to_vidra_file(&file, "title", &props).unwrap();
        assert!(changed);

        let out = std::fs::read_to_string(&file).unwrap();
        assert!(out.contains("animate(X)"));
        assert!(out.contains("loop(Infinite)"));
        assert!(out.contains("1.5s -> 100"));
        assert!(out.contains("2.25s -> 500 ~ Linear"));

        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn apply_layer_animation_property_alias_maps_to_canonical() {
        let root = std::env::temp_dir().join(format!("vidra_mcp_anim_prop_alias_{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(&root).unwrap();

        let file = root.join("main.vidra");
        std::fs::write(
            &file,
            r#"project(1920, 1080, 60) {
    scene("main", 5s) {
        layer("title") {
            text("Hello")
            animate(Opacity) {
                0s -> 0
                1s -> 1 ~ EaseInOut
            }
        }
    }
}
"#,
        )
        .unwrap();

        let props = serde_json::json!({
            "animate": [
                {
                    "property": "alpha",
                    "from": 0.1,
                    "to": 0.95,
                    "start": 0.2,
                    "end": 1.2,
                    "easing": "Linear"
                },
                {
                    "property": "posX",
                    "from": 0,
                    "to": 250,
                    "start": 0,
                    "end": 1,
                    "easing": "EaseOut"
                }
            ]
        });

        let changed = apply_layer_properties_to_vidra_file(&file, "title", &props).unwrap();
        assert!(changed);

        let out = std::fs::read_to_string(&file).unwrap();
        assert_eq!(out.matches("animate(Opacity)").count(), 1);
        assert!(out.contains("0.2s -> 0.1"));
        assert!(out.contains("1.2s -> 0.95 ~ Linear"));
        assert!(out.contains("animate(X)"));

        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn apply_layer_animation_value_aliases_are_supported() {
        let root = std::env::temp_dir().join(format!("vidra_mcp_anim_value_alias_{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(&root).unwrap();

        let file = root.join("main.vidra");
        std::fs::write(
            &file,
            r#"project(1920, 1080, 60) {
    scene("main", 5s) {
        layer("title") {
            text("Hello")
        }
    }
}
"#,
        )
        .unwrap();

        let props = serde_json::json!({
            "animate": {
                "property": "rotate",
                "startValue": -15,
                "endValue": 45,
                "start": 0,
                "end": 1.25,
                "easing": "EaseOut"
            }
        });

        let changed = apply_layer_properties_to_vidra_file(&file, "title", &props).unwrap();
        assert!(changed);

        let out = std::fs::read_to_string(&file).unwrap();
        assert!(out.contains("animate(Rotation)"));
        assert!(out.contains("0s -> -15"));
        assert!(out.contains("1.25s -> 45 ~ EaseOut"));

        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn apply_layer_animation_easing_aliases_are_normalized() {
        let root = std::env::temp_dir().join(format!("vidra_mcp_anim_easing_alias_{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(&root).unwrap();

        let file = root.join("main.vidra");
        std::fs::write(
            &file,
            r#"project(1920, 1080, 60) {
    scene("main", 5s) {
        layer("title") {
            text("Hello")
        }
    }
}
"#,
        )
        .unwrap();

        let props = serde_json::json!({
            "animate": {
                "property": "opacity",
                "from": 0,
                "to": 1,
                "start": 0,
                "end": 1,
                "timingFunction": "ease-out"
            }
        });

        let changed = apply_layer_properties_to_vidra_file(&file, "title", &props).unwrap();
        assert!(changed);

        let out = std::fs::read_to_string(&file).unwrap();
        assert!(out.contains("animate(Opacity)"));
        assert!(out.contains("1s -> 1 ~ EaseOut"));

        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn apply_layer_animation_keyframes_are_rendered_and_sorted() {
        let root = std::env::temp_dir().join(format!("vidra_mcp_anim_keyframes_{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(&root).unwrap();

        let file = root.join("main.vidra");
        std::fs::write(
            &file,
            r#"project(1920, 1080, 60) {
    scene("main", 5s) {
        layer("title") {
            text("Hello")
        }
    }
}
"#,
        )
        .unwrap();

        let props = serde_json::json!({
            "animate": {
                "property": "posY",
                "curve": "ease-in",
                "keyframes": [
                    { "time": 1.5, "value": 90 },
                    { "time": 0.0, "value": 10 },
                    { "time": 0.5, "value": 40 }
                ]
            }
        });

        let changed = apply_layer_properties_to_vidra_file(&file, "title", &props).unwrap();
        assert!(changed);

        let out = std::fs::read_to_string(&file).unwrap();
        assert!(out.contains("animate(Y)"));
        let i0 = out.find("0s -> 10").unwrap();
        let i1 = out.find("0.5s -> 40 ~ EaseIn").unwrap();
        let i2 = out.find("1.5s -> 90 ~ EaseIn").unwrap();
        assert!(i0 < i1 && i1 < i2);

        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn apply_layer_animation_keyframes_respect_per_frame_easing() {
        let root = std::env::temp_dir().join(format!("vidra_mcp_anim_keyframe_easing_{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(&root).unwrap();

        let file = root.join("main.vidra");
        std::fs::write(
            &file,
            r#"project(1920, 1080, 60) {
    scene("main", 5s) {
        layer("title") {
            text("Hello")
        }
    }
}
"#,
        )
        .unwrap();

        let props = serde_json::json!({
            "animate": {
                "property": "x",
                "curve": "linear",
                "keyframes": [
                    { "time": 0, "value": 0 },
                    { "time": 1, "value": 200, "easing": "ease-out" },
                    { "time": 2, "value": 300 }
                ]
            }
        });

        let changed = apply_layer_properties_to_vidra_file(&file, "title", &props).unwrap();
        assert!(changed);

        let out = std::fs::read_to_string(&file).unwrap();
        assert!(out.contains("animate(X)"));
        assert!(out.contains("1s -> 200 ~ EaseOut"));
        assert!(out.contains("2s -> 300 ~ Linear"));

        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn apply_layer_animation_keyframe_percent_times_map_to_span() {
        let root = std::env::temp_dir().join(format!("vidra_mcp_anim_keyframe_percent_{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(&root).unwrap();

        let file = root.join("main.vidra");
        std::fs::write(
            &file,
            r#"project(1920, 1080, 60) {
    scene("main", 5s) {
        layer("title") {
            text("Hello")
        }
    }
}
"#,
        )
        .unwrap();

        let props = serde_json::json!({
            "animate": {
                "property": "opacity",
                "start": 1,
                "duration": 4,
                "curve": "linear",
                "keyframes": [
                    { "time": "0%", "value": 0 },
                    { "time": "50%", "value": 0.5 },
                    { "time": "100%", "value": 1 }
                ]
            }
        });

        let changed = apply_layer_properties_to_vidra_file(&file, "title", &props).unwrap();
        assert!(changed);

        let out = std::fs::read_to_string(&file).unwrap();
        assert!(out.contains("animate(Opacity)"));
        assert!(out.contains("1s -> 0"));
        assert!(out.contains("3s -> 0.5 ~ Linear"));
        assert!(out.contains("5s -> 1 ~ Linear"));

        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn apply_layer_animation_object_values_map_by_property() {
        let root = std::env::temp_dir().join(format!("vidra_mcp_anim_object_values_{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(&root).unwrap();

        let file = root.join("main.vidra");
        std::fs::write(
            &file,
            r#"project(1920, 1080, 60) {
    scene("main", 5s) {
        layer("title") {
            text("Hello")
        }
    }
}
"#,
        )
        .unwrap();

        let props = serde_json::json!({
            "animate": {
                "property": "posX",
                "from": { "x": 10, "y": 999 },
                "to": { "x": 300, "y": 888 },
                "start": 0,
                "end": 2,
                "curve": "linear"
            }
        });

        let changed = apply_layer_properties_to_vidra_file(&file, "title", &props).unwrap();
        assert!(changed);

        let out = std::fs::read_to_string(&file).unwrap();
        assert!(out.contains("animate(X)"));
        assert!(out.contains("0s -> 10"));
        assert!(out.contains("2s -> 300 ~ Linear"));
        assert!(!out.contains("999"));
        assert!(!out.contains("888"));

        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn apply_layer_animation_scale_uses_composite_object_average() {
        let root = std::env::temp_dir().join(format!("vidra_mcp_anim_scale_object_{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(&root).unwrap();

        let file = root.join("main.vidra");
        std::fs::write(
            &file,
            r#"project(1920, 1080, 60) {
    scene("main", 5s) {
        layer("title") {
            text("Hello")
        }
    }
}
"#,
        )
        .unwrap();

        let props = serde_json::json!({
            "animate": {
                "property": "scale",
                "from": { "scaleX": 1.2, "scaleY": 0.8 },
                "to": { "x": 2.0, "y": 1.0 },
                "start": 0,
                "end": 2,
                "curve": "linear"
            }
        });

        let changed = apply_layer_properties_to_vidra_file(&file, "title", &props).unwrap();
        assert!(changed);

        let out = std::fs::read_to_string(&file).unwrap();
        assert!(out.contains("animate(Scale)"));
        assert!(out.contains("0s -> 1"));
        assert!(out.contains("2s -> 1.5 ~ Linear"));

        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn apply_layer_animation_scale_reads_array_values() {
        let root = std::env::temp_dir().join(format!("vidra_mcp_anim_scale_array_{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(&root).unwrap();

        let file = root.join("main.vidra");
        std::fs::write(
            &file,
            r#"project(1920, 1080, 60) {
    scene("main", 5s) {
        layer("title") {
            text("Hello")
        }
    }
}
"#,
        )
        .unwrap();

        let props = serde_json::json!({
            "animate": {
                "property": "scale",
                "keyframes": [
                    { "time": 0, "value": [1.0, 1.2] },
                    { "time": 1, "value": [2.0, 1.0] }
                ],
                "curve": "linear"
            }
        });

        let changed = apply_layer_properties_to_vidra_file(&file, "title", &props).unwrap();
        assert!(changed);

        let out = std::fs::read_to_string(&file).unwrap();
        assert!(out.contains("animate(Scale)"));
        assert!(out.contains("0s -> 1.1"));
        assert!(out.contains("1s -> 1.5 ~ Linear"));

        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn apply_layer_animation_keyframes_without_time_are_distributed() {
        let root = std::env::temp_dir().join(format!("vidra_mcp_anim_keyframes_implicit_{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(&root).unwrap();

        let file = root.join("main.vidra");
        std::fs::write(
            &file,
            r#"project(1920, 1080, 60) {
    scene("main", 5s) {
        layer("title") {
            text("Hello")
        }
    }
}
"#,
        )
        .unwrap();

        let props = serde_json::json!({
            "animate": {
                "property": "opacity",
                "start": 1,
                "end": 4,
                "curve": "linear",
                "keyframes": [
                    { "value": 0.0 },
                    { "value": 0.5 },
                    { "value": 1.0 }
                ]
            }
        });

        let changed = apply_layer_properties_to_vidra_file(&file, "title", &props).unwrap();
        assert!(changed);

        let out = std::fs::read_to_string(&file).unwrap();
        assert!(out.contains("1s -> 0"));
        assert!(out.contains("2.5s -> 0.5 ~ Linear"));
        assert!(out.contains("4s -> 1 ~ Linear"));

        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn apply_layer_animation_keyframes_support_offset_fraction() {
        let root = std::env::temp_dir().join(format!("vidra_mcp_anim_keyframes_offset_{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(&root).unwrap();

        let file = root.join("main.vidra");
        std::fs::write(
            &file,
            r#"project(1920, 1080, 60) {
    scene("main", 5s) {
        layer("title") {
            text("Hello")
        }
    }
}
"#,
        )
        .unwrap();

        let props = serde_json::json!({
            "animate": {
                "property": "x",
                "start": 2,
                "duration": 4,
                "curve": "ease-out",
                "keyframes": [
                    { "offset": 0.0, "value": 10 },
                    { "offset": 0.25, "value": 50 },
                    { "offset": 1.0, "value": 100 }
                ]
            }
        });

        let changed = apply_layer_properties_to_vidra_file(&file, "title", &props).unwrap();
        assert!(changed);

        let out = std::fs::read_to_string(&file).unwrap();
        assert!(out.contains("animate(X)"));
        assert!(out.contains("2s -> 10"));
        assert!(out.contains("3s -> 50 ~ EaseOut"));
        assert!(out.contains("6s -> 100 ~ EaseOut"));

        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn apply_layer_animation_keyframes_mixed_times_are_interpolated() {
        let root = std::env::temp_dir().join(format!("vidra_mcp_anim_keyframes_mixed_{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(&root).unwrap();

        let file = root.join("main.vidra");
        std::fs::write(
            &file,
            r#"project(1920, 1080, 60) {
    scene("main", 5s) {
        layer("title") {
            text("Hello")
        }
    }
}
"#,
        )
        .unwrap();

        let props = serde_json::json!({
            "animate": {
                "property": "x",
                "start": 0,
                "end": 10,
                "curve": "linear",
                "keyframes": [
                    { "time": 0, "value": 0 },
                    { "value": 40 },
                    { "offset": 0.8, "value": 80 },
                    { "value": 90 },
                    { "time": 10, "value": 100 }
                ]
            }
        });

        let changed = apply_layer_properties_to_vidra_file(&file, "title", &props).unwrap();
        assert!(changed);

        let out = std::fs::read_to_string(&file).unwrap();
        assert!(out.contains("animate(X)"));
        assert!(out.contains("0s -> 0"));
        assert!(out.contains("4s -> 40 ~ Linear"));
        assert!(out.contains("8s -> 80 ~ Linear"));
        assert!(out.contains("9s -> 90 ~ Linear"));
        assert!(out.contains("10s -> 100 ~ Linear"));

        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn apply_layer_animation_keyframes_support_offset_percent_number() {
        let root = std::env::temp_dir().join(format!("vidra_mcp_anim_keyframes_offset_percent_num_{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(&root).unwrap();

        let file = root.join("main.vidra");
        std::fs::write(
            &file,
            r#"project(1920, 1080, 60) {
    scene("main", 5s) {
        layer("title") {
            text("Hello")
        }
    }
}
"#,
        )
        .unwrap();

        let props = serde_json::json!({
            "animate": {
                "property": "x",
                "start": 2,
                "end": 6,
                "curve": "linear",
                "keyframes": [
                    { "offset": 0, "value": 10 },
                    { "offset": 25, "value": 30 },
                    { "offset": 100, "value": 90 }
                ]
            }
        });

        let changed = apply_layer_properties_to_vidra_file(&file, "title", &props).unwrap();
        assert!(changed);

        let out = std::fs::read_to_string(&file).unwrap();
        assert!(out.contains("animate(X)"));
        assert!(out.contains("2s -> 10"));
        assert!(out.contains("3s -> 30 ~ Linear"));
        assert!(out.contains("6s -> 90 ~ Linear"));

        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn apply_layer_animation_keyframes_clamp_out_of_range_offsets() {
        let root = std::env::temp_dir().join(format!("vidra_mcp_anim_keyframes_offset_clamp_{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(&root).unwrap();

        let file = root.join("main.vidra");
        std::fs::write(
            &file,
            r#"project(1920, 1080, 60) {
    scene("main", 5s) {
        layer("title") {
            text("Hello")
        }
    }
}
"#,
        )
        .unwrap();

        let props = serde_json::json!({
            "animate": {
                "property": "opacity",
                "start": 2,
                "end": 6,
                "curve": "linear",
                "keyframes": [
                    { "offset": -10, "value": 0.0 },
                    { "offset": "120%", "value": 1.0 },
                    { "offset": 1.5, "value": 0.5 }
                ]
            }
        });

        let changed = apply_layer_properties_to_vidra_file(&file, "title", &props).unwrap();
        assert!(changed);

        let out = std::fs::read_to_string(&file).unwrap();
        assert!(out.contains("animate(Opacity)"));
        assert!(out.contains("2s -> 0"));
        assert!(out.contains("6s -> 1 ~ Linear"));
        assert!(out.contains("6s -> 0.5 ~ Linear"));

        let _ = std::fs::remove_dir_all(&root);
    }
}
