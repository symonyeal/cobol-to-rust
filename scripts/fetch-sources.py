#!/usr/bin/env python3
"""Fetch byte-pinned COBOL sources and licenses into an external directory."""
import argparse
import hashlib
import json
from pathlib import Path, PurePosixPath
from urllib.request import Request, urlopen


def fetch(url: str) -> bytes:
    request = Request(url, headers={"User-Agent": "cobol-to-rust-source-fetcher/1"})
    with urlopen(request, timeout=30) as response:
        return response.read()


def safe_parts(value: str) -> tuple[str, ...]:
    parts = PurePosixPath(value).parts
    if not parts or PurePosixPath(value).is_absolute() or any(c in value for c in ("\\", ":")) or any(part in ("", ".", "..") for part in parts):
        raise ValueError(f"unsafe relative path: {value}")
    return parts


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("destination", type=Path, help="external source root to populate")
    args = parser.parse_args()
    metadata_path = Path(__file__).resolve().parents[1] / "docs" / "sources.json"
    metadata = json.loads(metadata_path.read_text(encoding="utf-8-sig"))
    if metadata.get("schema_version") != 1:
        raise ValueError("unsupported source metadata version")

    prepared = []
    for source in metadata["sources"]:
        if not all(key in source for key in ("id", "folder", "filename", "source_url", "source_sha256", "license_url", "license_sha256")):
            raise ValueError("source metadata is incomplete")
        source_parts = safe_parts(source["folder"]) + safe_parts(source["filename"])
        license_parts = safe_parts(source["folder"]) + ("LICENSE",)
        source_bytes = fetch(source["source_url"])
        license_bytes = fetch(source["license_url"])
        for label, content, expected in (
            ("source", source_bytes, source["source_sha256"]),
            ("license", license_bytes, source["license_sha256"]),
        ):
            actual = hashlib.sha256(content).hexdigest()
            if actual != expected:
                raise ValueError(f"{source['id']} {label} SHA-256 mismatch: {actual}")
        prepared.append((source, source_parts, source_bytes, license_parts, license_bytes))

    destination = args.destination
    for source, source_parts, source_bytes, license_parts, license_bytes in prepared:
        for parts, content in ((source_parts, source_bytes), (license_parts, license_bytes)):
            path = destination.joinpath(*parts)
            path.parent.mkdir(parents=True, exist_ok=True)
            path.write_bytes(content)
        print(f"{source['id']}: source and license SHA-256 verified")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
