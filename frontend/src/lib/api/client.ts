import { z } from "zod";
import * as t from "./types";

// On the server (RSC / route handlers) we hit the backend directly; in the
// browser we use a relative path so the Next rewrite proxies it (same origin → no CORS).
const SERVER_BASE = process.env.BACKEND_INTERNAL_URL ?? "http://localhost:8080";

export class ApiError extends Error {
  status: number;
  constructor(status: number, message: string) {
    super(message);
    this.status = status;
    this.name = "ApiError";
  }
}

async function request<T>(
  path: string,
  schema: z.ZodType<T>,
  init?: RequestInit
): Promise<T> {
  const base = typeof window === "undefined" ? SERVER_BASE : "";
  const res = await fetch(`${base}${path}`, {
    ...init,
    headers: { "Content-Type": "application/json", ...(init?.headers ?? {}) },
    cache: "no-store",
  });

  if (!res.ok) {
    let msg = `Erreur ${res.status}`;
    try {
      const body = (await res.json()) as { error?: string };
      if (body?.error) msg = body.error;
    } catch {
      /* ignore non-JSON error bodies */
    }
    throw new ApiError(res.status, msg);
  }

  if (res.status === 204) return undefined as T;
  return schema.parse(await res.json());
}

const sourceDocSchema = z.object({
  id: z.string(),
  subject_id: z.string(),
  block_id: z.string().nullable(),
  title: z.string(),
  content: z.string(),
  created_at: z.string(),
});

const generateAckSchema = z.object({ job_id: z.string(), status: z.string() });

const providerDefaultsSchema = z.object({
  base_url: z.string(),
  model: z.string().nullable(),
});

const aiSettingsSchema = z.object({
  provider: z.enum(["gemini", "openai", "anthropic"]),
  model: z.string(),
  base_url: z.string(),
  api_key_set: z.boolean(),
  api_key_hint: z.string().nullable(),
  key_required: z.boolean(),
  configured: z.boolean(),
  defaults: z.object({
    gemini: providerDefaultsSchema,
    openai: providerDefaultsSchema,
    anthropic: providerDefaultsSchema,
  }),
});
export type AiSettings = z.infer<typeof aiSettingsSchema>;

const aiTestSchema = z.object({
  ok: z.boolean(),
  model: z.string().optional(),
  latency_ms: z.number().optional(),
  error: z.string().optional(),
});

const closeSessionSchema = z.object({
  session: t.studySessionSchema,
  duration_min: z.number(),
  over_cap: z.boolean(),
  nudge: z.string().nullable(),
});

