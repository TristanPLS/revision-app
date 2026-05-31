"use client";

import { useState } from "react";
import Link from "next/link";
import { useParams } from "next/navigation";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { ArrowLeft, Plus, Sparkles, Trash2 } from "lucide-react";
import { toast } from "sonner";
import { api, ApiError } from "@/lib/api/client";
import { cn } from "@/lib/utils";
import { Button, buttonVariants } from "@/components/ui/button";
import { Card, CardContent } from "@/components/ui/card";
import { Input } from "@/components/ui/input";
import { Badge } from "@/components/ui/badge";
import { Skeleton } from "@/components/ui/skeleton";
import { LeitnerBoard } from "@/components/app/leitner-board";

export default function SubjectPage() {
  const { id } = useParams<{ id: string }>();
  const qc = useQueryClient();

  const subject = useQuery({ queryKey: ["subject", id], queryFn: () => api.subjects.get(id) });
  const stats = useQuery({ queryKey: ["stats", id], queryFn: () => api.subjects.stats(id) });
  const blocks = useQuery({ queryKey: ["blocks", id], queryFn: () => api.blocks.list(id) });
  const cards = useQuery({ queryKey: ["flashcards", id], queryFn: () => api.flashcards.list(id) });
  const exams = useQuery({ queryKey: ["exams", id], queryFn: () => api.exams.list(id) });
  const feynman = useQuery({ queryKey: ["feynman", id], queryFn: () => api.feynman.list(id) });
  const cornell = useQuery({ queryKey: ["cornell", id], queryFn: () => api.cornell.list(id) });

  const [blockTitle, setBlockTitle] = useState("");

  const addBlock = useMutation({
    mutationFn: () => api.blocks.create(id, { title: blockTitle.trim() }),
    onSuccess: () => {
      setBlockTitle("");
      qc.invalidateQueries({ queryKey: ["blocks", id] });
    },
    onError: (e: unknown) =>
      toast.error(e instanceof ApiError ? e.message : "Échec"),
  });

  const deleteCard = useMutation({
    mutationFn: (cardId: string) => api.flashcards.remove(cardId),
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: ["flashcards", id] });
      qc.invalidateQueries({ queryKey: ["stats", id] });
    },
  });

  const deleteExam = useMutation({
    mutationFn: (examId: string) => api.exams.remove(examId),
    onSuccess: () => qc.invalidateQueries({ queryKey: ["exams", id] }),
  });

  const dueNow = stats.data?.due_now ?? 0;

  return (
    <div className="space-y-10">
      <Link
        href="/"
        className="inline-flex items-center gap-1.5 text-sm text-muted-foreground hover:text-foreground"
      >
        <ArrowLeft className="size-4" /> Accueil
      </Link>

      <header className="flex flex-wrap items-end justify-between gap-4">
        <div className="space-y-1">
          <h1 className="text-3xl font-semibold tracking-tight">
            {subject.data?.name ?? <Skeleton className="h-9 w-48" />}
          </h1>
          {subject.data?.description && (
            <p className="text-muted-foreground">{subject.data.description}</p>
          )}
        </div>
        <div className="flex gap-2">
          <Link
            href={`/subjects/${id}/generate`}
            className={cn(buttonVariants({ variant: "outline" }))}
          >
            <Sparkles /> Générer (IA)
          </Link>
          <Link
            href={`/review?subject=${id}`}
            className={cn(buttonVariants(), dueNow === 0 && "pointer-events-none opacity-50")}
            aria-disabled={dueNow === 0}
          >
            Réviser{dueNow > 0 ? ` (${dueNow})` : ""}
          </Link>
        </div>
      </header>

      {/* Stats + Leitner */}
      <section className="space-y-4">
        <h2 className="text-sm font-medium text-muted-foreground">Progression</h2>
        <Card>
          <CardContent className="space-y-5 pt-6">
            <div className="flex flex-wrap gap-6 text-sm">
              <Stat label="Cartes" value={stats.data?.total_cards} />
              <Stat label="À réviser" value={stats.data?.due_now} />
              <Stat label="Révisions" value={stats.data?.reviews_total} />
            </div>
            {stats.data && <LeitnerBoard byBox={stats.data.by_box} />}
            {stats.data && stats.data.weakest_blocks.length > 0 && (
              <div className="space-y-1.5">
                <p className="text-xs text-muted-foreground">Blocs les plus faibles</p>
                <div className="flex flex-wrap gap-2">
                  {stats.data.weakest_blocks.map((b) => (
                    <Badge key={b.block_id ?? b.title} variant="muted">
                      {b.title} · {Math.round(b.mastery * 100)}%
                    </Badge>
                  ))}
                </div>
              </div>
            )}
          </CardContent>
        </Card>
      </section>

      {/* Exams */}
      <section className="space-y-4">
        <div className="flex items-center justify-between">
          <h2 className="text-sm font-medium text-muted-foreground">Examens blancs</h2>
          <Link
            href={`/subjects/${id}/generate`}
            className={cn(buttonVariants({ variant: "outline", size: "sm" }))}
          >
            <Sparkles className="size-4" /> Générer
          </Link>
        </div>
        {exams.data && exams.data.length > 0 ? (
          <ul className="divide-y rounded-lg border">
            {exams.data.map((e) => (
              <li key={e.id} className="flex items-center gap-3 px-4 py-3">
                <div className="min-w-0 flex-1">
                  <p className="truncate text-sm font-medium">{e.title}</p>
                  <p className="tabular text-sm text-muted-foreground">
                    {e.question_count} question{e.question_count > 1 ? "s" : ""}
                    {e.time_limit_s ? ` · ${Math.round(e.time_limit_s / 60)} min` : ""}
                    {e.best_score != null && e.max_score
                      ? ` · meilleur : ${Math.round(e.best_score)}/${Math.round(e.max_score)}`
                      : ""}
                  </p>
                </div>
                <Link href={`/exams/${e.id}/run`} className={cn(buttonVariants({ size: "sm" }))}>
                  Passer
                </Link>
                <Button
                  variant="ghost"
                  size="icon"
                  aria-label="Supprimer l'examen"
                  onClick={() => deleteExam.mutate(e.id)}
                >
                  <Trash2 className="size-4" />
                </Button>
              </li>
            ))}
          </ul>
        ) : (
          <p className="text-sm text-muted-foreground">
            Aucun examen.{" "}
            <Link href={`/subjects/${id}/generate`} className="text-primary hover:underline">
              Génère un examen blanc depuis ton cours →
            </Link>
          </p>
        )}
      </section>

      {/* Feynman */}
      <section className="space-y-4">
        <div className="flex items-center justify-between">
          <h2 className="text-sm font-medium text-muted-foreground">Menu Feynman</h2>
          <Link
            href={`/subjects/${id}/generate`}
            className={cn(buttonVariants({ variant: "outline", size: "sm" }))}
          >
            <Sparkles className="size-4" /> Générer
          </Link>
        </div>
        {feynman.data && feynman.data.length > 0 ? (
          <ul className="divide-y rounded-lg border">
            {feynman.data.map((c) => (
              <li key={c.id} className="flex items-center gap-3 px-4 py-3">
                <div className="min-w-0 flex-1">
                  <p className="truncate text-sm font-medium">{c.title}</p>
                  <p className="tabular text-sm text-muted-foreground">
                    {c.attempts} essai{c.attempts > 1 ? "s" : ""}
                    {c.last_rating != null ? ` · dernière auto-éval ${c.last_rating}/5` : ""}
                  </p>
                </div>
                <Link href={`/feynman/${c.id}`} className={cn(buttonVariants({ size: "sm" }))}>
                  S&apos;entraîner
                </Link>
              </li>
            ))}
          </ul>
        ) : (
          <p className="text-sm text-muted-foreground">
            Aucun concept.{" "}
            <Link href={`/subjects/${id}/generate`} className="text-primary hover:underline">
              Génère un menu Feynman →
            </Link>
          </p>
        )}
      </section>

      {/* Cornell */}
      <section className="space-y-4">
        <div className="flex items-center justify-between">
          <h2 className="text-sm font-medium text-muted-foreground">Notes Cornell</h2>
          <Link
            href={`/subjects/${id}/cornell`}
            className={cn(buttonVariants({ variant: "outline", size: "sm" }))}
          >
            <Plus className="size-4" /> Notes
          </Link>
        </div>
        {cornell.data && cornell.data.length > 0 ? (
          <ul className="divide-y rounded-lg border">
            {cornell.data.map((n) => (
              <li key={n.id} className="flex items-center gap-3 px-4 py-3">
                <div className="min-w-0 flex-1">
                  <p className="truncate text-sm font-medium">{n.title}</p>
                  <p className="tabular text-sm text-muted-foreground">
                    {n.cue_count} question{n.cue_count > 1 ? "s" : ""} de marge
                  </p>
                </div>
                <Link
                  href={`/subjects/${id}/cornell?note=${n.id}`}
                  className={cn(buttonVariants({ variant: "outline", size: "sm" }))}
                >
                  Ouvrir
                </Link>
              </li>
            ))}
          </ul>
        ) : (
          <p className="text-sm text-muted-foreground">
            Aucune note.{" "}
            <Link href={`/subjects/${id}/cornell`} className="text-primary hover:underline">
              Crée une note Cornell →
            </Link>
          </p>
        )}
      </section>

      {/* Blocks */}
      <section className="space-y-4">
        <h2 className="text-sm font-medium text-muted-foreground">Blocs / thématiques</h2>
        <form
          className="flex gap-2"
          onSubmit={(e) => {
            e.preventDefault();
            if (blockTitle.trim() && !addBlock.isPending) addBlock.mutate();
          }}
        >
          <Input
            value={blockTitle}
            onChange={(e) => setBlockTitle(e.target.value)}
            placeholder="Nouveau bloc (ex. Causes)"
            aria-label="Titre du bloc"
          />
          <Button type="submit" variant="outline" disabled={!blockTitle.trim() || addBlock.isPending}>
            <Plus /> Bloc
          </Button>
        </form>
        <div className="flex flex-wrap gap-2">
          {blocks.data?.map((b) => (
            <Badge key={b.id} variant="secondary" className="px-3 py-1">
              {b.code ? `${b.code} · ` : ""}
              {b.title}
            </Badge>
          ))}
          {blocks.data?.length === 0 && (
            <p className="text-sm text-muted-foreground">
              Aucun bloc — l&apos;IA peut en proposer, ou ajoute-les à la main.
            </p>
          )}
        </div>
      </section>

      {/* Flashcards */}
      <section className="space-y-4">
        <h2 className="text-sm font-medium text-muted-foreground">
          Flashcards{cards.data ? ` (${cards.data.length})` : ""}
        </h2>
        {cards.isLoading ? (
          <Skeleton className="h-24" />
        ) : cards.data && cards.data.length > 0 ? (
          <ul className="divide-y rounded-lg border">
            {cards.data.slice(0, 50).map((c) => (
              <li key={c.id} className="flex items-center gap-3 px-4 py-3">
                <div className="min-w-0 flex-1">
                  <p className="truncate text-sm font-medium">{c.front}</p>
                  <p className="truncate text-sm text-muted-foreground">{c.back}</p>
                </div>
                {c.source === "ai" && <Badge variant="muted">IA</Badge>}
                <Button
                  variant="ghost"
                  size="icon"
                  aria-label="Supprimer la carte"
                  onClick={() => deleteCard.mutate(c.id)}
                >
                  <Trash2 className="size-4" />
                </Button>
              </li>
            ))}
          </ul>
        ) : (
          <p className="text-sm text-muted-foreground">
            Pas encore de cartes.{" "}
            <Link href={`/subjects/${id}/generate`} className="text-primary hover:underline">
              Génère-en depuis ton cours →
            </Link>
          </p>
        )}
      </section>
    </div>
  );
}

function Stat({ label, value }: { label: string; value?: number }) {
  return (
    <div>
      <p className="text-xs text-muted-foreground">{label}</p>
      <p className="tabular text-2xl font-semibold">{value ?? "—"}</p>
    </div>
  );
}
