"use client";

import { useEffect, useState } from "react";
import Link from "next/link";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import {
  ArrowRight,
  Check,
  Flag,
  Loader2,
  RotateCcw,
  TriangleAlert,
  Trophy,
  X,
} from "lucide-react";
import { toast } from "sonner";
import { api, ApiError } from "@/lib/api/client";
import { cn, flagEmoji } from "@/lib/utils";
import { EmptyState } from "@/components/app/empty-state";
import { Badge } from "@/components/ui/badge";
import { Button, buttonVariants } from "@/components/ui/button";
import { Progress } from "@/components/ui/progress";
import { Skeleton } from "@/components/ui/skeleton";
import type { GeoAnswerResponse, GeoFlagItem, GeoStats } from "@/lib/api/types";

const CONTINENTS = ["Afrique", "Amériques", "Asie", "Europe", "Océanie"];
const QUEUE_SIZE = 20;

type Feedback = GeoAnswerResponse & { given: string };
type Score = { good: number; total: number };

export default function GeoFlagsPage() {
  const qc = useQueryClient();
  const [continent, setContinent] = useState("");
  // `round` entre dans la clé de requête : sans lui, relancer une série ressert
  // la file du cache (cartes déjà jouées) le temps que le refetch réponde.
  const [round, setRound] = useState(0);
  const [pos, setPos] = useState(0);
  const [feedback, setFeedback] = useState<Feedback | null>(null);
  const [score, setScore] = useState<Score>({ good: 0, total: 0 });

  const queue = useQuery({
    queryKey: ["geo-queue", "flag", continent, round],
    queryFn: () =>
      api.geo.queue("flag", { continent: continent || undefined, limit: QUEUE_SIZE }),
  });

  const cards = (queue.data ?? []).filter(
    (item): item is GeoFlagItem => item.kind === "flag"
  );
  const total = cards.length;
  const current = cards[pos];
  const finished = total > 0 && !current;

  const stats = useQuery({
    queryKey: ["geo-stats", "flag"],
    queryFn: () => api.geo.stats("flag"),
    enabled: finished,
  });

  useEffect(() => {
    if (!queue.isError) return;
    toast.error(
      queue.error instanceof ApiError
        ? queue.error.message
        : "Impossible de charger les drapeaux"
    );
  }, [queue.isError, queue.error]);

  const answerMut = useMutation({
    mutationFn: (v: { cardId: string; given: string }) =>
      api.geo.answer(v.cardId, v.given),
    onSuccess: (res, v) => {
      setFeedback({ ...res, given: v.given });
      setScore((s) => ({
        good: s.good + (res.correct ? 1 : 0),
        total: s.total + 1,
      }));
    },
    onError: (e: unknown) =>
      toast.error(e instanceof ApiError ? e.message : "Réponse non enregistrée"),
    onSettled: () => qc.invalidateQueries({ queryKey: ["geo-stats", "flag"] }),
  });

  function choose(option: string) {
    if (!current || feedback || answerMut.isPending) return;
    answerMut.mutate({ cardId: current.card_id, given: option });
  }

  function next() {
    setFeedback(null);
    setPos((p) => p + 1);
  }

  function newRound() {
    setFeedback(null);
    setPos(0);
    setRound((r) => r + 1);
  }

  // Changer de filtre = nouvelle série : le score repart de zéro pour rester
  // celui d'un ensemble de cartes cohérent.
  function changeContinent(value: string) {
    setContinent(value);
    setScore({ good: 0, total: 0 });
    newRound();
  }

  const isLast = pos === total - 1;

  return (
    <div className="mx-auto flex min-h-dvh max-w-lg flex-col px-4 pt-4 pb-[max(1rem,env(safe-area-inset-bottom))]">
      <h1 className="sr-only">Devine le drapeau</h1>

      <header className="flex items-center gap-2">
        <Link
          href="/"
          aria-label="Quitter"
          className="-ml-3 flex size-11 shrink-0 items-center justify-center text-muted-foreground hover:text-foreground"
        >
          <X className="size-5" />
        </Link>
        <select
          aria-label="Filtrer par continent"
          value={continent}
          onChange={(e) => changeContinent(e.target.value)}
          className="h-11 min-w-0 flex-1 rounded-md border border-input bg-background px-3 text-base focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring sm:text-sm"
        >
          <option value="">Tous les continents</option>
          {CONTINENTS.map((c) => (
            <option key={c} value={c}>
              {c}
            </option>
          ))}
        </select>
        <span className="tabular flex shrink-0 items-center gap-1 text-sm font-medium text-muted-foreground">
          <Check className="size-4 text-rate-good" />
          {score.good}/{score.total}
        </span>
      </header>

      {current && (
        <div className="mt-3 flex items-center gap-3">
          <Progress className="flex-1" value={(pos / total) * 100} />
          <span className="tabular shrink-0 text-xs text-muted-foreground">
            {pos + 1} / {total}
          </span>
        </div>
      )}

      <main className="flex flex-1 flex-col justify-center py-6">
        {queue.isPending ? (
          <QueueSkeleton />
        ) : queue.isError ? (
          <EmptyState
            icon={TriangleAlert}
            action={
              <Button
                variant="outline"
                onClick={() => queue.refetch()}
                disabled={queue.isFetching}
                className="gap-2"
              >
                {queue.isFetching ? <Loader2 className="animate-spin" /> : <RotateCcw />}
                Réessayer
              </Button>
            }
          >
            Les drapeaux ne se chargent pas. Vérifie ta connexion au serveur.
          </EmptyState>
        ) : total === 0 ? (
          <EmptyState
            icon={Flag}
            action={
              continent ? (
                <Button variant="outline" onClick={() => changeContinent("")}>
                  Voir tous les continents
                </Button>
              ) : (
                <Link href="/" className={cn(buttonVariants({ variant: "outline" }))}>
                  Accueil
                </Link>
              )
            }
          >
            Aucun drapeau à réviser dans cette sélection — la répétition espacée les
            ramènera le moment venu.
            {score.total > 0 && ` Score de la session : ${score.good}/${score.total}.`}
          </EmptyState>
        ) : current ? (
          <div key={current.card_id} className="animate-reveal flex flex-1 flex-col">
            <div className="flex flex-1 flex-col items-center justify-center gap-3 py-2">
              <p className="text-sm text-muted-foreground">Quel pays ?</p>
              {/* aria-label neutre : l'emoji est annoncé « drapeau de … » par
                  certains lecteurs d'écran, ce qui donnerait la réponse. */}
              <span
                role="img"
                aria-label="Drapeau à identifier"
                className="text-[length:clamp(5rem,30vw,9rem)] leading-none"
              >
                {flagEmoji(current.iso2) || current.iso2.toUpperCase()}
              </span>
              <Badge variant="muted">{current.continent}</Badge>
            </div>

            <div className="grid gap-2.5">
              {current.options.map((option) => {
                const expected = !!feedback && option === feedback.expected;
                const given = !!feedback && option === feedback.given;
                const pending =
                  answerMut.isPending && answerMut.variables?.given === option;
                return (
                  <button
                    key={option}
                    onClick={() => choose(option)}
                    disabled={!!feedback || answerMut.isPending}
                    className={cn(
                      buttonVariants({ variant: "outline" }),
                      "h-auto min-h-14 w-full justify-between whitespace-normal px-4 py-3 text-left text-base [&_svg]:size-5",
                      // Les options gelées gardent leur couleur (le vert et le
                      // rouge de la correction ne doivent pas être délavés) :
                      // d'où un disabled:opacity-* plutôt qu'un opacity-* nu,
                      // que la spécificité de `:disabled` écraserait.
                      "focus-visible:ring-2 focus-visible:ring-ring disabled:opacity-100",
                      feedback && !expected && !given && "disabled:opacity-60",
                      expected && "border-transparent bg-rate-good text-rate-foreground",
                      given &&
                        !expected &&
                        "border-transparent bg-rate-again text-rate-foreground"
                    )}
                  >
                    <span>{option}</span>
                    {pending && <Loader2 className="shrink-0 animate-spin" />}
                    {expected && <Check className="shrink-0" />}
                    {given && !expected && <X className="shrink-0" />}
                  </button>
                );
              })}
            </div>

            <p
              aria-live="polite"
              className={cn(
                "mt-4 min-h-5 text-center text-sm font-medium",
                feedback && (feedback.correct ? "text-rate-good" : "text-rate-again")
              )}
            >
              {feedback
                ? feedback.correct
                  ? "Bien vu !"
                  : `Raté — c'était ${feedback.expected}`
                : ""}
            </p>

            <Button
              size="lg"
              onClick={next}
              disabled={!feedback}
              className="mt-2 w-full gap-2 focus-visible:ring-2 focus-visible:ring-ring"
            >
              {isLast ? "Voir le score" : "Suivant"}
              <ArrowRight />
            </Button>
          </div>
        ) : (
          <Done score={score} stats={stats.data} onContinue={newRound} />
        )}
      </main>
    </div>
  );
}

