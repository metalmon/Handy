"""Generate the SVG path data for the ХЭНДИ wordmark from a local font.

One-off generator: run it, paste the emitted `<path>` into
`src/components/icons/HandyTextLogo.tsx`, and the font never ships. It uses
fontTools so nothing is downloaded at generation time.

Usage:
    python scripts/generate-wordmark.py

To regenerate for another face, change FONT below. The component API must stay
the same (`width`, `height`, `className`).
"""

from fontTools.pens.svgPathPen import SVGPathPen
from fontTools.pens.transformPen import TransformPen
from fontTools.ttLib import TTFont

FONT = r"C:\Windows\Fonts\segoeuib.ttf"
TEXT = "ХЭНДИ"
# Extra space between letters, as a fraction of the cap height.
TRACKING = 0.06
# Cap height of the emitted wordmark, in viewBox units.
CAP_HEIGHT = 140.0

font = TTFont(FONT)
glyph_set = font.getGlyphSet()
cmap = font.getBestCmap()

cap_height = getattr(font["OS/2"], "sCapHeight", None) or font["head"].unitsPerEm * 0.7
tracking_units = TRACKING * cap_height
scale = CAP_HEIGHT / cap_height


def glyph_path(ch: str, x_offset: float) -> str:
    glyph_name = cmap.get(ord(ch))
    if glyph_name is None:
        raise SystemExit(f"missing glyph for {ch!r} in {FONT}")

    pen = SVGPathPen(glyph_set, ntos=lambda v: f"{round(v * scale, 1):g}")
    # Flip Y so the baseline sits at y = 0 and the cap line at -CAP_HEIGHT.
    transform = TransformPen(pen, (scale, 0, 0, -scale, x_offset * scale, 0))
    glyph_set[glyph_name].draw(transform)
    return pen.getCommands()


cursor = 0.0
segments = []
for index, ch in enumerate(TEXT):
    segments.append(glyph_path(ch, cursor))
    cursor += glyph_set[cmap[ord(ch)]].width
    if index != len(TEXT) - 1:
        cursor += tracking_units

width = round(cursor * scale, 1)
path = " ".join(s for s in segments if s)

print(
    f'<!-- Generated from segoeuib.ttf (Segoe UI Bold) by scripts/generate-wordmark.py.\n'
    f'     Text {TEXT!r}, cap height {CAP_HEIGHT:g}, tracking {TRACKING:g}em.\n'
    f'     The font itself is not shipped — only these outlines are. -->'
)
print(f'viewBox="0 {-CAP_HEIGHT} {width} {CAP_HEIGHT}"')
print(f'<path d="{path}" className="logo-primary" />')