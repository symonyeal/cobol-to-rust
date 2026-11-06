#!/usr/bin/env python3
"""Prepare the approved metadata-only fxsort compiler-admission source."""
import argparse
import hashlib
from pathlib import Path

ORIGINAL_SHA256 = "971d185e34000f20e1200e505ec20a6fd1462884318a93f14c92fa02c75572a2"
SOURCE_RELATIVE = Path("IBM_cobol-is-fun") / "fxsort.cbl"
OUTPUT_RELATIVE = Path("candidates") / "adapted" / "fxsort.cbl"
PROGRAM_ID = b"       PROGRAM-ID.    FXSORT"
AUTHOR = b"       AUTHOR.        Jeff Bisti"
IDENTIFICATION = b"       IDENTIFICATION DIVISION."
NOTICE = b"      * Modified: metadata periods added to PROGRAM-ID and AUTHOR only."


def sha256(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def prepare(source_root: Path) -> tuple[Path, str]:
    """Add two approved metadata terminators and an Apache modification notice.

    INPUT: source_root, folder holding the fetched original source.
    OUTPUT: adapted source path and SHA-256; original must match the pinned hash.
    """
    original_path = source_root / SOURCE_RELATIVE
    original = original_path.read_bytes()
    actual = sha256(original)
    if actual != ORIGINAL_SHA256:
        raise ValueError(f"original fxsort hash mismatch: {actual}")
    for marker in (PROGRAM_ID, AUTHOR, IDENTIFICATION):
        if original.count(marker) != 1:
            raise ValueError(f"expected one source marker: {marker.decode('ascii')}")
    if len(NOTICE) > 72 or not NOTICE.startswith(b"      *"):
        raise ValueError("modification notice is not a fixed-format comment")

    adapted = original.replace(PROGRAM_ID, PROGRAM_ID + b".", 1)
    adapted = adapted.replace(AUTHOR, AUTHOR + b".", 1)
    adapted = adapted.replace(IDENTIFICATION, NOTICE + b"\n" + IDENTIFICATION, 1)
    target = source_root / OUTPUT_RELATIVE
    target.parent.mkdir(parents=True, exist_ok=True)
    target.write_bytes(adapted)
    return target, sha256(adapted)


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("source_root", type=Path, help="external root containing IBM_cobol-is-fun/fxsort.cbl")
    args = parser.parse_args()
    target, adapted_hash = prepare(args.source_root)
    print(f"adapted source written: {OUTPUT_RELATIVE.as_posix()}")
    print(f"original sha256: {ORIGINAL_SHA256}")
    print(f"adapted sha256: {adapted_hash}")


if __name__ == "__main__":
    main()
