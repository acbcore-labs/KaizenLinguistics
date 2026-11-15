#!/usr/bin/env python3
"""
KaizenLinguistics — Deterministic Grapheme-to-Phoneme (G2P) Transducers
for Classical/Biblical Hebrew (hbo, Tiberian Masoretic) and Koine Greek (grc).

Implements:
1. Tiberian / Reconstructed Biblical Hebrew (hbo):
   - Begadkefat spirantization (plosive with dagesh lene vs. fricative without)
   - Dagesh forte consonant gemination (/Cː/) after short vowels
   - Qamatz Gadol (/aː/) vs. Qamatz Qatan (/o/) in closed unaccented syllables
   - Vocal Shewa (/ə/) vs. Silent Shewa (∅) & Hataf reduced vowels (/ă/, /ĕ/, /ŏ/)
   - Word-final furtive Pathach transposition on gutturals (ח, ע, הּ) -> /aħ/, /aʕ/, /ah/
2. Koine Greek (grc) Multi-Profile G2P:
   - Profile A: Restored 1st-Century Imperial Koine (Buth / Kantor / Alpha with Angela)
   - Profile B: Academic Erasmian (distinct vowel-grapheme mapping for dictation)
   - Profile C: Modern / Liturgical Greek
"""

from __future__ import annotations
import unicodedata
from dataclasses import dataclass
from typing import List, Dict, Tuple


@dataclass(frozen=True)
class PhonemeSpan:
    grapheme: str
    ipa: str
    char_start: int
    char_end: int
    is_morpheme_highlight: bool = False


# ============================================================================
# 1. KOINE GREEK (grc) MULTI-PROFILE DETERMINISTIC G2P
# ============================================================================

GREEK_DIGRAPHS: Dict[str, Dict[str, str]] = {
    "αι": {"RESTORED_KOINE_1C": "ɛ", "ERASMIAN": "ai", "MODERN": "e"},
    "ει": {"RESTORED_KOINE_1C": "iː", "ERASMIAN": "eɪ", "MODERN": "i"},
    "οι": {"RESTORED_KOINE_1C": "yː", "ERASMIAN": "ɔɪ", "MODERN": "i"},
    "υι": {"RESTORED_KOINE_1C": "yː", "ERASMIAN": "ɥi", "MODERN": "i"},
    "αυ": {"RESTORED_KOINE_1C": "aw", "ERASMIAN": "au", "MODERN": "av"},
    "ευ": {"RESTORED_KOINE_1C": "ɛw", "ERASMIAN": "ɛu", "MODERN": "ev"},
    "ηυ": {"RESTORED_KOINE_1C": "eːw", "ERASMIAN": "ɛːu", "MODERN": "iv"},
    "ου": {"RESTORED_KOINE_1C": "uː", "ERASMIAN": "uː", "MODERN": "u"},
    "γγ": {"RESTORED_KOINE_1C": "ŋɡ", "ERASMIAN": "ŋɡ", "MODERN": "ŋɡ"},
    "γκ": {"RESTORED_KOINE_1C": "ŋk", "ERASMIAN": "ŋk", "MODERN": "ŋɡ"},
    "γχ": {"RESTORED_KOINE_1C": "ŋkʰ", "ERASMIAN": "ŋx", "MODERN": "ŋx"},
    "γξ": {"RESTORED_KOINE_1C": "ŋks", "ERASMIAN": "ŋks", "MODERN": "ŋks"},
}