function Done({
  score,
  stats,
  onContinue,
}: {
  score: Score;
  stats?: GeoStats;
  onContinue: () => void;
}) {
  const pct = score.total ? Math.round((score.good / score.total) * 100) : 0;
  return (
    <div className="flex flex-col items-center gap-4 text-center">
      <Trophy className="size-10 text-primary" />
      <div>
        <p className="text-xl font-semibold">Série terminée</p>
        <p className="tabular text-muted-foreground">
          {score.good} bonne{score.good > 1 ? "s" : ""} réponse
          {score.good > 1 ? "s" : ""} sur {score.total} — {pct} %
        </p>
      </div>
      {stats && (
        <p className="tabular text-xs text-muted-foreground">
          {stats.due_now} à revoir · {stats.mastered}/{stats.total_cards} maîtrisés
        </p>
      )}
      <div className="flex w-full flex-col gap-2 sm:flex-row sm:justify-center">
        <Button onClick={onContinue} className="gap-2">
          <RotateCcw /> Continuer
        </Button>
        <Link href="/" className={cn(buttonVariants({ variant: "outline" }))}>
          Terminer
        </Link>
      </div>
    </div>
  );
}

function QueueSkeleton() {
  return (
    <div className="flex flex-1 flex-col">
      <div className="flex flex-1 items-center justify-center py-6">
        <Skeleton className="size-32 rounded-2xl" />
      </div>
      <div className="grid gap-2.5">
        {[0, 1, 2, 3].map((i) => (
          <Skeleton key={i} className="h-14 w-full" />
        ))}
      </div>
      <Skeleton className="mt-7 h-12 w-full" />
    </div>
  );
}
