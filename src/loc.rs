use std::fmt::{Debug, Display, Formatter};

#[derive(Clone, PartialEq)]
pub struct Loc {
    path: Box<String>,
    line: usize,
    col: usize,
    char: usize,
}

impl Debug for Loc {
    fn fmt(&self, fmt: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
        write!(fmt, "{self}")
    }
}

impl Display for Loc {
    fn fmt(&self, fmt: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
        write!(fmt, "{}:{}:{}", self.path, self.line, self.col)
    }
}

impl Loc {
    pub fn new(path: &String) -> Self {
        Self {
            path: Box::new(path.clone()),
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
