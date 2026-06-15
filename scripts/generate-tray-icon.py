#!/usr/bin/env python3
"""Generate tray icons for Keyboard Locker — lock/unlock states, 32x32 PNG."""

import struct
import zlib
import os

OUT_DIR = os.path.join(os.path.dirname(__file__), "..", "src-tauri", "icons")

# App brand colors from DESIGN.md
UNLOCKED_BG = (37, 99, 235, 255)   # Safety Blue #2563EB — keyboard unlocked
LOCKED_BG   = (239, 68, 68, 255)   # Functional Red #EF4444 — keyboard locked

WHITE  = (255, 255, 255, 255)
TRANS  = (0, 0, 0, 0)

def make_png(pixels: list, width: int, height: int) -> bytes:
    """Encode a flat list of (r,g,b,a) tuples into a valid PNG byte string."""
    # Build raw scanlines (filter byte 0 + RGBA row)
    raw = b""
    for y in range(height):
        raw += b"\x00"  # filter: None
        for x in range(width):
            r, g, b, a = pixels[y * width + x]
            raw += struct.pack("BBBB", r, g, b, a)

    def chunk(ctype: bytes, data: bytes) -> bytes:
        c = ctype + data
        return struct.pack(">I", len(data)) + c + struct.pack(">I", zlib.crc32(c) & 0xFFFFFFFF)

    ihdr = chunk(b"IHDR", struct.pack(">IIBBBBB", width, height, 8, 6, 0, 0, 0))
    idat = chunk(b"IDAT", zlib.compress(raw))
    iend = chunk(b"IEND", b"")

    return b"\x89PNG\r\n\x1a\n" + ihdr + idat + iend


def draw_lock(pixels, w, bg_color, cx=16, cy=16):
    """Draw a centered padlock icon into a flat pixel array of size w*w."""
    # Shackle — U-shaped arch at top
    shackle_outer_r = 8
    shackle_inner_r = 4
    shackle_top = cy - 11      # row 5
    shackle_bottom = cy - 3    # row 13 (where body begins)

    # Body — rounded rectangle
    body_left   = cx - 6       # col 10
    body_right  = cx + 6       # col 22
    body_top    = shackle_bottom
    body_bottom = cy + 9       # row 25
    body_radius = 3

    # Keyhole params
    hole_cx, hole_cy = cx, cy + 3   # row 19

    for y in range(w):
        for x in range(w):
            i = y * w + x
            dx, dy = x - cx, y - shackle_top

            # ---- 1. Keyhole cutout (MUST be before body to be visible) ----
            in_keyhole = False
            if abs(x - hole_cx) <= 2 and abs(y - hole_cy) <= 3:
                dk = ((x - hole_cx)**2 + (y - hole_cy)**2) ** 0.5
                if y <= hole_cy and dk <= 2:
                    in_keyhole = True
                elif y > hole_cy and abs(x - hole_cx) <= 1:
                    in_keyhole = True
            if in_keyhole:
                pixels[i] = bg_color
                continue

            # ---- 2. Body background (rounded rectangle) ----
            in_body = False
            if body_top <= y <= body_bottom and body_left <= x <= body_right:
                in_body = True
                # carve rounded corners
                for qy in (body_top, body_bottom):
                    for qx in (body_left, body_right):
                        if abs(x - qx) < body_radius and abs(y - qy) < body_radius:
                            cd = ((x - qx)**2 + (y - qy)**2) ** 0.5
                            if cd > body_radius:
                                in_body = False
            if in_body:
                pixels[i] = WHITE
                continue

            # ---- 3. Shackle ring (top half of a thick circle) ----
            dist = (dx*dx + dy*dy) ** 0.5
            if dy <= 0 and shackle_inner_r <= dist <= shackle_outer_r:
                pixels[i] = WHITE
                continue

            # ---- 4. Shackle side bars ----
            if shackle_top < y <= shackle_bottom:
                if shackle_inner_r <= abs(dx) <= shackle_outer_r:
                    d_to_center = (dx*dx + (y - shackle_top)**2) ** 0.5
                    if d_to_center > shackle_inner_r:
                        pixels[i] = WHITE
                        continue


def main():
    os.makedirs(OUT_DIR, exist_ok=True)
    w = 32

    # ---- Unlocked icon (blue) ----
    px = [TRANS] * (w * w)
    draw_lock(px, w, UNLOCKED_BG)
    path = os.path.join(OUT_DIR, "tray-icon-unlocked.png")
    with open(path, "wb") as f:
        f.write(make_png(px, w, w))
    print(f"Created {path}")

    # ---- Locked icon (red) ----
    px = [TRANS] * (w * w)
    draw_lock(px, w, LOCKED_BG)
    path = os.path.join(OUT_DIR, "tray-icon-locked.png")
    with open(path, "wb") as f:
        f.write(make_png(px, w, w))
    print(f"Created {path}")

    # ---- Also create a default tray icon (same as unlocked) ----
    px = [TRANS] * (w * w)
    draw_lock(px, w, UNLOCKED_BG)
    path = os.path.join(OUT_DIR, "tray-icon.png")
    with open(path, "wb") as f:
        f.write(make_png(px, w, w))
    print(f"Created {path}")


if __name__ == "__main__":
    main()
