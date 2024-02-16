#!/usr/bin/python3

from typing import *
from lexer import lex

import sys

def main(argv: list[str]):
    argv.pop(0)

    path = argv.pop(0)

    with open(path, "r") as f:
        code = f.read()
    print(*lex(path, code), sep="\n")

if __name__ == "__main__":
    main(sys.argv)