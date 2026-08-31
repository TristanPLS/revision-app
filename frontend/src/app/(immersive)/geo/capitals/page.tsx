"use client";

import { useRef, useState } from "react";
import Link from "next/link";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import {
  Check,
  Globe,
  Loader2,
  PartyPopper,
  RotateCcw,
  TriangleAlert,
  X,
} from "lucide-react";
import { toast } from "sonner";
import { api, ApiError } from "@/lib/api/client";
import { cn, flagSrc } from "@/lib/utils";
import { Button, buttonVariants } from "@/components/ui/button";
import { Card } from "@/components/ui/card";
import { Input } from "@/components/ui/input";
import { Progress } from "@/components/ui/progress";
import { Skeleton } from "@/components/ui/skeleton";
import { EmptyState } from "@/components/app/empty-state";
import type { GeoAnswerResponse, GeoCapitalItem } from "@/lib/api/types";

// Libellés exacts attendus par le filtre backend (référentiel figé par la
// migration 0010_geo.sql) : évite de télécharger les 197 pays pour 6 boutons.
const CONTINENTS = ["Afrique", "Amériques", "Asie", "Europe", "Océanie"];
const BATCH = 20;

type Score = { answered: number; correct: number };

export default function GeoCapitalsPage() {
  const qc = useQueryClient();
  const inputRef = useRef<HTMLInputElement>(null);

  const [continent, setContinent] = useState<string | null>(null);
  // `round` entre dans la clé de requête : sans lui, relancer une série ressert
  // la file du cache (cartes déjà jouées) le temps que le refetch réponde.
  const [round, setRound] = useState(0);
  const [pos, setPos] = useState(0);
  const [given, setGiven] = useState("");
  const [result, setResult] = useState<GeoAnswerResponse | null>(null);
  const [score, setScore] = useState<Score>({ answered: 0, correct: 0 });

  const queue = useQuery({
    queryKey: ["geo-queue", "capital", continent, round],
    queryFn: () =>
      api.geo.queue("capital", {
        continent: continent ?? undefined,
        limit: BATCH,
      }),
  });

  const answer = useMutation({
    mutationFn: ({ cardId, value }: { cardId: string; value: string }) =>
      api.geo.answer(cardId, value),
    onSuccess: (r) => {
      setResult(r);
      setScore((s) => ({
        answered: s.answered + 1,
        correct: s.correct + (r.correct ? 1 : 0),
      }));
      qc.invalidateQueries({ queryKey: ["geo-stats"] });
    },
    onError: (e: unknown) =>
      toast.error(e instanceof ApiError ? e.message : "Réponse non enregistrée"),
  });

  // La file est une union discriminée (drapeaux + capitales) : on la réduit à la
  // branche « capital », seule à porter country_name.
  const cards = (queue.data ?? []).filter(
    (i): i is GeoCapitalItem => i.kind === "capital"
  );
  const total = cards.length;
  const current = cards[pos];
  const finished = total > 0 && pos >= total;

  /// Nouvelle série. Le score reste celui de la session (comme sur /geo/flags) ;
  /// seul un changement de filtre le remet à zéro, pour qu'il décrive toujours
  /// un ensemble de cartes cohérent.
  function newRound() {
    setPos(0);
    setGiven("");
    setResult(null);
    setRound((r) => r + 1);
  }

  function pickContinent(next: string | null) {
    if (next === continent) return;
    setContinent(next);
    setScore({ answered: 0, correct: 0 });
    newRound();
  }

  function submit(e: React.FormEvent) {
    e.preventDefault();
    // Focus réaffirmé dans le geste utilisateur : sur iOS un focus() différé ne
    // rouvre pas le clavier, et on veut enchaîner sans re-taper l'écran.
    inputRef.current?.focus();
    if (result) {
      setResult(null);
      setGiven("");
      setPos((p) => p + 1);
      return;
    }
    const value = given.trim();
    if (!value || !current || answer.isPending) return;
    answer.mutate({ cardId: current.card_id, value });
  }

  return (
    <div className="mx-auto flex min-h-dvh max-w-2xl flex-col px-4 pt-6">
      <div className="flex items-center gap-4">
        <Link
          href="/"
          className="-ml-3 flex size-11 shrink-0 items-center justify-center text-muted-foreground hover:text-foreground"
          aria-label="Quitter les capitales"
        >
          <X className="size-5" />
        </Link>
        <div className="flex-1">
          <Progress value={total ? (pos / total) * 100 : 0} />
        </div>
        {score.answered > 0 && (
          <span className="tabular text-sm text-muted-foreground">
            {score.correct}/{score.answered}
          </span>
        )}
      </div>

      <div className="mt-4 flex flex-wrap justify-center gap-2">
        {[null, ...CONTINENTS].map((c) => (
          <button
            key={c ?? "tous"}
            type="button"
            onClick={() => pickContinent(c)}
            aria-pressed={continent === c}
            className={cn(
              "h-11 rounded-full border px-4 text-sm font-medium transition-colors focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring",
              continent === c
                ? "border-primary bg-primary/10 text-primary"
                : "text-muted-foreground active:bg-accent"
            )}
          >
            {c ?? "Tous"}
          </button>
        ))}
      </div>

      <div className="flex flex-1 flex-col items-center justify-center py-6">
        {queue.isPending ? (
          <div className="w-full space-y-4">
            <Skeleton className="h-44 w-full" />
            <Skeleton className="h-11 w-full" />
          </div>
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
            La file n&apos;a pas pu être chargée. Vérifie que le serveur répond,
            puis réessaie.
          </EmptyState>
        ) : finished ? (
          <Done score={score} onContinue={newRound} />
        ) : current ? (
          <div className="w-full">
            <p className="tabular mb-2 text-center text-xs uppercase tracking-wide text-muted-foreground">
              {pos + 1} / {total}
            </p>
            {/* key = remontage à chaque carte → l'animation d'entrée rejoue */}
            <Card
              key={current.card_id}
              className="animate-reveal p-6 text-center sm:p-8"
            >
              {/* eslint-disable-next-line @next/next/no-img-element */}
              <img
                src={flagSrc(current.iso2)}
                alt=""
                width={72}
                height={48}
                className="mx-auto h-auto w-16 rounded border"
              />
              <h1 className="mt-3 break-words text-2xl font-semibold leading-snug">
                Capitale de {current.country_name} ?
              </h1>
            </Card>
            {result && <Correction result={result} />}
          </div>
        ) : (
          <EmptyState
            icon={Globe}
            action={
              continent ? (
                <Button variant="outline" onClick={() => pickContinent(null)}>
                  Tous les continents
                </Button>
              ) : (
                <Link href="/" className={cn(buttonVariants({ variant: "outline" }))}>
                  Accueil
                </Link>
              )
            }
          >
            Aucune carte de capitale {continent ? "sur ce continent" : "pour le moment"}.
          </EmptyState>
        )}
      </div>

      {/* Le champ reste monté (et modifiable) pendant la correction : le
          démonter fermerait le clavier virtuel entre deux questions. */}
      {current && !queue.isError && (
        <form
          onSubmit={submit}
          /* La marge négative ne vaut que tant que le conteneur occupe toute la
             largeur : passé max-w-2xl, elle déborderait du viewport. */
          className="sticky bottom-0 border-t bg-background px-4 pt-3 pb-[calc(0.75rem+env(safe-area-inset-bottom))] max-sm:-mx-4"
        >
          <Input
            ref={inputRef}
            value={given}
            onChange={(e) => setGiven(e.target.value)}
            placeholder="Tape la capitale…"
            aria-label={`Capitale de ${current.country_name}`}
            autoFocus
            autoCapitalize="off"
            autoCorrect="off"
            autoComplete="off"
            spellCheck={false}
            enterKeyHint={result ? "next" : "go"}
          />
          <p className="mt-1.5 text-xs text-muted-foreground">
            Les accents et une petite faute de frappe sont tolérés.
          </p>
          <Button
            type="submit"
            size="lg"
            className="mt-2 w-full"
            disabled={!result && (!given.trim() || answer.isPending)}
          >
            {answer.isPending && <Loader2 className="animate-spin" />}
            {result ? "Suivant" : "Valider"}
          </Button>
        </form>
      )}
    </div>
  );
}

