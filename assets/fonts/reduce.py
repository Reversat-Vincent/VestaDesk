"""Makes the reduced Noto fonts of VestaDesk from their original files.

Usage, with fontTools 4.67.0: python reduce.py <folder of the originals>

The folder holds the three original Noto fonts listed in the `fonts` module docs (src/fonts.rs),
under their original names. The reduced fonts replace the ones next to this script, and come out
the same, byte for byte, on every run.

Each reduced font keeps the characters of its `-characters.txt` file, and has no kerning. The
glyphs of the symbol fonts are also centered in one em, the width of a Japanese character, and
shrunk if they are wider; this removes their hinting.
"""

import hashlib
import sys
from pathlib import Path

import fontTools
from fontTools import subset
from fontTools.pens.transformPen import TransformPen
from fontTools.pens.ttGlyphPen import TTGlyphPen

# The version of fontTools that made the reduced fonts: another version may give other bytes.
FONTTOOLS_VERSION = "4.67.0"

# The fonts to reduce. For each one: its file name, the same for the original and the reduced font;
# the SHA-256 of the original; and whether its glyphs are centered in one em, which only the symbol
# fonts need, so that each symbol takes the width of a Japanese character.
FONTS = [
    ("NotoSansJP-Regular.otf", "dff723ba59d57d136764a04b9b2d03205544f7cd785a711442d6d2d085ac5073", False),
    ("NotoSansSymbols2-Regular.ttf", "c4a0a80f0041ce4be81e2478faad22776d23edb98ae3f0d19bd37044820ecf9d", True),
    ("NotoSansSymbols-Regular.ttf", "d0e98e9a2c046594c5021437273943be7e79e0fd980fde125279e22302212595", True),
]


def center_in_one_em(font):
    """Gives each glyph that has a width an advance of one em, centering it and shrinking it if it is wider."""
    # `unitsPerEm` is the number of font units in one em: a Japanese character is one em wide.
    em = font["head"].unitsPerEm
    # `glyf` holds the outline of each glyph, and `hmtx` its advance (the width it takes in a line)
    # and its left side bearing (the space before its outline).
    glyf, hmtx, glyphs = font["glyf"], font["hmtx"], font.getGlyphSet()
    # All the new outlines are drawn from the original ones first, and stored in the font after.
    outlines = {}
    for name in font.getGlyphOrder():
        advance = hmtx[name][0]
        if advance == 0:
            continue  # combining marks, drawn over the character before them, keep no width
        # A composite glyph is made of other glyphs, which are moved too: it would be moved twice.
        if glyf[name].isComposite():
            sys.exit(f"{name} is a composite glyph, which cannot be centered")
        # A glyph wider than one em shrinks to fit in it; a narrower one keeps its size.
        scale = min(1.0, em / advance)
        # The outline is redrawn scaled, and moved right by half of the space left in the em.
        pen = TTGlyphPen(glyphs)
        glyphs[name].draw(TransformPen(pen, (scale, 0, 0, scale, (em - advance * scale) / 2, 0)))
        outlines[name] = pen.glyph()
    for name, glyph in outlines.items():
        glyf[name] = glyph
        # The outline has moved, so its bounding box and left side bearing change; its advance is one em.
        glyph.recalcBounds(glyf)
        hmtx[name] = (em, getattr(glyph, "xMin", 0))


if len(sys.argv) != 2:
    sys.exit(__doc__)
if fontTools.version != FONTTOOLS_VERSION:
    sys.exit(f"fontTools {fontTools.version} is installed, but the fonts are made with fontTools {FONTTOOLS_VERSION}")

# `here` is assets/fonts, which holds the character lists and receives the reduced fonts.
here, originals = Path(__file__).parent, Path(sys.argv[1])
for name, sha256, centered in FONTS:
    # Any file other than the documented original is refused, so that the result is always the same.
    original = originals / name
    if hashlib.sha256(original.read_bytes()).hexdigest() != sha256:
        sys.exit(f"{original} is not the original font")

    # The options of fontTools' subsetter, the others keeping their defaults (which leave the date
    # of the font unchanged):
    # - name_IDs: all the names of the font are kept, its copyright and licence included;
    # - hinting: the symbol fonts lose their hinting, which would distort their moved outlines;
    # - recalc_bounds: the bounding boxes of the symbol fonts are recomputed when they are saved,
    #   since centering moves their outlines;
    # - harfbuzz_repacker: fontTools packs the glyph substitution tables itself, even when the
    #   HarfBuzz packer is installed, which would give other bytes;
    # - drop_tables: the GPOS table, which holds the kerning, is removed too.
    options = subset.Options(
        name_IDs=["*"], hinting=not centered, recalc_bounds=centered, harfbuzz_repacker=False
    )
    options.drop_tables.append("GPOS")
    # The glyph names are not needed: leaving them out makes the font smaller, as fontTools' own
    # subsetting command does.
    font = subset.load_font(original, options, dontLoadGlyphNames=True)

    # Only the glyphs of the listed characters are kept, with the other forms that the font can
    # replace them with, such as vertical forms. The line breaks of the list, including those that
    # Git may write as CR LF on Windows, are not characters to keep.
    subsetter = subset.Subsetter(options)
    characters = (here / f"{name.split('-')[0]}-characters.txt").read_text(encoding="utf-8")
    subsetter.populate(text=characters.replace("\r", "").replace("\n", ""))
    subsetter.subset(font)

    if centered:
        center_in_one_em(font)
    subset.save_font(font, here / name, options)
    print(f"{name}: {(here / name).stat().st_size} bytes")
