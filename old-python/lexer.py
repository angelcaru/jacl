from dataclasses import dataclass
from enum import StrEnum, auto
from typing import *

from loc import Loc

class TokenKind(StrEnum):
    NAME      = auto()
    LPAREN    = auto()
    RPAREN    = auto()
    STRING    = auto()
    SEMICOLON = auto()

@dataclass
class Token:
    loc: Loc
    kind: TokenKind
    val: str

SIMPLE_TOKENS = {
    "(": TokenKind.LPAREN,
    ")": TokenKind.RPAREN,
    ";": TokenKind.SEMICOLON,
}

def lex(path: str, code: str) -> Iterable[Token]:
    loc = Loc(path)
    while loc.char < len(code):
        char = code[loc.char]
        curr_loc = loc.copy()
        if char.isspace():
            loc.advance(code)
        elif char in SIMPLE_TOKENS:
            yield Token(curr_loc, SIMPLE_TOKENS[char], char)
            loc.advance(code)
        elif char.isalpha():
            name = ""
            while loc.char < len(code) and code[loc.char].isalnum():
                name += code[loc.char]
                loc.advance(code)

            yield Token(curr_loc, TokenKind.NAME, name)
        elif char == '"':
            loc.advance(code)
            string = ""
            while loc.char < len(code) and code[loc.char] != '"':
                string += code[loc.char]
                loc.advance(code)
            
            loc.advance(code)
            yield Token(curr_loc, TokenKind.STRING, string)
        else:
            print(loc)
            assert False, "unreachable"

def llex(path: str, code: str) -> list[Token]:
    return list(lex(path, code))
