//! LRC 歌词解析与时间轴同步。
//!
//! 提供 LRC 格式解析、按播放时间定位当前行、翻译合并、时间偏移等能力，
//! 供播放器在播放时高亮当前歌词行。仅依赖 std，无外部依赖。

use crate::models::{Lyric, LyricLine};

/// 尝试将行首的 `[...]` 标签解析为时间戳，返回 `(time_ms, 剩余字符串)`。
///
/// 支持 `[mm:ss]`、`[mm:ss.xx]`、`[mm:ss.xxx]` 格式，按
/// `time_ms = mm*60000 + ss*1000 + xx` 计算（xx 为小数部分整数，直接作为毫秒）。
/// 若标签非合法时间戳（如 `ti`、`ar`、`al`、`by`、`offset` 等元数据），返回 `None`。
///
/// 通过字节扫描 `[` `]` 配对，内部按 `:` 分割 mm:ss[.xx]，
/// 仅在 ASCII 边界切分，避免 UTF-8 切片 panic。
fn parse_timestamp_tag(line: &str) -> Option<(u64, &str)> {
    let bytes = line.as_bytes();
    if bytes.is_empty() || bytes[0] != b'[' {
        return None;
    }
    let close = line.find(']')?;
    let inner = &line[1..close];
    let rest = &line[close + 1..];
    // 内部按 ':' 分割为 mm:ss[.xx]
    let colon = inner.find(':')?;
    let mm_str = &inner[..colon];
    let after = &inner[colon + 1..];
    // after 形如 ss.xx / ss.xxx / ss
    let (ss_str, frac_str) = match after.find('.') {
        Some(dot) => (&after[..dot], Some(&after[dot + 1..])),
        None => (after, None),
    };
    // mm、ss 必须为纯数字（过滤掉 ti/ar/al 等元数据键）
    if mm_str.is_empty() || !mm_str.bytes().all(|b| b.is_ascii_digit()) {
        return None;
    }
    if ss_str.is_empty() || !ss_str.bytes().all(|b| b.is_ascii_digit()) {
        return None;
    }
    let mm: u64 = mm_str.parse().ok()?;
    let ss: u64 = ss_str.parse().ok()?;
    let ms: u64 = match frac_str {
        Some(s) if !s.is_empty() && s.bytes().all(|b| b.is_ascii_digit()) => s.parse().ok()?,
        _ => 0,
    };
    Some((mm * 60000 + ss * 1000 + ms, rest))
}

/// 解析 LRC 文本为 [`Lyric`]。
///
/// - 逐行解析，每行可能含多个时间戳前缀（如 `[00:12.34][00:45.00]同一行` 表示
///   该行在两个时间点出现，会展开为两行）。
/// - 元数据行（`ti`/`ar`/`al`/`by`/`offset` 等非时间戳 `[key:value]`）不计入歌词行。
/// - 无时间戳且不以 `[` 开头的非空行保留为 `time_ms = None` 的纯文本歌词行。
/// - 空行（仅空白）跳过，文本去除首尾空白。
/// - 行按 `time_ms` 升序排序，`None` 行置于末尾（保持原相对顺序）。
pub fn parse_lrc(content: &str) -> Lyric {
    let mut timed: Vec<LyricLine> = Vec::new();
    let mut untimed: Vec<LyricLine> = Vec::new();

    for raw_line in content.lines() {
        let line = raw_line.trim();
        if line.is_empty() {
            continue;
        }
        // 扫描行首连续的 [...] 标签，收集所有合法时间戳
        let mut times: Vec<u64> = Vec::new();
        let mut cursor = line;
        let mut had_bracket = false;
        while cursor.starts_with('[') {
            had_bracket = true;
            match parse_timestamp_tag(cursor) {
                Some((t, rest)) => {
                    times.push(t);
                    cursor = rest;
                }
                None => break,
            }
        }

        if !times.is_empty() {
            // 至少一个合法时间戳：每个时间戳生成一行，共享剩余文本
            let text = cursor.trim().to_string();
            for t in times {
                timed.push(LyricLine {
                    time_ms: Some(t),
                    text: text.clone(),
                });
            }
        } else if !had_bracket {
            // 不以 [ 开头：纯文本歌词行
            untimed.push(LyricLine {
                time_ms: None,
                text: line.to_string(),
            });
        }
        // had_bracket 但无合法时间戳：视为元数据行，跳过
    }

    // 时间戳行按 time_ms 升序（稳定排序，同序保持原相对顺序）
    timed.sort_by_key(|l| l.time_ms.unwrap_or(0));
    let mut lines = timed;
    lines.extend(untimed);

    Lyric {
        lines,
        translation: None,
    }
}

