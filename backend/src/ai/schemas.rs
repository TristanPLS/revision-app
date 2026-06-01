use serde_json::{json, Value};

/// `responseSchema` for flashcard generation: `{ flashcards: [{front, back, block_hint?}] }`.
pub fn flashcards_schema() -> Value {
    json!({
        "type": "object",
        "properties": {
            "flashcards": {
                "type": "array",
                "items": {
                    "type": "object",
                    "properties": {
                        "front": { "type": "string" },
                        "back": { "type": "string" },
                        "block_hint": { "type": "string" }
                    },
                    "required": ["front", "back"]
                }
            }
        },
        "required": ["flashcards"]
    })
}

/// `responseSchema` for exam generation.
pub fn exam_schema() -> Value {
    json!({
        "type": "object",
        "properties": {
            "questions": {
                "type": "array",
                "items": {
                    "type": "object",
                    "properties": {
                        "qtype": {
                            "type": "string",
                            "enum": ["mcq", "true_false", "short_answer", "open_ended"]
                        },
                        "prompt": { "type": "string" },
                        "options": {
                            "type": "array",
                            "items": {
                                "type": "object",
                                "properties": {
                                    "key": { "type": "string" },
                                    "text": { "type": "string" }
                                },
                                "required": ["key", "text"]
                            }
                        },
                        "answer_key": { "type": "string" },
                        "explanation": { "type": "string" },
                        "points": { "type": "integer" },
                        "block_hint": { "type": "string" }
                    },
                    "required": ["qtype", "prompt"]
                }
            }
        },
        "required": ["questions"]
    })
}

/// `responseSchema` for Feynman concept-menu generation.
pub fn feynman_schema() -> Value {
    json!({
        "type": "object",
        "properties": {
            "concepts": {
                "type": "array",
                "items": {
                    "type": "object",
                    "properties": {
                        "title": { "type": "string" },
                        "hint": { "type": "string" },
                        "block_hint": { "type": "string" }
                    },
                    "required": ["title"]
                }
            }
        },
        "required": ["concepts"]
    })
}

/// `responseSchema` for the planning pass: a block breakdown + a proposed
/// quantity for each support, sized to the course. The model decides the
/// numbers; the user can adjust them before generating.
pub fn plan_schema() -> Value {
    json!({
        "type": "object",
        "properties": {
            "blocks": {
                "type": "array",
                "items": {
                    "type": "object",
                    "properties": {
                        "title": { "type": "string" },
                        "code": { "type": "string" },
                        "summary": { "type": "string" }
                    },
                    "required": ["title"]
                }
            },
            "flashcards": { "type": "integer" },
            "exam_questions": { "type": "integer" },
            "feynman_concepts": { "type": "integer" },
            "map_nodes": { "type": "integer" },
            "cornell_cues": { "type": "integer" },
            "schemas": { "type": "integer" }
        },
        "required": [
            "blocks", "flashcards", "exam_questions", "feynman_concepts", "map_nodes",
            "cornell_cues", "schemas"
        ]
    })
}

/// `responseSchema` for Cornell note generation: structured body + summary +
/// margin recall questions (cues).
pub fn cornell_schema() -> Value {
    json!({
        "type": "object",
        "properties": {
            "title": { "type": "string" },
            "body": { "type": "string" },
            "summary": { "type": "string" },
            "cues": {
                "type": "array",
                "items": {
                    "type": "object",
                    "properties": {
                        "question": { "type": "string" },
                        "answer": { "type": "string" }
                    },
                    "required": ["question"]
                }
            }
        },
        "required": ["title", "body", "cues"]
    })
}

/// `responseSchema` for schema-stub generation (dual coding): each item is a
/// diagram the learner should draw, with a title and a reference of what it must
/// contain. The drawing itself is left to the learner (active encoding).
pub fn schemas_schema() -> Value {
    json!({
        "type": "object",
        "properties": {
            "schemas": {
                "type": "array",
                "items": {
                    "type": "object",
                    "properties": {
                        "title": { "type": "string" },
                        "reference": { "type": "string" },
                        "block_hint": { "type": "string" }
                    },
                    "required": ["title", "reference"]
                }
            }
        },
        "required": ["schemas"]
    })
}

/// `responseSchema` for concept-map generation (hierarchical nodes + edges).
pub fn concept_map_schema() -> Value {
    json!({
        "type": "object",
        "properties": {
            "title": { "type": "string" },
            "nodes": {
                "type": "array",
                "items": {
                    "type": "object",
                    "properties": {
                        "id": { "type": "string" },
                        "label": { "type": "string" },
                        "parent": { "type": "string" }
                    },
                    "required": ["id", "label"]
                }
            },
            "edges": {
                "type": "array",
                "items": {
                    "type": "object",
                    "properties": {
                        "from": { "type": "string" },
                        "to": { "type": "string" },
                        "label": { "type": "string" }
                    },
                    "required": ["from", "to"]
                }
            }
        },
        "required": ["nodes"]
    })
}

/// `responseSchema` for AI grading of a free-text answer.
pub fn grade_schema() -> Value {
    json!({
        "type": "object",
        "properties": {
            "score": { "type": "number" },
            "feedback": { "type": "string" }
        },
        "required": ["score", "feedback"]
    })
}
