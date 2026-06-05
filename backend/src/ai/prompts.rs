use super::truncate;

/// When generating across a whole course (the "tout générer" bundle), we pass
/// the list of block titles so the model classifies each item by setting
/// `block_hint` to the matching block. Empty list → single-scope generation,
/// no classification instruction (legacy behaviour preserved).
fn blocks_menu_rule(blocks: &[String]) -> String {
    if blocks.is_empty() {
        return String::new();
    }
    let list = blocks
        .iter()
        .map(|b| format!("           • {b}"))
        .collect::<Vec<_>>()
        .join("\n");
    format!(
        "\n         - CLASSE chaque élément dans l'UN de ces blocs en recopiant son titre EXACT dans 'block_hint' :\n{list}",
    )
}

/// Prompt for atomic active-recall flashcards. Encodes the study guide's rules
/// (one fact per card, chunking, focus on figures/mechanisms/examples).
pub fn flashcards_prompt(
    source: &str,
    count: i32,
    block_title: Option<&str>,
    blocks_menu: &[String],
) -> String {
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
         - 'block_hint' (optionnel) = le thème de la carte, pour la classer.{menu}\n\
         - Langue : la même que le COURS.\n\
         - Réponds UNIQUEMENT en JSON conforme au schéma, sans texte autour.{scope}\n\n\
         COURS :\n\"\"\"\n{src}\n\"\"\"",
        count = count,
        scope = scope,
        menu = blocks_menu_rule(blocks_menu),
        src = truncate(source, super::max_source_chars()),
    )
}

/// Prompt for a mixed-type mock exam (QCM, vrai/faux, court, ouvert), interleaving blocks.
pub fn exam_prompt(
    source: &str,
    count: i32,
    block_title: Option<&str>,
    blocks_menu: &[String],
) -> String {
    let scope = match block_title {
        Some(t) => format!(" Concentre-toi sur le thème : « {t} »."),
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
         - Varie les thèmes d'une question à l'autre (entrelacement). 'block_hint' = thème.{menu}\n\
         - Langue : celle du COURS. Réponds UNIQUEMENT en JSON conforme au schéma.{scope}\n\n\
         COURS :\n\"\"\"\n{src}\n\"\"\"",
        count = count,
        scope = scope,
        menu = blocks_menu_rule(blocks_menu),
        src = truncate(source, super::max_source_chars()),
    )
}

/// Prompt for a Feynman menu: concepts worth explaining "comme à un enfant".
pub fn feynman_prompt(
    source: &str,
    count: i32,
    block_title: Option<&str>,
    blocks_menu: &[String],
) -> String {
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
         - 'hint' = les 2-3 points-clés qu'une bonne explication doit contenir.{menu}\n\
         - Langue : celle du COURS. Réponds UNIQUEMENT en JSON conforme au schéma.{scope}\n\n\
         COURS :\n\"\"\"\n{src}\n\"\"\"",
        count = count,
        scope = scope,
        menu = blocks_menu_rule(blocks_menu),
        src = truncate(source, super::max_source_chars()),
    )
}

/// Prompt for a hierarchical concept map. `target_nodes` lets the planning pass
/// size the map; `None` keeps the default 8–15 range.
pub fn concept_map_prompt(
    source: &str,
    block_title: Option<&str>,
    target_nodes: Option<i32>,
) -> String {
    let scope = match block_title {
        Some(t) => format!(" Concentre-toi sur le thème : « {t} »."),
        None => String::new(),
    };
    let nodes_rule = match target_nodes {
        Some(n) => format!("environ {n} concepts", n = n.clamp(6, 20)),
        None => "8 à 15 concepts".to_string(),
    };
    format!(
        "Tu es un pédagogue. À partir du COURS ci-dessous, construis une CARTE CONCEPTUELLE hiérarchique.\n\n\
         RÈGLES :\n\
         - 'title' : titre court de la carte.\n\
         - 'nodes' : {nodes_rule}. Chaque nœud a un 'id' court unique (ex. \"n1\"), un 'label' bref, \
           et 'parent' = l'id du concept parent (omis ou vide pour LA racine unique).\n\
         - 'edges' : liens transversaux entre concepts ('from'/'to' = ids de nœuds, 'label' = nature du lien, ex. « entraîne »).\n\
         - Une seule racine. Langue : celle du COURS. Réponds UNIQUEMENT en JSON conforme au schéma.{scope}\n\n\
         COURS :\n\"\"\"\n{src}\n\"\"\"",
        nodes_rule = nodes_rule,
        scope = scope,
        src = truncate(source, super::max_source_chars()),
    )
}

