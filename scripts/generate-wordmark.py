"""Generate the SVG path data for the ХЭНДИ wordmark from a local font.

One-off generator: run it, paste the emitted `<path>` into
`src/components/icons/HandyTextLogo.tsx`, and the font never ships. It uses
fontTools so nothing is downloaded at generation time.

Usage:
    python scripts/generate-wordmark.py

To regenerate for another face, change FONT below. The component API must stay
the same (`width`, `height`, `className`).
"""

from fontTools.pens.boundsPen import BoundsPen
from fontTools.pens.svgPathPen import SVGPathPen
from fontTools.pens.transformPen import TransformPen
from fontTools.ttLib import TTFont

FONT = r"C:\Windows\Fonts\segoeuib.ttf"
TEXT = "ХЭНДИ"
# Extra space between letters, as a fraction of the cap height.
TRACKING = 0.06
# Cap height of the emitted wordmark, in viewBox units.
CAP_HEIGHT = 140.0
# Empty space kept around the ink so nothing is clipped by the viewBox.
PADDING = 2.0

font = TTFont(FONT)
glyph_set = font.getGlyphSet()
cmap = font.getBestCmap()

cap_height = getattr(font["OS/2"], "sCapHeight", None) or font["head"].unitsPerEm * 0.7
tracking_units = TRACKING * cap_height
scale = CAP_HEIGHT / cap_height


def glyph_name(ch: str) -> str:
    name = cmap.get(ord(ch))
    if name is None:
        raise SystemExit(f"missing glyph for {ch!r} in {FONT}")
    return name


def glyph_path(ch: str, x_offset: float) -> str:
    pen = SVGPathPen(glyph_set, ntos=lambda v: f"{round(v, 1):g}")
    # Flip Y so the baseline sits at y = 0 and the cap line at -CAP_HEIGHT.
    transform = TransformPen(pen, (scale, 0, 0, -scale, x_offset * scale, 0))
    glyph_set[glyph_name(ch)].draw(transform)
    return pen.getCommands()


def glyph_bounds(ch: str, x_offset: float) -> tuple[float, float, float, float]:
    pen = BoundsPen(glyph_set)
    transform = TransformPen(pen, (scale, 0, 0, -scale, x_offset * scale, 0))
    glyph_set[glyph_name(ch)].draw(transform)
    return pen.bounds


cursor = 0.0
segments = []
bounds = []
for index, ch in enumerate(TEXT):
    segments.append(glyph_path(ch, cursor))
    bounds.append(glyph_bounds(ch, cursor))
    cursor += glyph_set[glyph_name(ch)].width
    if index != len(TEXT) - 1:
        cursor += tracking_units

path = " ".join(s for s in segments if s)

# Tight box around the ink: Cyrillic Д has legs below the baseline and round
# strokes overshoot the cap line, so the box is taller than the cap height.
min_x = min(b[0] for b in bounds) - PADDING
min_y = min(b[1] for b in bounds) - PADDING
max_x = max(b[2] for b in bounds) + PADDING
max_y = max(b[3] for b in bounds) + PADDING

print(
    f'<!-- Generated from segoeuib.ttf (Segoe UI Bold) by scripts/generate-wordmark.py.\n'
    f'     Text {TEXT!r}, cap height {CAP_HEIGHT:g}, tracking {TRACKING:g}em.\n'
    f'     The font itself is not shipped — only these outlines are. -->'
)
print(f"viewBox=\"{min_x:.1f} {min_y:.1f} {max_x - min_x:.1f} {max_y - min_y:.1f}\"")
print(f'<path d="{path}" className="logo-primary" />')