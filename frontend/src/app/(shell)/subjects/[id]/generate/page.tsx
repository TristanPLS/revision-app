"use client";

import { useEffect, useState } from "react";
import Link from "next/link";
import { useParams } from "next/navigation";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { ArrowLeft, Loader2, Sparkles, CheckCircle2, Wand2, Plus, Minus, Trash2 } from "lucide-react";
import { toast } from "sonner";
import { api, ApiError } from "@/lib/api/client";
import type { StudyPlan } from "@/lib/api/types";
import { cn } from "@/lib/utils";
import { Button, buttonVariants } from "@/components/ui/button";
import { Card, CardContent } from "@/components/ui/card";
import { Textarea } from "@/components/ui/textarea";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";

type Kind = "flashcards" | "exam" | "feynman" | "concept_map" | "cornell" | "schema";
type Mode = "all" | "single";

const KIND_LABELS: Record<Kind, string> = {
  flashcards: "Flashcards",
  exam: "Examen",
  feynman: "Feynman",
  concept_map: "Carte",
  cornell: "Cornell",
  schema: "Schémas",
};

export default function GeneratePage() {
  const { id } = useParams<{ id: string }>();
  const qc = useQueryClient();

  const [mode, setMode] = useState<Mode>("all");
  const [content, setContent] = useState("");
  const [title, setTitle] = useState("");
  const [count, setCount] = useState(10);
  const [blockId, setBlockId] = useState("");
  const [kind, setKind] = useState<Kind>("flashcards");
  const [generatedKind, setGeneratedKind] = useState<Kind>("flashcards");
  const [jobId, setJobId] = useState<string | null>(null);

  // Bundle ("Tout générer") flow.
  const [plan, setPlan] = useState<StudyPlan | null>(null);
  const [planSourceId, setPlanSourceId] = useState<string | null>(null);
  const [bundleJobId, setBundleJobId] = useState<string | null>(null);

  const blocks = useQuery({ queryKey: ["blocks", id], queryFn: () => api.blocks.list(id) });
  const settings = useQuery({ queryKey: ["settings"], queryFn: api.settings.get });
  const aiReady = settings.data?.configured !== false;

  const job = useQuery({
    queryKey: ["job", jobId],
    queryFn: () => api.jobs.get(jobId as string),
    enabled: !!jobId,
    refetchInterval: (query) => {
      const status = query.state.data?.status;
      return status === "done" || status === "failed" ? false : 1500;
    },
  });

  const bundleJob = useQuery({
    queryKey: ["job", bundleJobId],
    queryFn: () => api.jobs.get(bundleJobId as string),
    enabled: !!bundleJobId,
    refetchInterval: (query) => {
      const status = query.state.data?.status;
      return status === "done" || status === "failed" ? false : 1500;
    },
  });

  // --- Single-support generation (unchanged behaviour) ---
  const start = useMutation({
    mutationFn: async () => {
      const src = await api.sources.create(id, {
        title: title.trim() || "Cours",
        content,
        block_id: blockId || undefined,
      });
      const ack = await api.sources.generate(src.id, {
        kind,
        count,
        block_id: blockId || undefined,
        title: title.trim() || undefined,
      });
      return ack.job_id;
    },
    onSuccess: (jid) => setJobId(jid),
    onError: (e: unknown) =>
      toast.error(e instanceof ApiError ? e.message : "Échec du lancement"),
  });

  // --- Bundle: analyse → editable plan → generate all ---
  const analyze = useMutation({
    mutationFn: async () => {
      const src = await api.sources.create(id, {
        title: title.trim() || "Cours",
        content,
      });
      const proposed = await api.sources.plan(src.id);
      return { sourceId: src.id, proposed };
    },
    onSuccess: ({ sourceId, proposed }) => {
      setPlanSourceId(sourceId);
      setPlan(proposed);
      setBundleJobId(null);
    },
    onError: (e: unknown) =>
      toast.error(e instanceof ApiError ? e.message : "Échec de l'analyse"),
  });

  const generateAll = useMutation({
    mutationFn: async () => {
      if (!planSourceId || !plan) throw new Error("plan manquant");
      const ack = await api.sources.generateAll(planSourceId, {
        plan,
        title: title.trim() || undefined,
      });
      return ack.job_id;
    },
    onSuccess: (jid) => setBundleJobId(jid),
    onError: (e: unknown) =>
      toast.error(e instanceof ApiError ? e.message : "Échec du lancement"),
  });

  // When a single job finishes, refresh the subject's data once.
  useEffect(() => {
    if (job.data?.status === "done") {
      qc.invalidateQueries({ queryKey: ["flashcards", id] });
      qc.invalidateQueries({ queryKey: ["stats", id] });
      qc.invalidateQueries({ queryKey: ["subjects"] });
    }
  }, [job.data?.status, id, qc]);

  // When the bundle finishes, refresh everything it may have created.
  useEffect(() => {
    if (bundleJob.data?.status === "done") {
      for (const key of ["flashcards", "exams", "feynman", "maps", "blocks", "stats"]) {
        qc.invalidateQueries({ queryKey: [key, id] });
      }
      qc.invalidateQueries({ queryKey: ["subjects"] });
    }
  }, [bundleJob.data?.status, id, qc]);

  // ---- Single-mode derived state ----
  const running =
    start.isPending ||
    (!!jobId && (job.data?.status === "pending" || job.data?.status === "running"));
  const done = job.data?.status === "done";
  const failed = job.data?.status === "failed";
  const result =
    done && job.data?.result && typeof job.data.result === "object"
      ? (job.data.result as {
          created?: number;
          skipped?: number;
          exam_id?: string;
          map_id?: string;
          nodes?: number;
          cues?: number;
          note_id?: string;
        })
      : null;
  const doneCount =
    generatedKind === "concept_map"
      ? (result?.nodes ?? 0)
      : generatedKind === "cornell"
        ? (result?.cues ?? 0)
        : (result?.created ?? 0);
  const doneNoun =
    generatedKind === "exam" || generatedKind === "cornell"
      ? "question"
      : generatedKind === "feynman"
        ? "concept"
        : generatedKind === "concept_map"
          ? "nœud"
          : generatedKind === "schema"
            ? "schéma"
            : "carte";
  const doneSubtitle =
    generatedKind === "exam"
      ? "Examen prêt — conditions réelles, chrono."
      : generatedKind === "feynman"
        ? "Concepts prêts à expliquer à voix haute."
        : generatedKind === "concept_map"
          ? "Carte conceptuelle prête."
          : generatedKind === "cornell"
            ? "Fiche Cornell prête — teste-toi avec les questions de marge."
            : generatedKind === "schema"
              ? "Schémas à dessiner de mémoire (dual coding)."
              : "Prêtes à réviser en répétition espacée.";

  // ---- Bundle-mode derived state ----
  const bundleRunning =
    analyze.isPending ||
    generateAll.isPending ||
    (!!bundleJobId &&
      (bundleJob.data?.status === "pending" || bundleJob.data?.status === "running"));
  const bundleDone = bundleJob.data?.status === "done";
  const bundleFailed = bundleJob.data?.status === "failed";
  const bundleResult =
    bundleDone && bundleJob.data?.result && typeof bundleJob.data.result === "object"
      ? (bundleJob.data.result as Record<string, unknown>)
      : null;

  function setPlanField(key: keyof Omit<StudyPlan, "blocks">, value: number) {
    setPlan((p) => (p ? { ...p, [key]: Math.max(0, value || 0) } : p));
  }

  return (
    <div className="space-y-8">
      <Link
        href={`/subjects/${id}`}
        className="inline-flex items-center gap-1.5 py-3 -my-3 text-sm text-muted-foreground hover:text-foreground"
      >
        <ArrowLeft className="size-4" /> Retour à la matière
      </Link>

      {!aiReady && (
        <Card className="border-primary/40 bg-primary/5">
          <CardContent className="flex flex-wrap items-center justify-between gap-3 py-4">
            <p className="text-sm text-muted-foreground">
              <span className="font-medium text-foreground">
                L&apos;IA n&apos;est pas encore configurée
              </span>{" "}
              — ajoute ta clé API pour lancer une génération.
            </p>
            <Link href="/settings" className={cn(buttonVariants({ size: "sm" }))}>
              Configurer
            </Link>
          </CardContent>
        </Card>
      )}

      <header className="space-y-1">
        <h1 className="text-2xl font-semibold tracking-tight sm:text-3xl">Générer avec l&apos;IA</h1>
        <p className="text-muted-foreground">
          Colle ton cours : l&apos;IA en extrait tout ton matériel de révision. Tu valides, tu
          révises, tu recommences.
        </p>
      </header>

      {/* Mode switch */}
      <div className="flex flex-wrap gap-2 rounded-lg border p-1">
        <button
          type="button"
          onClick={() => setMode("all")}
          disabled={running || bundleRunning}
          className={cn(
            "flex min-h-11 items-center gap-1.5 rounded-md px-3 py-1.5 text-sm font-medium transition-colors sm:min-h-0",
            mode === "all"
              ? "bg-primary text-primary-foreground"
              : "text-muted-foreground hover:text-foreground"
          )}
        >
          <Wand2 className="size-4" /> Tout générer
        </button>
        <button
          type="button"
          onClick={() => setMode("single")}
          disabled={running || bundleRunning}
          className={cn(
            "min-h-11 rounded-md px-3 py-1.5 text-sm font-medium transition-colors sm:min-h-0",
            mode === "single"
              ? "bg-primary text-primary-foreground"
              : "text-muted-foreground hover:text-foreground"
          )}
        >
          Un support à la fois
        </button>
      </div>

      <Card>
        <CardContent className="space-y-5 pt-6">
          {mode === "single" && (
            <div className="flex flex-wrap gap-2 rounded-lg border p-1">
              {(["flashcards", "exam", "feynman", "concept_map", "cornell", "schema"] as const).map(
                (k) => (
                  <button
                    key={k}
                    type="button"
                    onClick={() => setKind(k)}
                    disabled={running}
                    className={cn(
                      "min-h-11 rounded-md px-3 py-1.5 text-sm font-medium transition-colors sm:min-h-0",
                      kind === k
                        ? "bg-primary text-primary-foreground"
                        : "text-muted-foreground hover:text-foreground"
                    )}
                  >
                    {KIND_LABELS[k]}
                  </button>
                )
              )}
            </div>
          )}

          <div className="space-y-2">
            <Label htmlFor="content">Contenu du cours</Label>
            <Textarea
              id="content"
              value={content}
              onChange={(e) => setContent(e.target.value)}
              placeholder="Colle ici le texte du cours…"
              className="min-h-64"
              disabled={running || bundleRunning}
            />
          </div>

          <div
            className={cn(
              "grid gap-4",
              mode === "single" ? "sm:grid-cols-3" : "sm:grid-cols-2"
            )}
          >
            <div className="space-y-2">
              <Label htmlFor="title">Titre (optionnel)</Label>
              <Input
                id="title"
                value={title}
                onChange={(e) => setTitle(e.target.value)}
                placeholder="Chapitre 1"
                disabled={running || bundleRunning}
              />
            </div>
            {mode === "single" && kind !== "concept_map" && (
              <div className="space-y-2">
                <Label htmlFor="count">
                  {kind === "exam" || kind === "cornell"
                    ? "Nombre de questions"
                    : kind === "feynman"
                      ? "Nombre de concepts"
                      : kind === "schema"
                        ? "Nombre de schémas"
                        : "Nombre de cartes"}
                </Label>
                <NumberStepper
                  id="count"
                  min={1}
                  max={50}
                  value={count}
                  onChange={setCount}
                  disabled={running}
                />
              </div>
            )}
            {mode === "single" && (
              <div className="space-y-2">
                <Label htmlFor="block">Bloc (optionnel)</Label>
                <select
                  id="block"
                  value={blockId}
                  onChange={(e) => setBlockId(e.target.value)}
                  disabled={running}
                  className="flex h-11 w-full rounded-md border border-input bg-background px-3 py-2 text-sm focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring disabled:opacity-50"
                >
                  <option value="">— Aucun —</option>
                  {blocks.data?.map((b) => (
                    <option key={b.id} value={b.id}>
                      {b.code ? `${b.code} · ` : ""}
                      {b.title}
                    </option>
                  ))}
                </select>
              </div>
            )}
          </div>

          {/* Action row */}
          {mode === "single" ? (
            <div className="flex items-center gap-3">
              <Button
                onClick={() => {
                  setJobId(null);
                  setGeneratedKind(kind);
                  start.mutate();
                }}
                disabled={content.trim().length < 20 || running}
              >
                {running ? <Loader2 className="animate-spin" /> : <Sparkles />}
                {running ? "Génération…" : "Générer"}
              </Button>
              {content.trim().length > 0 && content.trim().length < 20 && (
                <span className="text-sm text-muted-foreground">Colle un peu plus de texte.</span>
              )}
            </div>
          ) : (
            <div className="flex items-center gap-3">
              <Button
                onClick={() => {
                  setBundleJobId(null);
                  setPlan(null);
                  analyze.mutate();
                }}
                disabled={content.trim().length < 20 || bundleRunning}
              >
                {analyze.isPending ? <Loader2 className="animate-spin" /> : <Wand2 />}
                {analyze.isPending ? "Analyse…" : "Analyser le cours"}
              </Button>
              {content.trim().length > 0 && content.trim().length < 20 && (
                <span className="text-sm text-muted-foreground">Colle un peu plus de texte.</span>
              )}
            </div>
          )}
        </CardContent>
      </Card>

      {/* Editable plan (bundle mode) */}
      {mode === "all" && plan && !bundleDone && (
        <Card className="border-primary/40">
          <CardContent className="space-y-6 pt-6">
            <div>
              <h2 className="font-medium">Plan proposé par l&apos;IA</h2>
              <p className="text-sm text-muted-foreground">
                Ajuste les quantités et les blocs si besoin, puis lance la génération complète.
              </p>
            </div>

            {/* Quantities */}
            <div className="grid gap-4 sm:grid-cols-2 lg:grid-cols-4">
              <PlanCount
                label="Flashcards"
                value={plan.flashcards}
                max={50}
                onChange={(v) => setPlanField("flashcards", v)}
                disabled={bundleRunning}
              />
              <PlanCount
                label="Questions d'examen"
                value={plan.exam_questions}
                max={50}
                onChange={(v) => setPlanField("exam_questions", v)}
                disabled={bundleRunning}
              />
              <PlanCount
                label="Concepts Feynman"
                value={plan.feynman_concepts}
                max={30}
                onChange={(v) => setPlanField("feynman_concepts", v)}
                disabled={bundleRunning}
              />
              <PlanCount
                label="Nœuds de carte"
                value={plan.map_nodes}
                max={20}
                onChange={(v) => setPlanField("map_nodes", v)}
                disabled={bundleRunning}
              />
              <PlanCount
                label="Questions Cornell"
                value={plan.cornell_cues}
                max={20}
                onChange={(v) => setPlanField("cornell_cues", v)}
                disabled={bundleRunning}
              />
              <PlanCount
                label="Schémas à dessiner"
                value={plan.schemas}
                max={8}
                onChange={(v) => setPlanField("schemas", v)}
                disabled={bundleRunning}
              />
            </div>
            <p className="text-xs text-muted-foreground">
              Mets une quantité à 0 pour ne pas générer ce support.
            </p>

            {/* Blocks */}
            <div className="space-y-2">
              <Label>Blocs détectés ({plan.blocks.length})</Label>
              <div className="space-y-2">
                {plan.blocks.map((b, i) => (
                  <div key={i} className="flex gap-2">
                    <Input
                      value={b.code ?? ""}
                      onChange={(e) =>
                        setPlan((p) => {
                          if (!p) return p;
                          const nb = [...p.blocks];
                          nb[i] = { ...nb[i], code: e.target.value || null };
                          return { ...p, blocks: nb };
                        })
                      }
                      placeholder="Code"
                      className="w-16 shrink-0"
                      disabled={bundleRunning}
                      aria-label={`Code du bloc ${i + 1}`}
                    />
                    <Input
                      value={b.title}
                      onChange={(e) =>
                        setPlan((p) => {
                          if (!p) return p;
                          const nb = [...p.blocks];
                          nb[i] = { ...nb[i], title: e.target.value };
                          return { ...p, blocks: nb };
                        })
                      }
                      placeholder="Titre du bloc"
                      className="min-w-0 flex-1"
                      disabled={bundleRunning}
                      aria-label={`Titre du bloc ${i + 1}`}
                    />
                    <Button
                      variant="ghost"
                      size="icon"
                      aria-label="Retirer le bloc"
                      disabled={bundleRunning}
                      onClick={() =>
                        setPlan((p) =>
                          p ? { ...p, blocks: p.blocks.filter((_, j) => j !== i) } : p
                        )
                      }
                    >
                      <Trash2 className="size-4" />
                    </Button>
                  </div>
                ))}
              </div>
              <Button
                variant="outline"
                size="sm"
                disabled={bundleRunning || plan.blocks.length >= 12}
                onClick={() =>
                  setPlan((p) =>
                    p
                      ? { ...p, blocks: [...p.blocks, { title: "", code: null, summary: null }] }
                      : p
                  )
                }
              >
                <Plus className="size-4" /> Ajouter un bloc
              </Button>
            </div>

            <div className="flex items-center gap-3">
              <Button onClick={() => generateAll.mutate()} disabled={bundleRunning}>
                {bundleRunning ? <Loader2 className="animate-spin" /> : <Sparkles />}
                {bundleRunning ? "Génération…" : "Tout générer"}
              </Button>
              <span className="text-sm text-muted-foreground">
                {bundleRunning
                  ? "L'IA produit chaque support, ça peut prendre une minute."
                  : "Crée les blocs puis tous les supports d'un coup."}
              </span>
            </div>
          </CardContent>
        </Card>
      )}

      {/* Bundle result */}
      {mode === "all" && bundleDone && bundleResult && (
        <Card className="border-primary/40">
          <CardContent className="space-y-4 pt-6">
            <div className="flex items-center gap-3">
              <CheckCircle2 className="size-8 text-primary" />
              <div>
                <p className="font-medium">Génération complète terminée</p>
                <p className="text-sm text-muted-foreground">
                  Ton cours est transformé en tout un kit de révision.
                </p>
              </div>
            </div>
            <ul className="grid gap-2 sm:grid-cols-2">
              <BundleLine label="Blocs" value={asCount(bundleResult.blocks)} />
              <BundleLine label="Flashcards" part={bundleResult.flashcards} kind="created" />
              <BundleLine label="Examen" part={bundleResult.exam} kind="created" />
              <BundleLine label="Concepts Feynman" part={bundleResult.feynman} kind="created" />
              <BundleLine label="Carte conceptuelle" part={bundleResult.concept_map} kind="nodes" />
              <BundleLine label="Fiche Cornell" part={bundleResult.cornell} kind="cues" />
              <BundleLine label="Schémas à dessiner" part={bundleResult.schemas} kind="created" />
            </ul>
            <div className="flex flex-wrap gap-2">
              <Link href={`/subjects/${id}`} className={cn(buttonVariants())}>
                Voir la matière
              </Link>
              <Link
                href={`/review?subject=${id}`}
                className={cn(buttonVariants({ variant: "outline" }))}
              >
                Réviser
              </Link>
            </div>
          </CardContent>
        </Card>
      )}

      {mode === "all" && bundleFailed && (
        <Card className="border-destructive/40">
          <CardContent className="pt-6">
            <p className="font-medium text-destructive">Échec de la génération</p>
            <p className="text-sm text-muted-foreground">
              {bundleJob.data?.error ?? "Vérifie ta clé API dans Réglages et réessaie."}
            </p>
          </CardContent>
        </Card>
      )}

      {/* Single-support result */}
      {mode === "single" && done && (
        <Card className="border-primary/40">
          <CardContent className="flex flex-wrap items-center gap-4 pt-6">
            <CheckCircle2 className="size-8 text-primary" />
            <div className="flex-1">
              <p className="font-medium">
                {doneCount} {doneNoun}
                {doneCount > 1 ? "s" : ""} générée{doneCount > 1 ? "s" : ""}
                {result?.skipped ? ` · ${result.skipped} ignorée(s)` : ""}
              </p>
              <p className="text-sm text-muted-foreground">{doneSubtitle}</p>
            </div>
            <div className="flex flex-wrap gap-2">
              <Link href={`/subjects/${id}`} className={cn(buttonVariants({ variant: "outline" }))}>
                Voir la matière
              </Link>
              {generatedKind === "exam" && result?.exam_id ? (
                <Link href={`/exams/${result.exam_id}/run`} className={cn(buttonVariants())}>
                  Passer l&apos;examen
                </Link>
              ) : generatedKind === "concept_map" && result?.map_id ? (
                <Link href={`/maps/${result.map_id}`} className={cn(buttonVariants())}>
                  Voir la carte
                </Link>
              ) : generatedKind === "cornell" ? (
                <Link
                  href={`/subjects/${id}/cornell${result?.note_id ? `?note=${result.note_id}` : ""}`}
                  className={cn(buttonVariants())}
                >
                  Ouvrir la fiche
                </Link>
              ) : generatedKind === "schema" ? (
                <Link href={`/subjects/${id}/schemas`} className={cn(buttonVariants())}>
                  Voir les schémas
                </Link>
              ) : generatedKind === "flashcards" ? (
                <Link href={`/review?subject=${id}`} className={cn(buttonVariants())}>
                  Réviser
                </Link>
              ) : null}
            </div>
          </CardContent>
        </Card>
      )}

      {mode === "single" && failed && (
        <Card className="border-destructive/40">
          <CardContent className="pt-6">
            <p className="font-medium text-destructive">Échec de la génération</p>
            <p className="text-sm text-muted-foreground">
              {job.data?.error ?? "Vérifie ta clé API dans Réglages et réessaie."}
            </p>
          </CardContent>
        </Card>
      )}
    </div>
  );
}

