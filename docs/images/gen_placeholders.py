#!/usr/bin/env python3
"""Generate placeholder screenshot images for the README.

Real screenshots should replace the PNGs this writes (same filenames). Run from
the repo root: `python3 docs/images/gen_placeholders.py` (needs Pillow).
"""

from PIL import Image, ImageDraw, ImageFont

BG = (13, 17, 23)  # page bg (github dark)
CARD = (22, 27, 34)  # window bg
BORDER = (48, 54, 61)
TITLE = (139, 148, 158)  # dim
ACCENT = (56, 189, 214)  # cyan
GREEN = (63, 185, 80)
RED = (248, 81, 73)
YELLOW = (210, 153, 34)
FAINT = (80, 90, 100)

MONO = [
    "/System/Library/Fonts/Menlo.ttc",
    "/System/Library/Fonts/SFNSMono.ttf",
    "/usr/share/fonts/truetype/dejavu/DejaVuSansMono.ttf",
]
SANS = [
    "/System/Library/Fonts/SFNS.ttf",
    "/System/Library/Fonts/Supplemental/Arial Bold.ttf",
    "/usr/share/fonts/truetype/dejavu/DejaVuSans-Bold.ttf",
]


def font(candidates, size):
    for p in candidates:
        try:
            return ImageFont.truetype(p, size)
        except Exception:
            continue
    return ImageFont.load_default()


def centered(d, cx, y, text, fnt, fill):
    l, t, r, b = d.textbbox((0, 0), text, font=fnt)
    d.text((cx - (r - l) / 2, y), text, font=fnt, fill=fill)


def window(w, h, title, big, small, hero=False):
    img = Image.new("RGB", (w, h), BG)
    d = ImageDraw.Draw(img)
    m = int(w * 0.04)
    d.rounded_rectangle([m, m, w - m, h - m], radius=14, fill=CARD, outline=BORDER, width=2)
    for i, c in enumerate((RED, YELLOW, GREEN)):
        cx = m + 26 + i * 26
        d.ellipse([cx, m + 20, cx + 14, m + 34], fill=c)
    centered(d, w / 2, m + 16, title, font(MONO, max(16, int(h * 0.026))), TITLE)
    d.line([m, m + 52, w - m, m + 52], fill=BORDER, width=2)
    cy = h / 2
    centered(d, w / 2, cy - int(h * 0.10), big,
             font(SANS, int(h * (0.075 if hero else 0.06))),
             ACCENT if hero else (230, 237, 243))
    centered(d, w / 2, cy + int(h * 0.02), small, font(MONO, int(h * 0.028)), TITLE)
    centered(d, w / 2, h - m - int(h * 0.07), f"{w}x{h} placeholder",
             font(MONO, int(h * 0.022)), FAINT)
    return img


SPECS = [
    ("hero.png", 1280, 640, "lazyprop", "lazyprop",
     "a lazygit-style TUI for MuleSoft secure properties", True),
    ("main.png", 1200, 760, "lazyprop — Main", "Main screen",
     "encrypt / decrypt against a saved environment", False),
    ("playground.png", 1200, 760, "lazyprop — Playground", "Playground screen",
     "one-off encrypt / decrypt, no saved environment", False),
    ("yaml.png", 1200, 760, "lazyprop — YAML", "YAML editor",
     "encrypt / decrypt values in place, comments preserved", False),
    ("about.png", 1200, 760, "lazyprop — About", "About / guides",
     "page-specific guides and keybindings", False),
]

if __name__ == "__main__":
    import os

    out = os.path.dirname(os.path.abspath(__file__))
    for name, w, h, title, big, small, hero in SPECS:
        window(w, h, title, big, small, hero).save(os.path.join(out, name))
        print("wrote", name)