/// Prompt for the planning pass: the model reads the whole course and proposes
/// a block breakdown + a quantity for each support, sized to the material. The
/// counts are the model's call (the user can still adjust them afterwards).
pub fn plan_prompt(source: &str) -> String {
    format!(
        "Tu es un ingénieur pédagogique. Analyse le COURS ci-dessous et propose un PLAN DE RÉVISION complet.\n\n\
         RÈGLES :\n\
         - 'blocks' : découpe le cours en 2 à 8 blocs thématiques cohérents (chapitres / grandes idées). \
           Chaque bloc a un 'title' bref, un 'code' court optionnel (ex. « B1 »), et un 'summary' d'une phrase.\n\
         - Choisis TOI-MÊME, en fonction de la densité et de la longueur réelle du cours, le nombre approprié de :\n\
           • 'flashcards' (faits atomiques à mémoriser) — typiquement 8 à 40 ;\n\
           • 'exam_questions' (questions d'examen blanc) — typiquement 5 à 20 ;\n\
           • 'feynman_concepts' (mécanismes à savoir expliquer) — typiquement 3 à 10 ;\n\
           • 'map_nodes' (nœuds de la carte conceptuelle) — typiquement 8 à 15 ;\n\
           • 'cornell_cues' (questions de rappel d'une fiche Cornell synthétisant le cours) — typiquement 5 à 12 ;\n\
           • 'schemas' (schémas à dessiner soi-même pour le dual coding) — typiquement 0 à 4, selon que le cours s'y prête (processus, structures, cycles).\n\
         - Vise une COUVERTURE complète sans redondance : un cours court mérite peu d'items, un cours dense davantage.\n\
         - Langue : celle du COURS. Réponds UNIQUEMENT en JSON conforme au schéma, sans texte autour.\n\n\
         COURS :\n\"\"\"\n{src}\n\"\"\"",
        src = truncate(source, super::max_source_chars()),
    )
}

/// Prompt for a Cornell note: structured body + summary + margin recall
/// questions (cues). `count` is the number of cues to produce.
pub fn cornell_prompt(source: &str, count: i32, block_title: Option<&str>) -> String {
    let scope = match block_title {
        Some(t) => format!(" Concentre-toi sur le thème : « {t} »."),
        None => String::new(),
    };
    format!(
        "Tu es un pédagogue expert de la méthode Cornell. À partir du COURS ci-dessous, rédige UNE fiche Cornell.\n\n\
         RÈGLES :\n\
         - 'title' : titre court de la fiche.\n\
         - 'body' : les notes principales, structurées et synthétiques (puces, abréviations, hiérarchie). C'est la colonne de droite.\n\
         - 'summary' : un résumé de 2-3 phrases en bas de fiche (la synthèse Cornell).\n\
         - 'cues' : exactement {count} questions de rappel actif dans la marge gauche, chacune avec sa 'question' (courte, ciblée) et sa 'answer' (la réponse attendue). Ces questions servent à se tester sans relire les notes.\n\
         - Langue : celle du COURS. Réponds UNIQUEMENT en JSON conforme au schéma.{scope}\n\n\
         COURS :\n\"\"\"\n{src}\n\"\"\"",
        count = count,
        scope = scope,
        src = truncate(source, super::max_source_chars()),
    )
}

/// Prompt for schema stubs (dual coding): diagrams the learner should draw,
/// each with a title and a reference of what it must contain. The learner draws
/// them (active encoding); the AI only scaffolds what to include.
pub fn schemas_prompt(
    source: &str,
    count: i32,
    block_title: Option<&str>,
    blocks_menu: &[String],
) -> String {
    let scope = match block_title {
        Some(t) => format!(" Concentre-toi sur le thème : « {t} »."),
        None => String::new(),
    };
    format!(
        "Tu es un pédagogue. À partir du COURS ci-dessous, propose {count} SCHÉMAS à dessiner soi-même \
         (dual coding : relier le verbal au visuel).\n\n\
         RÈGLES :\n\
         - Cible ce qui gagne à être visualisé : processus, cycles, structures, relations de cause à effet.\n\
         - 'title' = le sujet du schéma (ex. « Le cycle de Calvin »).\n\
         - 'reference' = la LISTE des éléments, légendes et relations que le schéma doit contenir (ce que l'élève vérifiera après l'avoir dessiné de mémoire). Ne dessine PAS à sa place : décris ce qu'il doit y faire figurer.\n\
         - 'block_hint' = le thème, pour classer le schéma.{menu}\n\
         - Langue : celle du COURS. Réponds UNIQUEMENT en JSON conforme au schéma.{scope}\n\n\
         COURS :\n\"\"\"\n{src}\n\"\"\"",
        count = count,
        scope = scope,
        menu = blocks_menu_rule(blocks_menu),
        src = truncate(source, super::max_source_chars()),
    )
}

/// Prompt to grade a free-text answer against a rubric, out of `max_points`.
pub fn grade_prompt(
    question: &str,
    rubric: Option<&str>,
    response: &str,
    max_points: i32,
) -> String {
    let rub = rubric
        .map(|r| format!("\nPoints-clés attendus (barème) :\n{r}\n"))
        .unwrap_or_default();
    format!(
        "Tu corriges une copie. Note la réponse de l'étudiant sur {max} points.\n\
         Sois juste mais exigeant ; valorise les bons éléments, signale les manques.\n\
         QUESTION : {q}\n{rub}\n\
         RÉPONSE DE L'ÉTUDIANT : {resp}\n\n\
         Réponds UNIQUEMENT en JSON : {{\"score\": <nombre 0..{max}>, \"feedback\": \"<retour bref et utile, dans la même langue que la QUESTION>\"}}.",
        max = max_points,
        q = question,
        rub = rub,
        resp = truncate(response, 4000),
    )
}
