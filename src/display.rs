use crate::ansi;
use crate::file::{File, FileType};
use crate::App;

impl App {
    pub fn display(&self) {
        let padding = 9;
        let space_for_text = self.width - padding;
        // 20% 40% 40%
        let first_col_width = space_for_text / 5;
        let second_col_width = first_col_width * 2;
        //assert!(self.width > (first_col_width + second_col_width + padding));
        print!("{}{}", ansi::CLEAR, ansi::RESET);
        self.show_breadcrumbs();

        let info_lines = match &self.selection_info {
            Some(info) => info.lines(),
            None => Vec::new(),
        };
        let (from, to) = self.rows_to_print(info_lines.len());
        for (i, line) in (from..to).enumerate() {
            let is_selected = line == self.current_selection;
            let selection_arrow = match is_selected {
                true => "->",
                false => "  ",
            };

            let formatted_current_item =
                display_file(self.current_directory_contents.get(line), second_col_width);

            let formatted_parent_item =
                display_file(self.parent_directory_contents.get(line), first_col_width);

            // padding is 3 + 2 + 4 characters
            let first_two_columns = format!(
                "{} | {selection_arrow} {} | ",
                formatted_parent_item, formatted_current_item
            );

            let formatted_info_line = truncate_with_ellipsis(
                info_lines.get(i).unwrap_or(&"".to_string()),
                second_col_width,
            );

            println!("{first_two_columns}{formatted_info_line}\r",);
        }

        println!("\r");
        for line in &self.debug_messages {
            println!("{}\r", line);
        }
    }

    fn rows_to_print(&self, info_lines_len: usize) -> (usize, usize) {
        let rows_to_show = (self.height - 2) - self.debug_messages.len();

        let max_lines = self
            .current_directory_contents
            .len()
            .max(self.parent_directory_contents.len())
            .max(info_lines_len);

        if max_lines > rows_to_show {
            let half = rows_to_show / 2;
            if self.current_selection < half {
                // limited from the start
                (0, rows_to_show)
            } else if self.current_selection > (max_lines - half) {
                // limited from the end
                (max_lines - rows_to_show, max_lines)
            } else {
                // if selection > half of rows_to_show, add the diff to the window
                let start_offset = self.current_selection - half;
                (start_offset, self.current_selection + half)
            }
        } else {
            (0, max_lines)
        }
    }

    fn show_breadcrumbs(&self) {
        println!("{}\r", self.current_directory.display());
    }
}

pub fn display_file(file: Option<&File>, max_length: usize) -> String {
    if let Some(file) = file {
        if file.name.starts_with('.') {
            display_hidden_file(file, max_length)
        } else {
            match file.ftype {
                FileType::File => display_normal_file(file, max_length),
                FileType::Directory => display_directory(file, max_length),
                FileType::Link => display_link(file, max_length),
                FileType::Unknown => display_unknown(file, max_length),
            }
        }
    } else {
        " ".repeat(max_length)
    }
}

fn display_hidden_file(file: &File, max_length: usize) -> String {
    format!(
        "{}{}{}",
        ansi::GRAY,
        truncate_with_ellipsis(&file.name, max_length),
        ansi::RESET
    )
}

fn display_normal_file(file: &File, max_length: usize) -> String {
    format!(
        "{}{}{}",
        ansi::WHITE,
        truncate_with_ellipsis(&file.name, max_length),
        ansi::RESET
    )
}
fn display_directory(file: &File, max_length: usize) -> String {
    format!(
        "{}{}{}",
        ansi::BLUE,
        truncate_with_ellipsis(&file.name, max_length),
        ansi::RESET
    )
}
fn display_link(file: &File, max_length: usize) -> String {
    format!(
        "{}{}{}",
        ansi::CYAN,
        truncate_with_ellipsis(&file.name, max_length),
        ansi::RESET
    )
}
fn display_unknown(file: &File, max_length: usize) -> String {
    format!(
        "{}{}{}",
        ansi::RED,
        truncate_with_ellipsis(&file.name, max_length),
        ansi::RESET
    )
}

fn truncate_with_ellipsis(input: &str, max_length: usize) -> String {
    if max_length < 3 {
        return "…".to_owned();
    }
    let chars_len = &input
        .chars().count();
    let wide_chars = &input.chars().filter(|ch| is_wide(*ch)).count();
    // Offset the number of wide_characters (as if they are 2x size)
    if chars_len > &(max_length - wide_chars) {
        format!(
            "{}…",
            &input.chars().take(max_length - 1 - wide_chars).collect::<String>()
        )
    } else {
        format!("{:<width$}", input, width = max_length - wide_chars)
    }
}


fn is_wide(ch: char) -> bool {
    // TODO: add more scripts
    let is_kanji = is_char_between_char_range(ch, KANJI_BEG, KANJI_END);
    let is_katakana = is_char_between_char_range(ch, KATAKANA_BEG, KATAKANA_END);
    let is_hiragana = is_char_between_char_range(ch, HIRAGANA_BEG, HIRAGANA_END);
    let is_fullwidth = is_char_between_char_range(ch, FULLWIDTH_BEG, FULLWIDTH_END);
    is_hiragana || is_katakana || is_kanji || is_fullwidth
}

// https://github.com/kitallis/konj
// taken from this project's `strings` file.
pub fn is_char_between_char_range(ch: char, range_beg: char, range_end: char) -> bool {
    if !(ch >= range_beg && ch <= range_end) {
        // || ch.is_whitespace()) {
        return false;
    }

    true
}

// https://github.com/kitallis/konj
// taken from this project's `constants` file.
pub const HIRAGANA_BEG: char = '\u{3040}';
pub const HIRAGANA_END: char = '\u{309F}';
pub const KATAKANA_BEG: char = '\u{30A0}';
pub const KATAKANA_END: char = '\u{30FF}';
pub const KANJI_BEG: char = '\u{4E00}';
pub const KANJI_END: char = '\u{9FAF}';
pub const FULLWIDTH_BEG: char = '\u{FF01}';
pub const FULLWIDTH_END: char = '\u{FF60}';
