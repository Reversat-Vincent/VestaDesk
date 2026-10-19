//! Font discovery (an installed Japanese font) and the egui `FontDefinitions` fallback chain, including Japanese.
//!
//! # Font choice
//!
//! egui draws each character with the first font of this chain that has it:
//!
//! 1. DejaVu Sans Mono, the main font, included in the app.
//! 2. Noto Sans JP 2.004, included in a reduced version.
//! 3. Noto Sans Symbols 2 and Noto Sans Symbols, reduced to the symbols that the fonts above lack.
//! 4. A Japanese font installed on the computer, if any, such as Yu Gothic on Windows or Noto Sans CJK on Linux.
//!
//! Text stays on the columns of the main font. Latin letters and DejaVu Sans Mono's symbols take
//! one column. Japanese characters and the Noto symbols take two: they are enlarged by 20%, and
//! each Noto symbol is centered in the width of a Japanese character, and shrunk if it is wider.
//! The reduced Noto fonts have no kerning, which would narrow pairs such as ア。.
//!
//! egui's built-in fonts are turned off, so the app has no emoji. The fonts above have every symbol
//! that egui's widgets use, such as ⏴⏵⏶⏷, except ⟲, which only its style editor shows. They also
//! cover 1,444 of the 1,725 symbols of the Unicode blocks of arrows, mathematical operators,
//! technical symbols, box drawing, geometric shapes, miscellaneous symbols and dingbats.
//!
//! With no font installed, the app can show these 7,478 characters of Japanese text:
//!
//! - every character of JIS X 0208: 2,965 level 1 kanji, 3,390 level 2 kanji and 524 others;
//! - those that code page 932 adds to it on Windows;
//! - the 2,136 Jōyō kanji, including the 4 that JIS X 0208 lacks: 剝, 塡, 頰 and 𠮟;
//! - the Unicode blocks Hiragana, Katakana, Katakana Phonetic Extensions, CJK Symbols and Punctuation, and Halfwidth and Fullwidth Forms but its halfwidth Hangul.
//!
//! The reduced Noto Sans JP keeps all of them but ≒, which DejaVu Sans Mono has. Rarer characters
//! come from the installed Japanese font, or show as a missing-character box without one. Every
//! font is used in its regular weight only, as egui shows emphasis with colour and size.
//!
//! # Bundled fonts
//!
//! The fonts of [`BUNDLED`] are in `assets/fonts`, each next to its licence, such as `DejaVuSansMono-LICENSE.txt`:
//!
//! | Font | Version | Original | SHA-256 of the original | Licence |
//! | --- | --- | --- | --- | --- |
//! | DejaVu Sans Mono | 2.37 | `ttf/DejaVuSansMono.ttf` in [dejavu-fonts-ttf-2.37.tar.bz2] | `b4a6c3e4faab8773f4ff761d56451646409f29abedd68f05d38c2df667d3c582` | Bitstream Vera and Arev |
//! | Noto Sans JP | 2.004 | [NotoSansJP-Regular.otf] | `dff723ba59d57d136764a04b9b2d03205544f7cd785a711442d6d2d085ac5073` | OFL 1.1 |
//! | Noto Sans Symbols 2 | 2.008 | `NotoSansSymbols2/hinted/ttf/NotoSansSymbols2-Regular.ttf` in [NotoSansSymbols2-v2.008.zip] | `c4a0a80f0041ce4be81e2478faad22776d23edb98ae3f0d19bd37044820ecf9d` | OFL 1.1 |
//! | Noto Sans Symbols | 2.003 | `NotoSansSymbols/hinted/ttf/NotoSansSymbols-Regular.ttf` in [NotoSansSymbols-v2.003.zip] | `d0e98e9a2c046594c5021437273943be7e79e0fd980fde125279e22302212595` | OFL 1.1 |
//!
//! [dejavu-fonts-ttf-2.37.tar.bz2]: https://github.com/dejavu-fonts/dejavu-fonts/releases/download/version_2_37/dejavu-fonts-ttf-2.37.tar.bz2
//! [NotoSansJP-Regular.otf]: https://github.com/notofonts/noto-cjk/raw/Sans2.004/Sans/SubsetOTF/JP/NotoSansJP-Regular.otf
//! [NotoSansSymbols2-v2.008.zip]: https://github.com/notofonts/symbols/releases/download/NotoSansSymbols2-v2.008/NotoSansSymbols2-v2.008.zip
//! [NotoSansSymbols-v2.003.zip]: https://github.com/notofonts/symbols/releases/download/NotoSansSymbols-v2.003/NotoSansSymbols-v2.003.zip
//!
//! DejaVu Sans Mono is included as is. The reduced Noto fonts keep the characters of their
//! `-characters.txt` file: `python assets/fonts/reduce.py <folder of the originals>` makes them,
//! with fontTools 4.67.0.
//!
//! `tests/fonts.rs` checks that each bundled font can be read and has the characters it provides,
//! that Noto Sans JP has the Japanese characters above, and that together the fonts weigh 2.5 MiB
//! at most.

use std::sync::Arc;

use egui::{Context, FontData, FontDefinitions, FontFamily};

/// A font file included in the app.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BundledFont {
    /// The font's name in egui's font definitions.
    pub name: &'static str,
    /// The content of the font file.
    pub data: &'static [u8],
}

/// The fonts included in the app, in the order that egui tries them for each character.
pub static BUNDLED: [BundledFont; 4] = [
    BundledFont {
        name: "DejaVu Sans Mono",
        data: include_bytes!("../assets/fonts/DejaVuSansMono.ttf"),
    },
    BundledFont {
        name: "Noto Sans JP",
        data: include_bytes!("../assets/fonts/NotoSansJP-Regular.otf"),
    },
    BundledFont {
        name: "Noto Sans Symbols 2",
        data: include_bytes!("../assets/fonts/NotoSansSymbols2-Regular.ttf"),
    },
    BundledFont {
        name: "Noto Sans Symbols",
        data: include_bytes!("../assets/fonts/NotoSansSymbols-Regular.ttf"),
    },
];

/// Returns egui's font definitions with the bundled fonts only, which all text tries in the order of [`BUNDLED`].
///
/// # Examples
///
/// ```
/// use egui::FontFamily;
/// use vesta_desk::fonts;
///
/// let definitions = fonts::definitions();
/// assert_eq!(definitions.families[&FontFamily::Proportional][0], "DejaVu Sans Mono");
/// ```
#[must_use]
pub fn definitions() -> FontDefinitions {
    let names: Vec<String> = BUNDLED.iter().map(|font| font.name.to_owned()).collect();
    FontDefinitions {
        font_data: BUNDLED
            .iter()
            .map(|font| {
                let data = Arc::new(FontData::from_static(font.data));
                (font.name.to_owned(), data)
            })
            .collect(),
        families: [FontFamily::Proportional, FontFamily::Monospace]
            .into_iter()
            .map(|family| (family, names.clone()))
            .collect(),
    }
}

/// Makes egui draw all text with the bundled fonts.
pub fn install(ctx: &Context) {
    ctx.set_fonts(definitions());
}
