//! KaizenLinguistics — Deterministic Grapheme-to-Phoneme (G2P) Transducers in Rust
//!
//! Implements exact historical phonological rules for:
//! 1. Koine Greek (`grc`):
//!    - Profile A: Restored 1st-Century Imperial Koine (Buth / Kantor / Alpha with Angela)
//!    - Profile B: Academic Erasmian (distinct vowel qualities for dictation)
//!    - Profile C: Modern / Liturgical Greek
//!    - Breathings, diphthongs, nasal assimilation, accents -> stress marks
//! 2. Biblical Hebrew (`hbo`):
//!    - Tiberian Masoretic vs. Academic Sephardic
//!    - Begadkefat spirantization (plosive stop with dagesh lene vs. fricative without)
//!    - Dagesh forte consonant gemination (/Cː/)
//!    - Matres lectionis (Vav as Shuruq /uː/ or Holam Male /oː/, Yod as /iː/)
//!    - Shewa classification (vocal /ə/ vs. silent ∅)
//!    - Word-final Furtive Pathach transposition on gutturals (חַ, עַ, הַּ -> /aħ/, /aʕ/, /ah/)

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum GreekPhonologyProfile {
    RestoredKoine1C,
    Erasmian,
    Modern,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum HebrewPhonologyProfile {
    TiberianMasoretic,
    AcademicSephardic,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GraphemeSpan {
    pub grapheme: String,
    pub ipa: String,
    pub char_start: usize,
    pub char_end: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct G2pResult {
    pub full_ipa: String,
    pub phone_tokens: Vec<String>,
    pub spans: Vec<GraphemeSpan>,
}

// ============================================================================
// 1. KOINE GREEK DETERMINISTIC G2P
// ============================================================================

const COMBINING_ROUGH_BREATHING: char = '\u{0314}'; // ῾
const COMBINING_ACUTE: char = '\u{0301}';
const COMBINING_GRAVE: char = '\u{0300}';
const COMBINING_CIRCUMFLEX: char = '\u{0342}';
const COMBINING_DIAERESIS: char = '\u{0308}';

fn decompose_greek_char(ch: char) -> (char, bool, bool, bool) {
    let mut base = ch.to_lowercase().next().unwrap_or(ch);
    let mut has_rough = false;
    let mut has_accent = false;
    let mut has_diaeresis = false;

    // Direct precomposed mapping fallback
    match ch {
        'ἁ' | 'ἑ' | 'ἡ' | 'ἱ' | 'ὁ' | 'ὑ' | 'ὡ' | 'ῥ' | 'ἇ' | 'ἧ' | 'ἷ' | 'ὗ' | 'ὧ' => {
            has_rough = true;
        }
        'ἅ' | 'ἕ' | 'ἥ' | 'ἵ' | 'ὅ' | 'ὕ' | 'ὥ' | 'ἣ' | 'ἃ' | 'ἓ' | 'ἳ' | 'ὃ' | 'ὓ' | 'ὣ' => {
            has_rough = true;
            has_accent = true;
        }
        'ἄ' | 'ἔ' | 'ἤ' | 'ἴ' | 'ὄ' | 'ὔ' | 'ὤ' | 'ὰ' | 'ὲ' | 'ὴ' | 'ὶ' | 'ὸ' | 'ὺ' | 'ὼ'
        | 'ᾶ' | 'ῆ' | 'ῖ' | 'ῦ' | 'ῶ' | 'ά' | 'έ' | 'ή' | 'ί' | 'ό' | 'ύ' | 'ώ' => {
            has_accent = true;
        }
        'ϊ' | 'ϋ' | 'ΐ' | 'ΰ' => {
            has_diaeresis = true;
            if ch == 'ΐ' || ch == 'ΰ' {
                has_accent = true;
            }
        }
        _ => {}
    }

    // Map Greek vowel variants to standard lowercase ASCII/Greek base
    match ch {
        'ἁ' | 'ἅ' | 'ἄ' | 'ὰ' | 'ᾶ' | 'ά' | 'ἀ' | 'ἂ' | 'ἃ' | 'ἆ' | 'ἇ' | 'ᾀ' | 'ᾁ' | 'ᾂ' | 'ᾃ' | 'ᾄ' | 'ᾅ' | 'ᾆ' | 'ᾇ' | 'ᾲ' | 'ᾳ' | 'ᾴ' | 'ᾷ' => base = 'α',
        'ἑ' | 'ἕ' | 'ἔ' | 'ὲ' | 'έ' | 'ἐ' | 'ἒ' | 'ἓ' => base = 'ε',
        'ἡ' | 'ἥ' | 'ἤ' | 'ὴ' | 'ῆ' | 'ή' | 'ἠ' | 'ἢ' | 'ἣ' | 'ἦ' | 'ἧ' | 'ᾐ' | 'ᾑ' | 'ᾒ' | 'ᾓ' | 'ᾔ' | 'ᾕ' | 'ᾖ' | 'ᾗ' | 'ῂ' | 'ῃ' | 'ῄ' | 'ῇ' => base = 'η',
        'ἱ' | 'ἵ' | 'ἴ' | 'ὶ' | 'ῖ' | 'ί' | 'ἰ' | 'ἲ' | 'ἳ' | 'ἶ' | 'ἷ' | 'ϊ' | 'ΐ' | 'ῒ' | 'ῗ' => base = 'ι',
        'ὁ' | 'ὅ' | 'ὄ' | 'ὸ' | 'ό' | 'ὀ' | 'ὂ' | 'ὃ' => base = 'ο',
        'ὑ' | 'ὕ' | 'ὔ' | 'ὺ' | 'ῦ' | 'ύ' | 'ὐ' | 'ὒ' | 'ὓ' | 'ὖ' | 'ὗ' | 'ϋ' | 'ΰ' | 'ῢ' | 'ῧ' => base = 'υ',
        'ὡ' | 'ὥ' | 'ὤ' | 'ὼ' | 'ῶ' | 'ώ' | 'ὠ' | 'ὢ' | 'ὣ' | 'ὦ' | 'ὧ' | 'ᾠ' | 'ᾡ' | 'ᾢ' | 'ᾣ' | 'ᾤ' | 'ᾥ' | 'ᾦ' | 'ᾧ' | 'ῲ' | 'ῳ' | 'ῴ' | 'ῷ' => base = 'ω',
        'ῥ' | 'ῤ' => base = 'ρ',
        _ => {}
    }

    // Check combining diacritics
    let s = ch.to_string();
    for c in s.chars() {
        if c == COMBINING_ROUGH_BREATHING {
            has_rough = true;
        } else if c == COMBINING_ACUTE || c == COMBINING_GRAVE || c == COMBINING_CIRCUMFLEX {
            has_accent = true;
        } else if c == COMBINING_DIAERESIS {
            has_diaeresis = true;
        }
    }

    (base, has_rough, has_accent, has_diaeresis)
}

fn greek_digraph_ipa(pair: &str, profile: GreekPhonologyProfile) -> Option<&'static str> {
    match (pair, profile) {
        ("αι", GreekPhonologyProfile::RestoredKoine1C) => Some("ɛ"),
        ("αι", GreekPhonologyProfile::Erasmian) => Some("ai"),
        ("αι", GreekPhonologyProfile::Modern) => Some("e"),

        ("ει", GreekPhonologyProfile::RestoredKoine1C) => Some("iː"),
        ("ει", GreekPhonologyProfile::Erasmian) => Some("eɪ"),
        ("ει", GreekPhonologyProfile::Modern) => Some("i"),

        ("οι", GreekPhonologyProfile::RestoredKoine1C) => Some("yː"),
        ("οι", GreekPhonologyProfile::Erasmian) => Some("ɔɪ"),
        ("οι", GreekPhonologyProfile::Modern) => Some("i"),

        ("υι", GreekPhonologyProfile::RestoredKoine1C) => Some("yː"),
        ("υι", GreekPhonologyProfile::Erasmian) => Some("ɥi"),
        ("υι", GreekPhonologyProfile::Modern) => Some("i"),

        ("αυ", GreekPhonologyProfile::RestoredKoine1C) => Some("aw"),
        ("αυ", GreekPhonologyProfile::Erasmian) => Some("au"),
        ("αυ", GreekPhonologyProfile::Modern) => Some("av"),

        ("ευ", GreekPhonologyProfile::RestoredKoine1C) => Some("ɛw"),
        ("ευ", GreekPhonologyProfile::Erasmian) => Some("ɛu"),
        ("ευ", GreekPhonologyProfile::Modern) => Some("ev"),

        ("ηυ", GreekPhonologyProfile::RestoredKoine1C) => Some("eːw"),
        ("ηυ", GreekPhonologyProfile::Erasmian) => Some("ɛːu"),
        ("ηυ", GreekPhonologyProfile::Modern) => Some("iv"),

        ("ου", GreekPhonologyProfile::RestoredKoine1C) => Some("uː"),
        ("ου", GreekPhonologyProfile::Erasmian) => Some("uː"),
        ("ου", GreekPhonologyProfile::Modern) => Some("u"),

        ("γγ", _) => Some("ŋɡ"),
        ("γκ", GreekPhonologyProfile::Modern) => Some("ŋɡ"),
        ("γκ", _) => Some("ŋk"),
        ("γχ", GreekPhonologyProfile::Erasmian) | ("γχ", GreekPhonologyProfile::Modern) => Some("ŋx"),
        ("γχ", GreekPhonologyProfile::RestoredKoine1C) => Some("ŋkʰ"),
        ("γξ", _) => Some("ŋks"),

        _ => None,
    }
}

fn greek_monograph_ipa(base: char, profile: GreekPhonologyProfile) -> Option<&'static str> {
    match (base, profile) {
        ('α', _) => Some("a"),
        ('β', GreekPhonologyProfile::Modern) => Some("v"),
        ('β', _) => Some("b"),
        ('γ', GreekPhonologyProfile::Modern) => Some("ɣ"),
        ('γ', _) => Some("ɡ"),
        ('δ', GreekPhonologyProfile::Modern) => Some("ð"),
        ('δ', _) => Some("d"),
        ('ε', GreekPhonologyProfile::Modern) => Some("e"),
        ('ε', _) => Some("ɛ"),
        ('ζ', GreekPhonologyProfile::Erasmian) => Some("dz"),
        ('ζ', _) => Some("z"),
        ('η', GreekPhonologyProfile::RestoredKoine1C) => Some("eː"),
        ('η', GreekPhonologyProfile::Erasmian) => Some("ɛː"),
        ('η', GreekPhonologyProfile::Modern) => Some("i"),
        ('θ', GreekPhonologyProfile::RestoredKoine1C) => Some("tʰ"),
        ('θ', _) => Some("θ"),
        ('ι', _) => Some("i"),
        ('κ', _) => Some("k"),
        ('λ', _) => Some("l"),
        ('μ', _) => Some("m"),
        ('ν', _) => Some("n"),
        ('ξ', _) => Some("ks"),
        ('ο', GreekPhonologyProfile::Erasmian) => Some("ɔ"),
        ('ο', _) => Some("o"),
        ('π', _) => Some("p"),
        ('ρ', _) => Some("r"),
        ('σ' | 'ς', _) => Some("s"),
        ('τ', _) => Some("t"),
        ('υ', GreekPhonologyProfile::Modern) => Some("i"),
        ('υ', _) => Some("y"),
        ('φ', GreekPhonologyProfile::RestoredKoine1C) => Some("pʰ"),
        ('φ', _) => Some("f"),
        ('χ', GreekPhonologyProfile::RestoredKoine1C) => Some("kʰ"),
        ('χ', _) => Some("x"),
        ('ψ', _) => Some("ps"),
        ('ω', _) => Some("oː"),
        _ => None,
    }
}

pub fn g2p_koine_greek(text: &str, profile: GreekPhonologyProfile) -> G2pResult {
    let chars: Vec<char> = text.chars().collect();
    let mut phones: Vec<String> = Vec::new();
    let mut spans: Vec<GraphemeSpan> = Vec::new();
    let mut word_ipa_parts: Vec<String> = Vec::new();
    let mut current_word_tokens: Vec<String> = Vec::new();

    let mut i = 0;
    let n = chars.len();

    while i < n {
        let ch = chars[i];
        if !ch.is_alphabetic() {
            if !current_word_tokens.is_empty() {
                word_ipa_parts.push(current_word_tokens.join(""));
                current_word_tokens.clear();
            }
            i += 1;
            continue;
        }

        let (base1, rough1, accent1, _) = decompose_greek_char(ch);

        // Check digraph
        if i + 1 < n && chars[i + 1].is_alphabetic() {
            let (base2, rough2, accent2, diaeresis2) = decompose_greek_char(chars[i + 1]);
            let pair = format!("{}{}", base1, base2);

            if !diaeresis2 {
                if let Some(ipa_val) = greek_digraph_ipa(&pair, profile) {
                    let has_rough = rough1 || rough2;
                    let has_accent = accent1 || accent2;

                    let mut prefix = String::new();
                    if has_rough && profile != GreekPhonologyProfile::Modern {
                        prefix.push('h');
                        phones.push("h".to_string());
                    }
                    if has_accent {
                        prefix.push('ˈ');
                    }
                    phones.push(ipa_val.to_string());

                    let tok = format!("{}{}", prefix, ipa_val);
                    current_word_tokens.push(tok.clone());

                    spans.push(GraphemeSpan {
                        grapheme: chars[i..=i + 1].iter().collect(),
                        ipa: tok,
                        char_start: i,
                        char_end: i + 2,
                    });

                    i += 2;
                    continue;
                }
            }
        }

        // Monograph fallback
        if let Some(mut ipa_val) = greek_monograph_ipa(base1, profile) {
            let mut prefix = String::new();
            if rough1 && profile != GreekPhonologyProfile::Modern {
                if base1 == 'ρ' {
                    ipa_val = "r̥";
                } else {
                    prefix.push('h');
                    phones.push("h".to_string());
                }
            }
            if accent1 {
                prefix.push('ˈ');
            }
            phones.push(ipa_val.to_string());

            let tok = format!("{}{}", prefix, ipa_val);
            current_word_tokens.push(tok.clone());

            spans.push(GraphemeSpan {
                grapheme: ch.to_string(),
                ipa: tok,
                char_start: i,
                char_end: i + 1,
            });
        }

        i += 1;
    }

    if !current_word_tokens.is_empty() {
        word_ipa_parts.push(current_word_tokens.join(""));
    }

    let full_ipa = format!("/{}/", word_ipa_parts.join(" "));
    G2pResult {
        full_ipa,
        phone_tokens: phones,
        spans,
    }
}

// ============================================================================
// 2. BIBLICAL HEBREW DETERMINISTIC G2P
// ============================================================================

const DAGESH_POINT: char = '\u{05BC}';
const SHIN_DOT: char = '\u{05C1}';
const SIN_DOT: char = '\u{05C2}';

fn hebrew_consonant_pair(ch: char) -> Option<(&'static str, &'static str)> {
    // (spirant_without_dagesh, plosive_with_dagesh)
    match ch {
        'א' => Some(("ʔ", "ʔ")),
        'ב' => Some(("v", "b")),
        'ג' => Some(("ɣ", "ɡ")),
        'ד' => Some(("ð", "d")),
        'ה' => Some(("h", "h")),
        'ו' => Some(("w", "w")),
        'ז' => Some(("z", "z")),
        'ח' => Some(("ħ", "ħ")),
        'ט' => Some(("tˤ", "tˤ")),
        'י' => Some(("j", "j")),
        'כ' | 'ך' => Some(("x", "k")),
        'ל' => Some(("l", "l")),
        'מ' | 'ם' => Some(("m", "m")),
        'נ' | 'ן' => Some(("n", "n")),
        'ס' => Some(("s", "s")),
        'ע' => Some(("ʕ", "ʕ")),
        'פ' | 'ף' => Some(("f", "p")),
        'צ' | 'ץ' => Some(("sˤ", "sˤ")),
        'ק' => Some(("q", "q")),
        'ר' => Some(("r", "r")),
        'ש' => Some(("ʃ", "ʃ")), // shin/sin dot resolves
        'ת' => Some(("θ", "t")),
        _ => None,
    }
}

fn hebrew_vowel_ipa(mark: char) -> Option<&'static str> {
    match mark {
        '\u{05B0}' => Some("ə"),   // Shewa
        '\u{05B1}' => Some("ĕ"),   // Hataf Segol
        '\u{05B2}' => Some("ă"),   // Hataf Pathach
        '\u{05B3}' => Some("ŏ"),   // Hataf Qamatz
        '\u{05B4}' => Some("i"),   // Hireq
        '\u{05B5}' => Some("eː"),  // Tsere
        '\u{05B6}' => Some("ɛ"),   // Segol
        '\u{05B7}' => Some("a"),   // Pathach
        '\u{05B8}' => Some("aː"),  // Qamatz
        '\u{05B9}' | '\u{05BA}' => Some("oː"), // Holam
        '\u{05BB}' => Some("u"),   // Qubutz
        _ => None,
    }
}

