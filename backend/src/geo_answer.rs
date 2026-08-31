//! Free-text answer matching for the geography sections (flags & capitals).
//!
//! The player types a country name or a capital; we decide whether it matches
//! one of the accepted answers (`name_accepted` / `capital_accepted`). The
//! comparison is accent-, case-, punctuation- and article-insensitive, and
//! tolerates a single typo on long enough answers. No external dependency:
//! accent folding covers the French/common repertoire by hand, and the
//! Levenshtein distance is computed with a two-row buffer.

/// Minimum number of *significant* chars (article and spaces excluded) in the
/// expected answer before a Levenshtein distance of 1 is tolerated. Below that,
/// close pairs (Mali/Bali, Iran/Irak, Le Cap/le cas) would be interchangeable.
const FUZZY_MIN_CHARS: usize = 5;

/// Leading tokens treated as optional articles ("Le Caire" ≈ "Caire").
const ARTICLES: [&str; 5] = ["le", "la", "les", "l", "the"];

/// Lowercase, fold French/common accents to ASCII, replace punctuation and
/// connectors (apostrophes — straight and typographic —, hyphens, dots, …)
/// with spaces, collapse whitespace runs, trim.
pub fn normalize(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            'é' | 'è' | 'ê' | 'ë' | 'É' | 'È' | 'Ê' | 'Ë' => out.push('e'),
            'à' | 'â' | 'ä' | 'á' | 'ã' | 'À' | 'Â' | 'Ä' | 'Á' | 'Ã' => out.push('a'),
            'î' | 'ï' | 'í' | 'Î' | 'Ï' | 'Í' => out.push('i'),
            'ô' | 'ö' | 'ó' | 'õ' | 'Ô' | 'Ö' | 'Ó' | 'Õ' => out.push('o'),
            'ù' | 'û' | 'ü' | 'ú' | 'Ù' | 'Û' | 'Ü' | 'Ú' => out.push('u'),
            'ç' | 'Ç' => out.push('c'),
            'ñ' | 'Ñ' => out.push('n'),
            'ÿ' | 'Ÿ' => out.push('y'),
            'œ' | 'Œ' => out.push_str("oe"),
            'æ' | 'Æ' => out.push_str("ae"),
            c if c.is_alphanumeric() => out.extend(c.to_lowercase()),
            // Everything else (whitespace, apostrophes, hyphens, dots, …)
            // acts as a word separator; runs collapse to a single space.
            _ => push_separator(&mut out),
        }
    }
    if out.ends_with(' ') {
        out.pop();
    }
    out
}

/// Strict match: normalization and an optional leading article only, no typo
/// tolerance. Used for the first pass, and to tell a slip from a real mistake:
/// a near-miss that is *exactly* another country's answer must not pass, since
/// real answers sit one edit apart (Irlande/Islande, Kingston/Kingstown).
pub fn matches_exact(given: &str, accepted: &[String]) -> bool {
    compare(given, accepted, false)
}

/// Same, plus a single typo once the expected answer is long enough. Callers
/// must still guard against a fuzzy hit that is in fact another country's real
/// answer — see `routes::geo`.
pub fn matches_typed(given: &str, accepted: &[String]) -> bool {
    compare(given, accepted, true)
}

fn compare(given: &str, accepted: &[String], fuzzy: bool) -> bool {
    let given = expand_abbreviations(&normalize(given));
    if given.is_empty() {
        return false;
    }
    accepted.iter().any(|a| {
        let expected = expand_abbreviations(&normalize(a));
        !expected.is_empty()
            && (close_enough(&given, &expected, fuzzy)
                || close_enough(strip_article(&given), strip_article(&expected), fuzzy))
    })
}

/// "St-Georges" ≡ "Saint-Georges": the abbreviation is common enough in typed
/// answers that refusing it reads as a bug. Applied to both sides, on already
/// normalized text (so tokens are space-separated).
fn expand_abbreviations(s: &str) -> String {
    if !s.contains("st") {
        return s.to_string();
    }
    s.split(' ')
        .map(|t| match t {
            "st" => "saint",
            "ste" => "sainte",
            other => other,
        })
        .collect::<Vec<_>>()
        .join(" ")
}

