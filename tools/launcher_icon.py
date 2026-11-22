#!/usr/bin/env python3
"""Generate a small 128x128 PNG application icon without dependencies."""

from __future__ import annotations

import math
import struct
import zlib
from pathlib import Path


def _chunk(tag: bytes, data: bytes) -> bytes:
    return (
        struct.pack(">I", len(data))
        + tag
        + data
        + struct.pack(">I", zlib.crc32(tag + data) & 0xFFFFFFFF)
    )


def _rounded(x: float, y: float, left: int, top: int, right: int, bottom: int, radius: int) -> bool:
    if not (left <= x < right and top <= y < bottom):
        return False
    nearest_x = min(max(x, left + radius), right - radius - 1)
    nearest_y = min(max(y, top + radius), bottom - radius - 1)
    return (x - nearest_x) ** 2 + (y - nearest_y) ** 2 <= radius**2


def write_icon(path: Path, color: str) -> None:
    """Write a 128x128 PNG with a dark tile, coloured border and white C."""
    color = color.lstrip("#")
    if len(color) != 6:
        raise ValueError("color must be #RRGGBB")
    accent = tuple(int(color[index : index + 2], 16) for index in (0, 2, 4))
    dark = (18, 20, 25)
    white = (240, 244, 250)

    rows: list[bytes] = []
    for y in range(128):
        row = bytearray()
        for x in range(128):
            alpha = 0
            pixel = (0, 0, 0)
            if _rounded(x, y, 6, 6, 122, 122, 24):
                pixel = accent
                alpha = 255
            if _rounded(x, y, 20, 20, 108, 108, 16):
                pixel = dark
            if _rounded(x, y, 30, 30, 98, 98, 12):
                dx = x - 64
                dy = y - 64
                radius = math.hypot(dx, dy)
                angle = math.degrees(math.atan2(dy, dx))
                in_c = 25 <= radius <= 39 and not (-48 <= angle <= 48)
                if in_c:
                    pixel = white
            row.extend((*pixel, alpha))
        rows.append(bytes(row))

    raw = b"".join(b"\x00" + row for row in rows)
    png = (
        b"\x89PNG\r\n\x1a\n"
        + _chunk(b"IHDR", struct.pack(">IIBBBBB", 128, 128, 8, 6, 0, 0, 0))
        + _chunk(b"IDAT", zlib.compress(raw, 9))
        + _chunk(b"IEND", b"")
    )
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_bytes(png)


if __name__ == "__main__":
    import sys

    output = Path(sys.argv[1] if len(sys.argv) > 1 else "icon-128.png")
    colour = sys.argv[2] if len(sys.argv) > 2 else "#e8573f"
    write_icon(output, colour)
    print(output)
