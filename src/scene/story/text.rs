use crate::ReadyState;
use crate::Rect;
use crate::font::FontMetrics;

/*
 * PAGE 0
 * -----------------------------------|
 * LINE It was the best of times      |
 * LINE it was the worst of times  [v]|
 * ------------------------------------
 * PAGE 1
 * -----------------------------------|
 * LINE I don't know what the other   |
 * LINE words are in that book but [v]|
 * ------------------------------------
 * PAGE 2
 * -----------------------------------|
 * LINE its a decent example.         |
 *                                 [>]|
 * ------------------------------------
*/
#[derive(Debug, Clone)]
pub struct TextBox {
    /// top left anchor for box, world coordinates or grid coordinates...?
    pub bounds: Rect,
    /// leading is the vertical space between lines.
    pub leading: i32,
    /// The height of each line (leading above and below not included)
    pub line_height: i32,
    /// The actual text, separated in pages that fit in the box's bounds
    pub text: Text,
}

impl TextBox {
    pub fn next_page(&mut self) {
        self.text.current_page += 1;
    }

    pub fn set_speed(&mut self, new_speed: LineSpeed) {
        for mut page in &mut self.text.pages {
            for mut line in &mut page.lines {
                line.line_speed = new_speed;
                line.timer = new_speed.into();
            }
        }
    }