fn push_separator(out: &mut String) {
    if !out.is_empty() && !out.ends_with(' ') {
        out.push(' ');
    }
}

/// Exact match, or — when `fuzzy` — one typo if the expected answer is long
/// enough. Both sides must already be normalized. The length gate counts only
/// significant chars (no article, no spaces), so "Le Cap" stays strict.
fn close_enough(given: &str, expected: &str, fuzzy: bool) -> bool {
    if given == expected {
        return true;
    }
    fuzzy && significant_len(expected) >= FUZZY_MIN_CHARS && levenshtein(given, expected) <= 1
}

fn significant_len(expected: &str) -> usize {
    strip_article(expected)
        .chars()
        .filter(|c| !c.is_whitespace())
        .count()
}

/// Drop a leading article token, unless it is the whole string.
fn strip_article(s: &str) -> &str {
    match s.split_once(' ') {
        Some((first, rest)) if ARTICLES.contains(&first) => rest,
        _ => s,
    }
}

fn levenshtein(a: &str, b: &str) -> usize {
    let a: Vec<char> = a.chars().collect();
    let b: Vec<char> = b.chars().collect();
    let mut prev: Vec<usize> = (0..=b.len()).collect();
    let mut curr: Vec<usize> = vec![0; b.len() + 1];
    for (i, ca) in a.iter().enumerate() {
        curr[0] = i + 1;
        for (j, cb) in b.iter().enumerate() {
            let cost = usize::from(ca != cb);
            curr[j + 1] = (prev[j] + cost).min(prev[j + 1] + 1).min(curr[j] + 1);
        }
        std::mem::swap(&mut prev, &mut curr);
    }
    prev[b.len()]
}

#[cfg(test)]
mod tests {
    use super::*;

    fn acc(items: &[&str]) -> Vec<String> {
        items.iter().map(|s| s.to_string()).collect()
    }

    // ---- normalize ----

    #[test]
    fn normalize_folds_case_accents_and_connectors() {
        assert_eq!(normalize("Côte d'Ivoire"), "cote d ivoire");
        assert_eq!(normalize("cote d ivoire"), "cote d ivoire");
        assert_eq!(
            normalize("Saint-Christophe-et-Niévès"),
            "saint christophe et nieves"
        );
        assert_eq!(
            normalize("éèêë àâä îï ôö ùûü ç ñ ÿ"),
            "eeee aaa ii oo uuu c n y"
        );
        assert_eq!(
            normalize("ÉÈÊË ÀÂÄ ÎÏ ÔÖ ÙÛÜ Ç Ñ Ÿ"),
            "eeee aaa ii oo uuu c n y"
        );
        assert_eq!(normalize("œŒæÆ"), "oeoeaeae");
        assert_eq!(normalize("São Tomé"), "sao tome");
        assert_eq!(normalize("Asunción"), "asuncion");
        assert_eq!(normalize("Brasília"), "brasilia");
    }

    #[test]
    fn normalize_handles_typographic_apostrophes_and_dots() {
        // U+2019 (’) and U+2018 (‘) behave like a straight apostrophe.
        assert_eq!(normalize("Côte d’Ivoire"), "cote d ivoire");
        assert_eq!(normalize("N‘Djamena"), "n djamena");
        assert_eq!(normalize("Washington D.C."), "washington d c");
    }

    #[test]
    fn normalize_collapses_spaces_and_trims() {
        assert_eq!(normalize("  le    caire  "), "le caire");
        assert_eq!(normalize("\tBuenos \n Aires "), "buenos aires");
    }

    #[test]
    fn normalize_degenerate_inputs() {
        assert_eq!(normalize(""), "");
        assert_eq!(normalize("   "), "");
        assert_eq!(normalize(" '-.’ "), "");
    }

    // ---- levenshtein ----

    #[test]
    fn levenshtein_basics() {
        assert_eq!(levenshtein("", ""), 0);
        assert_eq!(levenshtein("", "abc"), 3);
        assert_eq!(levenshtein("abc", ""), 3);
        assert_eq!(levenshtein("kitten", "sitting"), 3);
        assert_eq!(levenshtein("paris", "paris"), 0);
        assert_eq!(levenshtein("chine", "chili"), 2);
    }