function NumberStepper({
  id,
  value,
  min,
  max,
  onChange,
  disabled,
  ariaLabel,
}: {
  id?: string;
  value: number;
  min: number;
  max: number;
  onChange: (v: number) => void;
  disabled?: boolean;
  ariaLabel?: string;
}) {
  // Saisie libre pendant la frappe ; le clamp n'arrive qu'au blur.
  const [draft, setDraft] = useState<string | null>(null);
  const clamp = (n: number) => Math.min(max, Math.max(min, n));
  const step = (delta: number) => {
    setDraft(null);
    onChange(clamp(value + delta));
  };
  return (
    <div className="flex gap-2">
      <Button
        type="button"
        variant="outline"
        size="icon"
        aria-label="Diminuer"
        disabled={disabled || value <= min}
        onClick={() => step(-1)}
      >
        <Minus className="size-4" />
      </Button>
      <Input
        id={id}
        inputMode="numeric"
        pattern="[0-9]*"
        value={draft ?? String(value)}
        onChange={(e) => setDraft(e.target.value)}
        onBlur={(e) => {
          const n = Number(e.target.value);
          onChange(e.target.value.trim() !== "" && Number.isFinite(n) ? clamp(n) : min);
          setDraft(null);
        }}
        className="tabular min-w-0 flex-1 text-center"
        disabled={disabled}
        aria-label={ariaLabel}
      />
      <Button
        type="button"
        variant="outline"
        size="icon"
        aria-label="Augmenter"
        disabled={disabled || value >= max}
        onClick={() => step(1)}
      >
        <Plus className="size-4" />
      </Button>
    </div>
  );
}