pub fn g2p_biblical_hebrew(text: &str, profile: HebrewPhonologyProfile) -> G2pResult {
    let chars: Vec<char> = text.chars().collect();
    let n = chars.len();

    // Group consonants and attached vowel/diacritic marks
    struct Cluster {
        base: char,
        marks: Vec<char>,
        start: usize,
        end: usize,
    }

    let mut clusters: Vec<Cluster> = Vec::new();
    let mut i = 0;

    while i < n {
        let ch = chars[i];
        if hebrew_consonant_pair(ch).is_some() {
            let start = i;
            let mut marks = Vec::new();
            i += 1;
            while i < n && ('\u{0591}'..='\u{05CF}').contains(&chars[i]) && hebrew_consonant_pair(chars[i]).is_none() {
                marks.push(chars[i]);
                i += 1;
            }
            clusters.push(Cluster {
                base: ch,
                marks,
                start,
                end: i,
            });
        } else {
            clusters.push(Cluster {
                base: ch,
                marks: Vec::new(),
                start: i,
                end: i + 1,
            });
            i += 1;
        }
    }

    let mut phones: Vec<String> = Vec::new();
    let mut spans: Vec<GraphemeSpan> = Vec::new();
    let mut word_ipa_parts: Vec<String> = Vec::new();
    let mut current_word_tokens: Vec<String> = Vec::new();

    let mut prev_had_short_vowel = false;
    let mut is_word_initial = true;

    let mut idx = 0;
    while idx < clusters.len() {
        let c = &clusters[idx];
        let base_pair = hebrew_consonant_pair(c.base);

        if base_pair.is_none() {
            if c.base == ' ' || c.base == '\u{05BE}' { // space or Maqaf
                if !current_word_tokens.is_empty() {
                    word_ipa_parts.push(current_word_tokens.join(""));
                    current_word_tokens.clear();
                }
                is_word_initial = true;
                prev_had_short_vowel = false;
            }
            idx += 1;
            continue;
        }

        let has_dagesh = c.marks.contains(&DAGESH_POINT);
        let has_holam = c.marks.contains(&'\u{05B9}') || c.marks.contains(&'\u{05BA}');
        let vowel_marks: Vec<char> = c.marks.iter().copied().filter(|&m| hebrew_vowel_ipa(m).is_some()).collect();

        // Shuruq /uː/: Vav with dagesh and no vowel mark, not word-initial
        if c.base == 'ו' && !is_word_initial && has_dagesh && vowel_marks.is_empty() {
            phones.push("uː".to_string());
            current_word_tokens.push("uː".to_string());
            spans.push(GraphemeSpan {
                grapheme: chars[c.start..c.end].iter().collect(),
                ipa: "uː".to_string(),
                char_start: c.start,
                char_end: c.end,
            });
            prev_had_short_vowel = false;
            is_word_initial = false;
            idx += 1;
            continue;
        }

        // Holam Male /oː/: Vav with Holam
        if c.base == 'ו' && !is_word_initial && has_holam && vowel_marks.len() == 1 {
            phones.push("oː".to_string());
            current_word_tokens.push("oː".to_string());
            spans.push(GraphemeSpan {
                grapheme: chars[c.start..c.end].iter().collect(),
                ipa: "oː".to_string(),
                char_start: c.start,
                char_end: c.end,
            });
            prev_had_short_vowel = false;
            is_word_initial = false;
            idx += 1;
            continue;
        }

        let (mut spirant, plosive) = base_pair.unwrap();

        // Academic Sephardic overrides spirant g/d/t to plosive
        if profile == HebrewPhonologyProfile::AcademicSephardic {
            if c.base == 'ג' || c.base == 'ד' || c.base == 'ת' {
                spirant = plosive;
            }
        }

        let mut cons_ipa = if c.base == 'ש' {
            if c.marks.contains(&SIN_DOT) {
                "s".to_string()
            } else if c.marks.contains(&SHIN_DOT) {
                "ʃ".to_string()
            } else {
                "ʃ".to_string()
            }
        } else if has_dagesh {
            plosive.to_string()
        } else {
            spirant.to_string()
        };

        // Dagesh Forte gemination: after short vowel inside a word, except gutturals & resh
        if has_dagesh && !is_word_initial && prev_had_short_vowel && !matches!(c.base, 'א' | 'ה' | 'ח' | 'ע' | 'ר') {
            cons_ipa.push('ː');
        }

        // Silent final He/Aleph without vowel/dagesh
        let is_word_final = (idx + 1 == clusters.len()) || hebrew_consonant_pair(clusters[idx + 1].base).is_none();
        if is_word_final && (c.base == 'ה' || c.base == 'א') && vowel_marks.is_empty() && !has_dagesh {
            cons_ipa.clear();
        }

        // Vowel resolution
        let mut v_ipa = String::new();
        for &vm in &vowel_marks {
            if vm == '\u{05B0}' { // Shewa
                if is_word_final {
                    // Silent final shewa
                } else if is_word_initial || !prev_had_short_vowel || has_dagesh {
                    v_ipa = "ə".to_string(); // Vocal shewa
                }
            } else if let Some(v) = hebrew_vowel_ipa(vm) {
                v_ipa = v.to_string();
            }
        }

        // Check Mater Lectionis Yod after Hiriq/Tsere/Segol
        let mut consumed_mater = false;
        if idx + 1 < clusters.len()
            && clusters[idx + 1].base == 'י'
            && clusters[idx + 1].marks.is_empty()
            && matches!(v_ipa.as_str(), "i" | "eː" | "ɛ")
        {
            if v_ipa == "i" {
                v_ipa = "iː".to_string();
            }
            consumed_mater = true;
        }

        // Check Furtive Pathach on word-final gutturals (חַ, עַ, הַּ)
        let cluster_str = if is_word_final && matches!(c.base, 'ח' | 'ע' | 'ה') && v_ipa == "a" && !prev_had_short_vowel {
            phones.push("a".to_string());
            if !cons_ipa.is_empty() {
                phones.push(cons_ipa.clone());
            }
            format!("a{}", cons_ipa)
        } else {
            let res = format!("{}{}", cons_ipa, v_ipa);
            if !cons_ipa.is_empty() {
                phones.push(cons_ipa.clone());
            }
            if !v_ipa.is_empty() {
                phones.push(v_ipa.clone());
            }
            res
        };

        if !cluster_str.is_empty() {
            current_word_tokens.push(cluster_str.clone());
            let span_end = if consumed_mater { clusters[idx + 1].end } else { c.end };
            spans.push(GraphemeSpan {
                grapheme: chars[c.start..span_end].iter().collect(),
                ipa: cluster_str,
                char_start: c.start,
                char_end: span_end,
            });
        }

        prev_had_short_vowel = matches!(v_ipa.as_str(), "i" | "ɛ" | "a" | "u" | "ă" | "ĕ" | "ŏ");
        is_word_initial = false;

        if consumed_mater {
            idx += 2;
        } else {
            idx += 1;
        }
    }

    if !current_word_tokens.is_empty() {
        word_ipa_parts.push(current_word_tokens.join(""));
    }

    let full_ipa = format!("/{}/", word_ipa_parts.join(" "));
    G2pResult {
        full_ipa,
        phone_tokens: phones,
        spans,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_koine_greek_g2p_restored_profile() {
        let sentence = "ὁ ἄγγελος βλέπει τὸν ἀδελφόν";
        let res = g2p_koine_greek(sentence, GreekPhonologyProfile::RestoredKoine1C);

        assert!(res.full_ipa.contains("aŋɡɛlos"));
        assert!(res.full_ipa.contains("blˈɛpiː"));
        assert!(res.full_ipa.contains("adɛlpʰˈon"));
        assert!(res.phone_tokens.contains(&"h".to_string()));
        assert!(res.phone_tokens.contains(&"ŋɡ".to_string()));
        assert!(res.phone_tokens.contains(&"pʰ".to_string()));
    }

    #[test]
    fn test_koine_greek_profile_differences() {
        let text = "καὶ ἔβλεψεν";
        let restored = g2p_koine_greek(text, GreekPhonologyProfile::RestoredKoine1C);
        let erasmian = g2p_koine_greek(text, GreekPhonologyProfile::Erasmian);
        let modern = g2p_koine_greek(text, GreekPhonologyProfile::Modern);

        // Restored has αι -> /ɛ/ (with stress mark: /kˈɛ/)
        assert!(restored.full_ipa.contains("kˈɛ"));
        // Erasmian has αι -> /ai/ (with stress mark: /kˈai/)
        assert!(erasmian.full_ipa.contains("kˈai"));
        // Modern has αι -> /e/ and β -> /v/
        assert!(modern.full_ipa.contains("kˈe"));
        assert!(modern.full_ipa.contains("v"));
    }

    #[test]
    fn test_biblical_hebrew_tiberian_g2p() {
        let text = "הַנַּעַר יֹשֵׁב עַל הָאֶבֶן";
        let res = g2p_biblical_hebrew(text, HebrewPhonologyProfile::TiberianMasoretic);

        // Dagesh forte on nun after short pathach -> /nː/
        assert!(res.phone_tokens.contains(&"nː".to_string()));
        // Pharyngeal Ayin -> /ʕ/
        assert!(res.phone_tokens.contains(&"ʕ".to_string()));
        // Shin dot -> /ʃ/
        assert!(res.phone_tokens.contains(&"ʃ".to_string()));
        // Bet without dagesh in yoshev -> /v/
        assert!(res.phone_tokens.contains(&"v".to_string()));
    }

    #[test]
    fn test_hebrew_furtive_pathach() {
        // רוּחַ (Ruach) - furtive pathach must transpose before chet -> /ruːaħ/
        let text = "רוּחַ";
        let res = g2p_biblical_hebrew(text, HebrewPhonologyProfile::TiberianMasoretic);
        assert!(res.full_ipa.contains("ruːaħ"));
    }

    #[test]
    fn test_hebrew_dagesh_forte_piel() {
        // וַיְקַדֵּשׁ (Vayqaddesh - Piel geminated dalet /dː/)
        let text = "וַיְקַדֵּשׁ";
        let res = g2p_biblical_hebrew(text, HebrewPhonologyProfile::TiberianMasoretic);
        assert!(res.phone_tokens.contains(&"dː".to_string()));
    }
}