/// 返回当前播放时间应高亮的行索引。
///
/// 规则：返回 `time_ms` 最末一行（已按升序排序）的索引；
/// 无时间戳（`None`）的行不参与定位，但保留在列表中。
/// 若没有满足条件的行（如当前时间早于第一行），返回 `None`。
pub fn line_at_time(lyric: &Lyric, time_ms: u64) -> Option<usize> {
    let mut result: Option<usize> = None;
    for (i, line) in lyric.lines.iter().enumerate() {
        match line.time_ms {
            Some(t) if t <= time_ms => result = Some(i),
            Some(_) => break, // 后续时间戳更大，无需继续
            None => continue,  // 无时间戳行不参与定位
        }
    }
    result
}

/// 合并主歌词与翻译。
///
/// - 若 `translation` 为 `None`，返回主歌词副本（`translation = None`）。
/// - 若有翻译，按 `time_ms` 对齐：主歌词每行查找翻译中 `time_ms` 相同的行作为翻译文本，
///   返回 `Lyric.lines = main.lines`，`translation = 对齐后的 Vec<LyricLine>`
///   （主歌词中未匹配到翻译的行用空文本占位，`time_ms` 与主歌词对应行一致）。
pub fn merge_translation(main: &Lyric, translation: Option<&Lyric>) -> Lyric {
    let translation = match translation {
        None => {
            return Lyric {
                lines: main.lines.clone(),
                translation: None,
            }
        }
        Some(t) => t,
    };
    let aligned: Vec<LyricLine> = main
        .lines
        .iter()
        .map(|line| {
            let trans_text = match line.time_ms {
                Some(t) => translation
                    .lines
                    .iter()
                    .find(|tl| tl.time_ms == Some(t))
                    .map(|tl| tl.text.clone())
                    .unwrap_or_default(),
                None => String::new(),
            };
            LyricLine {
                time_ms: line.time_ms,
                text: trans_text,
            }
        })
        .collect();
    Lyric {
        lines: main.lines.clone(),
        translation: Some(aligned),
    }
}

