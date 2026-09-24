use unicode_bidi::BidiInfo;

/// Checks if a string contains any Right-to-Left (RTL) characters.
pub fn is_rtl(text: &str) -> bool {
    text.chars().any(|ch| matches!(ch as u32,
        0x0590..=0x08FF
        | 0xFB50..=0xFDFF
        | 0xFE70..=0xFEFF
    ))
}

/// Checks if a string contains any Arabic characters.
pub fn contains_arabic(text: &str) -> bool {
    text.chars().any(|ch| matches!(ch as u32,
        0x0600..=0x06FF
        | 0x0750..=0x077F
        | 0x08A0..=0x08FF
        | 0xFB50..=0xFDFF
        | 0xFE70..=0xFEFF
    ))
}

/// A line of text prepared for visual Left-to-Right layout in GPUI.
#[derive(Debug, Clone)]
pub struct BidiPreparedLine {
    /// The visually reordered text string to pass to the platform text shaper.
    pub visual_text: String,
    /// Maps byte offsets in `visual_text` to byte offsets in the original logical text.
    pub visual_to_logical: Vec<usize>,
}

fn is_harakat(ch: char) -> bool {
    matches!(ch as u32,
        0x0610..=0x061A
        | 0x064B..=0x065F
        | 0x0670
        | 0x06D6..=0x06DC
        | 0x06DF..=0x06E8
        | 0x06EA..=0x06ED
        | 0x08D4..=0x08E1
        | 0x08E3..=0x08FF
    )
}

fn map_reshaped_to_orig(text: &str, reshaped: &str) -> Vec<usize> {
    let mut reshaped_to_orig = Vec::with_capacity(reshaped.len() + 1);
    let mut text_chars = text.char_indices().peekable();

    for (_r_offset, r_char) in reshaped.char_indices() {
        if !is_harakat(r_char) {
            while let Some(&(_, t_char)) = text_chars.peek() {
                if is_harakat(t_char) {
                    text_chars.next();
                } else {
                    break;
                }
            }
        }

        let orig_byte = if let Some(&(t_offset, _)) = text_chars.peek() {
            if r_char == '\u{FDF2}' {
                text_chars.next();
                for _ in 0..3 {
                    while let Some(&(_, t_char)) = text_chars.peek() {
                        if is_harakat(t_char) {
                            text_chars.next();
                        } else {
                            break;
                        }
                    }
                    text_chars.next();
                }
            } else if matches!(r_char, '\u{FEF5}'..='\u{FEFC}') {
                text_chars.next();
                while let Some(&(_, t_char)) = text_chars.peek() {
                    if is_harakat(t_char) {
                        text_chars.next();
                    } else {
                        break;
                    }
                }
                text_chars.next();
            } else {
                text_chars.next();
            }
            t_offset
        } else {
            text.len()
        };

        let r_char_len = r_char.len_utf8();
        for _ in 0..r_char_len {
            reshaped_to_orig.push(orig_byte);
        }
    }

    while reshaped_to_orig.len() <= reshaped.len() {
        reshaped_to_orig.push(text.len());
    }

    reshaped_to_orig
}

/// Prepares a single line of text for visual LTR rendering in GPUI,
/// reshaping Arabic characters and applying Unicode BiDi reordering.
pub fn prepare_bidi_line(text: &str) -> BidiPreparedLine {
    if !is_rtl(text) {
        return BidiPreparedLine {
            visual_text: text.to_owned(),
            visual_to_logical: (0..=text.len()).collect(),
        };
    }

    // 1. Reshape Arabic letters into cursive presentation forms if needed.
    let reshaped = if contains_arabic(text) {
        arabic_reshaper::arabic_reshape(text)
    } else {
        text.to_owned()
    };

    let reshaped_to_orig = map_reshaped_to_orig(text, &reshaped);

    // 2. Perform BiDi visual reordering
    let bidi_info = BidiInfo::new(&reshaped, None);
    let mut visual = String::with_capacity(reshaped.len());
    let mut mapping = Vec::with_capacity(reshaped.len() + 1);

    for para in &bidi_info.paragraphs {
        let (levels, runs) = bidi_info.visual_runs(para, para.range.clone());
        for run in runs {
            let is_rtl_run = levels[run.start].is_rtl();
            let run_slice = &reshaped[run.clone()];
            let char_indices: Vec<(usize, char)> = run_slice.char_indices().collect();

            if is_rtl_run {
                for &(offset, ch) in char_indices.iter().rev() {
                    let reshaped_byte = run.start + offset;
                    let orig_byte = reshaped_to_orig
                        .get(reshaped_byte)
                        .copied()
                        .unwrap_or(text.len());
                    let v_start = visual.len();
                    visual.push(ch);
                    while mapping.len() < v_start {
                        mapping.push(orig_byte);
                    }
                    while mapping.len() < visual.len() {
                        mapping.push(orig_byte);
                    }
                }
            } else {
                for &(offset, ch) in &char_indices {
                    let reshaped_byte = run.start + offset;
                    let orig_byte = reshaped_to_orig
                        .get(reshaped_byte)
                        .copied()
                        .unwrap_or(text.len());
                    let v_start = visual.len();
                    visual.push(ch);
                    while mapping.len() < v_start {
                        mapping.push(orig_byte);
                    }
                    while mapping.len() < visual.len() {
                        mapping.push(orig_byte);
                    }
                }
            }
        }
    }

    while mapping.len() <= visual.len() {
        mapping.push(text.len());
    }

    BidiPreparedLine {
        visual_text: visual,
        visual_to_logical: mapping,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bidi_arabic_shaping() {
        let text = "زلزال";
        assert!(is_rtl(text));
        assert!(contains_arabic(text));
        let prepared = prepare_bidi_line(text);
        assert_ne!(prepared.visual_text, text);
        for &orig_idx in &prepared.visual_to_logical {
            assert!(orig_idx <= text.len(), "orig_idx {} must be <= text.len() {}", orig_idx, text.len());
        }
    }

    #[test]
    fn test_pure_arabic_artist_bounds() {
        let text = "محمد البصيلي";
        assert!(is_rtl(text));
        let prepared = prepare_bidi_line(text);
        for &orig_idx in &prepared.visual_to_logical {
            assert!(orig_idx <= text.len(), "orig_idx {} must be <= text.len() {}", orig_idx, text.len());
        }
    }

    #[test]
    fn test_arabic_ligatures_bounds() {
        let text = "السلام عليكم ورحمة الله";
        assert!(is_rtl(text));
        let prepared = prepare_bidi_line(text);
        for &orig_idx in &prepared.visual_to_logical {
            assert!(orig_idx <= text.len(), "orig_idx {} must be <= text.len() {}", orig_idx, text.len());
        }
    }

    #[test]
    fn test_mixed_bidi() {
        let text = "Hello كلام World";
        assert!(is_rtl(text));
        let prepared = prepare_bidi_line(text);
        for &orig_idx in &prepared.visual_to_logical {
            assert!(orig_idx <= text.len(), "orig_idx {} must be <= text.len() {}", orig_idx, text.len());
        }
        assert!(prepared.visual_text.starts_with("Hello "));
        assert!(prepared.visual_text.ends_with(" World"));
    }
}
