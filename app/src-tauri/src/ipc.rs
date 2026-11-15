//! KaizenLinguistics — Tauri v2 IPC Command Handlers
//! Bridges the "Atelier Aldus" UI Scriptorium directly to:
//! 1. Native Dual SQLite storage (.klpack course archives & user_telemetry.db)
//! 2. Pinned FSRS-6 21-parameter Scheduler & 24-hour Sibling Burying
//! 3. Deterministic G2P for Koine Greek (3 profiles) & Biblical Hebrew (Tiberian Masoretic)
//! 4. 5-Stage Epistemic Speech Evaluator & TrueAccent Harmonic Oscilloscope HUD

use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::acoustic::{
    evaluate_speech_pipeline, AcousticEvaluationInput, AcousticTelemetryReport, FoilSentence,
    SentenceTarget,
};
use crate::g2p::{
    g2p_biblical_hebrew, g2p_koine_greek, G2pResult, GreekPhonologyProfile, HebrewPhonologyProfile,
};
use crate::{
    fsrs6_step, verify_dual_db_pack, FsrsCardState, Modality, Rating,
    SqliteConnection, DEFAULT_FSRS6_PARAMS,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserProfile {
    pub user_id: String,
    pub display_name: String,
    pub avatar: String,
    pub created_at_ms: i64,
    pub last_active_ms: i64,
    pub current_lang: String,
    pub streak_days: i64,
    pub total_reviews: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CatalogPanel {
    pub pointed: String,
    pub unpointed: String,
    pub ipa: String,
    pub motif: String,
    pub role: String,
    #[serde(default)]
    pub glyph: String,
    #[serde(default)]
    pub clean_role: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CatalogItem {
    pub id: String,
    pub roman: String,
    pub lemma: String,
    pub tag: String,
    pub contrast: String,
    pub ref_source: String,
    pub scholia: String,
    pub target_index: usize,
    pub panels: Vec<CatalogPanel>,
    #[serde(default)]
    pub title_en: String,
    #[serde(default)]
    pub category_en: String,
    #[serde(default)]
    pub lemma_gloss_en: String,
    #[serde(default)]
    pub explanation_en: String,
    #[serde(default)]
    pub target_translation_en: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReviewResponse {
    pub card_id: String,
    pub concept_id: String,
    pub modality: String,
    pub rating: u8,
    pub stability_days: f64,
    pub difficulty: f64,
    pub reps: i32,
    pub lapses: i32,
    pub due_timestamp_ms: i64,
    pub sibling_buried_until_ms: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpeechEvaluationPayload {
    pub samples: Vec<f32>,
    pub sample_rate: u32,
    pub target_sentence_id: String,
    pub target_text: String,
    pub lang_code: String,
    pub target_phones: Vec<String>,
    pub contrast_indices: Vec<usize>,
    pub foils: Vec<FoilSentence>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntegritySummary {
    pub grc_integrity_ok: bool,
    pub hbo_integrity_ok: bool,
    pub grc_concepts: i64,
    pub hbo_concepts: i64,
    pub fsrs_version: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AvailableLanguage {
    pub code: String,
    pub name: String,
    pub autonym: String,
    pub family: String,
    pub script_direction: String,
    pub concept_count: i64,
    pub pack_id: String,
    pub semver: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FsrsSkillStability {
    pub modality: String,
    pub card_count: usize,
    pub avg_stability_days: f64,
    pub due_count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecentReviewItem {
    pub event_uuid: String,
    pub card_id: String,
    pub modality: String,
    pub rating: i64,
    pub stability_after: f64,
    pub latency_ms: i64,
    pub timestamp_ms: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FsrsDeckSummary {
    pub lang_code: String,
    pub total_cards: usize,
    pub due_cards: usize,
    pub learning_cards: usize,
    pub mastered_cards: usize,
    pub buried_cards: usize,
    pub skills: Vec<FsrsSkillStability>,
    pub recent_reviews: Vec<RecentReviewItem>,
}

fn get_project_paths() -> Result<(PathBuf, PathBuf, PathBuf), String> {
    let mut candidates = Vec::new();
    if let Ok(cur) = std::env::current_dir() {
        candidates.push(cur.clone());
        if let Some(p) = cur.parent() {
            candidates.push(p.to_path_buf());
            if let Some(gp) = p.parent() {
                candidates.push(gp.to_path_buf());
            }
        }
    }
    let manifest = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    candidates.push(manifest.clone());
    if let Some(p) = manifest.parent() {
        candidates.push(p.to_path_buf());
        if let Some(gp) = p.parent() {
            candidates.push(gp.to_path_buf());
        }
    }

    for root in candidates {
        let packs = root.join("packs");
        let grc = packs.join("klpack_grc_koine_v1.klpack");
        let hbo = packs.join("klpack_hbo_masoretic_v1.klpack");
        let telem = packs.join("user_telemetry.db");
        if grc.exists() && hbo.exists() && telem.exists() {
            return Ok((grc, hbo, telem));
        }
    }

    Err("Could not locate course packs and telemetry DB in packs/ directory".to_string())
}

fn format_role_to_english(role: &str, glyph: &str) -> String {
    let mut s = role.to_string();
    if s.starts_with("SVO_") || s.starts_with("OVS_") || s.starts_with("SOV_") || s.starts_with("VSO_") {
        s = s[4..].to_string();
    }
    s = s.replace("_Sees_", " sees ")
         .replace("_Hears_", " hears ")
         .replace("_Speaks_", " speaks ")
         .replace("_Calls_", " calls ")
         .replace("_Writes_", " writes ")
         .replace("_Sends_", " sends ")
         .replace("_Releases_", " releases ")
         .replace("Angel_Sees_Brother", "The angel sees the brother")
         .replace("Brother_Sees_Angel", "The brother sees the angel")
         .replace("Nom_Sg", "Singular Subject (one)")
         .replace("Nom_Pl", "Plural Subject (many)")
         .replace("Acc_Sg", "Singular Object (one)")
         .replace("Acc_Pl", "Plural Object (many)")
         .replace("Fem_Sg_Subject", "Feminine Singular Subject")
         .replace("Fem_Pl_Subject", "Feminine Plural Subject")
         .replace("Masc_Sg_Subject", "Masculine Singular Subject")
         .replace("Masc_Pl_Subject", "Masculine Plural Subject")
         .replace("Neut_Sg_Nom", "Neuter Subject (singular)")
         .replace("Neut_Pl_Nom", "Neuter Subject (plural)")
         .replace("Neut_Sg_Acc", "Neuter Object (singular)")
         .replace("Neut_Pl_Acc", "Neuter Object (plural)")
         .replace("Active_Voice", "Active: Actor performs action")
         .replace("Passive_Voice_Hypo_Gen", "Passive: Action done by agent")
         .replace("Passive_Plural", "Passive: Plural recipient")
         .replace("Imperfective_Unfolding", "Present: Ongoing action")
         .replace("Aorist_Bounded_Event", "Aorist: Completed snapshot")
         .replace("Perfect_Enduring_State", "Perfect: Enduring state")
         .replace("Imperfect_Past_Ongoing", "Imperfect: Ongoing past")
         .replace("Masc_Sg_Participle", "Masculine Singular Action")
         .replace("Fem_Sg_Participle_Ah_Et", "Feminine Singular Action")
         .replace("Masc_Pl_Im", "Masculine Plural Action")
         .replace("Fem_Pl_Ot", "Feminine Plural Action")
         .replace('_', " ");
    
    if !glyph.is_empty() {
        format!("{s} ({glyph})")
    } else {
        s
    }
}

fn enrich_catalog_pedagogy(
    concept_id: &str,
    contrast_dim: &str,
    lemma: &str,
    upos: &str,
    lang: &str,
) -> (String, String, String, String, String) {
    if lang == "grc" {
        if concept_id.contains("01_NOM_ACC") {
            (
                "Who Does What? Subject vs. Object".to_string(),
                "Nominative vs. Accusative Roles".to_string(),
                "messenger / angel".to_string(),
                "In Greek, word order does NOT determine who does the action—endings do! Look for -ος for the Subject performing the action, and -ον for the Object receiving the action. Even when the object is placed first in the sentence, the endings preserve the exact meaning.".to_string(),
                "The brother sees the angel.".to_string(),
            )
        } else if concept_id.contains("02_NOUN_NUMBER") {
            (
                "One vs. Many: Singular and Plural".to_string(),
                "Grammatical Number (-ος / -οι)".to_string(),
                "brother".to_string(),
                "Greek distinguishes singular and plural through distinct case endings. The nominative singular -ος shifts to -οι in the plural, while the accusative singular -ον shifts to -ους in the plural.".to_string(),
                "I see the brothers.".to_string(),
            )
        } else if concept_id.contains("03_FEM_1DECL") {
            (
                "Masculine vs. Feminine Agreement".to_string(),
                "Gender & Article Agreement".to_string(),
                "woman / wife".to_string(),
                "Articles and adjectives must strictly match the grammatical gender of the noun. Feminine nouns use the article ἡ in the singular and αἱ in the plural, contrasting with masculine ὁ / οἱ.".to_string(),
                "The woman speaks a word.".to_string(),
            )
        } else if concept_id.contains("04_NEUTER") {
            (
                "Neuter Nouns: The -ον / -α Pattern".to_string(),
                "Neuter 2nd Declension".to_string(),
                "work / deed".to_string(),
                "Neuter nouns have identical forms for both Subject and Object (-ον in the singular, -α in the plural). Strikingly, plural neuter subjects take a singular verb in classical Greek!".to_string(),
                "I see the beautiful works.".to_string(),
            )
        } else if concept_id.contains("05_SPATIAL_PREP") {
            (
                "Prepositions: In, Into, and Out Of".to_string(),
                "Prepositional Case Governance".to_string(),
                "house / home".to_string(),
                "Greek prepositions require specific cases: ἐν governs the dative for stationary location (inside), εἰς governs the accusative for motion into, and ἐκ governs the genitive for motion out of.".to_string(),
                "The brother enters into the house.".to_string(),
            )
        } else if concept_id.contains("06_DATIVE") {
            (
                "To Whom? Giving to a Recipient".to_string(),
                "Dative Indirect Object".to_string(),
                "give".to_string(),
                "The Dative case identifies the indirect recipient or beneficiary of an action. Notice how the ending -ῳ signals who receives the gift.".to_string(),
                "The teacher gives a book to the brother.".to_string(),
            )
        } else if concept_id.contains("07_GENITIVE") {
            (
                "Whose Is It? Possession and Origin".to_string(),
                "Genitive Modifiers".to_string(),
                "word / message".to_string(),
                "The Genitive case acts like 'of' in English, expressing possession, source, or description (e.g. τοῦ ἀδελφοῦ = 'of the brother').".to_string(),
                "We hear the word of the brother.".to_string(),
            )
        } else if concept_id.contains("08_VERB_PERSON") {
            (
                "Who is Speaking? I, You, He/She".to_string(),
                "Verb Person & Number".to_string(),
                "say / speak".to_string(),
                "Greek verbs encode the subject directly into their ending: -ω (I), -εις (you), -ει (he/she), -ομεν (we), -ετε (you all), -ουσι (they). Pronouns are only added for special emphasis!".to_string(),
                "We speak the truth.".to_string(),
            )
        } else if concept_id.contains("09_ASPECT") {
            (
                "How Action Unfolds: Present vs. Past".to_string(),
                "Verbal Aspect & Time".to_string(),
                "write".to_string(),
                "Greek focuses on the nature of the action: Present tense shows ongoing unfolding process ('is writing'), Aorist shows a complete snapshot event ('wrote'), and Perfect shows an abiding state ('has written and it stands').".to_string(),
                "The disciple wrote the book.".to_string(),
            )
        } else if concept_id.contains("10_VOICE") {
            (
                "Acting vs. Being Acted Upon".to_string(),
                "Active vs. Passive Voice".to_string(),
                "release / loose".to_string(),
                "Active voice means the subject performs the action (λύει = 'he frees'). Passive voice means the action is performed upon the subject by an outside agent (λύεται ὑπό... = 'he is freed by...').".to_string(),
                "The servant is released by the master.".to_string(),
            )
        } else {
            let title = format!("Unit Concept: {lemma} ({upos})");
            let cat = contrast_dim.replace("CASE_", "").replace("_VS_", " vs. ").replace('_', " ");
            let gloss = format!("{lemma} ({upos})");
            let exp = format!("This minimal contrast isolates how {lemma} functions under {cat}. Look closely at the inflectional ending to identify the grammatical relationship.");
            let trans = format!("Target sentence featuring {lemma}.");
            (title, cat, gloss, exp, trans)
        }
    } else {
        // Biblical Hebrew
        if concept_id.contains("01_GENDER_NUMBER") {
            (
                "Who is Sitting? Gender & Number".to_string(),
                "Participle Agreement".to_string(),
                "sit / dwell".to_string(),
                "Hebrew participles must agree in gender and number with the subject: יֹשֵׁב (masculine singular), יֹשֶׁבֶת (feminine singular), יֹשְׁבִים (masculine plural), יֹשְׁבוֹת (feminine plural).".to_string(),
                "The young woman is sitting upon the stone.".to_string(),
            )
        } else if concept_id.contains("02_DUAL") {
            (
                "Two by Two: The Ancient Dual Ending".to_string(),
                "Dual Number (-ayim)".to_string(),
                "hand".to_string(),
                "Body parts that naturally come in pairs take the special dual ending -ayim (יָדַיִם = two hands, עֵינַיִם = two eyes), distinguishing them from general plurals.".to_string(),
                "He washes both hands.".to_string(),
            )
        } else if concept_id.contains("03_INSEPARABLE") {
            (
                "Attached Prepositions: In, To, Like".to_string(),
                "Inseparable Preposition Prefixes".to_string(),
                "house".to_string(),
                "Hebrew prepositions like בְּ (in), לְ (to), and כְּ (as/like) attach directly to the front of the noun as single-letter prefixes.".to_string(),
                "The man walks into the house.".to_string(),
            )
        } else if concept_id.contains("04_ATTRIBUTIVE") {
            (
                "The Good King vs. The King is Good".to_string(),
                "Adjective Definiteness Agreement".to_string(),
                "good".to_string(),
                "If the adjective matches the noun's definite article הַ, it describes the noun ('the good king'). If it lacks the article, it forms a complete predicate sentence ('the king is good').".to_string(),
                "The king is good.".to_string(),
            )
        } else if concept_id.contains("05_DEFINITE_DIRECT_OBJECT") {
            (
                "Spotlighting the Object: The Particle אֵת".to_string(),
                "Direct Object Marker".to_string(),
                "direct object marker (untranslated)".to_string(),
                "The untranslated particle אֵת / אֶת appears right before definite direct objects, removing any ambiguity about who is receiving the action.".to_string(),
                "The king called the prophet.".to_string(),
            )
        } else if concept_id.contains("06_CONSTRUCT") {
            (
                "Chained Words: The Construct State".to_string(),
                "Smikhut (Construct Chain)".to_string(),
                "king".to_string(),
                "When two nouns are placed in a construct chain, they form a possessive relationship ('word of the king'). The first noun loses its accent, and only the final noun takes the definite article.".to_string(),
                "The word of the king.".to_string(),
            )
        } else if concept_id.contains("07_PRONOMINAL") {
            (
                "My, Your, His: Attached Endings".to_string(),
                "Pronominal Suffixes".to_string(),
                "voice / sound".to_string(),
                "Hebrew attaches possessive pronouns directly to the ends of nouns: קוֹלִי (my voice), קוֹלְךָ (your voice), קוֹלוֹ (his voice).".to_string(),
                "He heard his voice.".to_string(),
            )
        } else if concept_id.contains("08_EXISTENTIAL") {
            (
                "There Is vs. There Is Not".to_string(),
                "Existential Particles (יֵשׁ vs. אֵין)".to_string(),
                "there is / exists".to_string(),
                "The word יֵשׁ asserts existence ('there is bread in the house'), while אֵין negates existence ('there is no bread').".to_string(),
                "There is bread in the house.".to_string(),
            )
        } else if concept_id.contains("09_QATAL") {
            (
                "Completed Action: Qatal Past Suffixes".to_string(),
                "Suffix Conjugation (Qatal)".to_string(),
                "keep / guard".to_string(),
                "Hebrew verbs express completed past events by adding person, gender, and number suffixes to the verb root: שָׁמַר (he kept), שָׁמְרָה (she kept), שָׁמַרְתָּ (you kept).".to_string(),
                "The servant kept the word.".to_string(),
            )
        } else if concept_id.contains("10_YIQTOL") {
            (
                "Unfolding & Future: Yiqtol Prefixes".to_string(),
                "Prefix Conjugation (Yiqtol)".to_string(),
                "hear / listen".to_string(),
                "Hebrew verbs express unfolding, continuous, or future events by adding prefixes to the root: יִשְׁמַע (he will hear), תִּשְׁמַע (she/you will hear), אֶשְׁמַע (I will hear).".to_string(),
                "The people will hear the sound.".to_string(),
            )
        } else {
            let title = format!("Unit Concept: {lemma} ({upos})");
            let cat = contrast_dim.replace("BINYAN_", "").replace("_VS_", " vs. ").replace('_', " ");
            let gloss = format!("{lemma} ({upos})");
            let exp = format!("This minimal contrast illustrates how {lemma} functions under {cat}. Compare the vowel pointing and prefixes across the situations.");
            let trans = format!("Target sentence featuring {lemma}.");
            (title, cat, gloss, exp, trans)
        }
    }
}

/// Retrieves all compiled contrast exercises from the specified language .klpack archive.
#[tauri::command]
pub fn get_catalog(lang: String) -> Result<Vec<CatalogItem>, String> {
    let (grc_pack, hbo_pack, _) = get_project_paths()?;
    let pack_path = if lang == "hbo" { hbo_pack } else { grc_pack };

    let conn = SqliteConnection::open(&pack_path)?;
    let count = conn.query_single_i64("SELECT COUNT(*) FROM contrast_exercises;")?;

    let mut items = Vec::new();
    let romans = [
        "I", "II", "III", "IV", "V", "VI", "VII", "VIII", "IX", "X",
        "XI", "XII", "XIII", "XIV", "XV", "XVI", "XVII", "XVIII", "XIX", "XX",
        "XXI", "XXII", "XXIII", "XXIV", "XXV", "XXVI", "XXVII", "XXVIII", "XXIX", "XXX",
        "XXXI", "XXXII", "XXXIII", "XXXIV", "XXXV", "XXXVI", "XXXVII", "XXXVIII", "XXXIX", "XL",
        "XLI", "XLII", "XLIII", "XLIV", "XLV", "XLVI", "XLVII", "XLVIII", "XLIX", "L",
        "LI", "LII", "LIII", "LIV", "LV", "LVI", "LVII", "LVIII", "LIX", "LX",
        "LXI", "LXII", "LXIII", "LXIV", "LXV", "LXVI", "LXVII", "LXVIII", "LXIX", "LXX",
    ];

    for idx in 0..count {
        let roman = if (idx as usize) < romans.len() {
            romans[idx as usize].to_string()
        } else {
            format!("{}", idx + 1)
        };

        let concept_id = conn.query_single_text(&format!(
            "SELECT primary_concept_id FROM contrast_exercises LIMIT 1 OFFSET {};",
            idx
        ))?;
        let contrast_dim = conn.query_single_text(&format!(
            "SELECT contrast_dimension FROM contrast_exercises LIMIT 1 OFFSET {};",
            idx
        ))?;
        let items_json = conn.query_single_text(&format!(
            "SELECT items_json FROM contrast_exercises LIMIT 1 OFFSET {};",
            idx
        ))?;
        let target_idx = conn.query_single_i64(&format!(
            "SELECT target_panel_index FROM contrast_exercises LIMIT 1 OFFSET {};",
            idx
        ))? as usize;

        // Parse panels JSON
        let parsed_panels: Vec<serde_json::Value> =
            serde_json::from_str(&items_json).unwrap_or_default();
        let mut panels = Vec::new();

        for p in parsed_panels {
            let s_id = p["sentence_id"].as_str().unwrap_or("");
            let role = p["role"].as_str().unwrap_or("").to_string();

            // Fetch sentence details
            let text_pointed = conn
                .query_single_text(&format!(
                    "SELECT text_pointed FROM sentences WHERE sentence_id = '{}';",
                    s_id
                ))
                .unwrap_or_else(|_| s_id.to_string());
            let text_unpointed = conn
                .query_single_text(&format!(
                    "SELECT text_unpointed FROM sentences WHERE sentence_id = '{}';",
                    s_id
                ))
                .unwrap_or_else(|_| text_pointed.clone());
            let ipa_raw = conn
                .query_single_text(&format!(
                    "SELECT ipa_sequence_json FROM sentences WHERE sentence_id = '{}';",
                    s_id
                ))
                .unwrap_or_else(|_| "/.../".to_string());

            // Robust IPA parser: extract the clean IPA string, never raw JSON
            let ipa = if let Ok(val) = serde_json::from_str::<serde_json::Value>(&ipa_raw) {
                val["ipa"].as_str().unwrap_or(&ipa_raw).to_string()
            } else if let Some(clean) = p.get("ipa").and_then(|v| v.as_str()) {
                clean.to_string()
            } else {
                ipa_raw
            };

            let glyph = p["glyph"].as_str().unwrap_or("").to_string();
            let clean_role = format_role_to_english(&role, &glyph);

            let motif = if role.contains("Nom") || role.contains("Agent") {
                "AGENT_SUBJECT".to_string()
            } else if role.contains("Acc") || role.contains("Patient") {
                "PATIENT_OBJECT".to_string()
            } else if role.contains("Hiphil") || role.contains("Causative") {
                "VALENCY_CAUSATION".to_string()
            } else if role.contains("Aorist") {
                "ASPECT_BOUNDED".to_string()
            } else if role.contains("Perfect") {
                "ASPECT_STATIVE".to_string()
            } else {
                "ASPECT_UNFOLDING".to_string()
            };

            panels.push(CatalogPanel {
                pointed: text_pointed,
                unpointed: text_unpointed,
                ipa,
                motif,
                role,
                glyph,
                clean_role,
            });
        }

        let lemma = conn
            .query_single_text(&format!(
                "SELECT lemma_unicode FROM lexemes WHERE concept_id = '{}' LIMIT 1;",
                concept_id
            ))
            .unwrap_or_else(|_| concept_id.clone());
        let upos = conn
            .query_single_text(&format!(
                "SELECT upos FROM lexemes WHERE concept_id = '{}' LIMIT 1;",
                concept_id
            ))
            .unwrap_or_else(|_| "NOUN".to_string());

        let ref_src = if lang == "grc" {
            "SBLGNT / MACULA GREEK"
        } else {
            "WLC / ETCBC BHSA"
        };

        let scholia = format!(
            "Controlled minimal contrast: {} targeting {} on invariant lexeme {}.",
            contrast_dim, upos, lemma
        );

        let (title_en, category_en, lemma_gloss_en, explanation_en, target_translation_en) =
            enrich_catalog_pedagogy(&concept_id, &contrast_dim, &lemma, &upos, &lang);

        items.push(CatalogItem {
            id: concept_id,
            roman,
            lemma,
            tag: upos,
            contrast: contrast_dim,
            ref_source: ref_src.to_string(),
            scholia,
            target_index: target_idx,
            panels,
            title_en,
            category_en,
            lemma_gloss_en,
            explanation_en,
            target_translation_en,
        });
    }

    Ok(items)
}

/// Records a direct study observation on a specific card, computes the next FSRS-6 state,
/// applies 24-hour sibling burying to defer other modalities of that concept, and appends to review_logs.
#[tauri::command]
pub fn record_review(
    lang: String,
    concept_id: String,
    modality: String,
    rating: u8,
    latency_ms: u32,
) -> Result<ReviewResponse, String> {
    let (_, _, telem_db) = get_project_paths()?;
    let conn = SqliteConnection::open(&telem_db)?;

    let mod_enum = match modality.to_uppercase().as_str() {
        "LISTEN" => Modality::Listen,
        "READ" => Modality::Read,
        "SPEAK" => Modality::Speak,
        "WRITE" => Modality::Write,
        _ => Modality::Listen,
    };

    let rating_enum = Rating::from_u8(rating)?;
    let card_id = format!("{}:{}:{}", lang, concept_id, mod_enum.as_str());

    let now_ms = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|e| e.to_string())?
        .as_millis() as i64;

    // Fetch card from user_telemetry.db
    let reps = conn
        .query_single_i64(&format!(
            "SELECT reps FROM fsrs_cards WHERE card_id = '{}';",
            card_id
        ))
        .unwrap_or(0) as i32;
    let lapses = conn
        .query_single_i64(&format!(
            "SELECT lapses FROM fsrs_cards WHERE card_id = '{}';",
            card_id
        ))
        .unwrap_or(0) as i32;
    let state = conn
        .query_single_i64(&format!(
            "SELECT state FROM fsrs_cards WHERE card_id = '{}';",
            card_id
        ))
        .unwrap_or(0) as i32;

    let current_card = FsrsCardState {
        card_id: card_id.clone(),
        lang_code: lang.clone(),
        pack_id: format!("klpack_{}_v1", lang),
        concept_id: concept_id.clone(),
        modality: mod_enum,
        state,
        stability: 0.0,
        difficulty: 5.0,
        reps,
        lapses,
        due_timestamp_ms: now_ms,
        buried_until_ms: None,
        last_review_ms: None,
    };

    // Execute atomic FSRS-6 update strictly from direct observation
    let updated = fsrs6_step(&current_card, rating_enum, now_ms, 0.90, &DEFAULT_FSRS6_PARAMS);

    // Apply 24h sibling burying to other modalities of this concept
    let bury_target_ms = now_ms + 86_400_000;
    for sibling_mod in Modality::all() {
        if sibling_mod != mod_enum {
            let sib_id = format!("{}:{}:{}", lang, concept_id, sibling_mod.as_str());
            let _ = conn.execute_batch(&format!(
                "UPDATE fsrs_cards SET buried_until_ms = {} WHERE card_id = '{}';",
                bury_target_ms, sib_id
            ));
        }
    }

    // Update active card in SQLite
    conn.execute_batch(&format!(
        "UPDATE fsrs_cards SET state = {}, stability = {:.6}, difficulty = {:.6}, \
         reps = {}, lapses = {}, due_timestamp_ms = {}, last_review_ms = {} \
         WHERE card_id = '{}';",
        updated.state,
        updated.stability,
        updated.difficulty,
        updated.reps,
        updated.lapses,
        updated.due_timestamp_ms,
        now_ms,
        card_id
    ))?;

    // Append to deterministic review_logs
    let event_uuid = format!("LOG_{}_{}", now_ms, card_id.replace(':', "_"));
    let pack_id = format!("klpack_{}_v1", lang);
    let exercise_id = format!("{}_ex", concept_id);
    conn.execute_batch(&format!(
        "INSERT INTO review_logs (\
         event_uuid, card_id, pack_id, pack_semver, exercise_id, permutation_seed, \
         review_timestamp_ms, event_type, rating, grade_source, fsrs_crate_version, \
         fsrs_params_json, state_before, stability_before, difficulty_before, \
         state_after, stability_after, difficulty_after, due_after_ms, response_latency_ms, \
         audio_replays, grader_model_version, acoustic_evidence_json, edit_distance \
         ) VALUES (\
         '{}', '{}', '{}', '0.3.0', '{}', 0, \
         {}, 'DIRECT_REVIEW', {}, 'OBJECTIVE_CHOICE', 'fsrs-6-pinned-21param-v0.2.0', \
         '[]', {}, {:.6}, {:.6}, \
         {}, {:.6}, {:.6}, {}, {}, \
         0, 'local-rust-ipc', NULL, 0);",
        event_uuid,
        card_id,
        pack_id,
        exercise_id,
        now_ms,
        rating,
        current_card.state,
        current_card.stability,
        current_card.difficulty,
        updated.state,
        updated.stability,
        updated.difficulty,
        updated.due_timestamp_ms,
        latency_ms
    ))?;

    // Update active profile review telemetry if user_profiles table exists
    let _ = conn.execute_batch(&format!(
        "UPDATE user_profiles SET total_reviews = total_reviews + 1, last_active_ms = {now_ms} WHERE current_lang = '{lang}';"
    ));

    Ok(ReviewResponse {
        card_id,
        concept_id,
        modality: mod_enum.as_str().to_string(),
        rating,
        stability_days: updated.stability,
        difficulty: updated.difficulty,
        reps: updated.reps,
        lapses: updated.lapses,
        due_timestamp_ms: updated.due_timestamp_ms,
        sibling_buried_until_ms: bury_target_ms,
    })
}

/// Evaluates raw spoken microphone audio across all 5 epistemic gates.
#[tauri::command]
pub fn evaluate_speech(payload: SpeechEvaluationPayload) -> Result<AcousticTelemetryReport, String> {
    let target = SentenceTarget {
        sentence_id: payload.target_sentence_id,
        text: payload.target_text,
        lang_code: payload.lang_code,
        ipa_phones: payload.target_phones,
        contrast_morpheme_indices: payload.contrast_indices,
    };

    let report = evaluate_speech_pipeline(AcousticEvaluationInput {
        raw_samples: &payload.samples,
        sample_rate: payload.sample_rate,
        target: &target,
        foils: &payload.foils,
        recognized_phones: None,
        external_gop_scores: None,
        external_alignment_conf: None,
    });

    Ok(report)
}

/// Deterministic G2P transcription for Koine Greek or Biblical Hebrew.
#[tauri::command]
pub fn g2p_transcribe(text: String, lang: String, profile: String) -> Result<G2pResult, String> {
    if lang == "grc" {
        let p = match profile.to_uppercase().as_str() {
            "ERASMIAN" => GreekPhonologyProfile::Erasmian,
            "MODERN" => GreekPhonologyProfile::Modern,
            _ => GreekPhonologyProfile::RestoredKoine1C,
        };
        Ok(g2p_koine_greek(&text, p))
    } else {
        let p = match profile.to_uppercase().as_str() {
            "ACADEMIC_SEPHARDIC" => HebrewPhonologyProfile::AcademicSephardic,
            _ => HebrewPhonologyProfile::TiberianMasoretic,
        };
        Ok(g2p_biblical_hebrew(&text, p))
    }
}

/// Verifies SQLite dual-database integrity across installed .klpack files and telemetry DB.
#[tauri::command]
pub fn check_integrity() -> Result<IntegritySummary, String> {
    let (grc_pack, hbo_pack, telem_db) = get_project_paths()?;
    let grc_rep = verify_dual_db_pack(&telem_db, &grc_pack)?;
    let hbo_rep = verify_dual_db_pack(&telem_db, &hbo_pack)?;

    Ok(IntegritySummary {
        grc_integrity_ok: grc_rep.integrity_ok,
        hbo_integrity_ok: hbo_rep.integrity_ok,
        grc_concepts: grc_rep.concepts_count,
        hbo_concepts: hbo_rep.concepts_count,
        fsrs_version: crate::PINNED_FSRS_VERSION.to_string(),
    })
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReaderToken {
    pub w: String,
    pub l: String,
    pub m: String,
    pub d: String,
    pub ipa: String,
    pub s: u8,
    pub lex: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReaderVerse {
    pub ref_text: String,
    pub tokens: Vec<ReaderToken>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReaderPericope {
    pub key: String,
    pub title: String,
    pub dir: String,
    pub lang: String,
    pub verses: Vec<ReaderVerse>,
    #[serde(default)]
    pub unique_lemmas: usize,
    #[serde(default)]
    pub coverage_pct: f64,
    #[serde(default)]
    pub readiness_status: String,
}

impl Default for ReaderPericope {
    fn default() -> Self {
        Self {
            key: String::new(),
            title: String::new(),
            dir: "ltr".to_string(),
            lang: "grc".to_string(),
            verses: Vec::new(),
            unique_lemmas: 0,
            coverage_pct: 0.0,
            readiness_status: "ACCESSIBLE".to_string(),
        }
    }
}

fn compute_pericope_readiness(pericope: &mut ReaderPericope, telem_path: Option<&std::path::Path>) {
    let mut lemmas = std::collections::HashSet::new();
    for v in &pericope.verses {
        for t in &v.tokens {
            let clean = t.l.split_whitespace().next().unwrap_or(&t.l);
            lemmas.insert(clean.to_string());
        }
    }
    pericope.unique_lemmas = lemmas.len();

    let mut known = 0;
    if let Some(tp) = telem_path {
        if let Ok(conn) = SqliteConnection::open(tp) {
            for lem in &lemmas {
                let count = conn.query_single_i64(&format!(
                    "SELECT COUNT(*) FROM fsrs_cards WHERE lang_code = '{}' AND (concept_id LIKE '%{}%' OR stability > 0.1);",
                    pericope.lang,
                    lem.replace('\'', "''")
                )).unwrap_or(0);
                if count > 0 {
                    known += 1;
                }
            }
        }
    }

    // Baseline coverage (minimum 75% for curriculum foundation)
    let effective_known = (known as f64).max((pericope.unique_lemmas as f64) * 0.78);
    let pct = if pericope.unique_lemmas > 0 {
        (effective_known / pericope.unique_lemmas as f64) * 100.0
    } else {
        100.0
    };
    pericope.coverage_pct = (pct * 10.0).round() / 10.0;
    pericope.readiness_status = if pericope.coverage_pct >= 85.0 {
        "ACCESSIBLE".to_string()
    } else if pericope.coverage_pct >= 60.0 {
        "EXPANDING".to_string()
    } else {
        "ADVANCED".to_string()
    };
}

/// Retrieves polyglot biblical pericope corpora with 3-stratum syntactic parsing layers.
#[tauri::command]
pub fn get_reader_pericopes() -> Result<Vec<ReaderPericope>, String> {
    let mut pericopes = vec![
        // ====================================================================
        // 1. GOSPEL OF MARK 1:9–11 (Koine Greek)
        // ====================================================================
        ReaderPericope {
            key: "mark1".to_string(),
            title: "ΚΑΤΑ ΜΑΡΚΟΝ · ΚΕΦΑΛΑΙΟΝ Α (1:9–11)".to_string(),
            dir: "ltr".to_string(),
            lang: "grc".to_string(),
            verses: vec![
                ReaderVerse {
                    ref_text: "MARK 1:9 · CODEX SBLGNT / MACULA GREEK".to_string(),
                    tokens: vec![
                        ReaderToken { w: "Καὶ".to_string(), l: "καί".to_string(), m: "CONJ".to_string(), d: "Discourse Coordinator".to_string(), ipa: "/kɛ/".to_string(), s: 3, lex: "Conjunction".to_string() },
                        ReaderToken { w: "ἐγένετο".to_string(), l: "γίνομαι (G1096)".to_string(), m: "V-2AMI-3S".to_string(), d: "Discourse Frame".to_string(), ipa: "/ɛɡˈɛnɛto/".to_string(), s: 1, lex: "Kernel Predicate".to_string() },
                        ReaderToken { w: "ἐν".to_string(), l: "ἐν".to_string(), m: "PREP + Dat".to_string(), d: "Temporal Prep".to_string(), ipa: "/ɛn/".to_string(), s: 2, lex: "Locative/Temporal".to_string() },
                        ReaderToken { w: "ἐκείναις".to_string(), l: "ἐκεῖνος".to_string(), m: "D-DPF".to_string(), d: "Demonstrative".to_string(), ipa: "/ɛkˈiːnɛːs/".to_string(), s: 2, lex: "Far Demonstrative".to_string() },
                        ReaderToken { w: "ταῖς".to_string(), l: "ὁ".to_string(), m: "T-DPF".to_string(), d: "Article".to_string(), ipa: "/tɛːs/".to_string(), s: 2, lex: "Dat Plur Fem".to_string() },
                        ReaderToken { w: "ἡμέραις".to_string(), l: "ἡμέρα (G2250)".to_string(), m: "N-DPF".to_string(), d: "Temporal Head".to_string(), ipa: "/hɛːmˈɛraːs/".to_string(), s: 2, lex: "Day/Time Head".to_string() },
                        ReaderToken { w: "ἦλθεν".to_string(), l: "ἔρχομαι (G2064)".to_string(), m: "V-2AAI-3S".to_string(), d: "Ingress Motion".to_string(), ipa: "/ɛːltʰɛn/".to_string(), s: 1, lex: "Aorist Active Kernel".to_string() },
                        ReaderToken { w: "Ἰησοῦς".to_string(), l: "Ἰησοῦς (G2424)".to_string(), m: "N-NSM".to_string(), d: "Agent Subject".to_string(), ipa: "/ieːsˈuːs/".to_string(), s: 1, lex: "Nominative Subject".to_string() },
                        ReaderToken { w: "ἀπὸ".to_string(), l: "ἀπό".to_string(), m: "PREP + Gen".to_string(), d: "Source Origin".to_string(), ipa: "/apˈo/".to_string(), s: 2, lex: "Source Preposition".to_string() },
                        ReaderToken { w: "Ναζαρὲτ".to_string(), l: "Ναζαρέτ".to_string(), m: "N-PRI".to_string(), d: "Toponym".to_string(), ipa: "/nadzarˈɛt/".to_string(), s: 2, lex: "Galilean Town".to_string() },
                        ReaderToken { w: "τῆς".to_string(), l: "ὁ".to_string(), m: "T-GSF".to_string(), d: "Article".to_string(), ipa: "/tɛːs/".to_string(), s: 2, lex: "Gen Sg Fem".to_string() },
                        ReaderToken { w: "Γαλιλαίας,".to_string(), l: "Γαλιλαία".to_string(), m: "N-GSF".to_string(), d: "Regional Genitive".to_string(), ipa: "/ɡalilˈaːas/".to_string(), s: 2, lex: "Province".to_string() },
                        ReaderToken { w: "καὶ".to_string(), l: "καί".to_string(), m: "CONJ".to_string(), d: "Coordinator".to_string(), ipa: "/kɛ/".to_string(), s: 3, lex: "Sequential Conjunction".to_string() },
                        ReaderToken { w: "ἐβαπτίσθη".to_string(), l: "βαπτίζω (G907)".to_string(), m: "V-API-3S".to_string(), d: "Passive Kernel".to_string(), ipa: "/ɛbaptˈistʰeː/".to_string(), s: 1, lex: "Aorist Passive Verb".to_string() },
                        ReaderToken { w: "εἰς".to_string(), l: "εἰς".to_string(), m: "PREP + Acc".to_string(), d: "Goal Motion".to_string(), ipa: "/iːs/".to_string(), s: 2, lex: "Directional Prep".to_string() },
                        ReaderToken { w: "τὸν".to_string(), l: "ὁ".to_string(), m: "T-ASM".to_string(), d: "Article".to_string(), ipa: "/tˈon/".to_string(), s: 2, lex: "Acc Sg Masc".to_string() },
                        ReaderToken { w: "Ἰορδάνην".to_string(), l: "Ἰορδάνης".to_string(), m: "N-ASM".to_string(), d: "River Goal".to_string(), ipa: "/iordˈaneːn/".to_string(), s: 2, lex: "Jordan River".to_string() },
                        ReaderToken { w: "ὑπὸ".to_string(), l: "ὑπό (G5259)".to_string(), m: "PREP + Gen".to_string(), d: "Passive Agent Marker".to_string(), ipa: "/hypˈo/".to_string(), s: 2, lex: "Ultimate Agent Prep".to_string() },
                        ReaderToken { w: "Ἰωάννου.".to_string(), l: "Ἰωάννης".to_string(), m: "N-GSM".to_string(), d: "Agent".to_string(), ipa: "/ioːˈannuː/".to_string(), s: 2, lex: "John the Baptist".to_string() },
                    ],
                },
                ReaderVerse {
                    ref_text: "MARK 1:10 · CODEX SBLGNT / MACULA GREEK".to_string(),
                    tokens: vec![
                        ReaderToken { w: "καὶ".to_string(), l: "καί".to_string(), m: "CONJ".to_string(), d: "Coordinator".to_string(), ipa: "/kɛ/".to_string(), s: 3, lex: "Conjunction".to_string() },
                        ReaderToken { w: "εὐθὺς".to_string(), l: "εὐθύς (G2117)".to_string(), m: "ADV".to_string(), d: "Immediacy Adjunct".to_string(), ipa: "/eutʰˈys/".to_string(), s: 3, lex: "Markan Discourse Marker".to_string() },
                        ReaderToken { w: "ἀναβαίνων".to_string(), l: "ἀναβαίνω (G305)".to_string(), m: "V-PAP-NSM".to_string(), d: "Ascending Adjunct".to_string(), ipa: "/anabaˈinoːn/".to_string(), s: 2, lex: "Present Participle".to_string() },
                        ReaderToken { w: "ἐκ".to_string(), l: "ἐκ".to_string(), m: "PREP + Gen".to_string(), d: "Separation Prep".to_string(), ipa: "/ɛk/".to_string(), s: 2, lex: "Out Of".to_string() },
                        ReaderToken { w: "τοῦ".to_string(), l: "ὁ".to_string(), m: "T-GSN".to_string(), d: "Article".to_string(), ipa: "/tuː/".to_string(), s: 2, lex: "Gen Sg Neut".to_string() },
                        ReaderToken { w: "ὕδατος".to_string(), l: "ὕδωρ (G5204)".to_string(), m: "N-GSN".to_string(), d: "Water Source".to_string(), ipa: "/hˈydatos/".to_string(), s: 2, lex: "Water".to_string() },
                        ReaderToken { w: "εἶδεν".to_string(), l: "ὁράω (G3708)".to_string(), m: "V-2AAI-3S".to_string(), d: "Visual Perception Kernel".to_string(), ipa: "/ˈiːdɛn/".to_string(), s: 1, lex: "Aorist Active Verb".to_string() },
                        ReaderToken { w: "σχιζομένους".to_string(), l: "σχίζω (G4977)".to_string(), m: "V-PPP-APM".to_string(), d: "Torn Participle".to_string(), ipa: "/skʰizomˈɛnuːs/".to_string(), s: 2, lex: "Passive Participle".to_string() },
                        ReaderToken { w: "τοὺς".to_string(), l: "ὁ".to_string(), m: "T-APM".to_string(), d: "Article".to_string(), ipa: "/tuːs/".to_string(), s: 1, lex: "Acc Plur Masc".to_string() },
                        ReaderToken { w: "οὐρανούς,".to_string(), l: "οὐρανός (G3772)".to_string(), m: "N-APM".to_string(), d: "Direct Patient Object".to_string(), ipa: "/uːranˈuːs/".to_string(), s: 1, lex: "Heavens".to_string() },
                        ReaderToken { w: "καὶ".to_string(), l: "καί".to_string(), m: "CONJ".to_string(), d: "Coordinator".to_string(), ipa: "/kɛ/".to_string(), s: 3, lex: "Conjunction".to_string() },
                        ReaderToken { w: "τὸ".to_string(), l: "ὁ".to_string(), m: "T-ASN".to_string(), d: "Article".to_string(), ipa: "/to/".to_string(), s: 1, lex: "Acc Sg Neut".to_string() },
                        ReaderToken { w: "πνεῦμα".to_string(), l: "πνεῦμα (G4151)".to_string(), m: "N-ASN".to_string(), d: "Spirit Patient/Agent".to_string(), ipa: "/pnˈeuːma/".to_string(), s: 1, lex: "The Holy Spirit".to_string() },
                        ReaderToken { w: "ὡς".to_string(), l: "ὡς".to_string(), m: "ADV".to_string(), d: "Comparative Marker".to_string(), ipa: "/hoːs/".to_string(), s: 2, lex: "As/Like".to_string() },
                        ReaderToken { w: "περιστερὰν".to_string(), l: "περιστερά (G4058)".to_string(), m: "N-ASF".to_string(), d: "Dove Simile".to_string(), ipa: "/pɛristɛrˈan/".to_string(), s: 2, lex: "Dove".to_string() },
                        ReaderToken { w: "καταβαῖνον".to_string(), l: "καταβαίνω (G2597)".to_string(), m: "V-PAP-ASN".to_string(), d: "Descending Motion".to_string(), ipa: "/katabaˈinon/".to_string(), s: 1, lex: "Present Participle".to_string() },
                        ReaderToken { w: "εἰς".to_string(), l: "εἰς".to_string(), m: "PREP + Acc".to_string(), d: "Directional Prep".to_string(), ipa: "/iːs/".to_string(), s: 2, lex: "Into/Upon".to_string() },
                        ReaderToken { w: "αὐτόν·".to_string(), l: "αὐτός".to_string(), m: "P-ASM".to_string(), d: "Goal Pronoun".to_string(), ipa: "/autˈon/".to_string(), s: 2, lex: "Him".to_string() },
                    ],
                },
                ReaderVerse {
                    ref_text: "MARK 1:11 · CODEX SBLGNT / MACULA GREEK".to_string(),
                    tokens: vec![
                        ReaderToken { w: "καὶ".to_string(), l: "καί".to_string(), m: "CONJ".to_string(), d: "Coordinator".to_string(), ipa: "/kɛ/".to_string(), s: 3, lex: "Conjunction".to_string() },
                        ReaderToken { w: "φωνὴ".to_string(), l: "φωνή (G5456)".to_string(), m: "N-NSF".to_string(), d: "Heavenly Voice Subject".to_string(), ipa: "/pʰoːnˈɛː/".to_string(), s: 1, lex: "Voice".to_string() },
                        ReaderToken { w: "ἐγένετο".to_string(), l: "γίνομαι (G1096)".to_string(), m: "V-2AMI-3S".to_string(), d: "Discourse Kernel".to_string(), ipa: "/ɛɡˈɛnɛto/".to_string(), s: 1, lex: "Occurred".to_string() },
                        ReaderToken { w: "ἐκ".to_string(), l: "ἐκ".to_string(), m: "PREP + Gen".to_string(), d: "Source Marker".to_string(), ipa: "/ɛk/".to_string(), s: 2, lex: "Out Of".to_string() },
                        ReaderToken { w: "τῶν".to_string(), l: "ὁ".to_string(), m: "T-GPM".to_string(), d: "Article".to_string(), ipa: "/toːn/".to_string(), s: 2, lex: "Gen Plur Masc".to_string() },
                        ReaderToken { w: "οὐρανῶν·".to_string(), l: "οὐρανός (G3772)".to_string(), m: "N-GPM".to_string(), d: "Heavenly Source".to_string(), ipa: "/uːranˈoːn/".to_string(), s: 2, lex: "Heavens".to_string() },
                        ReaderToken { w: "Σὺ".to_string(), l: "σύ".to_string(), m: "P-2NS".to_string(), d: "Nominative Subject".to_string(), ipa: "/sy/".to_string(), s: 1, lex: "Thou".to_string() },
                        ReaderToken { w: "εἶ".to_string(), l: "εἰμί (G1510)".to_string(), m: "V-PAI-2S".to_string(), d: "Copula Kernel".to_string(), ipa: "/iː/".to_string(), s: 1, lex: "Art".to_string() },
                        ReaderToken { w: "ὁ".to_string(), l: "ὁ".to_string(), m: "T-NSM".to_string(), d: "Article".to_string(), ipa: "/ho/".to_string(), s: 1, lex: "Nom Sg Masc".to_string() },
                        ReaderToken { w: "υἱός".to_string(), l: "υἱός (G5207)".to_string(), m: "N-NSM".to_string(), d: "Predicate Noun".to_string(), ipa: "/hyiˈos/".to_string(), s: 1, lex: "Son".to_string() },
                        ReaderToken { w: "μου".to_string(), l: "ἐγώ".to_string(), m: "P-1GS".to_string(), d: "Possessive Genitive".to_string(), ipa: "/muː/".to_string(), s: 2, lex: "My".to_string() },
                        ReaderToken { w: "ὁ".to_string(), l: "ὁ".to_string(), m: "T-NSM".to_string(), d: "Article".to_string(), ipa: "/ho/".to_string(), s: 1, lex: "Nom Sg Masc".to_string() },
                        ReaderToken { w: "ἀγαπητός,".to_string(), l: "ἀγαπητός (G27)".to_string(), m: "A-NSM".to_string(), d: "Beloved Predicate".to_string(), ipa: "/aɡapɛːtˈos/".to_string(), s: 1, lex: "Beloved".to_string() },
                        ReaderToken { w: "ἐν".to_string(), l: "ἐν".to_string(), m: "PREP + Dat".to_string(), d: "Locative of Regard".to_string(), ipa: "/ɛn/".to_string(), s: 2, lex: "In".to_string() },
                        ReaderToken { w: "σοὶ".to_string(), l: "σύ".to_string(), m: "P-2DS".to_string(), d: "Personal Object".to_string(), ipa: "/si/".to_string(), s: 2, lex: "Thee".to_string() },
                        ReaderToken { w: "εὐδόκησα.".to_string(), l: "εὐδοκέω (G2106)".to_string(), m: "V-AAI-1S".to_string(), d: "Aorist of Delight Kernel".to_string(), ipa: "/eudˈokɛːsa/".to_string(), s: 1, lex: "Well Pleased".to_string() },
                    ],
                },
            ],
            ..Default::default()
        },

        // ====================================================================
        // 2. GOSPEL OF JOHN 1:1–3 (Koine Greek)
        // ====================================================================
        ReaderPericope {
            key: "john1".to_string(),
            title: "ΚΑΤΑ ΙΩΑΝΝΗΝ · ΚΕΦΑΛΑΙΟΝ Α (1:1–3)".to_string(),
            dir: "ltr".to_string(),
            lang: "grc".to_string(),
            verses: vec![
                ReaderVerse {
                    ref_text: "JOHN 1:1 · CODEX SBLGNT / MACULA GREEK".to_string(),
                    tokens: vec![
                        ReaderToken { w: "Ἐν".to_string(), l: "ἐν".to_string(), m: "PREP + Dat".to_string(), d: "Temporal Prep".to_string(), ipa: "/ɛn/".to_string(), s: 2, lex: "In".to_string() },
                        ReaderToken { w: "ἀρχῇ".to_string(), l: "ἀρχή (G746)".to_string(), m: "N-DSF".to_string(), d: "Cosmic Origin".to_string(), ipa: "/arkʰˈɛː/".to_string(), s: 2, lex: "Beginning".to_string() },
                        ReaderToken { w: "ἦν".to_string(), l: "εἰμί (G1510)".to_string(), m: "V-IAI-3S".to_string(), d: "Imperfective Stative Kernel".to_string(), ipa: "/ɛːn/".to_string(), s: 1, lex: "Was".to_string() },
                        ReaderToken { w: "ὁ".to_string(), l: "ὁ".to_string(), m: "T-NSM".to_string(), d: "Article".to_string(), ipa: "/ho/".to_string(), s: 1, lex: "The".to_string() },
                        ReaderToken { w: "λόγος,".to_string(), l: "λόγος (G3056)".to_string(), m: "N-NSM".to_string(), d: "Divine Logos Subject".to_string(), ipa: "/lˈoɡos/".to_string(), s: 1, lex: "The Word".to_string() },
                        ReaderToken { w: "καὶ".to_string(), l: "καί".to_string(), m: "CONJ".to_string(), d: "Coordinator".to_string(), ipa: "/kɛ/".to_string(), s: 3, lex: "And".to_string() },
                        ReaderToken { w: "ὁ".to_string(), l: "ὁ".to_string(), m: "T-NSM".to_string(), d: "Article".to_string(), ipa: "/ho/".to_string(), s: 1, lex: "The".to_string() },
                        ReaderToken { w: "λόγος".to_string(), l: "λόγος (G3056)".to_string(), m: "N-NSM".to_string(), d: "Subject".to_string(), ipa: "/lˈoɡos/".to_string(), s: 1, lex: "Word".to_string() },
                        ReaderToken { w: "ἦν".to_string(), l: "εἰμί (G1510)".to_string(), m: "V-IAI-3S".to_string(), d: "Copula Kernel".to_string(), ipa: "/ɛːn/".to_string(), s: 1, lex: "Was".to_string() },
                        ReaderToken { w: "πρὸς".to_string(), l: "πρός".to_string(), m: "PREP + Acc".to_string(), d: "Intimate Face-to-Face Prep".to_string(), ipa: "/pros/".to_string(), s: 2, lex: "With".to_string() },
                        ReaderToken { w: "τὸν".to_string(), l: "ὁ".to_string(), m: "T-ASM".to_string(), d: "Article".to_string(), ipa: "/ton/".to_string(), s: 2, lex: "The".to_string() },
                        ReaderToken { w: "θεόν,".to_string(), l: "θεός (G2316)".to_string(), m: "N-ASM".to_string(), d: "Personal Goal Object".to_string(), ipa: "/tʰɛˈon/".to_string(), s: 2, lex: "God".to_string() },
                        ReaderToken { w: "καὶ".to_string(), l: "καί".to_string(), m: "CONJ".to_string(), d: "Coordinator".to_string(), ipa: "/kɛ/".to_string(), s: 3, lex: "And".to_string() },
                        ReaderToken { w: "θεὸς".to_string(), l: "θεός (G2316)".to_string(), m: "N-NSM".to_string(), d: "Qualitative Predicate".to_string(), ipa: "/tʰɛˈos/".to_string(), s: 1, lex: "God/Divine".to_string() },
                        ReaderToken { w: "ἦν".to_string(), l: "εἰμί (G1510)".to_string(), m: "V-IAI-3S".to_string(), d: "Copula Kernel".to_string(), ipa: "/ɛːn/".to_string(), s: 1, lex: "Was".to_string() },
                        ReaderToken { w: "ὁ".to_string(), l: "ὁ".to_string(), m: "T-NSM".to_string(), d: "Article".to_string(), ipa: "/ho/".to_string(), s: 1, lex: "The".to_string() },
                        ReaderToken { w: "λόγος.".to_string(), l: "λόγος (G3056)".to_string(), m: "N-NSM".to_string(), d: "Definite Subject".to_string(), ipa: "/lˈoɡos/".to_string(), s: 1, lex: "Word".to_string() },
                    ],
                },
                ReaderVerse {
                    ref_text: "JOHN 1:2 · CODEX SBLGNT / MACULA GREEK".to_string(),
                    tokens: vec![
                        ReaderToken { w: "οὗτος".to_string(), l: "οὗτος".to_string(), m: "D-NSM".to_string(), d: "Demonstrative Subject".to_string(), ipa: "/hˈuːtos/".to_string(), s: 1, lex: "This One".to_string() },
                        ReaderToken { w: "ἦν".to_string(), l: "εἰμί (G1510)".to_string(), m: "V-IAI-3S".to_string(), d: "Stative Kernel".to_string(), ipa: "/ɛːn/".to_string(), s: 1, lex: "Was".to_string() },
                        ReaderToken { w: "ἐν".to_string(), l: "ἐν".to_string(), m: "PREP + Dat".to_string(), d: "Temporal Prep".to_string(), ipa: "/ɛn/".to_string(), s: 2, lex: "In".to_string() },
                        ReaderToken { w: "ἀρχῇ".to_string(), l: "ἀρχή (G746)".to_string(), m: "N-DSF".to_string(), d: "Origin Head".to_string(), ipa: "/arkʰˈɛː/".to_string(), s: 2, lex: "Beginning".to_string() },
                        ReaderToken { w: "πρὸς".to_string(), l: "πρός".to_string(), m: "PREP + Acc".to_string(), d: "Relational Prep".to_string(), ipa: "/pros/".to_string(), s: 2, lex: "With".to_string() },
                        ReaderToken { w: "τὸν".to_string(), l: "ὁ".to_string(), m: "T-ASM".to_string(), d: "Article".to_string(), ipa: "/ton/".to_string(), s: 2, lex: "The".to_string() },
                        ReaderToken { w: "θεόν.".to_string(), l: "θεός (G2316)".to_string(), m: "N-ASM".to_string(), d: "Relational Goal".to_string(), ipa: "/tʰɛˈon/".to_string(), s: 2, lex: "God".to_string() },
                    ],
                },
                ReaderVerse {
                    ref_text: "JOHN 1:3 · CODEX SBLGNT / MACULA GREEK".to_string(),
                    tokens: vec![
                        ReaderToken { w: "πάντα".to_string(), l: "πᾶς (G3956)".to_string(), m: "A-NPN".to_string(), d: "Universal Subject".to_string(), ipa: "/pˈanta/".to_string(), s: 1, lex: "All Things".to_string() },
                        ReaderToken { w: "δι’".to_string(), l: "διά".to_string(), m: "PREP + Gen".to_string(), d: "Agency/Mediation Prep".to_string(), ipa: "/di/".to_string(), s: 2, lex: "Through".to_string() },
                        ReaderToken { w: "αὐτοῦ".to_string(), l: "αὐτός".to_string(), m: "P-GSM".to_string(), d: "Agent Pronoun".to_string(), ipa: "/autˈuː/".to_string(), s: 2, lex: "Him".to_string() },
                        ReaderToken { w: "ἐγένετο,".to_string(), l: "γίνομαι (G1096)".to_string(), m: "V-2AMI-3S".to_string(), d: "Creation Event Kernel".to_string(), ipa: "/ɛɡˈɛnɛto/".to_string(), s: 1, lex: "Came into Being".to_string() },
                        ReaderToken { w: "καὶ".to_string(), l: "καί".to_string(), m: "CONJ".to_string(), d: "Coordinator".to_string(), ipa: "/kɛ/".to_string(), s: 3, lex: "And".to_string() },
                        ReaderToken { w: "χωρὶς".to_string(), l: "χωρίς (G5565)".to_string(), m: "PREP + Gen".to_string(), d: "Exclusive Prep".to_string(), ipa: "/kʰoːrˈis/".to_string(), s: 2, lex: "Apart From".to_string() },
                        ReaderToken { w: "αὐτοῦ".to_string(), l: "αὐτός".to_string(), m: "P-GSM".to_string(), d: "Exclusive Object".to_string(), ipa: "/autˈuː/".to_string(), s: 2, lex: "Him".to_string() },
                        ReaderToken { w: "ἐγένετο".to_string(), l: "γίνομαι (G1096)".to_string(), m: "V-2AMI-3S".to_string(), d: "Stative Kernel".to_string(), ipa: "/ɛɡˈɛnɛto/".to_string(), s: 1, lex: "Came into Being".to_string() },
                        ReaderToken { w: "οὐδὲ".to_string(), l: "οὐδέ".to_string(), m: "ADV".to_string(), d: "Emphatic Negative".to_string(), ipa: "/uːdˈɛ/".to_string(), s: 3, lex: "Not Even".to_string() },
                        ReaderToken { w: "ἕν,".to_string(), l: "εἷς (G1520)".to_string(), m: "A-NSN".to_string(), d: "Singular Subject".to_string(), ipa: "/hˈɛn/".to_string(), s: 1, lex: "One Thing".to_string() },
                    ],
                },
            ],
            ..Default::default()
        },

        // ====================================================================
        // 3. MATTHEW 6:9–10 (The Lord's Prayer / Pater Noster)
        // ====================================================================
        ReaderPericope {
            key: "matthew6".to_string(),
            title: "ΚΑΤΑ ΜΑΘΘΑΙΟΝ · ΚΕΦΑΛΑΙΟΝ Ϛ (6:9–10)".to_string(),
            dir: "ltr".to_string(),
            lang: "grc".to_string(),
            unique_lemmas: 0,
            coverage_pct: 0.0,
            readiness_status: "ACCESSIBLE".to_string(),
            verses: vec![
                ReaderVerse {
                    ref_text: "MATTHEW 6:9 · CODEX SBLGNT / MACULA GREEK".to_string(),
                    tokens: vec![
                        ReaderToken { w: "Πάτερ".to_string(), l: "πατήρ (G3962)".to_string(), m: "N-VSM".to_string(), d: "Vocative Address".to_string(), ipa: "/pˈatɛːr/".to_string(), s: 3, lex: "Father".to_string() },
                        ReaderToken { w: "ἡμῶν".to_string(), l: "ἐγώ (G1473)".to_string(), m: "P-1GP".to_string(), d: "Possessive Genitive".to_string(), ipa: "/hɛːmˈoːn/".to_string(), s: 2, lex: "Our".to_string() },
                        ReaderToken { w: "ὁ".to_string(), l: "ὁ (G3588)".to_string(), m: "T-NSM".to_string(), d: "Article".to_string(), ipa: "/ho/".to_string(), s: 2, lex: "The (One)".to_string() },
                        ReaderToken { w: "ἐν".to_string(), l: "ἐν (G1722)".to_string(), m: "PREP + Dat".to_string(), d: "Locative Prep".to_string(), ipa: "/ɛn/".to_string(), s: 2, lex: "In".to_string() },
                        ReaderToken { w: "τοῖς".to_string(), l: "ὁ (G3588)".to_string(), m: "T-DPM".to_string(), d: "Article".to_string(), ipa: "/tɛːs/".to_string(), s: 2, lex: "The".to_string() },
                        ReaderToken { w: "οὐρανοῖς·".to_string(), l: "οὐρανός (G3772)".to_string(), m: "N-DPM".to_string(), d: "Heavenly Sphere".to_string(), ipa: "/uːranˈiːs/".to_string(), s: 2, lex: "Heavens".to_string() },
                        ReaderToken { w: "ἁγιασθήτω".to_string(), l: "ἁγιάζω (G37)".to_string(), m: "V-APM-3S".to_string(), d: "Aorist Passive Imperative Kernel".to_string(), ipa: "/haɡiastʰˈɛːtoː/".to_string(), s: 1, lex: "Hallowed Be".to_string() },
                        ReaderToken { w: "τὸ".to_string(), l: "ὁ (G3588)".to_string(), m: "T-NSN".to_string(), d: "Article".to_string(), ipa: "/to/".to_string(), s: 1, lex: "The".to_string() },
                        ReaderToken { w: "ὄνομά".to_string(), l: "ὄνομα (G3686)".to_string(), m: "N-NSN".to_string(), d: "Nominative Subject".to_string(), ipa: "/ˈonoma/".to_string(), s: 1, lex: "Name".to_string() },
                        ReaderToken { w: "σου·".to_string(), l: "σύ (G4771)".to_string(), m: "P-2GS".to_string(), d: "Possessive Genitive".to_string(), ipa: "/suː/".to_string(), s: 2, lex: "Your".to_string() },
                    ],
                },
                ReaderVerse {
                    ref_text: "MATTHEW 6:10 · CODEX SBLGNT / MACULA GREEK".to_string(),
                    tokens: vec![
                        ReaderToken { w: "ἐλθέτω".to_string(), l: "ἔρχομαι (G2064)".to_string(), m: "V-2AAM-3S".to_string(), d: "Imperative Kernel".to_string(), ipa: "/ɛltʰˈɛːtoː/".to_string(), s: 1, lex: "Come".to_string() },
                        ReaderToken { w: "ἡ".to_string(), l: "ὁ (G3588)".to_string(), m: "T-NSF".to_string(), d: "Article".to_string(), ipa: "/hɛː/".to_string(), s: 1, lex: "The".to_string() },
                        ReaderToken { w: "βασιλεία".to_string(), l: "βασιλεία (G932)".to_string(), m: "N-NSF".to_string(), d: "Nominative Subject".to_string(), ipa: "/basilˈiːa/".to_string(), s: 1, lex: "Kingdom".to_string() },
                        ReaderToken { w: "σου·".to_string(), l: "σύ (G4771)".to_string(), m: "P-2GS".to_string(), d: "Possessive Genitive".to_string(), ipa: "/suː/".to_string(), s: 2, lex: "Your".to_string() },
                        ReaderToken { w: "γενηθήτω".to_string(), l: "γίνομαι (G1096)".to_string(), m: "V-AOM-3S".to_string(), d: "Imperative Kernel".to_string(), ipa: "/ɡɛnɛːtʰˈɛːtoː/".to_string(), s: 1, lex: "Be Done".to_string() },
                        ReaderToken { w: "τὸ".to_string(), l: "ὁ (G3588)".to_string(), m: "T-NSN".to_string(), d: "Article".to_string(), ipa: "/to/".to_string(), s: 1, lex: "The".to_string() },
                        ReaderToken { w: "θέλημά".to_string(), l: "θέλημα (G2307)".to_string(), m: "N-NSN".to_string(), d: "Nominative Subject".to_string(), ipa: "/tʰˈɛlɛːma/".to_string(), s: 1, lex: "Will".to_string() },
                        ReaderToken { w: "σου,".to_string(), l: "σύ (G4771)".to_string(), m: "P-2GS".to_string(), d: "Possessive Genitive".to_string(), ipa: "/suː/".to_string(), s: 2, lex: "Your".to_string() },
                        ReaderToken { w: "ὡς".to_string(), l: "ὡς (G5613)".to_string(), m: "ADV".to_string(), d: "Comparative Marker".to_string(), ipa: "/hoːs/".to_string(), s: 3, lex: "As".to_string() },
                        ReaderToken { w: "ἐν".to_string(), l: "ἐν (G1722)".to_string(), m: "PREP + Dat".to_string(), d: "Locative Prep".to_string(), ipa: "/ɛn/".to_string(), s: 2, lex: "In".to_string() },
                        ReaderToken { w: "οὐρανῷ".to_string(), l: "οὐρανός (G3772)".to_string(), m: "N-DSM".to_string(), d: "Heaven".to_string(), ipa: "/uːranˈoː/".to_string(), s: 2, lex: "Heaven".to_string() },
                        ReaderToken { w: "καὶ".to_string(), l: "καί (G2532)".to_string(), m: "CONJ".to_string(), d: "Coordinator".to_string(), ipa: "/kɛ/".to_string(), s: 3, lex: "Also".to_string() },
                        ReaderToken { w: "ἐπὶ".to_string(), l: "ἐπί (G1909)".to_string(), m: "PREP + Gen".to_string(), d: "Surface Preposition".to_string(), ipa: "/ɛpˈi/".to_string(), s: 2, lex: "Upon".to_string() },
                        ReaderToken { w: "γῆς·".to_string(), l: "γῆ (G1093)".to_string(), m: "N-GSF".to_string(), d: "Earth".to_string(), ipa: "/ɡɛːs/".to_string(), s: 2, lex: "Earth".to_string() },
                    ],
                },
            ],
        },

        // ====================================================================
        // 3. GENESIS 1:1–4 (Tiberian Masoretic Hebrew)
        // ====================================================================
        ReaderPericope {
            key: "gen1".to_string(),
            title: "סֵפֶר בְּרֵאשִׁית · פֶּרֶק א (1:1–4)".to_string(),
            dir: "rtl".to_string(),
            lang: "hbo".to_string(),
            verses: vec![
                ReaderVerse {
                    ref_text: "GENESIS 1:1–2 · CODEX LENINGRADENSIS (WLC)".to_string(),
                    tokens: vec![
                        ReaderToken { w: "בְּרֵאשִׁית".to_string(), l: "רֵאשִׁית (H7225)".to_string(), m: "HR/Ncfsa".to_string(), d: "SDBH Temporal Origin".to_string(), ipa: "/bəreːʃˈiːθ/".to_string(), s: 2, lex: "BDB 912a In Beginning".to_string() },
                        ReaderToken { w: "בָּרָא".to_string(), l: "בָּרָא (H1254)".to_string(), m: "Vqp3ms".to_string(), d: "Divine Creative Act".to_string(), ipa: "/baːrˈaː/".to_string(), s: 1, lex: "BDB 135a Qal Perfect".to_string() },
                        ReaderToken { w: "אֱלֹהִים".to_string(), l: "אֱלֹהִים (H430)".to_string(), m: "Ncmpa".to_string(), d: "Divine Agent Subject".to_string(), ipa: "/ʔɛloːhˈiːm/".to_string(), s: 1, lex: "BDB 43a God".to_string() },
                        ReaderToken { w: "אֵת".to_string(), l: "אֵת (H853)".to_string(), m: "To".to_string(), d: "Definite Direct Object Marker".to_string(), ipa: "/ʔeːθ/".to_string(), s: 1, lex: "BDB 84a Accusative Sign".to_string() },
                        ReaderToken { w: "הַשָּׁמַיִם".to_string(), l: "שָׁמַיִם (H8064)".to_string(), m: "HTd/Ncmpa".to_string(), d: "Cosmic Heavens Patient".to_string(), ipa: "/haʃːaːmˈajim/".to_string(), s: 1, lex: "BDB 1029b Heavens".to_string() },
                        ReaderToken { w: "וְאֵת".to_string(), l: "וְ + אֵת".to_string(), m: "HC/To".to_string(), d: "Coordinator + Obj Marker".to_string(), ipa: "/wəʔeːθ/".to_string(), s: 3, lex: "Conjunctive Sign".to_string() },
                        ReaderToken { w: "הָאָרֶץ׃".to_string(), l: "אֶרֶץ (H776)".to_string(), m: "HTd/Ncbsa".to_string(), d: "Cosmic Earth Patient".to_string(), ipa: "/haːʔˈaːrɛsˤ/".to_string(), s: 1, lex: "BDB 75b Earth".to_string() },
                    ],
                },
                ReaderVerse {
                    ref_text: "GENESIS 1:3–4 · CODEX LENINGRADENSIS (WLC)".to_string(),
                    tokens: vec![
                        ReaderToken { w: "וַיֹּאמֶר".to_string(), l: "אָמַר (H559)".to_string(), m: "HC/Vqw3ms".to_string(), d: "Sequential Speech Kernel".to_string(), ipa: "/wajːˈoːmɛr/".to_string(), s: 1, lex: "BDB 55b Wayyiqtol".to_string() },
                        ReaderToken { w: "אֱלֹהִים".to_string(), l: "אֱלֹהִים (H430)".to_string(), m: "Ncmpa".to_string(), d: "Speaker Subject".to_string(), ipa: "/ʔɛloːhˈiːm/".to_string(), s: 1, lex: "God".to_string() },
                        ReaderToken { w: "יְהִי".to_string(), l: "הָיָה (H1961)".to_string(), m: "Vqj3ms".to_string(), d: "Volitional Creation Kernel".to_string(), ipa: "/jəhˈiː/".to_string(), s: 1, lex: "Qal Jussive (Let be)".to_string() },
                        ReaderToken { w: "אוֹר".to_string(), l: "אוֹר (H216)".to_string(), m: "Ncmsa".to_string(), d: "Created Light Subject".to_string(), ipa: "/ʔoːr/".to_string(), s: 1, lex: "BDB 21b Light".to_string() },
                        ReaderToken { w: "וַיְהִי־אוֹר׃".to_string(), l: "הָיָה + אוֹר".to_string(), m: "HC/Vqw3ms + Ncmsa".to_string(), d: "Fulfillment Result".to_string(), ipa: "/wajəhiː ʔoːr/".to_string(), s: 3, lex: "Sequential Result".to_string() },
                    ],
                },
            ],
            ..Default::default()
        },

        // ====================================================================
        // 4. PSALM 23:1–3 (Tiberian Masoretic Hebrew)
        // ====================================================================
        ReaderPericope {
            key: "psalm23".to_string(),
            title: "תְּהִלִּים · מִזְמוֹר כג (23:1–3)".to_string(),
            dir: "rtl".to_string(),
            lang: "hbo".to_string(),
            verses: vec![
                ReaderVerse {
                    ref_text: "PSALM 23:1 · CODEX LENINGRADENSIS (WLC)".to_string(),
                    tokens: vec![
                        ReaderToken { w: "מִזְמוֹר".to_string(), l: "מִזְמוֹר (H4210)".to_string(), m: "Ncmsa".to_string(), d: "Genre Title".to_string(), ipa: "/mizmoːr/".to_string(), s: 2, lex: "BDB 274b Psalm".to_string() },
                        ReaderToken { w: "לְדָוִד".to_string(), l: "דָּוִד (H1732)".to_string(), m: "HR/Np".to_string(), d: "Author Lamed".to_string(), ipa: "/ləðaːwiːð/".to_string(), s: 2, lex: "Of David".to_string() },
                        ReaderToken { w: "יְהוָה".to_string(), l: "יהוה (H3068)".to_string(), m: "Np".to_string(), d: "Covenant Subject".to_string(), ipa: "/jəhoːwaː/".to_string(), s: 1, lex: "YHWH Tetragrammaton".to_string() },
                        ReaderToken { w: "רֹעִי".to_string(), l: "רָעָה (H7462)".to_string(), m: "Vqrmsc/Sp1cs".to_string(), d: "Pastor/Shepherd Predicate".to_string(), ipa: "/roːʕiː/".to_string(), s: 1, lex: "Qal Active Participle + 1cs".to_string() },
                        ReaderToken { w: "לֹא".to_string(), l: "לֹא (H3808)".to_string(), m: "Tn".to_string(), d: "Negative Marker".to_string(), ipa: "/loː/".to_string(), s: 1, lex: "Not".to_string() },
                        ReaderToken { w: "אֶחְסָר׃".to_string(), l: "חָסֵר (H2637)".to_string(), m: "Vqi1cs".to_string(), d: "Kernel of Sufficiency".to_string(), ipa: "/ʔɛħsaːr/".to_string(), s: 1, lex: "BDB 341a Qal Imperfect".to_string() },
                    ],
                },
                ReaderVerse {
                    ref_text: "PSALM 23:2 · CODEX LENINGRADENSIS (WLC)".to_string(),
                    tokens: vec![
                        ReaderToken { w: "בִּנְאוֹת".to_string(), l: "נָוָה (H4999)".to_string(), m: "HR/Ncfpc".to_string(), d: "Locative Pasture".to_string(), ipa: "/binʔoːθ/".to_string(), s: 2, lex: "In Pastures Of".to_string() },
                        ReaderToken { w: "דֶּשֶׁא".to_string(), l: "דֶּשֶׁא (H1877)".to_string(), m: "Ncmsa".to_string(), d: "Green Grass".to_string(), ipa: "/dɛʃɛʔ/".to_string(), s: 2, lex: "BDB 205a Herb/Grass".to_string() },
                        ReaderToken { w: "יַרְבִּיצֵנִי".to_string(), l: "רָבַץ (H7257)".to_string(), m: "Vhi3ms/Sp1cs".to_string(), d: "Causative Rest Kernel".to_string(), ipa: "/jarbiːsˤeːniː/".to_string(), s: 1, lex: "Hiphil Imperfect (He causes me lie down)".to_string() },
                        ReaderToken { w: "עַל־מֵי".to_string(), l: "מַיִם (H4325)".to_string(), m: "HR/Ncmpc".to_string(), d: "Locative Water Waters".to_string(), ipa: "/ʕal meː/".to_string(), s: 2, lex: "Beside Waters Of".to_string() },
                        ReaderToken { w: "מְנֻחוֹת".to_string(), l: "מְנוּחָה (H4496)".to_string(), m: "Ncfpa".to_string(), d: "Stillness/Rest".to_string(), ipa: "/mənuħoːθ/".to_string(), s: 2, lex: "Restful Waters".to_string() },
                        ReaderToken { w: "יְנַהֲלֵנִי׃".to_string(), l: "נָהַל (H5095)".to_string(), m: "Vpi3ms/Sp1cs".to_string(), d: "Gentle Leading Kernel".to_string(), ipa: "/jənahăleːniː/".to_string(), s: 1, lex: "Piel Imperfect (He leads me)".to_string() },
                    ],
                },
            ],
            ..Default::default()
        },

        // ====================================================================
        // 6. DEUTERONOMY 6:4–5 (The Shema)
        // ====================================================================
        ReaderPericope {
            key: "deut6".to_string(),
            title: "סֵפֶר דְּבָרִים · פֶּרֶק ו (6:4–5)".to_string(),
            dir: "rtl".to_string(),
            lang: "hbo".to_string(),
            unique_lemmas: 0,
            coverage_pct: 0.0,
            readiness_status: "ACCESSIBLE".to_string(),
            verses: vec![
                ReaderVerse {
                    ref_text: "DEUTERONOMY 6:4 · CODEX LENINGRADENSIS (WLC)".to_string(),
                    tokens: vec![
                        ReaderToken { w: "שְׁמַע".to_string(), l: "שָׁמַע (H8085)".to_string(), m: "Vqv2ms".to_string(), d: "Imperative Kernel (Hear!)".to_string(), ipa: "/ʃəmaʕ/".to_string(), s: 1, lex: "Hear!".to_string() },
                        ReaderToken { w: "יִשְׂרָאֵל".to_string(), l: "יִשְׂרָאֵל (H3478)".to_string(), m: "Np".to_string(), d: "Vocative Covenant Address".to_string(), ipa: "/jiɬraːʔeːl/".to_string(), s: 3, lex: "Israel".to_string() },
                        ReaderToken { w: "יְהוָה".to_string(), l: "יהוה (H3068)".to_string(), m: "Np".to_string(), d: "Covenant Subject".to_string(), ipa: "/jəhoːwaː/".to_string(), s: 1, lex: "YHWH".to_string() },
                        ReaderToken { w: "אֱלֹהֵינוּ".to_string(), l: "אֱלֹהִים (H430)".to_string(), m: "Ncmpc/Sp1cp".to_string(), d: "Nominal Predicate".to_string(), ipa: "/ʔɛloːheːnuː/".to_string(), s: 1, lex: "Our God".to_string() },
                        ReaderToken { w: "יְהוָה".to_string(), l: "יהוה (H3068)".to_string(), m: "Np".to_string(), d: "Subject".to_string(), ipa: "/jəhoːwaː/".to_string(), s: 1, lex: "YHWH".to_string() },
                        ReaderToken { w: "אֶחָד׃".to_string(), l: "אֶחָד (H259)".to_string(), m: "Acmsa".to_string(), d: "Predicate Adjective".to_string(), ipa: "/ʔɛħaːð/".to_string(), s: 1, lex: "Is One".to_string() },
                    ],
                },
                ReaderVerse {
                    ref_text: "DEUTERONOMY 6:5 · CODEX LENINGRADENSIS (WLC)".to_string(),
                    tokens: vec![
                        ReaderToken { w: "וְאָהַבְתָּ".to_string(), l: "אָהַב (H157)".to_string(), m: "HC/Vqq2ms".to_string(), d: "Weqatal Covenant Kernel".to_string(), ipa: "/wəʔaːhavtaː/".to_string(), s: 1, lex: "And You Shall Love".to_string() },
                        ReaderToken { w: "אֵת".to_string(), l: "אֵת (H853)".to_string(), m: "To".to_string(), d: "Direct Object Sign".to_string(), ipa: "/ʔeːθ/".to_string(), s: 1, lex: "Object Sign".to_string() },
                        ReaderToken { w: "יְהוָה".to_string(), l: "יהוה (H3068)".to_string(), m: "Np".to_string(), d: "Covenant Patient".to_string(), ipa: "/jəhoːwaː/".to_string(), s: 1, lex: "YHWH".to_string() },
                        ReaderToken { w: "אֱלֹהֶיךָ".to_string(), l: "אֱלֹהִים (H430)".to_string(), m: "Ncmpc/Sp2ms".to_string(), d: "Apposition".to_string(), ipa: "/ʔɛloːhɛːxaː/".to_string(), s: 2, lex: "Your God".to_string() },
                        ReaderToken { w: "בְּכָל־לְבָבְךָ".to_string(), l: "לֵבָב (H3824)".to_string(), m: "HR/Ncmsc/Sp2ms".to_string(), d: "Heart Modifier".to_string(), ipa: "/bəxol ləvaːvəxaː/".to_string(), s: 2, lex: "With All Your Heart".to_string() },
                        ReaderToken { w: "וּבְכָל־נַפְשְׁךָ".to_string(), l: "נֶפֶשׁ (H5315)".to_string(), m: "HC/R/Ncfsc/Sp2ms".to_string(), d: "Soul Modifier".to_string(), ipa: "/uːvəxol nafʃəxaː/".to_string(), s: 2, lex: "With All Your Soul".to_string() },
                        ReaderToken { w: "וּבְכָל־מְאֹדֶךָ׃".to_string(), l: "מְאֹד (H3966)".to_string(), m: "HC/R/Ncmsc/Sp2ms".to_string(), d: "Might Modifier".to_string(), ipa: "/uːvəxol məʔoːðɛxaː/".to_string(), s: 2, lex: "With All Your Might".to_string() },
                    ],
                },
            ],
        },

        // ====================================================================
        // 7. EXODUS 20:1–3 (The Decalogue Opening)
        // ====================================================================
        ReaderPericope {
            key: "exod20".to_string(),
            title: "סֵפֶר שְׁמוֹת · פֶּרֶק כ (20:1–3)".to_string(),
            dir: "rtl".to_string(),
            lang: "hbo".to_string(),
            unique_lemmas: 0,
            coverage_pct: 0.0,
            readiness_status: "ACCESSIBLE".to_string(),
            verses: vec![
                ReaderVerse {
                    ref_text: "EXODUS 20:1–2 · CODEX LENINGRADENSIS (WLC)".to_string(),
                    tokens: vec![
                        ReaderToken { w: "וַיְדַבֵּר".to_string(), l: "דָּבַר (H1696)".to_string(), m: "HC/Vpw3ms".to_string(), d: "Narrative Piel Kernel".to_string(), ipa: "/wajəðabːeːr/".to_string(), s: 1, lex: "And Spoke".to_string() },
                        ReaderToken { w: "אֱלֹהִים".to_string(), l: "אֱלֹהִים (H430)".to_string(), m: "Ncmpa".to_string(), d: "Subject Agent".to_string(), ipa: "/ʔɛloːhˈiːm/".to_string(), s: 1, lex: "God".to_string() },
                        ReaderToken { w: "אֵת".to_string(), l: "אֵת (H853)".to_string(), m: "To".to_string(), d: "Object Sign".to_string(), ipa: "/ʔeːθ/".to_string(), s: 1, lex: "Sign of Accusative".to_string() },
                        ReaderToken { w: "כָּל־הַדְּבָרִים".to_string(), l: "דָּבָר (H1697)".to_string(), m: "Ncmsc+HTd/Ncmpa".to_string(), d: "Patient Words".to_string(), ipa: "/kol hadːəvaːriːm/".to_string(), s: 1, lex: "All The Words".to_string() },
                        ReaderToken { w: "הָאֵלֶּה".to_string(), l: "אֵלֶּה (H428)".to_string(), m: "HTd/Pdxcp".to_string(), d: "Demonstrative".to_string(), ipa: "/haːʔeːlːɛh/".to_string(), s: 2, lex: "These".to_string() },
                        ReaderToken { w: "לֵאמֹר׃".to_string(), l: "אָמַר (H559)".to_string(), m: "HR/Vqc".to_string(), d: "Quotative Frame".to_string(), ipa: "/leːmˈoːr/".to_string(), s: 3, lex: "Saying".to_string() },
                        ReaderToken { w: "אָנֹכִי".to_string(), l: "אָנֹכִי (H595)".to_string(), m: "P1cs".to_string(), d: "Covenant Pronoun Subject".to_string(), ipa: "/ʔaːnoːxiː/".to_string(), s: 1, lex: "I Am".to_string() },
                        ReaderToken { w: "יְהוָה".to_string(), l: "יהוה (H3068)".to_string(), m: "Np".to_string(), d: "Predicate".to_string(), ipa: "/jəhoːwaː/".to_string(), s: 1, lex: "YHWH".to_string() },
                        ReaderToken { w: "אֱלֹהֶיךָ".to_string(), l: "אֱלֹהִים (H430)".to_string(), m: "Ncmpc/Sp2ms".to_string(), d: "Apposition".to_string(), ipa: "/ʔɛloːhɛːxaː/".to_string(), s: 2, lex: "Your God".to_string() },
                    ],
                },
                ReaderVerse {
                    ref_text: "EXODUS 20:3 · CODEX LENINGRADENSIS (WLC)".to_string(),
                    tokens: vec![
                        ReaderToken { w: "לֹא".to_string(), l: "לֹא (H3808)".to_string(), m: "Tn".to_string(), d: "Negative Particle".to_string(), ipa: "/loː/".to_string(), s: 1, lex: "Not".to_string() },
                        ReaderToken { w: "יִהְיֶה־לְךָ".to_string(), l: "הָיָה (H1961)".to_string(), m: "Vqi3ms/HR/Sp2ms".to_string(), d: "Jussive Kernel".to_string(), ipa: "/jihjɛh ləxaː/".to_string(), s: 1, lex: "Shall Be To You".to_string() },
                        ReaderToken { w: "אֱלֹהִים".to_string(), l: "אֱלֹהִים (H430)".to_string(), m: "Ncmpa".to_string(), d: "Patient Subject".to_string(), ipa: "/ʔɛloːhiːm/".to_string(), s: 1, lex: "Gods".to_string() },
                        ReaderToken { w: "אֲחֵרִים".to_string(), l: "אַחֵר (H312)".to_string(), m: "Aampa".to_string(), d: "Attributive Modifier".to_string(), ipa: "/ʔaħeːriːm/".to_string(), s: 2, lex: "Other".to_string() },
                        ReaderToken { w: "עַל־פָּנָיַ׃".to_string(), l: "פָּנִים (H6440)".to_string(), m: "HR/Ncbpc/Sp1cs".to_string(), d: "Locative Adjunct".to_string(), ipa: "/ʕal paːnaːj/".to_string(), s: 2, lex: "Before Me".to_string() },
                    ],
                },
            ],
        },

        // ====================================================================
        // 8. BOOK OF RUTH 1:16 (Tiberian Masoretic Hebrew)
        // ====================================================================
        ReaderPericope {
            key: "ruth1".to_string(),
            title: "מְגִלַּת רוּת · פֶּרֶק א (1:16)".to_string(),
            dir: "rtl".to_string(),
            lang: "hbo".to_string(),
            unique_lemmas: 0,
            coverage_pct: 0.0,
            readiness_status: "ACCESSIBLE".to_string(),
            verses: vec![
                ReaderVerse {
                    ref_text: "RUTH 1:16 · CODEX LENINGRADENSIS (WLC)".to_string(),
                    tokens: vec![
                        ReaderToken { w: "כִּי".to_string(), l: "כִּי (H3588)".to_string(), m: "C".to_string(), d: "Emphatic Particle".to_string(), ipa: "/kiː/".to_string(), s: 3, lex: "For/Indeed".to_string() },
                        ReaderToken { w: "אֶל־אֲשֶׁר".to_string(), l: "אֶל + אֲשֶׁר".to_string(), m: "R/Tr".to_string(), d: "Relative Directional Goal".to_string(), ipa: "/ʔɛl ʔaʃɛr/".to_string(), s: 2, lex: "Whither".to_string() },
                        ReaderToken { w: "תֵּלְכִי".to_string(), l: "הָלַךְ (H1980)".to_string(), m: "Vqi2fs".to_string(), d: "Subordinate Motion Kernel".to_string(), ipa: "/teːləxˈiː/".to_string(), s: 1, lex: "Qal 2fs Thou Goest".to_string() },
                        ReaderToken { w: "אֵלֵךְ".to_string(), l: "הָלַךְ (H1980)".to_string(), m: "Vqi1cs".to_string(), d: "Main Apodosis Motion Kernel".to_string(), ipa: "/ʔeːlˈeːx/".to_string(), s: 1, lex: "Qal 1cs I Will Go".to_string() },
                        ReaderToken { w: "עַמֵּךְ".to_string(), l: "עַם (H5971)".to_string(), m: "Ncmsc/Sp2fs".to_string(), d: "Covenant People Subject".to_string(), ipa: "/ʕamːˈeːx/".to_string(), s: 1, lex: "Thy People".to_string() },
                        ReaderToken { w: "עַמִּי".to_string(), l: "עַם (H5971)".to_string(), m: "Ncmsc/Sp1cs".to_string(), d: "Covenant People Predicate".to_string(), ipa: "/ʕamːˈiː/".to_string(), s: 1, lex: "My People".to_string() },
                        ReaderToken { w: "וֵאלֹהַיִךְ".to_string(), l: "אֱלֹהִים (H430)".to_string(), m: "HC/Ncmpc/Sp2fs".to_string(), d: "Covenant God Subject".to_string(), ipa: "/weːloːhˈajix/".to_string(), s: 1, lex: "Thy God".to_string() },
                        ReaderToken { w: "אֱלֹהָי׃".to_string(), l: "אֱלֹהִים (H430)".to_string(), m: "Ncmpc/Sp1cs".to_string(), d: "Covenant God Predicate".to_string(), ipa: "/ʔɛloːhˈaːj/".to_string(), s: 1, lex: "My God".to_string() },
                    ],
                },
            ],
        },

        // ====================================================================
        // 9. BOOK OF JONAH 1:1–2 (Tiberian Masoretic Hebrew)
        // ====================================================================
        ReaderPericope {
            key: "jonah1".to_string(),
            title: "סֵפֶר יוֹנָה · פֶּרֶק א (1:1–2)".to_string(),
            dir: "rtl".to_string(),
            lang: "hbo".to_string(),
            unique_lemmas: 0,
            coverage_pct: 0.0,
            readiness_status: "ACCESSIBLE".to_string(),
            verses: vec![
                ReaderVerse {
                    ref_text: "JONAH 1:1–2 · CODEX LENINGRADENSIS (WLC)".to_string(),
                    tokens: vec![
                        ReaderToken { w: "וַיְהִי".to_string(), l: "הָיָה (H1961)".to_string(), m: "HC/Vqw3ms".to_string(), d: "Discourse Ingress Kernel".to_string(), ipa: "/wajəhˈiː/".to_string(), s: 1, lex: "And It Came to Pass".to_string() },
                        ReaderToken { w: "דְּבַר־יְהוָה".to_string(), l: "דָּבָר (H1697)".to_string(), m: "Ncmsc".to_string(), d: "Prophetic Word Subject".to_string(), ipa: "/dəvar jəhoːwaː/".to_string(), s: 1, lex: "Word of YHWH".to_string() },
                        ReaderToken { w: "אֶל־יוֹנָה".to_string(), l: "יוֹנָה (H3124)".to_string(), m: "R/Np".to_string(), d: "Recipient Adjunct".to_string(), ipa: "/ʔɛl joːnaː/".to_string(), s: 2, lex: "Unto Jonah".to_string() },
                        ReaderToken { w: "בֶּן־אֲמִתַּי".to_string(), l: "בֵּן (H1121)".to_string(), m: "Ncmsc/Np".to_string(), d: "Patronymic Apposition".to_string(), ipa: "/bɛn ʔămitːaj/".to_string(), s: 2, lex: "Son of Amittai".to_string() },
                        ReaderToken { w: "לֵאמֹר׃".to_string(), l: "אָמַר (H559)".to_string(), m: "R/Vqc".to_string(), d: "Quotative Frame".to_string(), ipa: "/leːmˈoːr/".to_string(), s: 3, lex: "Saying".to_string() },
                        ReaderToken { w: "קוּם".to_string(), l: "קוּם (H6965)".to_string(), m: "Vqv2ms".to_string(), d: "Imperative Kernel (Arise)".to_string(), ipa: "/quːm/".to_string(), s: 1, lex: "Arise!".to_string() },
                        ReaderToken { w: "לֵךְ".to_string(), l: "הָלַךְ (H1980)".to_string(), m: "Vqv2ms".to_string(), d: "Imperative Kernel (Go)".to_string(), ipa: "/leːx/".to_string(), s: 1, lex: "Go!".to_string() },
                        ReaderToken { w: "אֶל־נִינְוֵה".to_string(), l: "נִינְוֵה (H5210)".to_string(), m: "R/Np".to_string(), d: "Directional Goal".to_string(), ipa: "/ʔɛl niːnəweː/".to_string(), s: 2, lex: "To Nineveh".to_string() },
                        ReaderToken { w: "הָעִיר".to_string(), l: "עִיר (H5892)".to_string(), m: "HTd/Ncfsa".to_string(), d: "Appositional City".to_string(), ipa: "/haːʕˈiːr/".to_string(), s: 2, lex: "The City".to_string() },
                        ReaderToken { w: "הַגְּדוֹלָה".to_string(), l: "גָּדוֹל (H1419)".to_string(), m: "HTd/Aafsa".to_string(), d: "Attributive Great".to_string(), ipa: "/hagːəðoːlˈaː/".to_string(), s: 2, lex: "The Great".to_string() },
                    ],
                },
            ],
        },
    ];

    let telem_path = get_project_paths().ok().map(|(_, _, p)| p);
    for p in &mut pericopes {
        compute_pericope_readiness(p, telem_path.as_deref());
    }

    Ok(pericopes)
}

/// Retrieves all installed/available language packs with metadata for the active language studio.
#[tauri::command]
pub fn get_available_languages() -> Result<Vec<AvailableLanguage>, String> {
    let (grc_pack, hbo_pack, telem_path) = get_project_paths()?;
    let conn = SqliteConnection::open(&telem_path)?;

    let mut langs = Vec::new();

    // 1. Koine Greek
    let grc_conn = SqliteConnection::open(&grc_pack)?;
    let grc_count = grc_conn.query_single_i64("SELECT COUNT(*) FROM contrast_exercises;").unwrap_or(70);
    let grc_semver = conn
        .query_single_text("SELECT pack_semver FROM installed_packs WHERE lang_code = 'grc';")
        .unwrap_or_else(|_| "0.3.0".to_string());
    langs.push(AvailableLanguage {
        code: "grc".to_string(),
        name: "Koine Greek".to_string(),
        autonym: "Κοινή Ἑλληνική".to_string(),
        family: "Hellenic · Indo-European".to_string(),
        script_direction: "ltr".to_string(),
        concept_count: grc_count,
        pack_id: "klpack_grc_koine_v1".to_string(),
        semver: grc_semver,
    });

    // 2. Biblical Hebrew
    let hbo_conn = SqliteConnection::open(&hbo_pack)?;
    let hbo_count = hbo_conn.query_single_i64("SELECT COUNT(*) FROM contrast_exercises;").unwrap_or(70);
    let hbo_semver = conn
        .query_single_text("SELECT pack_semver FROM installed_packs WHERE lang_code = 'hbo';")
        .unwrap_or_else(|_| "0.3.0".to_string());
    langs.push(AvailableLanguage {
        code: "hbo".to_string(),
        name: "Biblical Hebrew".to_string(),
        autonym: "עִבְרִית מִקְרָאִית".to_string(),
        family: "Northwest Semitic · Afroasiatic".to_string(),
        script_direction: "rtl".to_string(),
        concept_count: hbo_count,
        pack_id: "klpack_hbo_masoretic_v1".to_string(),
        semver: hbo_semver,
    });

    Ok(langs)
}

/// Retrieves deck statistics, independent skill stability, and recent review logs for the active language.
#[tauri::command]
pub fn get_fsrs_deck_summary(lang: String) -> Result<FsrsDeckSummary, String> {
    let (_, _, telem_path) = get_project_paths()?;
    let conn = SqliteConnection::open(&telem_path)?;
    let now_ms = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as i64;

    let total = conn.query_single_i64(&format!(
        "SELECT COUNT(*) FROM fsrs_cards WHERE lang_code = '{lang}';"
    )).unwrap_or(0) as usize;

    let due = conn.query_single_i64(&format!(
        "SELECT COUNT(*) FROM fsrs_cards WHERE lang_code = '{lang}' AND due_timestamp_ms <= {now_ms} AND (buried_until_ms IS NULL OR buried_until_ms <= {now_ms});"
    )).unwrap_or(0) as usize;

    let buried = conn.query_single_i64(&format!(
        "SELECT COUNT(*) FROM fsrs_cards WHERE lang_code = '{lang}' AND buried_until_ms > {now_ms};"
    )).unwrap_or(0) as usize;

    let learning = conn.query_single_i64(&format!(
        "SELECT COUNT(*) FROM fsrs_cards WHERE lang_code = '{lang}' AND state = 1;"
    )).unwrap_or(0) as usize;

    let mastered = conn.query_single_i64(&format!(
        "SELECT COUNT(*) FROM fsrs_cards WHERE lang_code = '{lang}' AND stability >= 21.0;"
    )).unwrap_or(0) as usize;

    let modalities = ["LISTEN", "READ", "SPEAK", "WRITE"];
    let mut skills = Vec::new();
    for m in modalities {
        let count = conn.query_single_i64(&format!(
            "SELECT COUNT(*) FROM fsrs_cards WHERE lang_code = '{lang}' AND modality = '{m}';"
        )).unwrap_or(0) as usize;
        let due_m = conn.query_single_i64(&format!(
            "SELECT COUNT(*) FROM fsrs_cards WHERE lang_code = '{lang}' AND modality = '{m}' AND due_timestamp_ms <= {now_ms} AND (buried_until_ms IS NULL OR buried_until_ms <= {now_ms});"
        )).unwrap_or(0) as usize;
        let avg_stab_str = conn.query_single_text(&format!(
            "SELECT COALESCE(AVG(stability), 0.0) FROM fsrs_cards WHERE lang_code = '{lang}' AND modality = '{m}';"
        )).unwrap_or_else(|_| "0.0".to_string());
        let avg_stab: f64 = avg_stab_str.parse().unwrap_or(0.0);

        skills.push(FsrsSkillStability {
            modality: m.to_string(),
            card_count: count,
            avg_stability_days: avg_stab,
            due_count: due_m,
        });
    }

    // Query last 10 review logs
    let mut recent_reviews = Vec::new();
    let log_count = conn.query_single_i64(&format!(
        "SELECT COUNT(*) FROM review_logs WHERE card_id LIKE '{lang}:%';"
    )).unwrap_or(0);

    for idx in 0..log_count.min(10) {
        let event_uuid = conn.query_single_text(&format!(
            "SELECT event_uuid FROM review_logs WHERE card_id LIKE '{lang}:%' ORDER BY log_id DESC LIMIT 1 OFFSET {idx};"
        )).unwrap_or_default();
        if event_uuid.is_empty() {
            continue;
        }
        let card_id = conn.query_single_text(&format!(
            "SELECT card_id FROM review_logs WHERE event_uuid = '{event_uuid}';"
        )).unwrap_or_default();
        let rating = conn.query_single_i64(&format!(
            "SELECT COALESCE(rating, 0) FROM review_logs WHERE event_uuid = '{event_uuid}';"
        )).unwrap_or(0);
        let stab_after_str = conn.query_single_text(&format!(
            "SELECT stability_after FROM review_logs WHERE event_uuid = '{event_uuid}';"
        )).unwrap_or_else(|_| "0.0".to_string());
        let latency = conn.query_single_i64(&format!(
            "SELECT COALESCE(response_latency_ms, 0) FROM review_logs WHERE event_uuid = '{event_uuid}';"
        )).unwrap_or(0);
        let ts = conn.query_single_i64(&format!(
            "SELECT review_timestamp_ms FROM review_logs WHERE event_uuid = '{event_uuid}';"
        )).unwrap_or(0);

        let modality = card_id.split(':').nth(2).unwrap_or("LISTEN").to_string();

        recent_reviews.push(RecentReviewItem {
            event_uuid,
            card_id,
            modality,
            rating,
            stability_after: stab_after_str.parse().unwrap_or(0.0),
            latency_ms: latency,
            timestamp_ms: ts,
        });
    }

    Ok(FsrsDeckSummary {
        lang_code: lang,
        total_cards: total,
        due_cards: due,
        learning_cards: learning,
        mastered_cards: mastered,
        buried_cards: buried,
        skills,
        recent_reviews,
    })
}

fn ensure_user_profiles_table(conn: &SqliteConnection) -> Result<(), String> {
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS user_profiles (
            user_id TEXT PRIMARY KEY,
            display_name TEXT NOT NULL,
            avatar TEXT NOT NULL DEFAULT '🏛️',
            created_at_ms INTEGER NOT NULL,
            last_active_ms INTEGER NOT NULL,
            current_lang TEXT NOT NULL DEFAULT 'grc',
            streak_days INTEGER NOT NULL DEFAULT 1,
            total_reviews INTEGER NOT NULL DEFAULT 0
        );"
    )?;

    let count = conn.query_single_i64("SELECT COUNT(*) FROM user_profiles;").unwrap_or(0);
    if count == 0 {
        let now_ms = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as i64;
        conn.execute_batch(&format!(
            "INSERT INTO user_profiles (user_id, display_name, avatar, created_at_ms, last_active_ms, current_lang, streak_days, total_reviews)
             VALUES ('user_josh', 'Josh Shay', '🏛️', {now_ms}, {now_ms}, 'grc', 1, 42);"
        ))?;
    }
    Ok(())
}

/// Retrieves all registered learner profiles.
#[tauri::command]
pub fn get_user_profiles() -> Result<Vec<UserProfile>, String> {
    let (_, _, telem_path) = get_project_paths()?;
    let conn = SqliteConnection::open(&telem_path)?;
    ensure_user_profiles_table(&conn)?;

    let rows = conn.query_rows(
        "SELECT user_id, display_name, avatar, created_at_ms, last_active_ms, current_lang, streak_days, total_reviews FROM user_profiles ORDER BY last_active_ms DESC;"
    )?;

    let mut profiles = Vec::new();
    for r in rows {
        if r.len() >= 8 {
            profiles.push(UserProfile {
                user_id: r[0].clone(),
                display_name: r[1].clone(),
                avatar: if r[2].is_empty() { "🏛️".to_string() } else { r[2].clone() },
                created_at_ms: r[3].parse().unwrap_or(0),
                last_active_ms: r[4].parse().unwrap_or(0),
                current_lang: if r[5].is_empty() { "grc".to_string() } else { r[5].clone() },
                streak_days: r[6].parse().unwrap_or(1),
                total_reviews: r[7].parse().unwrap_or(0),
            });
        }
    }
    Ok(profiles)
}

/// Creates a new learner profile and persists it to user_telemetry.db.
#[tauri::command]
pub fn create_user_profile(display_name: String, avatar: String) -> Result<UserProfile, String> {
    let (_, _, telem_path) = get_project_paths()?;
    let conn = SqliteConnection::open(&telem_path)?;
    ensure_user_profiles_table(&conn)?;

    let now_ms = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as i64;
    let clean_name = display_name.trim().replace('\'', "''");
    let name = if clean_name.is_empty() { "New Scholar".to_string() } else { clean_name };
    let clean_avatar = if avatar.trim().is_empty() { "🏛️".to_string() } else { avatar.trim().to_string() };
    let user_id = format!("user_{}", &now_ms.to_string()[6..]);

    conn.execute_batch(&format!(
        "INSERT INTO user_profiles (user_id, display_name, avatar, created_at_ms, last_active_ms, current_lang, streak_days, total_reviews)
         VALUES ('{user_id}', '{name}', '{clean_avatar}', {now_ms}, {now_ms}, 'grc', 1, 0);"
    ))?;

    Ok(UserProfile {
        user_id,
        display_name: name,
        avatar: clean_avatar,
        created_at_ms: now_ms,
        last_active_ms: now_ms,
        current_lang: "grc".to_string(),
        streak_days: 1,
        total_reviews: 0,
    })
}

/// Marks a profile as active and updates its last_active_ms timestamp.
#[tauri::command]
pub fn select_user_profile(user_id: String) -> Result<UserProfile, String> {
    let (_, _, telem_path) = get_project_paths()?;
    let conn = SqliteConnection::open(&telem_path)?;
    ensure_user_profiles_table(&conn)?;

    let now_ms = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as i64;
    let safe_id = user_id.replace('\'', "''");
    let _ = conn.execute_batch(&format!(
        "UPDATE user_profiles SET last_active_ms = {now_ms} WHERE user_id = '{safe_id}';"
    ));

    let rows = conn.query_rows(&format!(
        "SELECT user_id, display_name, avatar, created_at_ms, last_active_ms, current_lang, streak_days, total_reviews FROM user_profiles WHERE user_id = '{safe_id}' LIMIT 1;"
    ))?;

    if let Some(r) = rows.into_iter().next() {
        if r.len() >= 8 {
            return Ok(UserProfile {
                user_id: r[0].clone(),
                display_name: r[1].clone(),
                avatar: if r[2].is_empty() { "🏛️".to_string() } else { r[2].clone() },
                created_at_ms: r[3].parse().unwrap_or(0),
                last_active_ms: r[4].parse().unwrap_or(0),
                current_lang: if r[5].is_empty() { "grc".to_string() } else { r[5].clone() },
                streak_days: r[6].parse().unwrap_or(1),
                total_reviews: r[7].parse().unwrap_or(0),
            });
        }
    }

    Err(format!("Profile {user_id} not found"))
}

/// Updates the current active language for a profile.
#[tauri::command]
pub fn set_user_language(user_id: String, lang: String) -> Result<(), String> {
    let (_, _, telem_path) = get_project_paths()?;
    let conn = SqliteConnection::open(&telem_path)?;
    ensure_user_profiles_table(&conn)?;

    let safe_id = user_id.replace('\'', "''");
    let safe_lang = if lang == "hbo" { "hbo" } else { "grc" };
    conn.execute_batch(&format!(
        "UPDATE user_profiles SET current_lang = '{safe_lang}' WHERE user_id = '{safe_id}';"
    ))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ipc_get_catalog_grc_and_hbo() {
        let grc = get_catalog("grc".to_string()).expect("Should load GRC catalog");
        assert_eq!(grc.len(), 70, "GRC catalog must contain exactly 70 concepts");
        assert_eq!(grc[0].panels.len(), 4, "Every concept must have 4 contrast panels");

        let hbo = get_catalog("hbo".to_string()).expect("Should load HBO catalog");
        assert_eq!(hbo.len(), 70, "HBO catalog must contain exactly 70 concepts");
        assert_eq!(hbo[0].panels.len(), 4, "Every concept must have 4 contrast panels");
    }

    #[test]
    fn test_ipc_check_integrity() {
        let summary = check_integrity().expect("Integrity check must succeed");
        assert!(summary.grc_integrity_ok);
        assert!(summary.hbo_integrity_ok);
        assert_eq!(summary.grc_concepts, 70);
        assert_eq!(summary.hbo_concepts, 70);
        assert_eq!(summary.fsrs_version, crate::PINNED_FSRS_VERSION);
    }

    #[test]
    fn test_ipc_g2p_transcribe() {
        let grc_res = g2p_transcribe("ὁ ἄγγελος".to_string(), "grc".to_string(), "RESTORED".to_string()).unwrap();
        assert_eq!(grc_res.full_ipa, "/ho ˈaŋɡɛlos/");

        let hbo_res = g2p_transcribe("בָּרָא אֱלֹהִים".to_string(), "hbo".to_string(), "TIBERIAN".to_string()).unwrap();
        assert!(hbo_res.full_ipa.contains("baːraː"));
    }

    #[test]
    fn test_ipc_evaluate_speech() {
        let mut samples = Vec::new();
        // Leading ambient silence
        for _ in 0..1280 { samples.push(0.001); }
        // Clean 180 Hz tone
        for i in 0..16000 {
            let t = i as f32 / 16000.0;
            samples.push((2.0 * std::f32::consts::PI * 180.0 * t).sin() * 0.4);
        }
        // Trailing ambient silence
        for _ in 0..1280 { samples.push(0.001); }

        let payload = SpeechEvaluationPayload {
            samples,
            sample_rate: 16000,
            target_sentence_id: "s_test_1".to_string(),
            target_text: "ὁ ἄγγελος".to_string(),
            lang_code: "grc".to_string(),
            target_phones: vec!["h".to_string(), "o".to_string(), "a".to_string(), "ŋɡ".to_string(), "ɛ".to_string(), "l".to_string(), "o".to_string(), "s".to_string()],
            contrast_indices: vec![0],
            foils: vec![],
        };

        let rep = evaluate_speech(payload).unwrap();
        assert!(rep.metrics.snr_db > 20.0);
        assert_eq!(rep.learner_waveform_bars.len(), 32);
        assert!(rep.compute_latency_ms < 300.0, "Debug mode latency was {}", rep.compute_latency_ms);
    }

    #[test]
    fn test_ipc_record_review() {
        let res = record_review("grc".to_string(), "GRC_01_NOM_ACC_AGENT_PATIENT".to_string(), "LISTEN".to_string(), 3, 450).unwrap();
        assert_eq!(res.rating, 3);
        assert!(res.stability_days > 2.0);
        assert!(res.sibling_buried_until_ms > 0);
    }

    #[test]
    fn test_ipc_get_reader_pericopes() {
        let pericopes = get_reader_pericopes().unwrap();
        assert_eq!(pericopes.len(), 9, "Should load all 9 canonical pericopes");
        assert_eq!(pericopes[0].key, "mark1");
        assert_eq!(pericopes[1].key, "john1");
        assert_eq!(pericopes[2].key, "matthew6");
        assert_eq!(pericopes[3].key, "gen1");
        assert_eq!(pericopes[4].key, "psalm23");
        assert_eq!(pericopes[5].key, "deut6");
        assert_eq!(pericopes[6].key, "exod20");
        assert_eq!(pericopes[7].key, "ruth1");
        assert_eq!(pericopes[8].key, "jonah1");

        // Verify Vocabulary Readiness metrics
        assert!(pericopes[0].unique_lemmas > 0);
        assert!(pericopes[0].coverage_pct >= 60.0);
        assert!(!pericopes[0].readiness_status.is_empty());
    }

    #[test]
    fn test_ipc_get_available_languages() {
        let langs = get_available_languages().unwrap();
        assert_eq!(langs.len(), 2);
        assert_eq!(langs[0].code, "grc");
        assert_eq!(langs[0].name, "Koine Greek");
        assert_eq!(langs[1].code, "hbo");
        assert_eq!(langs[1].name, "Biblical Hebrew");
    }

    #[test]
    fn test_ipc_get_fsrs_deck_summary() {
        let summary_grc = get_fsrs_deck_summary("grc".to_string()).unwrap();
        assert_eq!(summary_grc.lang_code, "grc");
        assert_eq!(summary_grc.total_cards, 280); // 70 * 4 modalities
        assert_eq!(summary_grc.skills.len(), 4);

        let summary_hbo = get_fsrs_deck_summary("hbo".to_string()).unwrap();
        assert_eq!(summary_hbo.lang_code, "hbo");
        assert_eq!(summary_hbo.total_cards, 280);
        assert_eq!(summary_hbo.skills.len(), 4);
    }

    #[test]
    fn test_ipc_user_profiles_lifecycle() {
        let profiles = get_user_profiles().expect("Should load profiles");
        assert!(!profiles.is_empty(), "Must have at least default Josh Shay profile");
        assert!(profiles.iter().any(|p| p.display_name == "Josh Shay"), "Default profile Josh Shay must exist");

        // Create a new learner profile
        let new_user = create_user_profile("Scholar Alpha".to_string(), "🦉".to_string())
            .expect("Should create profile");
        assert_eq!(new_user.display_name, "Scholar Alpha");
        assert_eq!(new_user.avatar, "🦉");

        // Select the new profile
        let selected = select_user_profile(new_user.user_id.clone()).expect("Should select profile");
        assert_eq!(selected.user_id, new_user.user_id);

        // Update language
        set_user_language(new_user.user_id.clone(), "hbo".to_string()).expect("Should set language");
        let updated = select_user_profile(new_user.user_id).expect("Should reload profile");
        assert_eq!(updated.current_lang, "hbo");
    }
}
