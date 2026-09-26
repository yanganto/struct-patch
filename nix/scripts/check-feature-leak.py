#!/usr/bin/env python3
"""Check that no `#[cfg(feature = ...)]` appears inside a quote!{}/quote!() block.

`cfg` attributes inside generated code are evaluated with the features of the
crate that *contains* the derive, not the user crate, so they must be resolved
before quote!{}/quote!() and emitted via #(#cfg)* repetition instead.

Usage: check-feature-leak.py <PATH>...   (e.g. derive/src)
Exit code 1 if any violation is found, 2 on usage errors.
"""

import sys
import pathlib

CFG_FEATURE = "#[cfg(feature"
QUOTE = "quote"


OPENERS = {"{": "}", "(": ")", "[": "]"}


def scan(text):
    """Yield (line_number, snippet) for each `#[cfg(feature` inside quote!{}/quote!()."""
    findings = []
    i, n = 0, len(text)
    # Stack entries: (belongs_to_quote_block, expected_closer).
    stack = []
    quote_blocks = 0

    def ident_start(idx):
        return idx == 0 or not (text[idx - 1].isalnum() or text[idx - 1] in "_")

    while i < n:
        c = text[i]

        # comments
        if text.startswith("//", i):
            i = text.find("\n", i)
            i = n if i == -1 else i + 1
            continue
        if text.startswith("/*", i):
            depth, i = 1, i + 2
            while i < n and depth:
                if text.startswith("/*", i):
                    depth, i = depth + 1, i + 2
                elif text.startswith("*/", i):
                    depth, i = depth - 1, i + 2
                else:
                    i += 1
            continue

        # string literals (also raw strings like r#"..."#)
        if c == '"' or (
            c == "r" and i + 1 < n and text[i + 1] in '#"' and ident_start(i)
        ):
            if c == "r":
                i += 1
                hashes = 0
                while i < n and text[i] == "#":
                    hashes, i = hashes + 1, i + 1
                if i >= n or text[i] != '"':  # not a raw string, plain ident
                    i += 1
                    continue
            i += 1
            while i < n:
                if text[i] == "\\":
                    i += 2
                elif text[i] == '"':
                    i += 1
                    break
                else:
                    i += 1
            continue

        # char literals ('x', '\n') but not lifetimes ('a, '_)
        if c == "'" and i + 1 < n and text[i + 1] in "\\(":
            if text[i + 1] == "\\":
                i += 4  # '\n' and friends
            else:
                close = text.find("}'", i + 2)
                i = close + 1 if close != -1 else i + 3
            continue

        if text.startswith(QUOTE, i) and ident_start(i):
            j = i + len(QUOTE)
            while j < n and text[j].isspace():
                j += 1
            if j < n and text[j] == "!":
                k = j + 1
                while k < n and text[k].isspace():
                    k += 1
                if k < n and text[k] in OPENERS:
                    stack.append((True, OPENERS[text[k]]))
                    quote_blocks += 1
                    i = k + 1
                    continue

        if c in OPENERS:
            stack.append((False, OPENERS[c]))
            i += 1
            continue
        if c in "})]":
            if stack:
                is_quote, _ = stack.pop()
                if is_quote:
                    quote_blocks -= 1
            i += 1
            continue

        if text.startswith(CFG_FEATURE, i) and quote_blocks:
            line = text.count("\n", 0, i) + 1
            end = text.find("]", i)
            snippet = text[i : (end + 1 if end != -1 else n)]
            snippet = " ".join(snippet.split())[:80]
            findings.append((line, snippet))
            i += len(CFG_FEATURE)
            continue

        i += 1

    return findings


def main():
    if len(sys.argv) < 2:
        print("usage: check-feature-leak.py <PATH>...", file=sys.stderr)
        return 2
    paths = [pathlib.Path(p) for p in sys.argv[1:]]

    missing = [str(p) for p in paths if not p.exists()]
    if missing:
        print(
            f"check-feature-leak.py: path(s) not found: {', '.join(missing)}",
            file=sys.stderr,
        )
        return 2

    files = []
    for p in paths:
        if p.is_dir():
            files.extend(p.rglob("*.rs"))
        else:
            files.append(p)

    violations = 0
    for f in sorted(files):
        try:
            text = f.read_text()
        except OSError as e:
            print(f"{f}: {e}", file=sys.stderr)
            continue
        for line, snippet in scan(text):
            print(f"{f}:{line}: `!! Feature leak detected: {snippet}")
            violations += 1

    if violations:
        print(f"\n{violations} violation(s) found.", file=sys.stderr)
        return 1
    dirs = [str(p) for p in paths if p.is_dir()]
    where = f" under {', '.join(dirs)}" if dirs else ""
    print(f"OK: no feature leak detects in {len(files)} file(s){where}.")
    return 0


if __name__ == "__main__":
    sys.exit(main())