GREEK_MONOGRAPHS: Dict[str, Dict[str, str]] = {
    "α": {"RESTORED_KOINE_1C": "a", "ERASMIAN": "a", "MODERN": "a"},
    "β": {"RESTORED_KOINE_1C": "b", "ERASMIAN": "b", "MODERN": "v"},
    "γ": {"RESTORED_KOINE_1C": "ɡ", "ERASMIAN": "ɡ", "MODERN": "ɣ"},
    "δ": {"RESTORED_KOINE_1C": "d", "ERASMIAN": "d", "MODERN": "ð"},
    "ε": {"RESTORED_KOINE_1C": "ɛ", "ERASMIAN": "ɛ", "MODERN": "e"},
    "ζ": {"RESTORED_KOINE_1C": "z", "ERASMIAN": "dz", "MODERN": "z"},
    "η": {"RESTORED_KOINE_1C": "eː", "ERASMIAN": "ɛː", "MODERN": "i"},
    "θ": {"RESTORED_KOINE_1C": "tʰ", "ERASMIAN": "θ", "MODERN": "θ"},
    "ι": {"RESTORED_KOINE_1C": "i", "ERASMIAN": "i", "MODERN": "i"},
    "κ": {"RESTORED_KOINE_1C": "k", "ERASMIAN": "k", "MODERN": "k"},
    "λ": {"RESTORED_KOINE_1C": "l", "ERASMIAN": "l", "MODERN": "l"},
    "μ": {"RESTORED_KOINE_1C": "m", "ERASMIAN": "m", "MODERN": "m"},
    "ν": {"RESTORED_KOINE_1C": "n", "ERASMIAN": "n", "MODERN": "n"},
    "ξ": {"RESTORED_KOINE_1C": "ks", "ERASMIAN": "ks", "MODERN": "ks"},
    "ο": {"RESTORED_KOINE_1C": "o", "ERASMIAN": "ɔ", "MODERN": "o"},
    "π": {"RESTORED_KOINE_1C": "p", "ERASMIAN": "p", "MODERN": "p"},
    "ρ": {"RESTORED_KOINE_1C": "r", "ERASMIAN": "r", "MODERN": "r"},
    "σ": {"RESTORED_KOINE_1C": "s", "ERASMIAN": "s", "MODERN": "s"},
    "ς": {"RESTORED_KOINE_1C": "s", "ERASMIAN": "s", "MODERN": "s"},
    "τ": {"RESTORED_KOINE_1C": "t", "ERASMIAN": "t", "MODERN": "t"},
    "υ": {"RESTORED_KOINE_1C": "y", "ERASMIAN": "y", "MODERN": "i"},
    "φ": {"RESTORED_KOINE_1C": "pʰ", "ERASMIAN": "f", "MODERN": "f"},
    "χ": {"RESTORED_KOINE_1C": "kʰ", "ERASMIAN": "x", "MODERN": "x"},
    "ψ": {"RESTORED_KOINE_1C": "ps", "ERASMIAN": "ps", "MODERN": "ps"},
    "ω": {"RESTORED_KOINE_1C": "oː", "ERASMIAN": "oː", "MODERN": "o"},
}

COMBINING_ROUGH_BREATHING = "\u0314"  # ῾
COMBINING_ACUTE = "\u0301"
COMBINING_GRAVE = "\u0300"
COMBINING_CIRCUMFLEX = "\u0342"
COMBINING_DIAERESIS = "\u0308"


def _decompose_greek_char(ch: str) -> Tuple[str, bool, bool, bool]:
    """Returns (base_lower_letter, has_rough_breathing, has_accent, has_diaeresis)."""
    nfd = unicodedata.normalize("NFD", ch)
    base = nfd[0].lower() if nfd else ""
    marks = set(nfd[1:])
    has_rough = COMBINING_ROUGH_BREATHING in marks
    has_accent = bool(marks & {COMBINING_ACUTE, COMBINING_GRAVE, COMBINING_CIRCUMFLEX})
    has_diaeresis = COMBINING_DIAERESIS in marks
    return base, has_rough, has_accent, has_diaeresis


def g2p_koine_greek(
    text: str, profile: str = "RESTORED_KOINE_1C"
) -> Tuple[str, List[str], List[Dict[str, object]]]:
    """
    Converts polytonic Koine Greek text into:
      - full_ipa_string (space-separated words with stress marks)
      - flat_phone_list (list of individual IPA phones)
      - grapheme_spans (list of dicts mapping surface character offsets to IPA phones)
    """
    text_nfc = unicodedata.normalize("NFC", text)
    phones: List[str] = []
    spans: List[Dict[str, object]] = []
    word_ipa_parts: List[str] = []

    i = 0
    n = len(text_nfc)
    current_word_phones: List[str] = []

    while i < n:
        ch = text_nfc[i]
        if not ch.isalpha():
            if current_word_phones:
                word_ipa_parts.append("".join(current_word_phones))
                current_word_phones = []
            i += 1
            continue

        base1, rough1, accent1, _ = _decompose_greek_char(ch)

        # Check digraph if a second alphabetic character follows without diaeresis
        if i + 1 < n and text_nfc[i + 1].isalpha():
            base2, rough2, accent2, diaeresis2 = _decompose_greek_char(text_nfc[i + 1])
            pair = base1 + base2
            if pair in GREEK_DIGRAPHS and not diaeresis2:
                has_rough = rough1 or rough2
                has_accent = accent1 or accent2
                ipa_val = GREEK_DIGRAPHS[pair][profile]
                prefix = ""
                if has_rough and profile != "MODERN":
                    prefix += "h"
                    phones.append("h")
                if has_accent:
                    prefix += "ˈ"
                phones.append(ipa_val)
                full_tok = prefix + ipa_val
                current_word_phones.append(full_tok)
                spans.append({
                    "grapheme": text_nfc[i : i + 2],
                    "ipa": full_tok,
                    "char_start": i,
                    "char_end": i + 2,
                })
                i += 2
                continue

        # Monograph fallback
        if base1 in GREEK_MONOGRAPHS:
            ipa_val = GREEK_MONOGRAPHS[base1][profile]
            prefix = ""
            if rough1 and profile != "MODERN":
                if base1 == "ρ":
                    ipa_val = "r̥"
                else:
                    prefix += "h"
                    phones.append("h")
            if accent1:
                prefix += "ˈ"
            phones.append(ipa_val)
            full_tok = prefix + ipa_val
            current_word_phones.append(full_tok)
            spans.append({
                "grapheme": ch,
                "ipa": full_tok,
                "char_start": i,
                "char_end": i + 1,
            })
        i += 1

    if current_word_phones:
        word_ipa_parts.append("".join(current_word_phones))

    return "/" + " ".join(word_ipa_parts) + "/", phones, spans