export const api = {
  subjects: {
    list: () => request("/api/subjects", z.array(t.subjectListItemSchema)),
    get: (id: string) => request(`/api/subjects/${id}`, t.subjectSchema),
    create: (body: { name: string; description?: string; exam_date?: string }) =>
      request("/api/subjects", t.subjectSchema, {
        method: "POST",
        body: JSON.stringify(body),
      }),
    remove: (id: string) =>
      request(`/api/subjects/${id}`, z.void(), { method: "DELETE" }),
    stats: (id: string) =>
      request(`/api/subjects/${id}/stats`, t.subjectStatsSchema),
  },
  blocks: {
    list: (subjectId: string) =>
      request(`/api/subjects/${subjectId}/blocks`, z.array(t.blockSchema)),
    create: (
      subjectId: string,
      body: { title: string; code?: string; summary?: string; position?: number }
    ) =>
      request(`/api/subjects/${subjectId}/blocks`, t.blockSchema, {
        method: "POST",
        body: JSON.stringify(body),
      }),
  },
  flashcards: {
    list: (subjectId: string, blockId?: string) =>
      request(
        `/api/subjects/${subjectId}/flashcards${blockId ? `?block_id=${blockId}` : ""}`,
        z.array(t.flashcardSchema)
      ),
    create: (
      subjectId: string,
      body: { front: string; back: string; hint?: string; block_id?: string }
    ) =>
      request(`/api/subjects/${subjectId}/flashcards`, t.flashcardSchema, {
        method: "POST",
        body: JSON.stringify(body),
      }),
    queue: (subjectId: string, limit = 20) =>
      request(
        `/api/subjects/${subjectId}/flashcards/queue?limit=${limit}`,
        z.array(t.flashcardSchema)
      ),
    interleave: (subjectId: string, cards = 20) =>
      request(
        `/api/subjects/${subjectId}/interleave?cards=${cards}`,
        z.array(t.flashcardSchema)
      ),
    review: (id: string, rating: 1 | 2 | 3 | 4, sessionId?: string) =>
      request(`/api/flashcards/${id}/review`, t.reviewResponseSchema, {
        method: "POST",
        body: JSON.stringify({ rating, session_id: sessionId ?? null }),
      }),
    remove: (id: string) =>
      request(`/api/flashcards/${id}`, z.void(), { method: "DELETE" }),
  },
  sources: {
    create: (
      subjectId: string,
      body: { title: string; content: string; block_id?: string }
    ) =>
      request(`/api/subjects/${subjectId}/sources`, sourceDocSchema, {
        method: "POST",
        body: JSON.stringify(body),
      }),
    generate: (
      sourceId: string,
      body: { kind: string; count?: number; block_id?: string; title?: string }
    ) =>
      request(`/api/sources/${sourceId}/generate`, generateAckSchema, {
        method: "POST",
        body: JSON.stringify(body),
      }),
    // Planning pass: the AI proposes an (editable) study plan for the whole course.
    plan: (sourceId: string) =>
      request(`/api/sources/${sourceId}/plan`, t.studyPlanSchema, { method: "POST" }),
    // Bundle generation: all supports at once from a (possibly edited) plan.
    generateAll: (sourceId: string, body: { plan: t.StudyPlan; title?: string }) =>
      request(`/api/sources/${sourceId}/generate-all`, generateAckSchema, {
        method: "POST",
        body: JSON.stringify(body),
      }),
  },
  exams: {
    list: (subjectId: string) =>
      request(`/api/subjects/${subjectId}/exams`, z.array(t.examListItemSchema)),
    get: (examId: string) => request(`/api/exams/${examId}`, t.examDetailSchema),
    remove: (examId: string) =>
      request(`/api/exams/${examId}`, z.void(), { method: "DELETE" }),
    startAttempt: (examId: string) =>
      request(`/api/exams/${examId}/attempts`, t.attemptStartSchema, { method: "POST" }),
    submit: (
      attemptId: string,
      answers: { question_id: string; response: string | null }[]
    ) =>
      request(`/api/attempts/${attemptId}/submit`, t.attemptResultSchema, {
        method: "POST",
        body: JSON.stringify({ answers }),
      }),
    attempt: (attemptId: string) =>
      request(`/api/attempts/${attemptId}`, t.attemptResultSchema),
  },
  feynman: {
    list: (subjectId: string) =>
      request(`/api/subjects/${subjectId}/feynman`, z.array(t.feynmanConceptItemSchema)),
    get: (id: string) => request(`/api/feynman/${id}`, t.feynmanConceptSchema),
    create: (
      subjectId: string,
      body: { title: string; hint?: string; block_id?: string }
    ) =>
      request(`/api/subjects/${subjectId}/feynman`, t.feynmanConceptSchema, {
        method: "POST",
        body: JSON.stringify(body),
      }),
    remove: (id: string) => request(`/api/feynman/${id}`, z.void(), { method: "DELETE" }),
    attempt: (
      id: string,
      body: {
        self_rating?: number;
        hesitations?: number;
        duration_s?: number;
        explanation?: string;
      }
    ) =>
      request(`/api/feynman/${id}/attempts`, t.feynmanAttemptSchema, {
        method: "POST",
        body: JSON.stringify(body),
      }),
    history: (id: string) =>
      request(`/api/feynman/${id}/attempts`, z.array(t.feynmanAttemptSchema)),
  },
  cornell: {
    list: (subjectId: string) =>
      request(`/api/subjects/${subjectId}/cornell`, z.array(t.cornellNoteItemSchema)),
    get: (id: string) => request(`/api/cornell/${id}`, t.cornellNoteDetailSchema),
    create: (
      subjectId: string,
      body: {
        title: string;
        body: string;
        summary?: string;
        block_id?: string;
        cues: { question: string; answer?: string }[];
      }
    ) =>
      request(`/api/subjects/${subjectId}/cornell`, t.cornellNoteDetailSchema, {
        method: "POST",
        body: JSON.stringify(body),
      }),
    remove: (id: string) => request(`/api/cornell/${id}`, z.void(), { method: "DELETE" }),
    cueToFlashcard: (cueId: string) =>
      request(`/api/cornell/cues/${cueId}/to-flashcard`, t.flashcardSchema, { method: "POST" }),
  },
  jobs: {
    get: (id: string) => request(`/api/jobs/${id}`, t.generationJobSchema),
  },
  sessions: {
    start: (body: { subject_id?: string; mode?: string }) =>
      request("/api/sessions", t.studySessionSchema, {
        method: "POST",
        body: JSON.stringify(body),
      }),
    close: (id: string) =>
      request(`/api/sessions/${id}/close`, closeSessionSchema, { method: "POST" }),
  },
  maps: {
    list: (subjectId: string) =>
      request(`/api/subjects/${subjectId}/maps`, z.array(t.conceptMapListItemSchema)),
    get: (id: string) => request(`/api/maps/${id}`, t.conceptMapDetailSchema),
    remove: (id: string) => request(`/api/maps/${id}`, z.void(), { method: "DELETE" }),
  },
  schemas: {
    list: (subjectId: string) =>
      request(`/api/subjects/${subjectId}/schemas`, z.array(t.schemaListItemSchema)),
    get: (id: string) => request(`/api/schemas/${id}`, t.schemaAssetSchema),
    create: (
      subjectId: string,
      body: { title: string; reference?: string; block_id?: string }
    ) =>
      request(`/api/subjects/${subjectId}/schemas`, t.schemaAssetSchema, {
        method: "POST",
        body: JSON.stringify(body),
      }),
    update: (
      id: string,
      body: { title?: string; reference?: string; drawing?: unknown }
    ) =>
      request(`/api/schemas/${id}`, t.schemaAssetSchema, {
        method: "PATCH",
        body: JSON.stringify(body),
      }),
    remove: (id: string) => request(`/api/schemas/${id}`, z.void(), { method: "DELETE" }),
  },
  settings: {
    get: () => request("/api/settings", aiSettingsSchema),
    update: (body: {
      provider?: string;
      model?: string;
      base_url?: string;
      api_key?: string;
    }) =>
      request("/api/settings", aiSettingsSchema, {
        method: "PUT",
        body: JSON.stringify(body),
      }),
    test: () => request("/api/settings/ai/test", aiTestSchema, { method: "POST" }),
  },
  fsrsInsights: (subjectId: string) =>
    request(`/api/subjects/${subjectId}/fsrs-insights`, t.fsrsInsightsSchema),
  guardrails: () => request("/api/guardrails", t.guardrailsSchema),
};