    // ---- matches: exact, accents, case, spacing ----

    #[test]
    fn matches_exact_modulo_normalization() {
        assert!(matches_typed("cote d ivoire", &acc(&["Côte d'Ivoire"])));
        assert!(matches_typed("Côte d’Ivoire", &acc(&["Côte d'Ivoire"])));
        assert!(matches_typed("PARIS", &acc(&["Paris"])));
        assert!(matches_typed("  buenos    aires ", &acc(&["Buenos Aires"])));
        assert!(matches_typed(
            "saint christophe et nieves",
            &acc(&["Saint-Christophe-et-Niévès"])
        ));
        assert!(matches_typed("Sao Tome", &acc(&["São Tomé"])));
    }

    #[test]
    fn matches_scans_all_accepted_answers() {
        let accepted = acc(&["Pretoria", "Le Cap", "Cape Town", "Bloemfontein"]);
        assert!(matches_typed("cape town", &accepted));
        assert!(matches_typed("bloemfontein", &accepted));
        assert!(!matches_typed("johannesburg", &accepted));
    }

    // ---- matches: typo tolerance ----

    #[test]
    fn one_typo_accepted_on_long_answers() {
        // missing letter, substitution, extra letter
        assert!(matches_typed("Ouagadugou", &acc(&["Ouagadougou"])));
        assert!(matches_typed("Antananarive", &acc(&["Antananarivo"])));
        assert!(matches_typed("Yamoussoukroo", &acc(&["Yamoussoukro"])));
    }

    #[test]
    fn two_typos_refused() {
        assert!(!matches_typed("Ouagadugu", &acc(&["Ouagadougou"])));
        assert!(!matches_typed("Antananarevi", &acc(&["Antananarivo"])));
        assert!(!matches_typed("Ymoussoukroo", &acc(&["Yamoussoukro"])));
    }

    #[test]
    fn short_trap_pairs_never_fuzzy_match() {
        assert!(!matches_typed("bali", &acc(&["Mali"])));
        assert!(!matches_typed("mali", &acc(&["Bali"])));
        assert!(!matches_typed("irak", &acc(&["Iran"])));
        assert!(!matches_typed("iran", &acc(&["Irak"])));
        assert!(!matches_typed("chili", &acc(&["Chine"])));
        assert!(!matches_typed("chine", &acc(&["Chili"])));
    }

    #[test]
    fn short_answers_still_match_exactly() {
        assert!(matches_typed("mali", &acc(&["Mali"])));
        assert!(matches_typed("Iran", &acc(&["Iran"])));
        assert!(matches_typed("chine", &acc(&["Chine"])));
    }

    #[test]
    fn fuzzy_threshold_is_on_the_expected_side() {
        // given is 5 chars but expected "Mali" is 4: no tolerance.
        assert!(!matches_typed("malia", &acc(&["Mali"])));
    }

    // ---- matches: articles ----

    #[test]
    fn leading_article_is_optional_on_both_sides() {
        assert!(matches_typed("caire", &acc(&["Le Caire"])));
        assert!(matches_typed("le caire", &acc(&["Caire"])));
        assert!(matches_typed("valette", &acc(&["La Valette"])));
        assert!(matches_typed("la valette", &acc(&["Valette"])));
        assert!(matches_typed("bahamas", &acc(&["The Bahamas"])));
        assert!(matches_typed("aquila", &acc(&["L'Aquila"])));
    }

    #[test]
    fn article_and_typo_combine() {
        assert!(matches_typed("la valete", &acc(&["La Valette"])));
        assert!(matches_typed("valete", &acc(&["La Valette"])));
    }

    #[test]
    fn article_stripping_does_not_create_false_positives() {
        // A lone article never matches.
        assert!(!matches_typed("la", &acc(&["La Valette"])));
        assert!(!matches_typed("le", &acc(&["Le Caire"])));
        // Stripping must not unlock fuzziness on short remainders.
        assert!(!matches_typed("le pas", &acc(&["La Paz"])));
        // A word merely *starting* with an article is not stripped.
        assert!(!matches_typed("os", &acc(&["Laos"])));
        assert!(matches_typed("laos", &acc(&["Laos"])));
        // Article as part of the name still matches directly.
        assert!(matches_typed("la havane", &acc(&["La Havane"])));
        assert!(matches_typed("havane", &acc(&["La Havane"])));
    }

