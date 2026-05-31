"use client";

import Link from "next/link";
import { useParams } from "next/navigation";
import { useQuery } from "@tanstack/react-query";
import { ArrowLeft, Sparkles } from "lucide-react";
import { api } from "@/lib/api/client";
import { cn } from "@/lib/utils";
import { Card, CardContent } from "@/components/ui/card";
import { Skeleton } from "@/components/ui/skeleton";
import { LeitnerBoard } from "@/components/app/leitner-board";

const RATING_META = [
  { label: "À revoir", color: "bg-rate-again" },
  { label: "Difficile", color: "bg-rate-hard" },
  { label: "Correct", color: "bg-rate-good" },
  { label: "Facile", color: "bg-rate-easy" },
];

export default function StatsPage() {
  const { id } = useParams<{ id: string }>();
  const subject = useQuery({ queryKey: ["subject", id], queryFn: () => api.subjects.get(id) });
  const stats = useQuery({ queryKey: ["stats", id], queryFn: () => api.subjects.stats(id) });
  const insights = useQuery({ queryKey: ["fsrs-insights", id], queryFn: () => api.fsrsInsights(id) });

  const ins = insights.data;
  const ratingTotal = ins ? ins.rating_counts.reduce((a, b) => a + b, 0) : 0;

  const fmtPct = (v: number | null) => (v == null ? "—" : `${Math.round(v * 100)}%`);

  return (
    <div className="space-y-8">
      <Link
        href={`/subjects/${id}`}
        className="inline-flex items-center gap-1.5 text-sm text-muted-foreground hover:text-foreground"
      >
        <ArrowLeft className="size-4" /> Retour à la matière
      </Link>

      <header className="space-y-1">
        <h1 className="text-2xl font-semibold tracking-tight sm:text-3xl">Insights FSRS</h1>
        <p className="text-muted-foreground">{subject.data?.name}</p>
      </header>

      {insights.isLoading || !ins ? (
        <Skeleton className="h-48" />
      ) : (
        <>
          <Card>
            <CardContent className="space-y-5 pt-6">
              <div className="flex flex-wrap items-end gap-x-8 gap-y-3">
                <div>
                  <p className="text-xs text-muted-foreground">Rétention mesurée</p>
                  <p className="tabular text-4xl font-semibold leading-none">
                    {fmtPct(ins.measured_retention)}
                  </p>
                </div>
                <div>
                  <p className="text-xs text-muted-foreground">Cible</p>
                  <p className="tabular text-2xl font-medium">{fmtPct(ins.target_retention)}</p>
                </div>
                <div>
                  <p className="text-xs text-muted-foreground">Prédite (modèle)</p>
                  <p className="tabular text-2xl font-medium">{fmtPct(ins.predicted_retention)}</p>
                </div>
              </div>

              <div className="flex items-start gap-2 rounded-md bg-accent px-4 py-3 text-sm text-accent-foreground">
                <Sparkles className="mt-0.5 size-4 shrink-0" />
                <span>{ins.recommendation}</span>
              </div>

              <div className="flex flex-wrap gap-6 text-sm">
                <Stat label="Révisions" value={ins.reviews_total} />
                <Stat label="Cartes vues" value={ins.cards_reviewed} />
                <Stat
                  label="Intervalle médian"
                  value={ins.median_interval_days != null ? `${ins.median_interval_days} j` : "—"}
                />
              </div>
            </CardContent>
          </Card>

          {/* Rating distribution */}
          <section className="space-y-4">
            <h2 className="text-sm font-medium text-muted-foreground">Distribution des notes</h2>
            <Card>
              <CardContent className="space-y-3 pt-6">
                {RATING_META.map((r, i) => {
                  const c = ins.rating_counts[i];
                  const pct = ratingTotal > 0 ? Math.round((c / ratingTotal) * 100) : 0;
                  return (
                    <div key={r.label} className="space-y-1">
                      <div className="flex justify-between text-sm">
                        <span>{r.label}</span>
                        <span className="tabular text-muted-foreground">
                          {c} · {pct}%
                        </span>
                      </div>
                      <div className="h-2 w-full overflow-hidden rounded-full bg-muted">
                        <div className={cn("h-full rounded-full", r.color)} style={{ width: `${pct}%` }} />
                      </div>
                    </div>
                  );
                })}
              </CardContent>
            </Card>
          </section>

          {/* Leitner */}
          {stats.data && (
            <section className="space-y-4">
              <h2 className="text-sm font-medium text-muted-foreground">Boîtes de Leitner</h2>
              <Card>
                <CardContent className="pt-6">
                  <LeitnerBoard byBox={stats.data.by_box} />
                </CardContent>
              </Card>
            </section>
          )}
        </>
      )}
    </div>
  );
}

function Stat({ label, value }: { label: string; value: number | string }) {
  return (
    <div>
      <p className="text-xs text-muted-foreground">{label}</p>
      <p className="tabular text-2xl font-semibold">{value}</p>
    </div>
  );
}