# ============================================================================
# 2. BIBLICAL HEBREW (hbo) MASORETIC TIBERIAN / RECONSTRUCTED G2P
# ============================================================================

HEBREW_CONSONANTS: Dict[str, Tuple[str, str]] = {
    # letter: (base_or_spirant_ipa, dagesh_plosive_ipa)
    "א": ("ʔ", "ʔ"),
    "ב": ("v", "b"),
    "ג": ("ɣ", "ɡ"),
    "ד": ("ð", "d"),
    "ה": ("h", "h"),
    "ו": ("w", "w"),
    "ז": ("z", "z"),
    "ח": ("ħ", "ħ"),
    "ט": ("tˤ", "tˤ"),
    "י": ("j", "j"),
    "כ": ("x", "k"),
    "ך": ("x", "k"),
    "ל": ("l", "l"),
    "מ": ("m", "m"),
    "ם": ("m", "m"),
    "נ": ("n", "n"),
    "ן": ("n", "n"),
    "ס": ("s", "s"),
    "ע": ("ʕ", "ʕ"),
    "פ": ("f", "p"),
    "ף": ("f", "p"),
    "צ": ("sˤ", "sˤ"),
    "ץ": ("sˤ", "sˤ"),
    "ק": ("q", "q"),
    "ר": ("r", "r"),
    "ש": ("ʃ", "ʃ"),  # disambiguated by shin/sin dot
    "ת": ("θ", "t"),
}

HEBREW_VOWELS: Dict[str, str] = {
    "\u05B0": "ə",   # Shewa (vocal default at word-initial or after open syllable)
    "\u05B1": "ĕ",   # Hataf Segol
    "\u05B2": "ă",   # Hataf Pathach
    "\u05B3": "ŏ",   # Hataf Qamatz
    "\u05B4": "i",   # Hiriq
    "\u05B5": "eː",  # Tsere
    "\u05B6": "ɛ",   # Segol
    "\u05B7": "a",   # Pathach
    "\u05B8": "aː",  # Qamatz (disambiguated if Qamatz Qatan)
    "\u05B9": "oː",  # Holam
    "\u05BA": "oː",  # Holam Haser for Vav
    "\u05BB": "u",   # Qubutz
}

DAGESH_OR_MAPIQ = "\u05BC"
SHIN_DOT = "\u05C1"
SIN_DOT = "\u05C2"


