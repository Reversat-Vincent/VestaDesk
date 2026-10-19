//! Each bundled font can be read and has the characters it provides, Noto Sans JP has the Japanese characters of the font research, and together the fonts stay within their size limit.
#![cfg(test)]

use std::ops::RangeInclusive;

use skrifa::instance::{LocationRef, Size};
use skrifa::outline::pen::NullPen;
use skrifa::{FontRef, GlyphId, MetadataProvider as _};
use vesta_desk::fonts::{BUNDLED, BundledFont};

/// The largest total size of the bundled fonts, in bytes: 2.5 MiB.
const SIZE_LIMIT: usize = 5 * 1024 * 1024 / 2;

/// The characters of the Unicode blocks that the font research covers: CJK Symbols and Punctuation, Hiragana, Katakana, Katakana Phonetic Extensions, and Halfwidth and Fullwidth Forms but its halfwidth Hangul.
const JAPANESE_BLOCKS: [RangeInclusive<char>; 8] = [
    '\u{3000}'..='\u{303F}',
    '\u{3041}'..='\u{3096}',
    '\u{3099}'..='\u{309F}',
    '\u{30A0}'..='\u{30FF}',
    '\u{31F0}'..='\u{31FF}',
    '\u{FF01}'..='\u{FF9F}',
    '\u{FFE0}'..='\u{FFE6}',
    '\u{FFE8}'..='\u{FFEE}',
];

/// Returns the characters that `font` provides: for DejaVu Sans Mono, the Latin letters, digits and punctuation of French and English text, and ≒; for a reduced font, those of its `-characters.txt` file.
fn characters(font: &BundledFont) -> Vec<char> {
    let listed = |list: &str| -> Vec<char> { list.lines().flat_map(str::chars).collect() };
    match font.name {
        "DejaVu Sans Mono" => (' '..='~')
            .chain('\u{A0}'..='\u{17F}')
            .chain("–—‘’“”…€≒".chars())
            .collect(),
        "Noto Sans JP" => listed(include_str!("../assets/fonts/NotoSansJP-characters.txt")),
        "Noto Sans Symbols 2" => listed(include_str!(
            "../assets/fonts/NotoSansSymbols2-characters.txt"
        )),
        "Noto Sans Symbols" => listed(include_str!(
            "../assets/fonts/NotoSansSymbols-characters.txt"
        )),
        name => panic!("the characters of {name} are not listed"),
    }
}

/// Each bundled font maps each character it provides to a glyph whose outline can be drawn.
#[test]
fn each_bundled_font_can_be_read_and_has_its_characters() {
    for font in &BUNDLED {
        let face = FontRef::new(font.data).unwrap_or_else(|error| panic!("{}: {error}", font.name));
        let (charmap, outlines) = (face.charmap(), face.outline_glyphs());
        for character in characters(font) {
            let drawn = charmap
                .map(character)
                .filter(|&glyph| glyph != GlyphId::NOTDEF)
                .and_then(|glyph| outlines.get(glyph))
                .map(|outline| {
                    outline.draw((Size::unscaled(), LocationRef::default()), &mut NullPen)
                });
            let code = u32::from(character);
            assert!(
                matches!(drawn, Some(Ok(_))),
                "{} cannot draw {character} (U+{code:04X})",
                font.name
            );
        }
    }
}

/// Noto Sans JP provides the 7,478 Japanese characters of the font research but ≒, which DejaVu Sans Mono provides, and so every character of the Unicode blocks that the research covers.
#[test]
fn noto_sans_jp_has_the_japanese_characters_of_the_font_research() {
    let noto_sans_jp = BUNDLED
        .iter()
        .find(|font| font.name == "Noto Sans JP")
        .unwrap();
    let japanese = characters(noto_sans_jp);
    assert_eq!(japanese.len(), 7_478 - 1);
    for character in JAPANESE_BLOCKS.into_iter().flatten() {
        let code = u32::from(character);
        assert!(japanese.contains(&character), "U+{code:04X} is missing");
    }
}

#[test]
fn the_bundled_fonts_stay_within_their_size_limit() {
    let size: usize = BUNDLED.iter().map(|font| font.data.len()).sum();
    assert!(
        size <= SIZE_LIMIT,
        "{size} bytes, over the limit of {SIZE_LIMIT} bytes"
    );
}