    // ---- matches: rejections ----

    #[test]
    fn empty_or_blank_input_refused() {
        assert!(!matches_typed("", &acc(&["Paris"])));
        assert!(!matches_typed("   ", &acc(&["Paris"])));
        assert!(!matches_typed(" '- ", &acc(&["Paris"])));
        assert!(!matches_typed("paris", &[]));
        assert!(!matches_typed("", &acc(&[""])));
    }

    #[test]
    fn plain_wrong_answers_refused() {
        assert!(!matches_typed("lyon", &acc(&["Paris"])));
        assert!(!matches_typed("australie", &acc(&["Autriche"])));
    }

    // ---- matches_exact: collision guard (used by routes::geo) ----

    #[test]
    fn exact_mode_refuses_one_edit_neighbours() {
        // These are the pairs the guard must separate: each is a real country
        // name, so typing one for the other is a mistake, not a slip.
        assert!(!matches_exact("Islande", &acc(&["Irlande"])));
        assert!(!matches_exact("Irlande", &acc(&["Islande"])));
        assert!(!matches_exact("Zambie", &acc(&["Gambie"])));
        assert!(!matches_exact("Gambie", &acc(&["Zambie"])));
        assert!(!matches_exact("Niger", &acc(&["Nigeria"])));
    }

    #[test]
    fn exact_mode_still_accepts_the_right_answer() {
        assert!(matches_exact("Irlande", &acc(&["Irlande"])));
        assert!(matches_exact("cote d ivoire", &acc(&["Côte d'Ivoire"])));
        assert!(matches_exact("caire", &acc(&["Le Caire"])));
    }

    #[test]
    fn typed_mode_keeps_its_typo_tolerance() {
        assert!(matches_typed("Irlnde", &acc(&["Irlande"])));
    }

    // ---- collision guard building block (used by routes::geo) ----

    #[test]
    fn real_seed_collisions_are_not_exact_matches() {
        // These pairs are one edit apart AND both are real answers in the seed;
        // routes::geo refuses a fuzzy hit that is exactly another country's
        // answer, and this is the predicate it relies on.
        assert!(!matches_exact("Kingston", &acc(&["Kingstown"])));
        assert!(!matches_exact("Kingstown", &acc(&["Kingston"])));
        assert!(!matches_exact("Panama", &acc(&["Manama"])));
        assert!(!matches_exact("Manama", &acc(&["Panama"])));
    }

    // ---- article + length gate ----

    #[test]
    fn short_answer_behind_an_article_stays_strict() {
        // "Le Cap" is 6 normalized chars but only 3 significant ones.
        assert!(!matches_typed("le cas", &acc(&["Le Cap"])));
        assert!(!matches_typed("la pas", &acc(&["La Paz"])));
        assert!(matches_typed("le cap", &acc(&["Le Cap"])));
        assert!(matches_typed("cap", &acc(&["Le Cap"])));
        assert!(matches_typed("la paz", &acc(&["La Paz"])));
    }

    #[test]
    fn spaces_do_not_inflate_the_fuzzy_gate() {
        // "New York" → 7 significant chars, tolerance applies as expected.
        assert!(matches_typed("new yorc", &acc(&["New York"])));
    }

    // ---- Saint / St ----

    #[test]
    fn saint_abbreviation_is_accepted() {
        assert!(matches_typed("St-Georges", &acc(&["Saint-Georges"])));
        assert!(matches_typed("Ste-Lucie", &acc(&["Sainte-Lucie"])));
        assert!(matches_typed(
            "St Christophe et Nieves",
            &acc(&["Saint-Christophe-et-Niévès"])
        ));
        assert!(matches_typed("Saint-Marin", &acc(&["St-Marin"])));
        // and it does not turn unrelated names into matches
        assert!(!matches_typed("st", &acc(&["Saint-Marin"])));
        assert!(!matches_typed("Estonie", &acc(&["Saint-Marin"])));
    }
}
