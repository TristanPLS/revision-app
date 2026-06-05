"use client";

import { Suspense, useCallback, useEffect, useRef, useState } from "react";
import Link from "next/link";
import { useSearchParams } from "next/navigation";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { X, Eye, Lightbulb, PartyPopper, RotateCcw } from "lucide-react";
import { toast } from "sonner";
import { api, ApiError } from "@/lib/api/client";
import { cn } from "@/lib/utils";
import { buttonVariants } from "@/components/ui/button";
import { Progress } from "@/components/ui/progress";
import { Skeleton } from "@/components/ui/skeleton";
import type { Flashcard } from "@/lib/api/types";

const RATINGS = [
  { value: 1 as const, label: "À revoir", key: "1", color: "bg-rate-again" },
  { value: 2 as const, label: "Difficile", key: "2", color: "bg-rate-hard" },
  { value: 3 as const, label: "Correct", key: "3", color: "bg-rate-good" },
  { value: 4 as const, label: "Facile", key: "4", color: "bg-rate-easy" },
];

function ReviewInner() {
  const sp = useSearchParams();
  const subjectId = sp.get("subject");
  const qc = useQueryClient();

  const sessionRef = useRef<string | null>(null);
  const [pos, setPos] = useState(0);
  const [revealed, setRevealed] = useState(false);
  const [reviewed, setReviewed] = useState(0);
  const [elapsed, setElapsed] = useState(0);

  const queue = useQuery({
    queryKey: ["interleave", subjectId],
    queryFn: () => api.flashcards.interleave(subjectId as string, 20),
    enabled: !!subjectId,
  });

  // Start a study session (for the review log + guardrails); close on unmount.
  useEffect(() => {
    if (!subjectId) return;
    let active = true;
    api.sessions
      .start({ subject_id: subjectId, mode: "interleaved" })
      .then((s) => {
        if (active) sessionRef.current = s.id;
      })
      .catch(() => {});
    return () => {
      active = false;
      const sid = sessionRef.current;
      if (sid) {
        api.sessions.close(sid).catch(() => {});
        qc.invalidateQueries({ queryKey: ["guardrails"] });
      }
    };
  }, [subjectId, qc]);

  // Session timer (fatigue guardrail).
  useEffect(() => {
    const t = setInterval(() => setElapsed((e) => e + 1), 1000);
    return () => clearInterval(t);
  }, []);

  const reviewMut = useMutation({
    mutationFn: ({ cardId, rating }: { cardId: string; rating: 1 | 2 | 3 | 4 }) =>
      api.flashcards.review(cardId, rating, sessionRef.current ?? undefined),
    onError: (e: unknown) =>
      toast.error(e instanceof ApiError ? e.message : "Note non enregistrée"),
    onSettled: () => {
      qc.invalidateQueries({ queryKey: ["stats", subjectId] });
    },
  });

  const cards = queue.data ?? [];
  const current: Flashcard | undefined = cards[pos];

  const rate = useCallback(
    (rating: 1 | 2 | 3 | 4) => {
      if (!current || !revealed) return;
      reviewMut.mutate({ cardId: current.id, rating }); // fire-and-forget (optimistic advance)
      setReviewed((n) => n + 1);
      setRevealed(false);
      setPos((p) => p + 1);
    },
    [current, revealed, reviewMut]
  );

  // Keyboard: Space/Enter reveals, 1–4 rate.
  useEffect(() => {
    const onKey = (e: KeyboardEvent) => {
      if (!current) return;
      if (!revealed && (e.key === " " || e.key === "Enter")) {
        e.preventDefault();
        setRevealed(true);
      } else if (revealed && ["1", "2", "3", "4"].includes(e.key)) {
        rate(Number(e.key) as 1 | 2 | 3 | 4);
      }
    };
    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
  }, [current, revealed, rate]);

  const mm = String(Math.floor(elapsed / 60)).padStart(2, "0");
  const ss = String(elapsed % 60).padStart(2, "0");
  const overCap = elapsed >= 45 * 60;

  const total = cards.length;
  const finished = !!subjectId && !queue.isLoading && pos >= total;

  if (!subjectId) {
    return (
      <CenteredMessage>
        Choisis une matière depuis l&apos;accueil pour réviser.
        <Link href="/" className={cn(buttonVariants(), "mt-4")}>
          Accueil
        </Link>
      </CenteredMessage>
    );
  }

  return (
    <div className="mx-auto flex min-h-screen max-w-2xl flex-col px-4 py-6">
      {/* Top bar */}
      <div className="flex items-center gap-4">
        <Link
          href={`/subjects/${subjectId}`}
          className="text-muted-foreground hover:text-foreground"
          aria-label="Quitter la session"
        >
          <X className="size-5" />
        </Link>
        <div className="flex-1">
          <Progress value={total ? (pos / total) * 100 : 0} />
        </div>
        <span
          className={cn(
            "tabular text-sm",
            overCap ? "font-medium text-rate-hard" : "text-muted-foreground"
          )}
        >
          {mm}:{ss}
        </span>
      </div>

      {overCap && (
        <p className="mt-3 rounded-md bg-accent px-3 py-2 text-center text-sm text-accent-foreground">
          Plus de 45 min — fais une pause. Le rythme bat le marathon.
        </p>
      )}

      {/* Body */}
      <div className="flex flex-1 flex-col items-center justify-center">
        {queue.isLoading ? (
          <Skeleton className="h-64 w-full" />
        ) : finished ? (
          <Done reviewed={reviewed} subjectId={subjectId} onReload={() => { setPos(0); setReviewed(0); setRevealed(false); queue.refetch(); }} />
        ) : current ? (
          <div className="w-full">
            <p className="mb-2 text-center text-xs uppercase tracking-wide text-muted-foreground tabular">
              {pos + 1} / {total}
            </p>
            {/* key = remontage à chaque carte → l'animation d'entrée rejoue */}
            <div
              key={current.id}
              className="animate-reveal flex min-h-64 flex-col justify-center rounded-xl border bg-card p-6 text-center sm:p-8"
            >
              <p className="text-xl font-medium leading-relaxed">{current.front}</p>
              {revealed && (
                <div className="animate-reveal">
                  <hr className="my-6 border-border" />
                  <p className="text-lg leading-relaxed text-foreground">{current.back}</p>
                  {current.hint && (
                    <p className="mt-4 flex items-center justify-center gap-1.5 text-sm text-muted-foreground">
                      <Lightbulb className="size-4" /> {current.hint}
                    </p>
                  )}
                </div>
              )}
            </div>

            {/* Anti-fluence: rating only appears AFTER an attempt + reveal */}
            {revealed ? (
              <div className="animate-reveal mt-6 grid grid-cols-2 gap-2 sm:grid-cols-4">
                {RATINGS.map((r) => (
                  <button
                    key={r.value}
                    onClick={() => rate(r.value)}
                    className={cn(
                      "flex h-14 flex-col items-center justify-center rounded-lg text-rate-foreground transition hover:opacity-90 active:scale-[0.97] focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring",
                      r.color
                    )}
                  >
                    <span className="text-sm font-medium">{r.label}</span>
                    <span className="text-xs opacity-80">{r.key}</span>
                  </button>
                ))}
              </div>
            ) : (
              <button
                onClick={() => setRevealed(true)}
                className={cn(buttonVariants({ size: "lg" }), "mt-6 w-full gap-2")}
              >
                <Eye /> Afficher la réponse
                <span className="text-xs opacity-70">(Espace)</span>
              </button>
            )}
          </div>
        ) : (
          <CenteredMessage>
            <PartyPopper className="mb-2 size-8 text-primary" />
            Rien à réviser pour cette matière — tout est à jour. 🎉
            <Link href={`/subjects/${subjectId}`} className={cn(buttonVariants(), "mt-4")}>
              Retour
            </Link>
          </CenteredMessage>
        )}
      </div>
    </div>
  );
}