/** Verdict + réponse attendue — jamais rendu avant l'envoi de la réponse. */
function Correction({ result }: { result: GeoAnswerResponse }) {
  const ok = result.correct;
  return (
    <Card
      role="status"
      className={cn(
        "animate-reveal mt-4 border-2 p-5 text-center",
        ok ? "border-rate-good bg-rate-good/10" : "border-rate-again bg-rate-again/10"
      )}
    >
      <span
        className={cn(
          "inline-flex items-center gap-1.5 rounded-full px-3 py-1 text-sm font-medium text-rate-foreground",
          ok ? "bg-rate-good" : "bg-rate-again"
        )}
      >
        {ok ? <Check className="size-4" /> : <X className="size-4" />}
        {ok ? "Bonne réponse" : "Pas cette fois"}
      </span>
      <p className="mt-3 break-words text-2xl font-semibold">{result.expected}</p>
      {result.accepted_alternatives.length > 0 && (
        <p className="mt-2 break-words text-sm text-muted-foreground">
          aussi accepté : {result.accepted_alternatives.join(", ")}
        </p>
      )}
    </Card>
  );
}

function Done({ score, onContinue }: { score: Score; onContinue: () => void }) {
  const pct = score.answered
    ? Math.round((score.correct / score.answered) * 100)
    : 0;
  return (
    <div className="flex flex-col items-center gap-4 text-center">
      <PartyPopper className="size-10 text-primary" />
      <div>
        <p className="text-xl font-semibold">Série terminée</p>
        <p className="tabular text-muted-foreground">
          {score.correct} bonne{score.correct > 1 ? "s" : ""} réponse
          {score.correct > 1 ? "s" : ""} sur {score.answered} — {pct} %
        </p>
      </div>
      <div className="flex flex-wrap justify-center gap-2">
        <Button onClick={onContinue} className="gap-2">
          <RotateCcw className="size-4" /> Continuer
        </Button>
        <Link href="/" className={cn(buttonVariants({ variant: "outline" }))}>
          Terminer
        </Link>
      </div>
    </div>
  );
}
