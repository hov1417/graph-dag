use std::cmp::max;
use std::fmt;

#[derive(Clone)]
pub struct Screen {
    dim_x: usize,
    dim_y: usize,
    lines: Vec<Vec<char>>,
}

impl Default for Screen {
    fn default() -> Self {
        Self::new(0, 0)
    }
}

impl Screen {
    pub fn new(width: usize, height: usize) -> Self {
        let mut scr = Self {
            dim_x: width,
            dim_y: height,
            lines: Vec::new(),
        };
        scr.resize(width, height);
        scr
    }

    pub fn resize(&mut self, new_x: usize, new_y: usize) {
        self.dim_x = new_x;
        self.dim_y = new_y;
        self.lines.resize(new_y, vec![' '; new_x]);
        for row in &mut self.lines {
            row.resize(new_x, ' ');
        }
    }

    pub const fn width(&self) -> usize {
        self.dim_x
    }
    pub const fn height(&self) -> usize {
        self.dim_y
    }

    pub fn pixel(&mut self, x: usize, y: usize) -> &mut char {
        &mut self.lines[y][x]
    }

    pub fn draw_pixel(&mut self, x: usize, y: usize, c: char) {
        self.lines[y][x] = c;
    }

    pub fn draw_text(&mut self, x: usize, y: usize, text: &str) {
        for (i, ch) in text.chars().enumerate() {
            if x + i < self.dim_x {
                self.lines[y][x + i] = ch;
            }
        }
    }

    pub fn draw_text_in_box_center(
        &mut self,
        x: usize,
        y: usize,
        width: usize,
        text: &str,
    ) {
        let margin = (width - text.chars().count())/ 2;
        self.draw_text(x + margin, y + 1, text)
    }

    pub fn draw_boxed_text(&mut self, x: usize, y: usize, text: &str) {
        self.draw_text(x + 1, y + 1, text);
        self.draw_box(x, y, text.chars().count() + 2, 3);
    }

    pub fn draw_box(&mut self, x: usize, y: usize, w: usize, h: usize) {
        self.lines[y][x] = '┌';
        self.lines[y][x + w - 1] = '┐';
        self.lines[y + h - 1][x] = '└';
        self.lines[y + h - 1][x + w - 1] = '┘';

        for xx in 1..w - 1 {
            self.lines[y][x + xx] = '─';
            self.lines[y + h - 1][x + xx] = '─';
        }
        for yy in 1..h - 1 {
            self.lines[y + yy][x] = '│';
            self.lines[y + yy][x + w - 1] = '│';
        }
    }

    pub fn draw_horizontal_line(&mut self, left: usize, right: usize, y: usize, c: char) {
        for x in left..=right {
            self.lines[y][x] = c;
        }
    }

    pub fn draw_vertical_line(&mut self, top: usize, bottom: usize, x: usize, c: char) {
        for y in top..=bottom {
            self.lines[y][x] = c;
        }
    }

    /// Converts a "half-drawn" vertical composed of '─' intersections
    /// into correct box-drawing chars (mirrors C++ `DrawVerticalLineComplete`).
    pub fn draw_vertical_line_complete(&mut self, top: usize, bottom: usize, x: usize) {
        for y in top..=bottom {
            let ch = self.lines[y][x];
            let res = match ch {
                '─' => {
                    let left = x > 0 && self.lines[y][x - 1] != ' ';
                    let right = x + 1 < self.dim_x && self.lines[y][x + 1] != ' ';
                    match (y == top, y == bottom, left, right) {
                        (true, true, l, r) => {
                            if l && r {
                                '─'
                            } else {
                                '│'
                            }
                        }
                        (true, false, true, true) => '┬',
                        (true, false, true, false) => '┐',
                        (true, false, false, true) => '┌',
                        (false, true, true, true) => '┴',
                        (false, true, true, false) => '┘',
                        (false, true, false, true) => '└',
                        _ => '│',
                    }
                }
                '┐' | '┘' => '┤',
                '┌' | '└' => '├',
                '┬' | '┴' => '┼',
                _ => '│',
            };
            self.lines[y][x] = res;
        }
    }

    #[expect(clippy::match_same_arms)] // current formatting is more readably
    pub fn asciify(&mut self, style: u8) {
        for row in &mut self.lines {
            for ch in row {
                *ch = match (*ch, style) {
                    ('─', _) => '-',
                    ('│', _) => '|',
                    ('┐' | '┌', _) => '.',
                    ('┘' | '└', _) => '\'',
                    ('┬', 0) => '-',
                    ('┬', 1) => '.',
                    ('┴', 0) => '-',
                    ('┴', 1) => '\'',
                    ('├' | '┤', _) => '-',
                    ('△', _) => '^',
                    ('▽', _) => 'V',
                    _ => *ch,
                };
            }
        }
    }

    pub fn append(&mut self, other: &Self, x: usize, y: usize) {
        self.resize(
            max(self.dim_x, x + other.dim_x),
            max(self.dim_y, y + other.dim_y),
        );
        for (dy, row) in other.lines.iter().enumerate() {
            for (dx, &ch) in row.iter().enumerate() {
                self.lines[y + dy][x + dx] = ch;
            }
        }
    }

    pub fn stringify(&self) -> String {
        let mut out = String::with_capacity((self.dim_x + 1) * self.dim_y);
        for row in &self.lines {
            for &ch in row {
                out.push(ch);
            }
            out.push('\n');
        }
        out
    }
}

impl fmt::Display for Screen {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.stringify())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn smoke() {
        let mut s = Screen::new(10, 5);
        s.draw_box(0, 0, 10, 5);
        s.draw_boxed_text(1, 1, "Hi");
        println!("{s}");
    }
}
