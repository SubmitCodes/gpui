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
                    let logical_byte = run.start + offset;
                    let v_start = visual.len();
                    visual.push(ch);
                    while mapping.len() < v_start {
                        mapping.push(logical_byte);
                    }
                    while mapping.len() < visual.len() {
                        mapping.push(logical_byte);
                    }
                }
            } else {
                for &(offset, ch) in &char_indices {
                    let logical_byte = run.start + offset;
                    let v_start = visual.len();
                    visual.push(ch);
                    while mapping.len() < v_start {
                        mapping.push(logical_byte);
                    }
                    while mapping.len() < visual.len() {
                        mapping.push(logical_byte);
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
        eprintln!("ORIGINAL: {}", text);
        eprintln!("PREPARED: {}", prepared.visual_text);
    }

    #[test]
    fn test_mixed_bidi() {
        let text = "Hello كلام World";
        assert!(is_rtl(text));
        let prepared = prepare_bidi_line(text);
        eprintln!("MIXED ORIGINAL: {}", text);
        eprintln!("MIXED PREPARED: {}", prepared.visual_text);
        assert!(prepared.visual_text.starts_with("Hello "));
        assert!(prepared.visual_text.ends_with(" World"));
    }
}
