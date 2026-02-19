#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""Remove white background from icon, make transparent for macOS squircle effect."""

from PIL import Image
import sys
from pathlib import Path


def is_white(pixel, threshold=250):
    """Check if pixel is nearly white (background)."""
    if len(pixel) == 4:
        r, g, b, a = pixel
        if a < 128:
            return False
    else:
        r, g, b = pixel[:3]
    return r >= threshold and g >= threshold and b >= threshold


def flood_fill_white(data, width, height, start_x, start_y, threshold=250):
    """Flood fill from point, return set of (x,y) that are connected white pixels."""
    stack = [(start_x, start_y)]
    visited = set()
    while stack:
        x, y = stack.pop()
        if (x, y) in visited or x < 0 or x >= width or y < 0 or y >= height:
            continue
        idx = (y * width + x) * 4
        r, g, b, a = data[idx : idx + 4]
        if r >= threshold and g >= threshold and b >= threshold:
            visited.add((x, y))
            stack.extend([(x - 1, y), (x + 1, y), (x, y - 1), (x, y + 1)])
    return visited


def main():
    src = Path(__file__).parent.parent / "src-tauri" / "app-icon-source.png"
    out = Path(__file__).parent.parent / "src-tauri" / "app-icon-source.png"

    if len(sys.argv) >= 2:
        src = Path(sys.argv[1])
    if len(sys.argv) >= 3:
        out = Path(sys.argv[2])

    img = Image.open(src).convert("RGBA")
    data = bytearray(img.tobytes())
    w, h = img.size
    threshold = 252  # Slightly loose to catch anti-aliased edges

    # Flood fill from 4 corners to find background white
    corners = [(0, 0), (w - 1, 0), (0, h - 1), (w - 1, h - 1)]
    background = set()
    for cx, cy in corners:
        background.update(flood_fill_white(data, w, h, cx, cy, threshold))

    # Make background pixels transparent
    for x, y in background:
        idx = (y * w + x) * 4
        data[idx + 3] = 0

    out_img = Image.frombytes("RGBA", (w, h), bytes(data))
    out_img.save(out)
    print("Done:", out)


if __name__ == "__main__":
    main()