    pub fn new(
        bounds: Rect,
        text: String,
        font_metrics: &impl FontMetrics,
        speed: LineSpeed,
    ) -> TextBox {
        let leading = font_metrics.line_height() / 4;

        let total_width = font_metrics.measure_width(&text);
        let max_width = bounds.width as i32;
        if total_width < max_width {
            // There's one line!
            return TextBox {
                bounds,
                leading,
                line_height: font_metrics.line_height(),
                text: Text {
                    pages: vec![Page {
                        lines: vec![Line::text_at_speed(text, speed)],
                        current_line: 0,
                    }],
                    current_page: 0,
                },
            };
        }

        // We have to split the text up into multiple lines
        // How many lines can we fit into the bounds?
        let line_height = font_metrics.line_height() + leading * 2;
        let lines_per_page = (bounds.height as usize / line_height as usize).max(1);

        let mut size = 0;
        let words = text.split_ascii_whitespace();
        let mut lines = vec![];
        let mut line = String::new();
        for word in words {
            if word.len() == 0 {
                continue;
            }

            let size_of_word = font_metrics.measure_width(&format!("{} ", word));
            let size_of_line = size + size_of_word;
            if size_of_line >= max_width {
                // this pushes us too far, start a new line.
                lines.push(Line::text_at_speed(line, speed));
                line = word.to_string();
                size = size_of_word;
            } else {
                size = size_of_line;
                line.push_str(word);
            }
            line.push(' ');
        }
        if !line.is_empty() {
            lines.push(Line::text_at_speed(line, speed));
        }

        let mut pages = vec![];
        let mut page = Page {
            lines: vec![],
            current_line: 0,
        };
        for line in lines.drain(..) {
            page.lines.push(line);
            if page.lines.len() == lines_per_page {
                pages.push(page);
                page = Page {
                    lines: vec![],
                    current_line: 0,
                };
            }
        }
        if page.lines.len() > 0 {
            pages.push(page);
        }

        TextBox {
            bounds,
            leading,
            line_height: font_metrics.line_height(),
            text: Text {
                pages,
                current_page: 0,
            },
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum LineState {
    Hidden,
    TypeWriter { revealed_count: usize }, // one character drawn at a time
    Visible,
}

#[derive(Debug, Clone, Copy)]
pub enum LineSpeed {
    Slow,
    Normal,
    Fast,
    Faster,
    Instant,
}

impl From<LineSpeed> for ReadyState {
    fn from(line_speed: LineSpeed) -> ReadyState {
        let ticks_between_characters = match line_speed {
            LineSpeed::Slow => 10,
            LineSpeed::Normal => 5,
            LineSpeed::Fast => 2,
            LineSpeed::Faster => 1,
            LineSpeed::Instant => 0,
        };
        ReadyState::Cooldown {
            wait_for: ticks_between_characters,
            ticks_waited: 0,
        }
    }
}

#[derive(Debug, Clone)]
struct Line {
    text: String,
    line_state: LineState,
    line_speed: LineSpeed,
    timer: ReadyState,
}

impl Line {
    pub fn is_visible(&self) -> bool {
        if let LineState::Visible = self.line_state {
            true
        } else {
            false
        }
    }

    pub fn text_at_speed(text: String, speed: LineSpeed) -> Self {
        Self {
            text,
            line_state: LineState::Hidden,
            timer: speed.into(),
            line_speed: speed,
        }
    }

    pub fn active_text(&self) -> &str {
        match self.line_state {
            LineState::Hidden => &self.text[0..0],
            LineState::TypeWriter { revealed_count } => &self.text[0..revealed_count],
            _ => &self.text[..],
        }
    }

    pub fn raw_text(&self) -> &str {
        &self.text[..]
    }

    pub fn advance_typewriter(&mut self) {
        self.line_state = match self.line_state {
            LineState::Hidden => LineState::TypeWriter { revealed_count: 1 },
            LineState::Visible => LineState::Visible,
            LineState::TypeWriter { revealed_count } => {
                let max_chars = self.text.len();
                let revealed_count = max_chars.min(revealed_count + 1);
                if revealed_count == max_chars {
                    LineState::Visible
                } else {
                    LineState::TypeWriter { revealed_count }
                }
            }
        }
    }

    pub fn update(&mut self, ticks: u32) {
        self.timer = self.timer.advance(ticks);
        if let LineSpeed::Instant = self.line_speed {
            self.line_state = LineState::Visible;
            return;
        }
        if let ReadyState::Ready = self.timer {
            self.advance_typewriter();
            self.timer = self.line_speed.into();
        }
    }
}

#[derive(Debug, Clone)]
pub struct Page {
    // Should we include speaker in here to identify the speaker per page?
    lines: Vec<Line>,
    current_line: usize,
}

impl Page {
    pub fn is_visible(&self) -> bool {
        self.current_line >= self.lines.len()
    }

    pub fn update(&mut self, ticks: u32) {
        if self.is_visible() {
            return;
        }

        self.lines[self.current_line].update(ticks);
        if self.lines[self.current_line].is_visible() {
            self.current_line = self.current_line + 1;
        }
    }

    pub fn raw_text(&self) -> String {
        let mut v = String::new();
        for line in &self.lines {
            v.push_str(line.raw_text());
        }
        v
    }
}

#[derive(Debug, Clone)]
pub struct Text {
    pages: Vec<Page>,
    current_page: usize,
}

impl Text {
    pub fn update(&mut self, ticks: u32) {
        if self.current_page >= self.pages.len() {
            return;
        }

        self.pages[self.current_page].update(ticks);
        // TODO this is just for testing, this should trigger
        // on user input to advance, not automatically
        // self.next_page();
    }

    pub fn next_page(&mut self) {
        if self.pages[self.current_page].is_visible() {
            self.current_page += 1;
        }
    }

    pub fn lines(&self) -> Vec<&str> {
        let mut v = vec![];
        let page = &self.pages[self.current_page.min(self.pages.len() - 1)];
        for line in &page.lines {
            if !line.active_text().is_empty() {
                v.push(line.active_text());
            }
        }
        v
    }

    pub fn raw_text(&self) -> String {
        let mut v = String::new();
        for page in &self.pages {
            v.push_str(&page.raw_text());
        }
        v
    }
}

#[cfg(test)]
mod transition_scene_tests {
    use super::*;
    #[test]
    fn a_hidden_line_advanced_shows_a_character() {
        let speed = LineSpeed::Normal;
        let mut l = Line::text_at_speed("Hello there".to_owned(), speed);
        let not_advanced = l.active_text();
        assert_eq!("", not_advanced);
        if let ReadyState::Cooldown { wait_for, .. } = speed.into() {
            l.update(wait_for);
        } else {
            assert!(false, "something is very wrong");
        }
        let advanced = l.active_text();
        assert_eq!("H", advanced);
    }

    #[test]
    fn an_almost_complete_line_advanced_becomes_visible() {
        let speed = LineSpeed::Faster;
        let mut l = Line::text_at_speed("Hello".to_owned(), speed);
        let not_advanced = l.active_text();
        assert_eq!("", not_advanced);
        if let ReadyState::Cooldown { wait_for, .. } = speed.into() {
            // .update is a saturating update, meaning that if you call it with 100 and
            // 100 is more than needed to get to ready, it waits in the readystate until the next
            // update call. So do 5 calls to get 5 characters to advance
            l.update(wait_for);
            assert_eq!("H", l.active_text());
            l.update(wait_for);
            assert_eq!("He", l.active_text());
            l.update(wait_for);
            assert_eq!("Hel", l.active_text());
            l.update(wait_for);
            assert_eq!("Hell", l.active_text());
            l.update(wait_for);
        } else {
            assert!(false, "something is very wrong");
        }
        assert_eq!("Hello", l.active_text());
        assert!(match l.line_state {
            LineState::Visible => true,
            other => {
                eprintln!("Unexpected {:?}", other);
                false
            }
        });
    }

    #[test]
    fn instant_speed_shows_text_after_first_update() {
        let speed = LineSpeed::Instant;
        let mut l = Line::text_at_speed("Hello there".to_owned(), speed);
        assert_eq!("", l.active_text());
        l.update(1);
        let advanced = l.active_text();
        assert_eq!("Hello there", advanced);
    }
}
