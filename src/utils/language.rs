//! `Source/utils/language.cpp`: gettext `.mo`/`.gmo` translations and `_()`.
//!
//! The translation tables are process-wide read-only data after `language_initialize`, held in
//! a lock rather than threaded through every caller of `_()` (as in the original, which keeps
//! them in globals).

use std::collections::HashMap;
use std::sync::RwLock;

use crate::ctx::Ctx;
use crate::engine::assets::{find_asset, open_asset};

const MO_MAGIC: u32 = 0x9504_12de;
const EXTENSIONS: [&str; 2] = [".mo", ".gmo"];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum PluralRule {
    Zero,
    IfNotOne,
    GreaterThanOne,
    HrRu,
    Pl,
    Ro,
    Cs,
}

struct Translations {
    /// One map per plural form; `translation[0]` also holds singular translations.
    translation: Vec<HashMap<String, String>>,
    plural_forms: usize,
    plural: PluralRule,
}

static TRANSLATIONS: RwLock<Translations> =
    RwLock::new(Translations { translation: Vec::new(), plural_forms: 2, plural: PluralRule::IfNotOne });

/// Original: `PluralIfNotOne` and the lambdas installed by `SetPluralForm` (utils/language.cpp).
// @port utils/language.cpp|PluralIfNotOne(int n) sha=e55f90e5f446
fn plural_id(rule: PluralRule, n: i32) -> usize {
    match rule {
        PluralRule::Zero => 0,
        PluralRule::IfNotOne => (n != 1) as usize,
        PluralRule::GreaterThanOne => (n > 1) as usize,
        PluralRule::HrRu => {
            if n % 10 == 1 && n % 100 != 11 {
                0
            } else if n % 10 >= 2 && n % 10 <= 4 && (n % 100 < 12 || n % 100 > 14) {
                1
            } else {
                2
            }
        }
        PluralRule::Pl => {
            if n == 1 {
                0
            } else if n % 10 >= 2 && n % 10 <= 4 && (n % 100 < 12 || n % 100 > 14) {
                1
            } else {
                2
            }
        }
        PluralRule::Ro => {
            if n == 1 {
                0
            } else if n == 0 || (n != 1 && n % 100 >= 1 && n % 100 <= 19) {
                1
            } else {
                2
            }
        }
        PluralRule::Cs => {
            if n == 1 {
                0
            } else if (2..=4).contains(&n) {
                1
            } else {
                2
            }
        }
    }
}

/// Original: `TrimLeft` (utils/language.cpp).
// @port utils/language.cpp|TrimLeft(string_view str) sha=14ade506e8bf
fn trim_left(s: &str) -> &str {
    s.trim_start_matches([' ', '\t'])
}

/// Original: `TrimRight` (utils/language.cpp).
// @port utils/language.cpp|TrimRight(string_view str) sha=de22eb2815fb
fn trim_right(s: &str) -> &str {
    s.trim_end_matches([' ', '\t'])
}

/// Original: `SetPluralForm` (utils/language.cpp).
// @port utils/language.cpp|SetPluralForm(string_view expression) sha=6ebfa6accb84
fn set_plural_form(t: &mut Translations, expression: &str) {
    let key = "plural=";
    let Some(key_pos) = expression.find(key) else { return };
    let mut expression = &expression[key_pos + key.len()..];
    if let Some(semicolon) = expression.find(';') {
        expression = &expression[..semicolon];
    }
    let expression = trim_left(trim_right(expression));
    t.plural = match expression {
        "0" => PluralRule::Zero,
        "(n != 1)" => PluralRule::IfNotOne,
        "(n > 1)" => PluralRule::GreaterThanOne,
        "(n%10==1 && n%100!=11 ? 0 : n%10>=2 && n%10<=4 && (n%100<12 || n%100>14) ? 1 : 2)" => PluralRule::HrRu,
        "(n==1 ? 0 : n%10>=2 && n%10<=4 && (n%100<12 || n%100>14) ? 1 : 2)" => PluralRule::Pl,
        "(n==1 ? 0 : n==0 || (n!=1 && n%100>=1 && n%100<=19) ? 1 : 2)" => PluralRule::Ro,
        "(n==1) ? 0 : (n>=2 && n<=4) ? 1 : 2" => PluralRule::Cs,
        _ => {
            crate::platform::log::error!("Unknown plural expression: '{}'", expression);
            return;
        }
    };
}

