//! Font discovery (system Consolas) and the egui `FontDefinitions` fallback chain, including Japanese.
//!
//! # Font choice
//!
//! egui draws each character with the first font of this chain that has it:
//!
//! 1. Consolas, the main font, when it is installed: it is proprietary, so the app never includes it.
//! 2. DejaVu Sans Mono, included in the app, which replaces Consolas when it is missing.
//! 3. Noto Sans JP 2.004, included in a reduced version.
//! 4. Noto Sans Symbols 2 and Noto Sans Symbols, reduced to the symbols that the fonts above lack.
//! 5. A Japanese font installed on the computer, if any, such as Yu Gothic on Windows or Noto Sans CJK on Linux.
//!
//! Text stays on the columns of the main font. Latin letters and DejaVu Sans Mono's symbols take
//! one column: next to Consolas, DejaVu Sans Mono is narrowed by 9% to fit. Japanese characters and
//! the Noto symbols take two: they are enlarged by 10% with Consolas and 20% with DejaVu Sans Mono,
//! and each Noto symbol is centered in the width of a Japanese character, and shrunk if it is
//! wider. The reduced Noto fonts have no kerning, which would narrow pairs such as ア。.
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
//! - the Unicode blocks Hiragana, Katakana, Katakana Phonetic Extensions, CJK Symbols and Punctuation, and Halfwidth and Fullwidth Forms.
//!
//! The reduced Noto Sans JP keeps all of them but ≒, which DejaVu Sans Mono has. Rarer characters
//! come from the installed Japanese font, or show as a missing-character box without one. Every
//! font is used in its regular weight only, as egui shows emphasis with colour and size.
