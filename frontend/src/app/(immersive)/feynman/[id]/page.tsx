"use client";

import { useEffect, useRef, useState } from "react";
import Link from "next/link";
import { useParams } from "next/navigation";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { X, Lightbulb, Hand, Loader2, RotateCcw } from "lucide-react";
import { toast } from "sonner";
import { api, ApiError } from "@/lib/api/client";
import { cn } from "@/lib/utils";
import { Button, buttonVariants } from "@/components/ui/button";
import { Card, CardContent } from "@/components/ui/card";
import { Textarea } from "@/components/ui/textarea";
import { Skeleton } from "@/components/ui/skeleton";
import type { FeynmanAttempt } from "@/lib/api/types";

type Phase = "explain" | "rate" | "done";

export default function FeynmanPracticePage() {
  const { id } = useParams<{ id: string }>();
  const qc = useQueryClient();

  const concept = useQuery({ queryKey: ["feynman-concept", id], queryFn: () => api.feynman.get(id) });

  const [phase, setPhase] = useState<Phase>("explain");
  const [elapsed, setElapsed] = useState(0);
  const [duration, setDuration] = useState(0);
  const [hesitations, setHesitations] = useState(0);
  const [rating, setRating] = useState<number | null>(null);
  const [explanation, setExplanation] = useState("");
  const [result, setResult] = useState<FeynmanAttempt | null>(null);
  const timerOn = useRef(true);

  useEffect(() => {
    const t = setInterval(() => {
      if (timerOn.current) setElapsed((e) => e + 1);
    }, 1000);
    return () => clearInterval(t);
  }, []);

  const submit = useMutation({
    mutationFn: () =>
      api.feynman.attempt(id, {
        self_rating: rating ?? undefined,
        hesitations,
        duration_s: duration,
        explanation: explanation.trim() || undefined,
      }),
    onSuccess: (att) => {
      setResult(att);
      setPhase("done");
      qc.invalidateQueries({ queryKey: ["feynman", concept.data?.subject_id] });
    },
    onError: (e: unknown) =>
      toast.error(e instanceof ApiError ? e.message : "Échec de l'enregistrement"),
  });

  const mm = String(Math.floor(elapsed / 60)).padStart(2, "0");
  const ss = String(elapsed % 60).padStart(2, "0");
  const overTime = (phase === "explain" ? elapsed : duration) > 120;
  const tooMany = hesitations > 3;
  const needsReview = overTime || tooMany;
  const subjectId = concept.data?.subject_id;

  function reset() {
    setPhase("explain");
    setElapsed(0);
    setDuration(0);
    setHesitations(0);
    setRating(null);
    setExplanation("");
    setResult(null);
    timerOn.current = true;
  }

  return (
    <div className="mx-auto flex min-h-screen max-w-2xl flex-col px-4 py-6">
      <div className="flex items-center gap-4">
        <Link
          href={subjectId ? `/subjects/${subjectId}` : "/"}
          className="text-muted-foreground hover:text-foreground"
          aria-label="Quitter"
        >
          <X className="size-5" />
        </Link>
        <p className="flex-1 text-sm font-medium text-muted-foreground">Feynman</p>
        <span
          className={cn(
            "tabular text-sm",
            phase === "explain" && overTime ? "font-medium text-rate-hard" : "text-muted-foreground"
          )}
        >
          {mm}:{ss}
        </span>
      </div>

      <div className="flex flex-1 flex-col items-center justify-center">
        {concept.isLoading || !concept.data ? (
          <Skeleton className="h-48 w-full" />
        ) : phase === "explain" ? (
          <div className="w-full space-y-6 text-center">
            <p className="text-xs uppercase tracking-wide text-muted-foreground">
              Explique à voix haute, comme à un enfant
            </p>
            <h1 className="text-2xl font-semibold leading-snug">{concept.data.title}</h1>
            <div className="flex flex-col items-center gap-3">
              <Button
                variant="outline"
                onClick={() => setHesitations((h) => h + 1)}
                className="gap-2"
              >
                <Hand /> J&apos;hésite ({hesitations})
              </Button>
              <Button
                size="lg"
                onClick={() => {
                  timerOn.current = false;
                  setDuration(elapsed);
                  setPhase("rate");
                }}
              >
                J&apos;ai fini d&apos;expliquer
              </Button>
            </div>
          </div>
        ) : phase === "rate" ? (
          <div className="w-full space-y-5">
            <Card>
              <CardContent className="space-y-3 pt-6">
                <p className="font-medium">{concept.data.title}</p>
                {concept.data.hint && (
                  <p className="flex items-start gap-2 rounded-md bg-accent px-3 py-2 text-sm text-accent-foreground">
                    <Lightbulb className="mt-0.5 size-4 shrink-0" />
                    <span>{concept.data.hint}</span>
                  </p>
                )}
                <p className="tabular text-sm text-muted-foreground">
                  {Math.floor(duration / 60)}:{String(duration % 60).padStart(2, "0")} ·{" "}
                  {hesitations} hésitation{hesitations > 1 ? "s" : ""}
                  {needsReview && (
                    <span className="ml-1 font-medium text-rate-hard">→ à revoir</span>
                  )}
                </p>
              </CardContent>
            </Card>

            <div className="space-y-2">
              <p className="text-sm font-medium">Auto-évaluation</p>
              <div className="grid grid-cols-5 gap-2">
                {[1, 2, 3, 4, 5].map((r) => (
                  <button
                    key={r}
                    onClick={() => setRating(r)}
                    className={cn(
                      "h-11 rounded-md border text-sm font-medium transition-colors focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring",
                      rating === r ? "border-primary bg-primary/10" : "hover:bg-accent"
                    )}
                  >
                    {r}
                  </button>
                ))}
              </div>
            </div>

            <div className="space-y-2">
              <p className="text-sm font-medium">
                Retour IA <span className="text-muted-foreground">(optionnel)</span>
              </p>
              <Textarea
                value={explanation}
                onChange={(e) => setExplanation(e.target.value)}
                placeholder="Tape ton explication pour obtenir un retour de l'IA…"
              />
            </div>

            <Button
              className="w-full gap-2"
              disabled={rating === null || submit.isPending}
              onClick={() => submit.mutate()}
            >
              {submit.isPending && <Loader2 className="animate-spin" />}
              Valider
            </Button>
          </div>
        ) : (
          <div className="w-full space-y-4 text-center">
            <p className="text-xl font-semibold">Enregistré 👍</p>
            {result?.ai_score != null && (
              <p className="tabular text-muted-foreground">Note IA : {result.ai_score}/100</p>
            )}
            {result?.ai_feedback && (
              <p className="rounded-md bg-muted px-4 py-3 text-left text-sm text-muted-foreground">
                {result.ai_feedback}
              </p>
            )}
            <div className="flex justify-center gap-2">
              <Button variant="outline" onClick={reset} className="gap-2">
                <RotateCcw className="size-4" /> Recommencer
              </Button>
              <Link
                href={subjectId ? `/subjects/${subjectId}` : "/"}
                className={cn(buttonVariants())}
              >
                Terminer
              </Link>
            </div>
          </div>
        )}
      </div>
    </div>
  );
}
