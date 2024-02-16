from __future__ import annotations

from typing import *
from dataclasses import dataclass

from loc import Loc
from lexer import Token, TokenKind

@dataclass
class Node:
    loc: Loc

@dataclass
class FuncCallNode(Node):
    func_name: str
    args: list[Node]

@dataclass
class StringNode(Node):
    val: str

@dataclass
class BlockNode(Node):
    nodes: list[Node]

Fmt: TypeAlias = "TypeAlias | repeat | save"

@dataclass
class repeat:
    kinds: list[Fmt]
    def __init__(self, *kinds: Fmt):
        self.kinds = list(kinds)

@dataclass
class save:
    key: str
    kind: Fmt

@dataclass
class func:
    func: ParseKindsExt

ParseKindsExt: TypeAlias = "Callable[[Parser, ParseDict, str], bool]"
ParseDict = dict[str, list[Token]]

def parse_kinds_ext(func: Callable):
    def f(*args, **kwargs) -> ParseKindsExt:
        if "parser" in kwargs or "dictionary" in kwargs or "key" in kwargs:
            raise Exception("Thou shalt not break my fun architecture!")
        def ext(parser: Parser, dictionary: ParseDict, key: str) -> bool:
            return func(parser, dictionary, key, *args, **kwargs)
        return ext
    return f

@parse_kinds_ext
def opt(p: Parser, dct: ParseDict, key: str, *kinds: Fmt) -> Literal[True]:
    keys = set(dct)
    
    if not p.parse_kinds_impl(dct, kinds, key=key):    
        to_delete = []
        for key in dct:
            if key not in keys:
                to_delete.append(key)
        for key in to_delete:
            del dct[key]
    return True


@parse_kinds_ext
def list_(p: Parser, dct: ParseDict, _: str, key: str, elt: Fmt, sep: Fmt) -> bool:
    return p.parse_kinds_impl(dct, [
        opt(
            save(key, elt),
            repeat(sep, save(key, elt))
        )
    ], key=None)

class Parser:
    toks: list[Token]
    i: int

    def __init__(self, toks: list[Token]):
        self.toks = toks
        self.i = 0
    
    def parse(self) -> Node:
        return NotImplemented



    def parse_kinds(self, *kinds: Fmt) -> tuple[ParseDict, bool]:
        dct: ParseDict = {}
        return dct, self.parse_kinds_impl(dct, kinds, key=None)

    def parse_kinds_impl(
            self, dct: ParseDict, kinds: Iterable[Fmt], *, key: str | None = None
            ) -> bool:
        
        start = self.i
        for kind in kinds:
            if self.i >= len(self.toks):
                self.i = start
                return False
            
            tok = self.toks[self.i]
            
            match kind:
                case TokenKind():
                    if tok.kind != kind:
                        self.i = start
                        return False

                    if key is not None:
                        if key not in dct:
                            dct[key] = [tok]
                        else:
                            dct[key].append(tok)

                    self.i += 1
                case repeat(kinds):
                    success = True
                    while success:
                        success = self.parse_kinds_impl(dct, kind.kinds, key=key)
                case save(key, kind):
                    success = self.parse_kinds_impl(dct, (kind, ), key=key)
                    if not success:
                        self.i = start
                        return False
                case func(func):
                    success = func(self, dct)
                    if not success:
                        self.i = start
                        return False

        return True



# from lexer import *
# from pprint import PrettyPrinter

# p = Parser(llex("<test>", "(a;b;c;d;)"))
# dct, succ = p.parse_kinds(
#     TokenKind.LPAREN,
#     repeat(save("elt", TokenKind.NAME), save("semis", TokenKind.SEMICOLON)),
#     TokenKind.RPAREN
# )

# print(f"{succ=}")
# PrettyPrinter(indent=4).pprint(dct)