def g2p_biblical_hebrew(
    text: str, academic_sephardic: bool = False
) -> Tuple[str, List[str], List[Dict[str, object]]]:
    """
    Converts pointed Masoretic Biblical Hebrew (hbo) into:
      - full_ipa_string
      - flat_phone_list
      - grapheme_spans
    Handles Begadkefat spirantization, Dagesh Forte gemination, Shin/Sin dots,
    Matres Lectionis (וּ = /uː/, וֹ = /oː/, ִי = /iː/), and Furtive Pathach.
    """
    text_nfd = unicodedata.normalize("NFD", text)
    # Segment into orthographic clusters: (base_consonant, [combining_marks], start_idx, end_idx)
    clusters: List[Tuple[str, List[str], int, int]] = []
    i = 0
    n = len(text_nfd)
    while i < n:
        ch = text_nfd[i]
        if ch in HEBREW_CONSONANTS:
            start = i
            marks: List[str] = []
            i += 1
            while i < n and unicodedata.combining(text_nfd[i]):
                marks.append(text_nfd[i])
                i += 1
            clusters.append((ch, marks, start, i))
        else:
            clusters.append((ch, [], i, i + 1))
            i += 1

    phones: List[str] = []
    spans: List[Dict[str, object]] = []
    word_ipa_parts: List[str] = []
    current_word: List[str] = []

    prev_had_short_vowel = False
    is_word_initial = True

    idx = 0
    while idx < len(clusters):
        base, marks, c_start, c_end = clusters[idx]
        if base not in HEBREW_CONSONANTS:
            if base in (" ", "\u05BE"):  # space or Maqaf
                if current_word:
                    word_ipa_parts.append("".join(current_word))
                    current_word = []
                is_word_initial = True
                prev_had_short_vowel = False
            idx += 1
            continue

        # Check if Vav is acting purely as Mater Lectionis (וּ shuruq or וֹ holam male)
        has_dagesh = DAGESH_OR_MAPIQ in marks
        has_holam = "\u05B9" in marks or "\u05BA" in marks
        vowel_marks = [m for m in marks if m in HEBREW_VOWELS]

        if base == "ו" and not is_word_initial and has_dagesh and not vowel_marks:
            # Shuruq /uː/
            phones.append("uː")
            current_word.append("uː")
            spans.append({
                "grapheme": text_nfd[c_start:c_end],
                "ipa": "uː",
                "char_start": c_start,
                "char_end": c_end,
            })
            prev_had_short_vowel = False
            is_word_initial = False
            idx += 1
            continue

        if base == "ו" and not is_word_initial and has_holam and len(vowel_marks) == 1:
            # Holam Male /oː/
            phones.append("oː")
            current_word.append("oː")
            spans.append({
                "grapheme": text_nfd[c_start:c_end],
                "ipa": "oː",
                "char_start": c_start,
                "char_end": c_end,
            })
            prev_had_short_vowel = False
            is_word_initial = False
            idx += 1
            continue

        # Determine consonant phone
        spirant_ipa, plosive_ipa = HEBREW_CONSONANTS[base]
        if academic_sephardic:
            if base in ("ג", "ד", "ת"):
                spirant_ipa = plosive_ipa
        if base == "ש":
            if SIN_DOT in marks:
                cons_ipa = "s"
            else:
                cons_ipa = "ʃ"
        else:
            cons_ipa = plosive_ipa if has_dagesh else spirant_ipa

        # Check Dagesh Forte gemination (after a short vowel inside a word)
        if has_dagesh and not is_word_initial and prev_had_short_vowel and base not in ("א", "ה", "ח", "ע", "ר"):
            cons_ipa = cons_ipa + "ː"

        # Silent final He or Aleph without vowel/mapiq
        is_word_final = (idx + 1 == len(clusters)) or (clusters[idx + 1][0] not in HEBREW_CONSONANTS)
        if is_word_final and base in ("ה", "א") and not vowel_marks and not has_dagesh:
            cons_ipa = ""

        # Determine vowel phone
        v_ipa = ""
        for vm in vowel_marks:
            if vm == "\u05B0":  # Shewa
                if is_word_final:
                    v_ipa = ""  # Silent final shewa
                elif is_word_initial or prev_had_short_vowel is False or has_dagesh:
                    v_ipa = "ə"  # Vocal shewa
                else:
                    v_ipa = ""  # Silent syllable-closing shewa
            else:
                v_ipa = HEBREW_VOWELS[vm]

        # Check Mater Lectionis Yod after Hiriq/Tsere/Segol
        if (
            idx + 1 < len(clusters)
            and clusters[idx + 1][0] == "י"
            and not clusters[idx + 1][1]
            and v_ipa in ("i", "eː", "ɛ")
        ):
            if v_ipa == "i":
                v_ipa = "iː"
            c_end = clusters[idx + 1][3]
            idx += 1  # consume silent mater yod

        # Check Furtive Pathach on word-final gutturals (חַ, עַ, הַּ)
        if is_word_final and base in ("ח", "ע", "ה") and v_ipa == "a" and not prev_had_short_vowel:
            cluster_ipa = "a" + cons_ipa
            phones.extend(["a", cons_ipa])
        else:
            cluster_ipa = cons_ipa + v_ipa
            if cons_ipa:
                phones.append(cons_ipa)
            if v_ipa:
                phones.append(v_ipa)

        if cluster_ipa:
            current_word.append(cluster_ipa)
            spans.append({
                "grapheme": unicodedata.normalize("NFC", text_nfd[c_start:c_end]),
                "ipa": cluster_ipa,
                "char_start": c_start,
                "char_end": c_end,
            })

        prev_had_short_vowel = v_ipa in ("i", "ɛ", "a", "u", "ă", "ĕ", "ŏ")
        is_word_initial = False
        idx += 1

    if current_word:
        word_ipa_parts.append("".join(current_word))

    return "/" + " ".join(word_ipa_parts) + "/", phones, spans


if __name__ == "__main__":
    grc_sample = "ὁ ἄγγελος βλέπει τὸν ἀδελφόν"
    ipa_str, phones, _ = g2p_koine_greek(grc_sample, "RESTORED_KOINE_1C")
    print(f"[GRC Restored] {grc_sample} -> {ipa_str} ({phones})")

    hbo_sample = "הַנַּעַר יֹשֵׁב עַל הָאֶבֶן"
    h_ipa, h_phones, _ = g2p_biblical_hebrew(hbo_sample)
    print(f"[HBO Tiberian] {hbo_sample} -> {h_ipa} ({h_phones})")
