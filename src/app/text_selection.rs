use crate::app::tabs::SelectionRegion;
use crate::pdf::{PageText, TextChar};

/// Calculate text selection based on mouse coordinates
#[allow(clippy::too_many_arguments)]
pub fn calculate_text_selection(
    page_text: &PageText,
    pdf_width: f32,
    pdf_height: f32,
    page_width: u32,
    page_height: u32,
    start_x: f32,
    start_y: f32,
    end_x: f32,
    end_y: f32,
) -> (String, Vec<SelectionRegion>) {
    // Calculate selection rectangle in screen coordinates
    let screen_min_x = start_x.min(end_x);
    let screen_max_x = start_x.max(end_x);
    let screen_min_y = start_y.min(end_y);
    let screen_max_y = start_y.max(end_y);

    // Calculate scale factors
    let scale_x = pdf_width / page_width as f32;
    let scale_y = pdf_height / page_height as f32;

    // Group all characters by line first
    let mut all_lines: Vec<Vec<&TextChar>> = Vec::new();
    let line_tolerance = 5.0f32;

    for c in &page_text.chars {
        let char_screen_y = (pdf_height - c.y) / scale_y;

        if let Some(last_line) = all_lines.last_mut() {
            if let Some(last_char) = last_line.first() {
                let last_char_screen_y = (pdf_height - last_char.y) / scale_y;
                if (char_screen_y - last_char_screen_y).abs() < line_tolerance {
                    last_line.push(c);
                    continue;
                }
            }
        }

        all_lines.push(vec![c]);
    }

    // Sort lines by Y position (top to bottom)
    all_lines.sort_by(|a, b| {
        let a_y = (pdf_height - a.first().unwrap().y) / scale_y;
        let b_y = (pdf_height - b.first().unwrap().y) / scale_y;
        a_y.partial_cmp(&b_y).unwrap_or(std::cmp::Ordering::Equal)
    });

    // Sort characters within each line by X position
    for line in &mut all_lines {
        line.sort_by(|a, b| a.x.partial_cmp(&b.x).unwrap_or(std::cmp::Ordering::Equal));
    }

    // Find which lines are within the selection Y range
    let mut selected_lines: Vec<&Vec<&TextChar>> = Vec::new();

    for line in &all_lines {
        if line.is_empty() {
            continue;
        }
        let first_char = line.first().unwrap();
        let line_y = (pdf_height - first_char.y) / scale_y;
        let line_height = first_char.height / scale_y;
        let line_top = line_y - line_height;
        let line_bottom = line_y;

        // Check if this line overlaps with selection Y range
        if line_bottom >= screen_min_y && line_top <= screen_max_y {
            selected_lines.push(line);
        }
    }

    // Build selected text and selection regions
    let selected_text = build_selected_text(
        &selected_lines,
        scale_x,
        page_width,
        screen_min_x,
        screen_max_x,
    );
    let selection_regions = build_selection_regions(
        &selected_lines,
        scale_x,
        scale_y,
        pdf_height,
        page_width,
        screen_min_x,
        screen_max_x,
    );

    (selected_text, selection_regions)
}

fn build_selected_text(
    selected_lines: &[&Vec<&TextChar>],
    scale_x: f32,
    page_width: u32,
    screen_min_x: f32,
    screen_max_x: f32,
) -> String {
    let mut selected_text_parts: Vec<String> = Vec::new();
    let num_lines = selected_lines.len();
    let page_width_f32 = page_width as f32;

    for (line_idx, line) in selected_lines.iter().enumerate() {
        // Filter out abnormal characters first
        let valid_line_chars: Vec<&TextChar> = line
            .iter()
            .filter(|c| {
                let char_x = c.x / scale_x;
                let char_end_x = (c.x + c.width) / scale_x;
                char_x < page_width_f32 && char_end_x <= page_width_f32 + 10.0
            })
            .copied()
            .collect();

        let chars_for_line: Vec<&TextChar> = if num_lines == 1 {
            // Single line: only characters in selection X range
            valid_line_chars
                .iter()
                .filter(|c| {
                    let char_x = c.x / scale_x;
                    let char_end_x = (c.x + c.width) / scale_x;
                    char_end_x >= screen_min_x && char_x <= screen_max_x
                })
                .copied()
                .collect()
        } else if line_idx == 0 {
            // First line: from selection start to line end
            valid_line_chars
                .iter()
                .filter(|c| {
                    let char_end_x = (c.x + c.width) / scale_x;
                    char_end_x >= screen_min_x
                })
                .copied()
                .collect()
        } else if line_idx == num_lines - 1 {
            // Last line: from line start to selection end
            valid_line_chars
                .iter()
                .filter(|c| {
                    let char_x = c.x / scale_x;
                    char_x <= screen_max_x
                })
                .copied()
                .collect()
        } else {
            // Middle lines: entire line
            valid_line_chars
        };

        // Sort characters within this line by x position
        let mut line_chars_sorted = chars_for_line;
        line_chars_sorted
            .sort_by(|a, b| a.x.partial_cmp(&b.x).unwrap_or(std::cmp::Ordering::Equal));

        // Build text for this line
        let line_text: String = line_chars_sorted.iter().map(|c| c.char).collect();
        if !line_text.is_empty() {
            selected_text_parts.push(line_text);
        }
    }

    // Join lines with newline
    selected_text_parts.join("\n")
}

