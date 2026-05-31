use super::truncate;

/// Prompt for atomic active-recall flashcards. Encodes the study guide's rules
/// (one fact per card, chunking, focus on figures/mechanisms/examples).
pub fn flashcards_prompt(source: &str, count: i32, block_title: Option<&str>) -> String {
    let scope = match block_title {
        Some(t) => format!(" Concentre-toi sur le thème : « {t} »."),
        None => String::new(),
    };
    format!(
        "Tu es un assistant pédagogique expert en sciences cognitives de l'apprentissage.\n\
         À partir du COURS ci-dessous, génère exactement {count} flashcards de qualité pour l'active recall.\n\n\
         RÈGLES STRICTES :\n\
         - Une carte = UN seul fait atomique. Regroupe les paires liées en un seul chunk (ex. « 96 % / 4 % » = une carte, pas deux).\n\
         - Le recto (front) = une question courte et précise. Le verso (back) = la réponse exacte (avec le chiffre si pertinent).\n\
         - Privilégie : chiffres-clés, définitions, mécanismes (« pourquoi / comment »), exemples concrets marquants.\n\
         - Interdits : cartes triviales, doublons, questions vagues.\n\
         - 'block_hint' (optionnel) = le thème de la carte, pour la classer.\n\
         - Langue : la même que le COURS.\n\
         - Réponds UNIQUEMENT en JSON conforme au schéma, sans texte autour.{scope}\n\n\
         COURS :\n\"\"\"\n{src}\n\"\"\"",
        count = count,
        scope = scope,
        src = truncate(source, 16000),
    )
}

/// Prompt for a mixed-type mock exam (QCM, vrai/faux, court, ouvert), interleaving blocks.
pub fn exam_prompt(source: &str, count: i32, block_title: Option<&str>) -> String {
    let scope = match block_title {
        Some(t) => format!(" Concentre-toi sur le thème : « {t} ».", ),
        None => String::new(),
    };
    format!(
        "Tu es un examinateur. À partir du COURS ci-dessous, génère un examen blanc de {count} questions.\n\n\
         RÈGLES STRICTES :\n\
         - MÉLANGE les types : 'mcq' (4 options, une seule correcte), 'true_false', 'short_answer' (réponse courte), 'open_ended' (réflexion).\n\
         - Pour 'mcq' : fournis 'options' (clés a,b,c,d) et 'answer_key' = la clé correcte.\n\
         - Pour 'true_false' : 'answer_key' = \"true\" ou \"false\".\n\
         - Pour 'short_answer' et 'open_ended' : PAS d'answer_key ; mets dans 'explanation' les points-clés attendus (barème).\n\
         - 'points' : 1 pour mcq/true_false, 2-3 pour short_answer, 3-5 pour open_ended.\n\
         - Varie les thèmes d'une question à l'autre (entrelacement). 'block_hint' = thème.\n\
         - Langue : celle du COURS. Réponds UNIQUEMENT en JSON conforme au schéma.{scope}\n\n\
         COURS :\n\"\"\"\n{src}\n\"\"\"",
        count = count,
        scope = scope,
        src = truncate(source, 16000),
    )
}

/// Prompt for a Feynman menu: concepts worth explaining "comme à un enfant".
pub fn feynman_prompt(source: &str, count: i32, block_title: Option<&str>) -> String {
    let scope = match block_title {
        Some(t) => format!(" Concentre-toi sur le thème : « {t} »."),
        None => String::new(),
    };
    format!(
        "Tu es un pédagogue. À partir du COURS ci-dessous, propose {count} concepts à savoir EXPLIQUER \
         à voix haute « comme à un enfant » (technique Feynman).\n\n\
         RÈGLES :\n\
         - Cible les MÉCANISMES et les « pourquoi / comment », pas les simples chiffres.\n\
         - 'title' = une question ouverte (ex. « Pourquoi la monoculture est-elle risquée ? »).\n\
         - 'hint' = les 2-3 points-clés qu'une bonne explication doit contenir.\n\
         - Langue : celle du COURS. Réponds UNIQUEMENT en JSON conforme au schéma.{scope}\n\n\
         COURS :\n\"\"\"\n{src}\n\"\"\"",
        count = count,
        scope = scope,
        src = truncate(source, 16000),
    )
}

/// Prompt for a hierarchical concept map.
pub fn concept_map_prompt(source: &str, block_title: Option<&str>) -> String {
    let scope = match block_title {
        Some(t) => format!(" Concentre-toi sur le thème : « {t} »."),
        None => String::new(),
    };
    format!(
        "Tu es un pédagogue. À partir du COURS ci-dessous, construis une CARTE CONCEPTUELLE hiérarchique.\n\n\
         RÈGLES :\n\
         - 'title' : titre court de la carte.\n\
         - 'nodes' : 8 à 15 concepts. Chaque nœud a un 'id' court unique (ex. \"n1\"), un 'label' bref, \
           et 'parent' = l'id du concept parent (omis ou vide pour LA racine unique).\n\
         - 'edges' : liens transversaux entre concepts ('from'/'to' = ids de nœuds, 'label' = nature du lien, ex. « entraîne »).\n\
         - Une seule racine. Langue : celle du COURS. Réponds UNIQUEMENT en JSON conforme au schéma.{scope}\n\n\
         COURS :\n\"\"\"\n{src}\n\"\"\"",
        scope = scope,
        src = truncate(source, 16000),
    )
}

/// Prompt to grade a free-text answer against a rubric, out of `max_points`.
pub fn grade_prompt(question: &str, rubric: Option<&str>, response: &str, max_points: i32) -> String {
    let rub = rubric
        .map(|r| format!("\nPoints-clés attendus (barème) :\n{r}\n"))
        .unwrap_or_default();
    format!(
        "Tu corriges une copie. Note la réponse de l'étudiant sur {max} points.\n\
         Sois juste mais exigeant ; valorise les bons éléments, signale les manques.\n\
         QUESTION : {q}\n{rub}\n\
         RÉPONSE DE L'ÉTUDIANT : {resp}\n\n\
         Réponds UNIQUEMENT en JSON : {{\"score\": <nombre 0..{max}>, \"feedback\": \"<retour bref et utile, en français>\"}}.",
        max = max_points,
        q = question,
        rub = rub,
        resp = truncate(response, 4000),
    )
}
