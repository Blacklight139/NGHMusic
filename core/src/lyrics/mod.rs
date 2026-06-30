//! 歌词解析（LRC）与时间轴同步。

use crate::models::{LyricLine};

/// 解析 LRC 文本为带时间轴的歌词行
pub fn parse_lrc(text: &str) -> Vec<LyricLine> {
    let mut out = Vec::new();
    let tag_re = regex::Regex::new(r"\[(\d+):(\d+)(?:[.:](\d+))?\]").unwrap();
    for line in text.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        // 提取所有时间标签
        let times: Vec<f64> = tag_re
            .captures_iter(line)
            .filter_map(|c| {
                let m: u64 = c.get(1)?.as_str().parse().ok()?;
                let s: u64 = c.get(2)?.as_str().parse().ok()?;
                let ms: u64 = c.get(3).map(|x| x.as_str()).and_then(|s| {
                    if s.len() <= 2 {
                        s.parse::<u64>().ok().map(|v| v * 10)
                    } else {
                        s[..3].parse().ok()
                    }
                }).unwrap_or(0);
                Some(m as f64 * 60.0 + s as f64 + ms as f64 / 1000.0)
            })
            .collect();
        if times.is_empty() {
            continue;
        }
        let text_part = tag_re.replace_all(line, "").trim().to_string();
        for t in times {
            out.push(LyricLine {
                time: t,
                text: text_part.clone(),
                translation: None,
            });
        }
    }
    out.sort_by(|a, b| a.time.partial_cmp(&b.time).unwrap_or(std::cmp::Ordering::Equal));
    out
}

/// 在时间轴上定位当前应高亮的行索引
pub fn locate(lines: &[LyricLine], position_sec: f64) -> Option<usize> {
    let mut idx = None;
    for (i, l) in lines.iter().enumerate() {
        if l.time <= position_sec {
            idx = Some(i);
        } else {
            break;
        }
    }
    idx
}
