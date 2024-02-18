use std::fmt::{Debug, Display, Formatter};

#[derive(Clone, Copy, PartialEq)]
pub struct Loc<'a> {
    path: &'a String,
    line: usize,
    col: usize,
    char: usize,
}

impl Debug for Loc<'_> {
    fn fmt(&self, fmt: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
        write!(fmt, "{self}")
    }
}

impl Display for Loc<'_> {
    fn fmt(&self, fmt: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
        write!(fmt, "{}:{}:{}", self.path, self.line, self.col)
    }
}

impl<'a> Loc<'a> {
    pub fn new(path: &'a String) -> Self {
        Self {
            path,
            line: 1,
            col: 1,
            char: 0,
        }
    }

    // FIXME: line reporting is off on the first character of a line
    pub fn advance(&mut self, ch: char) -> char {
        self.char += 1;
        if ch == '\n' {
            self.line += 1;
            self.col = 1;
        } else {
            self.col += 1;
        }
        ch
    }
}