function Done({
  reviewed,
  subjectId,
  onReload,
}: {
  reviewed: number;
  subjectId: string;
  onReload: () => void;
}) {
  return (
    <div className="flex flex-col items-center gap-4 text-center">
      <PartyPopper className="size-10 text-primary" />
      <div>
        <p className="text-xl font-semibold">Session terminée</p>
        <p className="text-muted-foreground tabular">
          {reviewed} carte{reviewed > 1 ? "s" : ""} révisée{reviewed > 1 ? "s" : ""}
        </p>
      </div>
      <div className="flex gap-2">
        <button onClick={onReload} className={cn(buttonVariants({ variant: "outline" }), "gap-2")}>
          <RotateCcw className="size-4" /> Encore
        </button>
        <Link href={`/subjects/${subjectId}`} className={cn(buttonVariants())}>
          Terminer
        </Link>
      </div>
    </div>
  );
}

function CenteredMessage({ children }: { children: React.ReactNode }) {
  return (
    <div className="flex min-h-screen flex-col items-center justify-center px-4 text-center text-muted-foreground">
      <div className="flex max-w-sm flex-col items-center">{children}</div>
    </div>
  );
}

export default function ReviewPage() {
  return (
    <Suspense fallback={null}>
      <ReviewInner />
    </Suspense>
  );
}