fn build_selection_regions(
    selected_lines: &[&Vec<&TextChar>],
    scale_x: f32,
    scale_y: f32,
    pdf_height: f32,
    page_width: u32,
    screen_min_x: f32,
    screen_max_x: f32,
) -> Vec<SelectionRegion> {
    let mut selection_regions: Vec<SelectionRegion> = Vec::new();
    let num_lines = selected_lines.len();
    let page_width_f32 = page_width as f32;

    for (line_idx, line_chars) in selected_lines.iter().enumerate() {
        if line_chars.is_empty() {
            continue;
        }

        // Get line height from first character
        let first_char = line_chars.first().unwrap();
        let line_height = first_char.height / scale_y;
        let line_top = ((pdf_height - first_char.y) / scale_y) - line_height;

        // Filter out characters with abnormal x coordinates (e.g., newline markers)
        // Only keep characters that are within page bounds
        let valid_chars: Vec<&TextChar> = line_chars
            .iter()
            .filter(|c| {
                let char_x = c.x / scale_x;
                let char_end_x = (c.x + c.width) / scale_x;
                // Filter out characters that are beyond page width
                // (likely formatting markers like newlines)
                char_x < page_width_f32 && char_end_x <= page_width_f32 + 10.0
            })
            .copied()
            .collect();

        if valid_chars.is_empty() {
            continue;
        }

        // Calculate line bounds from valid characters only
        // Characters are already sorted by x, so first char has min_x, last char has max_x
        let line_min_x = valid_chars.first().unwrap().x / scale_x;
        let line_max_x =
            valid_chars.last().unwrap().x / scale_x + valid_chars.last().unwrap().width / scale_x;

        // Determine X bounds based on line position
        let (sel_min_x, sel_max_x) = if num_lines == 1 {
            // Single line: only characters in selection X range
            let chars_in_range: Vec<&TextChar> = valid_chars
                .iter()
                .filter(|c| {
                    let char_x = c.x / scale_x;
                    let char_end_x = (c.x + c.width) / scale_x;
                    char_end_x >= screen_min_x && char_x <= screen_max_x
                })
                .copied()
                .collect();
            if chars_in_range.is_empty() {
                continue;
            }
            let min_x = chars_in_range
                .iter()
                .map(|c| c.x / scale_x)
                .fold(f32::MAX, f32::min);
            let max_x = chars_in_range
                .iter()
                .map(|c| (c.x + c.width) / scale_x)
                .fold(f32::MIN, f32::max);
            (min_x, max_x)
        } else if line_idx == 0 {
            // First line: from selection start to line end (only actual characters)
            let chars_in_range: Vec<&TextChar> = valid_chars
                .iter()
                .filter(|c| {
                    let char_end_x = (c.x + c.width) / scale_x;
                    char_end_x >= screen_min_x
                })
                .copied()
                .collect();
            if chars_in_range.is_empty() {
                continue;
            }
            let min_x = chars_in_range
                .iter()
                .map(|c| c.x / scale_x)
                .fold(f32::MAX, f32::min);
            let max_x = line_max_x; // Use actual line end, not selection end
            (min_x, max_x)
        } else if line_idx == num_lines - 1 {
            // Last line: from line start to selection end (only actual characters)
            let chars_in_range: Vec<&TextChar> = valid_chars
                .iter()
                .filter(|c| {
                    let char_x = c.x / scale_x;
                    char_x <= screen_max_x
                })
                .copied()
                .collect();
            if chars_in_range.is_empty() {
                continue;
            }
            let min_x = line_min_x; // Use actual line start
            let max_x = chars_in_range
                .iter()
                .map(|c| (c.x + c.width) / scale_x)
                .fold(f32::MIN, f32::max);
            (min_x, max_x)
        } else {
            // Middle lines: entire line (only actual characters)
            (line_min_x, line_max_x)
        };

        // Clamp to image bounds
        let clamped_min_x = sel_min_x.max(0.0);
        let clamped_max_x = sel_max_x.min(page_width as f32);
        let final_width = clamped_max_x - clamped_min_x;

        if final_width > 0.0 {
            selection_regions.push(SelectionRegion {
                x: clamped_min_x,
                y: line_top,
                width: final_width,
                height: line_height,
            });
        }
    }

    selection_regions
}