/// Original: `ParsePluralForms` (utils/language.cpp).
// @port utils/language.cpp|ParsePluralForms(string_view string) sha=7d4aacaca3e7
fn parse_plural_forms(t: &mut Translations, string: &str) {
    let plurals_key = "nplurals";
    let Some(pos) = string.find(plurals_key) else { return };
    let string = &string[pos + plurals_key.len()..];
    let Some(eq) = string.find('=') else { return };
    let value = &string[eq + 1..];
    let Some(&first) = value.as_bytes().first() else { return };
    if first < b'0' {
        return;
    }
    let nplurals = (first - b'0') as usize;
    if nplurals == 0 {
        return;
    }
    t.plural_forms = nplurals;
    set_plural_form(t, value);
}

/// Original: `ParseMetadata` (utils/language.cpp).
// @port utils/language.cpp|ParseMetadata(string_view metadata) sha=17e8cbe02548
fn parse_metadata(t: &mut Translations, metadata: &str) {
    let mut metadata = metadata;
    while !metadata.is_empty() {
        let Some(delim) = metadata.find(':') else { break };
        let key = trim_left(&metadata[..delim]);
        let mut val = trim_left(&metadata[delim + 1..]);
        if let Some(nl) = val.find('\n') {
            val = &val[..nl];
            let consumed = (val.as_ptr() as usize - metadata.as_ptr() as usize) + val.len() + 1;
            metadata = &metadata[consumed..];
        } else {
            metadata = "";
        }
        if key == "Plural-Forms" {
            parse_plural_forms(t, val);
            break;
        }
    }
}

/// Original: `LanguageTranslate` (utils/language.cpp), the `_()` macro.
// @port utils/language.cpp|LanguageTranslate(const char *key) sha=3c69b907334a
pub fn tr(key: &str) -> String {
    let t = TRANSLATIONS.read().unwrap();
    t.translation.first().and_then(|m| m.get(key)).cloned().unwrap_or_else(|| key.to_string())
}

/// Original: `LanguagePluralTranslate` (utils/language.cpp), `ngettext`.
// @port utils/language.cpp|LanguagePluralTranslate(const char *singular, string_view plural, int count) sha=472f31042c2c
pub fn ngettext(singular: &str, plural: &str, count: i32) -> String {
    let t = TRANSLATIONS.read().unwrap();
    let n = plural_id(t.plural, count);
    match t.translation.get(n).and_then(|m| m.get(singular)) {
        Some(s) => s.clone(),
        None => if count != 1 { plural.to_string() } else { singular.to_string() },
    }
}

/// Original: `LanguageParticularTranslate` (utils/language.cpp), `pgettext`.
// @port utils/language.cpp|LanguageParticularTranslate(string_view context, string_view message) sha=26a2e02d66ee
pub fn pgettext(context: &str, message: &str) -> String {
    let key = format!("{context}\u{4}{message}");
    let t = TRANSLATIONS.read().unwrap();
    t.translation.first().and_then(|m| m.get(&key)).cloned().unwrap_or_else(|| message.to_string())
}

/// Original: `HasTranslation` (utils/language.cpp).
// @port utils/language.cpp|HasTranslation(const std::string &locale) sha=030113bd8542
pub fn has_translation(ctx: &mut Ctx, locale: &str) -> bool {
    if locale == "en" {
        return true;
    }
    EXTENSIONS.iter().any(|ext| find_asset(ctx, &format!("{locale}{ext}")).ok())
}

/// Original: `GetLanguageCode` (utils/language.cpp).
// @port utils/language.cpp|GetLanguageCode() sha=35a2e81e486d
pub fn get_language_code(ctx: &Ctx) -> String {
    if !ctx.diablo.force_locale.is_empty() {
        return ctx.diablo.force_locale.clone();
    }
    ctx.options.language.code.code().to_string()
}

