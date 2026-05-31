"use client";

import { useEffect, useState } from "react";
import Link from "next/link";
import { useParams } from "next/navigation";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { ArrowLeft, Loader2, Sparkles, CheckCircle2 } from "lucide-react";
import { toast } from "sonner";
import { api, ApiError } from "@/lib/api/client";
import { cn } from "@/lib/utils";
import { Button, buttonVariants } from "@/components/ui/button";
import { Card, CardContent } from "@/components/ui/card";
import { Textarea } from "@/components/ui/textarea";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";

export default function GeneratePage() {
  const { id } = useParams<{ id: string }>();
  const qc = useQueryClient();

  const [content, setContent] = useState("");
  const [title, setTitle] = useState("");
  const [count, setCount] = useState(10);
  const [blockId, setBlockId] = useState("");
  const [kind, setKind] = useState<"flashcards" | "exam" | "feynman">("flashcards");
  const [generatedKind, setGeneratedKind] = useState<"flashcards" | "exam" | "feynman">(
    "flashcards"
  );
  const [jobId, setJobId] = useState<string | null>(null);

  const blocks = useQuery({ queryKey: ["blocks", id], queryFn: () => api.blocks.list(id) });

  const job = useQuery({
    queryKey: ["job", jobId],
    queryFn: () => api.jobs.get(jobId as string),
    enabled: !!jobId,
    refetchInterval: (query) => {
      const status = query.state.data?.status;
      return status === "done" || status === "failed" ? false : 1500;
    },
  });

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

  // When a job finishes, refresh the subject's data once.
  useEffect(() => {
    if (job.data?.status === "done") {
      qc.invalidateQueries({ queryKey: ["flashcards", id] });
      qc.invalidateQueries({ queryKey: ["stats", id] });
      qc.invalidateQueries({ queryKey: ["subjects"] });
    }
  }, [job.data?.status, id, qc]);

  const running =
    start.isPending ||
    (!!jobId && (job.data?.status === "pending" || job.data?.status === "running"));
  const done = job.data?.status === "done";
  const failed = job.data?.status === "failed";
  const result =
    done && job.data?.result && typeof job.data.result === "object"
      ? (job.data.result as { created?: number; skipped?: number; exam_id?: string })
      : null;

  return (
    <div className="space-y-8">
      <Link
        href={`/subjects/${id}`}
        className="inline-flex items-center gap-1.5 text-sm text-muted-foreground hover:text-foreground"
      >
        <ArrowLeft className="size-4" /> Retour à la matière
      </Link>

      <header className="space-y-1">
        <h1 className="text-3xl font-semibold tracking-tight">Générer avec l&apos;IA</h1>
        <p className="text-muted-foreground">
          Colle ton cours : l&apos;IA en extrait des flashcards atomiques (active recall)
          ou un examen blanc. Tu valides, tu révises, tu recommences.
        </p>
      </header>

      <Card>
        <CardContent className="space-y-5 pt-6">
          <div className="inline-flex rounded-lg border p-1">
            {(["flashcards", "exam", "feynman"] as const).map((k) => (
              <button
                key={k}
                type="button"
                onClick={() => setKind(k)}
                disabled={running}
                className={cn(
                  "rounded-md px-4 py-1.5 text-sm font-medium transition-colors",
                  kind === k
                    ? "bg-primary text-primary-foreground"
                    : "text-muted-foreground hover:text-foreground"
                )}
              >
                {k === "flashcards" ? "Flashcards" : k === "exam" ? "Examen blanc" : "Menu Feynman"}
              </button>
            ))}
          </div>
          <div className="space-y-2">
            <Label htmlFor="content">Contenu du cours</Label>
            <Textarea
              id="content"
              value={content}
              onChange={(e) => setContent(e.target.value)}
              placeholder="Colle ici le texte du cours…"
              className="min-h-64"
              disabled={running}
            />
          </div>

          <div className="grid gap-4 sm:grid-cols-3">
            <div className="space-y-2">
              <Label htmlFor="title">Titre (optionnel)</Label>
              <Input
                id="title"
                value={title}
                onChange={(e) => setTitle(e.target.value)}
                placeholder="Chapitre 1"
                disabled={running}
              />
            </div>
            <div className="space-y-2">
              <Label htmlFor="count">
                {kind === "exam"
                  ? "Nombre de questions"
                  : kind === "feynman"
                    ? "Nombre de concepts"
                    : "Nombre de cartes"}
              </Label>
              <Input
                id="count"
                type="number"
                min={1}
                max={50}
                value={count}
                onChange={(e) => setCount(Math.min(50, Math.max(1, Number(e.target.value) || 1)))}
                className="tabular"
                disabled={running}
              />
            </div>
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
          </div>

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
              <span className="text-sm text-muted-foreground">
                Colle un peu plus de texte.
              </span>
            )}
          </div>
        </CardContent>
      </Card>

      {/* Result */}
      {done && (
        <Card className="border-primary/40">
          <CardContent className="flex flex-wrap items-center gap-4 pt-6">
            <CheckCircle2 className="size-8 text-primary" />
            <div className="flex-1">
              <p className="font-medium">
                {result?.created ?? 0}{" "}
                {generatedKind === "exam"
                  ? "question"
                  : generatedKind === "feynman"
                    ? "concept"
                    : "carte"}
                {(result?.created ?? 0) > 1 ? "s" : ""} générée
                {(result?.created ?? 0) > 1 ? "s" : ""}
                {result?.skipped ? ` · ${result.skipped} ignorée(s)` : ""}
              </p>
              <p className="text-sm text-muted-foreground">
                {generatedKind === "exam"
                  ? "Examen prêt — conditions réelles, chrono."
                  : generatedKind === "feynman"
                    ? "Concepts prêts à expliquer à voix haute."
                    : "Prêtes à réviser en répétition espacée."}
              </p>
            </div>
            <div className="flex gap-2">
              <Link href={`/subjects/${id}`} className={cn(buttonVariants({ variant: "outline" }))}>
                Voir la matière
              </Link>
              {generatedKind === "exam" && result?.exam_id ? (
                <Link href={`/exams/${result.exam_id}/run`} className={cn(buttonVariants())}>
                  Passer l&apos;examen
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

      {failed && (
        <Card className="border-destructive/40">
          <CardContent className="pt-6">
            <p className="font-medium text-destructive">Échec de la génération</p>
            <p className="text-sm text-muted-foreground">
              {job.data?.error ?? "Vérifie la clé GEMINI_API_KEY et réessaie."}
            </p>
          </CardContent>
        </Card>
      )}
    </div>
  );
}