function PlanCount({
  label,
  value,
  max,
  onChange,
  disabled,
}: {
  label: string;
  value: number;
  max: number;
  onChange: (v: number) => void;
  disabled?: boolean;
}) {
  return (
    <div className="space-y-2">
      <Label>{label}</Label>
      <NumberStepper
        value={value}
        min={0}
        max={max}
        onChange={onChange}
        disabled={disabled}
        ariaLabel={label}
      />
    </div>
  );
}

function asCount(v: unknown): number {
  return typeof v === "number" ? v : 0;
}

function BundleLine({
  label,
  value,
  part,
  kind,
}: {
  label: string;
  value?: number;
  part?: unknown;
  kind?: "created" | "nodes" | "cues";
}) {
  let text: string;
  let ok = true;
  if (value !== undefined) {
    text = String(value);
  } else if (part && typeof part === "object") {
    const p = part as Record<string, unknown>;
    if (typeof p.error === "string") {
      text = "échec";
      ok = false;
    } else if (kind === "nodes") {
      text = String(asCount(p.nodes));
    } else if (kind === "cues") {
      text = String(asCount(p.cues));
    } else {
      const created = asCount(p.created);
      const skipped = asCount(p.skipped);
      text = skipped > 0 ? `${created} · ${skipped} ignorée(s)` : String(created);
    }
  } else {
    text = "—";
    ok = false;
  }
  return (
    <li className="flex items-center justify-between rounded-md border px-3 py-2 text-sm">
      <span className="text-muted-foreground">{label}</span>
      <span className={cn("tabular font-medium", !ok && "text-muted-foreground")}>{text}</span>
    </li>
  );
}
