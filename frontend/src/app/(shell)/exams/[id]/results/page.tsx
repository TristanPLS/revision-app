"use client";

import { Suspense } from "react";
import Link from "next/link";
import { useParams, useSearchParams } from "next/navigation";
import { useQuery } from "@tanstack/react-query";
import { ArrowLeft, CheckCircle2, XCircle, MinusCircle } from "lucide-react";
import { api } from "@/lib/api/client";
import { cn } from "@/lib/utils";
import { buttonVariants } from "@/components/ui/button";
import { Card, CardContent } from "@/components/ui/card";
import { Badge } from "@/components/ui/badge";
import { Progress } from "@/components/ui/progress";
import { Skeleton } from "@/components/ui/skeleton";
import type { ResultItem } from "@/lib/api/types";

function ResultsInner() {
  const { id } = useParams<{ id: string }>();
  const sp = useSearchParams();
  const attemptId = sp.get("attempt");

  const exam = useQuery({ queryKey: ["exam", id], queryFn: () => api.exams.get(id) });
  const res = useQuery({
    queryKey: ["attempt", attemptId],
    queryFn: () => api.exams.attempt(attemptId as string),
    enabled: !!attemptId,
  });

  const subjectId = exam.data?.subject_id;

  if (!attemptId) {
    return <p className="text-muted-foreground">Tentative introuvable.</p>;
  }

  const score = res.data?.score ?? 0;
  const max = res.data?.max_score ?? 0;
  const pct = max > 0 ? Math.round((score / max) * 100) : 0;

  const weakest = [...(res.data?.by_block ?? [])]
    .filter((b) => b.max > 0)
    .sort((a, b) => a.awarded / a.max - b.awarded / b.max)
    .slice(0, 3);

  return (
    <div className="space-y-8">
      <Link
        href={subjectId ? `/subjects/${subjectId}` : "/"}
        className="inline-flex items-center gap-1.5 text-sm text-muted-foreground hover:text-foreground"
      >
        <ArrowLeft className="size-4" /> Retour à la matière
      </Link>

      <header className="space-y-1">
        <h1 className="text-3xl font-semibold tracking-tight">Résultat</h1>
        <p className="text-muted-foreground">{exam.data?.title}</p>
      </header>

      {res.isLoading ? (
        <Skeleton className="h-40" />
      ) : res.data ? (
        <>
          {/* Score */}
          <Card>
            <CardContent className="space-y-4 pt-6">
              <div className="flex items-end gap-3">
                <span className="tabular text-5xl font-semibold leading-none">{pct}%</span>
                <span className="tabular pb-1 text-muted-foreground">
                  {Math.round(score)} / {Math.round(max)} pts
                </span>
              </div>
              <Progress value={pct} />
            </CardContent>
          </Card>

          {/* Per-block breakdown */}
          {res.data.by_block.length > 0 && (
            <section className="space-y-4">
              <h2 className="text-sm font-medium text-muted-foreground">Par bloc</h2>
              <Card>
                <CardContent className="space-y-3 pt-6">
                  {res.data.by_block.map((b) => {
                    const p = b.max > 0 ? Math.round((b.awarded / b.max) * 100) : 0;
                    return (
                      <div key={b.block_id ?? b.title} className="space-y-1">
                        <div className="flex justify-between text-sm">
                          <span>{b.title}</span>
                          <span className="tabular text-muted-foreground">
                            {Math.round(b.awarded)}/{Math.round(b.max)} · {p}%
                          </span>
                        </div>
                        <Progress value={p} />
                      </div>
                    );
                  })}
                </CardContent>
              </Card>
              {weakest.length > 0 && (
                <div className="flex flex-wrap items-center gap-2">
                  <span className="text-xs text-muted-foreground">À renforcer :</span>
                  {weakest.map((b) => (
                    <Badge key={b.block_id ?? b.title} variant="muted">
                      {b.title}
                    </Badge>
                  ))}
                </div>
              )}
            </section>
          )}

          {/* Per-question review */}
          <section className="space-y-4">
            <h2 className="text-sm font-medium text-muted-foreground">Correction</h2>
            <div className="space-y-3">
              {res.data.items.map((it, i) => (
                <ResultRow key={it.question_id} item={it} index={i + 1} />
              ))}
            </div>
          </section>

          <div className="flex gap-2">
            {subjectId && (
              <Link href={`/subjects/${subjectId}`} className={cn(buttonVariants({ variant: "outline" }))}>
                Retour à la matière
              </Link>
            )}
            <Link href={`/exams/${id}/run`} className={cn(buttonVariants())}>
              Repasser
            </Link>
          </div>
        </>
      ) : (
        <p className="text-muted-foreground">Résultat indisponible.</p>
      )}
    </div>
  );
}

function ResultRow({ item, index }: { item: ResultItem; index: number }) {
  const correct = item.is_correct;
  const Icon = correct === true ? CheckCircle2 : correct === false ? XCircle : MinusCircle;
  const iconColor =
    correct === true
      ? "text-rate-good"
      : correct === false
        ? "text-rate-again"
        : "text-muted-foreground";

  return (
    <Card>
      <CardContent className="space-y-2 pt-6">
        <div className="flex items-start gap-3">
          <Icon className={cn("mt-0.5 size-5 shrink-0", iconColor)} />
          <div className="min-w-0 flex-1 space-y-2">
            <p className="font-medium">
              <span className="text-muted-foreground tabular">{index}. </span>
              {item.prompt}
            </p>
            <p className="text-sm">
              <span className="text-muted-foreground">Ta réponse : </span>
              {item.response?.trim() ? formatResponse(item) : <em className="text-muted-foreground">— vide —</em>}
            </p>
            {item.answer_key && (
              <p className="text-sm">
                <span className="text-muted-foreground">Attendu : </span>
                {formatKey(item)}
              </p>
            )}
            {item.ai_feedback && (
              <p className="rounded-md bg-muted px-3 py-2 text-sm text-muted-foreground">
                {item.ai_feedback}
              </p>
            )}
            {!item.ai_feedback && item.explanation && (
              <p className="text-sm text-muted-foreground">{item.explanation}</p>
            )}
          </div>
          <Badge variant="muted" className="shrink-0 tabular">
            {Math.round(item.awarded ?? 0)}/{item.points}
          </Badge>
        </div>
      </CardContent>
    </Card>
  );
}

// MCQ responses are option keys — show the option text when possible.
function formatResponse(item: ResultItem): string {
  if (item.qtype === "mcq" && item.options) {
    return item.options.find((o) => o.key === item.response)?.text ?? item.response ?? "";
  }
  if (item.qtype === "true_false") return item.response === "true" ? "Vrai" : "Faux";
  return item.response ?? "";
}

function formatKey(item: ResultItem): string {
  if (item.qtype === "mcq" && item.options) {
    return item.options.find((o) => o.key === item.answer_key)?.text ?? item.answer_key ?? "";
  }
  if (item.qtype === "true_false") return item.answer_key === "true" ? "Vrai" : "Faux";
  return item.answer_key ?? "";
}

export default function ExamResultsPage() {
  return (
    <Suspense fallback={null}>
      <ResultsInner />
    </Suspense>
  );
}