/// Original: `IsSmallFontTall` (utils/language.cpp).
// @port utils/language.cpp|IsSmallFontTall() sha=ba919932e94a
pub fn is_small_font_tall(ctx: &Ctx) -> bool {
    let code = get_language_code(ctx);
    let code = &code[..code.len().min(2)];
    code == "zh" || code == "ja" || code == "ko"
}

fn read_u32(d: &[u8], off: usize) -> Option<u32> {
    d.get(off..off + 4).map(|b| u32::from_le_bytes(b.try_into().unwrap()))
}

/// Original: `LanguageInitialize` (utils/language.cpp). MO files are read whole (the original
/// does so for MPQ-backed files too).
// @port utils/language.cpp|LanguageInitialize() sha=11b7f2437e05
pub fn language_initialize(ctx: &mut Ctx) {
    {
        let mut t = TRANSLATIONS.write().unwrap();
        t.translation = vec![HashMap::new(), HashMap::new()];
    }
    let lang = get_language_code(ctx);
    if lang == "en" {
        TRANSLATIONS.write().unwrap().plural = PluralRule::IfNotOne;
        return;
    }
    if is_small_font_tall(ctx) && !crate::engine::render::text_render::have_extra_fonts(ctx) {
        let msg = format!(
            "fonts.mpq is required for locale \"{}\"\n\nPlease download fonts.mpq from:\ngithub.com/diasurgical/\ndevilutionx-assets/releases",
            get_language_code(ctx)
        );
        crate::diablo_ui::dialogs::ui_error_ok_dialog(ctx, "Missing fonts.mpq", &msg, true);
        ctx.diablo.force_locale = "en".to_string();
        TRANSLATIONS.write().unwrap().plural = PluralRule::IfNotOne;
        return;
    }
    let mut handle = None;
    for ext in EXTENSIONS {
        let h = open_asset(ctx, &format!("{lang}{ext}"));
        if h.ok() {
            handle = Some(h);
            break;
        }
    }
    let Some(handle) = handle else {
        ctx.diablo.force_locale = "en".to_string();
        TRANSLATIONS.write().unwrap().plural = PluralRule::IfNotOne;
        return;
    };
    let data = handle.into_bytes();
    let (Some(magic), Some(rev), Some(nb), Some(src_off), Some(dst_off)) =
        (read_u32(&data, 0), read_u32(&data, 4), read_u32(&data, 8), read_u32(&data, 12), read_u32(&data, 16))
    else {
        return;
    };
    if magic != MO_MAGIC {
        return;
    }
    let (major, minor) = (rev & 0xFFFF, rev >> 16);
    if major > 1 || minor > 1 {
        return;
    }
    let entry = |base: u32, i: u32| -> Option<(usize, usize)> {
        let o = base as usize + i as usize * 8;
        Some((read_u32(&data, o)? as usize, read_u32(&data, o + 4)? as usize))
    };
    let string = |(len, off): (usize, usize)| -> Option<String> {
        data.get(off..off + len).map(|b| String::from_utf8_lossy(b).into_owned())
    };
    let (Some(src0), Some(dst0)) = (entry(src_off, 0), entry(dst_off, 0)) else { return };
    if src0.0 != 0 {
        return;
    }
    let Some(header) = string(dst0) else { return };
    let mut t = TRANSLATIONS.write().unwrap();
    parse_metadata(&mut t, &header);
    let forms = t.plural_forms;
    t.translation = (0..forms).map(|_| HashMap::new()).collect();
    for i in 1..nb {
        let (Some(s), Some(d)) = (entry(src_off, i), entry(dst_off, i)) else { continue };
        let (Some(key), Some(value)) = (string(s), string(d)) else { continue };
        // Plural keys are "singular\0plural"; only the singular takes part in lookup.
        // Plural values hold one \0-terminated string per form. `emplace` keeps the first.
        let key = key.split('\0').next().unwrap_or("").to_string();
        for (j, v) in value.split('\0').enumerate().take(forms) {
            t.translation[j].entry(key.clone()).or_insert_with(|| v.to_string());
        }
    }
}