/// 对所有 `time_ms` 应用时间偏移（毫秒，可正可负）。
///
/// 同时作用于歌词行与翻译行；下溢保护：若 `time_ms + offset_ms < 0`，则设为 `0`。
/// `time_ms` 为 `None` 的行保持不变。
pub fn apply_offset(lyric: &mut Lyric, offset_ms: i64) {
    let shift = |t: Option<u64>| -> Option<u64> {
        t.map(|v| {
            let new_v = v as i64 + offset_ms;
            if new_v < 0 {
                0
            } else {
                new_v as u64
            }
        })
    };
    for line in lyric.lines.iter_mut() {
        line.time_ms = shift(line.time_ms);
    }
    if let Some(trans) = lyric.translation.as_mut() {
        for line in trans.iter_mut() {
            line.time_ms = shift(line.time_ms);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 解析含多时间戳行、元数据行、无时间戳行的完整 LRC，验证行数与 time_ms。
    #[test]
    fn test_parse_lrc_full() {
        let content = "\
[ti:歌曲名]
[ar:艺术家]
[al:专辑]
[by:制作人]
[offset: 250]
[00:12.34][00:45.00]同一行歌词
[00:15.67]第二行歌词
[00:20.00]第三行歌词
纯文本歌词行";
        let lyric = parse_lrc(content);
        // 5 行元数据被跳过；多时间戳行展开为 2 行；普通时间戳 2 行；纯文本 1 行 = 5 行
        assert_eq!(lyric.lines.len(), 5);
        // 按时间升序：12034, 15067, 20000, 45000, None
        assert_eq!(lyric.lines[0].time_ms, Some(12034));
        assert_eq!(lyric.lines[0].text, "同一行歌词");
        assert_eq!(lyric.lines[1].time_ms, Some(15067));
        assert_eq!(lyric.lines[1].text, "第二行歌词");
        assert_eq!(lyric.lines[2].time_ms, Some(20000));
        assert_eq!(lyric.lines[2].text, "第三行歌词");
        assert_eq!(lyric.lines[3].time_ms, Some(45000));
        assert_eq!(lyric.lines[3].text, "同一行歌词");
        assert_eq!(lyric.lines[4].time_ms, None);
        assert_eq!(lyric.lines[4].text, "纯文本歌词行");
        // 默认无翻译
        assert!(lyric.translation.is_none());
    }

    /// 支持不同时间戳精度：`[mm:ss]` / `[mm:ss.xx]` / `[mm:ss.xxx]`。
    #[test]
    fn test_parse_lrc_timestamp_precision() {
        let lyric = parse_lrc("[01:30]分秒\n[00:05.5]带一位小数\n[00:01.234]三位小数");
        assert_eq!(lyric.lines.len(), 3);
        // 解析后按 time_ms 升序排序：1234, 5005, 90000
        // [00:01.234] = 1*1000 + 234 = 1234
        assert_eq!(lyric.lines[0].time_ms, Some(1234));
        assert_eq!(lyric.lines[0].text, "三位小数");
        // [00:05.5] = 5*1000 + 5 = 5005
        assert_eq!(lyric.lines[1].time_ms, Some(5005));
        assert_eq!(lyric.lines[1].text, "带一位小数");
        // [01:30] = 1*60000 + 30*1000 = 90000
        assert_eq!(lyric.lines[2].time_ms, Some(90000));
        assert_eq!(lyric.lines[2].text, "分秒");
    }

    /// line_at_time 按播放时间正确定位当前行。
    #[test]
    fn test_line_at_time() {
        let lyric = parse_lrc(
            "[00:12.34]第一行\n[00:15.67]第二行\n[00:20.00]第三行\n纯文本行",
        );
        // 早于第一行
        assert_eq!(line_at_time(&lyric, 5000), None);
        // 第一行期间
        assert_eq!(line_at_time(&lyric, 12034), Some(0));
        assert_eq!(line_at_time(&lyric, 14000), Some(0));
        // 第二行期间
        assert_eq!(line_at_time(&lyric, 15067), Some(1));
        assert_eq!(line_at_time(&lyric, 18000), Some(1));
        // 第三行期间
        assert_eq!(line_at_time(&lyric, 20000), Some(2));
        assert_eq!(line_at_time(&lyric, 30000), Some(2));
    }

    /// merge_translation 按 time_ms 对齐，未匹配行用空文本占位。
    #[test]
    fn test_merge_translation() {
        let main = parse_lrc("[00:12.34]第一行\n[00:15.67]第二行\n[00:20.00]第三行");
        let trans = parse_lrc("[00:12.34]First line\n[00:20.00]Third line");
        let merged = merge_translation(&main, Some(&trans));
        // 主歌词不变
        assert_eq!(merged.lines.len(), 3);
        assert_eq!(merged.lines[0].text, "第一行");
        assert_eq!(merged.lines[1].text, "第二行");
        assert_eq!(merged.lines[2].text, "第三行");
        // 翻译对齐
        let trans_lines = merged.translation.as_ref().expect("翻译应为 Some");
        assert_eq!(trans_lines.len(), 3);
        assert_eq!(trans_lines[0].time_ms, Some(12034));
        assert_eq!(trans_lines[0].text, "First line");
        // 第二行无对应翻译：空文本占位
        assert_eq!(trans_lines[1].time_ms, Some(15067));
        assert_eq!(trans_lines[1].text, "");
        assert_eq!(trans_lines[2].time_ms, Some(20000));
        assert_eq!(trans_lines[2].text, "Third line");
        // translation 为 None 时返回主歌词副本
        let no_trans = merge_translation(&main, None);
        assert!(no_trans.translation.is_none());
        assert_eq!(no_trans.lines, main.lines);
    }

    /// apply_offset 正/负偏移与下溢保护。
    #[test]
    fn test_apply_offset() {
        let mut lyric = parse_lrc("[00:12.34]第一行\n[00:15.67]第二行\n纯文本行");
        assert_eq!(lyric.lines[0].time_ms, Some(12034));
        assert_eq!(lyric.lines[1].time_ms, Some(15067));
        assert_eq!(lyric.lines[2].time_ms, None);
        // 正偏移
        apply_offset(&mut lyric, 1000);
        assert_eq!(lyric.lines[0].time_ms, Some(13034));
        assert_eq!(lyric.lines[1].time_ms, Some(16067));
        assert_eq!(lyric.lines[2].time_ms, None); // None 保持不变
        // 负偏移（不溢出）
        apply_offset(&mut lyric, -3000);
        assert_eq!(lyric.lines[0].time_ms, Some(10034));
        assert_eq!(lyric.lines[1].time_ms, Some(13067));
        // 下溢保护
        apply_offset(&mut lyric, -1_000_000);
        assert_eq!(lyric.lines[0].time_ms, Some(0));
        assert_eq!(lyric.lines[1].time_ms, Some(0));
    }

    /// apply_offset 同步作用于翻译行。
    #[test]
    fn test_apply_offset_translation() {
        let main = parse_lrc("[00:12.34]第一行\n[00:15.67]第二行");
        let trans = parse_lrc("[00:12.34]First\n[00:15.67]Second");
        let mut merged = merge_translation(&main, Some(&trans));
        apply_offset(&mut merged, 1000);
        let trans_lines = merged.translation.as_ref().unwrap();
        assert_eq!(merged.lines[0].time_ms, Some(13034));
        assert_eq!(trans_lines[0].time_ms, Some(13034));
        assert_eq!(trans_lines[1].time_ms, Some(16067));
    }

    /// 空字符串与仅空白输入返回空 Lyric。
    #[test]
    fn test_parse_lrc_empty() {
        let lyric = parse_lrc("");
        assert!(lyric.lines.is_empty());
        assert!(lyric.translation.is_none());
        assert_eq!(line_at_time(&lyric, 1000), None);

        let ws = parse_lrc("   \n\n  \t  \n");
        assert!(ws.lines.is_empty());
    }
}
