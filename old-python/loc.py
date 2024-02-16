from typing import *
from dataclasses import dataclass

@dataclass
class Loc:
    path: str
    line: int = 1
    col: int = 1
    char: int = 0

    def advance(self, code: str) -> Self:
        if code[self.char] == "\n":
            self.line += 1
            self.col = 1
        else:
            self.col += 1
        self.char += 1
        return self

    def copy(self) -> "Loc":
        return Loc(self.path, self.line, self.col, self.char)

    def __repr__(self):
        return f"{self.path}:{self.line}:{self.col}"
