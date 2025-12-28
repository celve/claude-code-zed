use std::fs;
use tower_lsp::lsp_types::Range;
use tracing::warn;

/// Convert LSP UTF-16 code unit position to Rust UTF-8 byte position
/// LSP uses UTF-16 code units for character positions per the specification
pub fn char_pos_to_byte_pos(line: &str, utf16_pos: usize) -> Option<usize> {
    let mut current_utf16_pos = 0;

    for (byte_pos, ch) in line.char_indices() {
        if current_utf16_pos == utf16_pos {
            return Some(byte_pos);
        }

        let char_utf16_len = ch.len_utf16();

        // If utf16_pos falls within this character's UTF-16 span, return this char's byte position
        if utf16_pos < current_utf16_pos + char_utf16_len {
            return Some(byte_pos);
        }

        current_utf16_pos += char_utf16_len;
    }

    // If utf16_pos is at the end of the string
    if current_utf16_pos == utf16_pos {
        return Some(line.len());
    }

    None
}

/// Read text content from a file within a specified range
pub fn read_text_from_range(file_path: &str, range: Range) -> String {
    let file_path = file_path.strip_prefix("file://").unwrap_or(file_path);

    match fs::read_to_string(file_path) {
        Ok(content) => {
            let lines: Vec<&str> = content.lines().collect();

            // Handle single line selection
            if range.start.line == range.end.line {
                if let Some(line) = lines.get(range.start.line as usize) {
                    let start_char = range.start.character as usize;
                    let end_char = range.end.character as usize;

                    if let (Some(start_byte), Some(end_byte)) = (
                        char_pos_to_byte_pos(line, start_char),
                        char_pos_to_byte_pos(line, end_char),
                    ) {
                        if start_byte <= end_byte {
                            return line[start_byte..end_byte].to_string();
                        }
                    }
                }
            } else {
                // Handle multi-line selection
                let mut selected_text = String::new();

                for (i, line_index) in (range.start.line..=range.end.line).enumerate() {
                    if let Some(line) = lines.get(line_index as usize) {
                        if i == 0 {
                            // First line - from start character to end
                            let start_char = range.start.character as usize;
                            if let Some(start_byte) = char_pos_to_byte_pos(line, start_char) {
                                selected_text.push_str(&line[start_byte..]);
                            }
                        } else if line_index == range.end.line {
                            // Last line - from start to end character
                            let end_char = range.end.character as usize;
                            if let Some(end_byte) = char_pos_to_byte_pos(line, end_char) {
                                selected_text.push_str(&line[..end_byte]);
                            }
                        } else {
                            // Middle lines - entire line
                            selected_text.push_str(line);
                        }

                        // Add newline except for the last line
                        if line_index < range.end.line {
                            selected_text.push('\n');
                        }
                    }
                }

                return selected_text;
            }
        }
        Err(e) => {
            warn!("Failed to read file {}: {}", file_path, e);
        }
    }

    String::new()
}
