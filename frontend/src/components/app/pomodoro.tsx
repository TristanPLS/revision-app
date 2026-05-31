"use client";

import { useEffect, useState } from "react";
import { Timer, Play, Pause, RotateCcw, X } from "lucide-react";
import { toast } from "sonner";
import { cn } from "@/lib/utils";
import { Button } from "@/components/ui/button";

const WORK = 25 * 60;
const BREAK = 5 * 60;

export function Pomodoro() {
  const [open, setOpen] = useState(false);
  const [mode, setMode] = useState<"work" | "break">("work");
  const [left, setLeft] = useState(WORK);
  const [running, setRunning] = useState(false);

  useEffect(() => {
    if (!running) return;
    if (left <= 0) {
      const next = mode === "work" ? "break" : "work";
      setMode(next);
      setLeft(next === "work" ? WORK : BREAK);
      toast(next === "break" ? "Pause de 5 min — éloigne-toi de l'écran." : "Reprise du travail.");
      return;
    }
    const t = setTimeout(() => setLeft((l) => l - 1), 1000);
    return () => clearTimeout(t);
  }, [running, left, mode]);

  const mm = String(Math.floor(left / 60)).padStart(2, "0");
  const ss = String(left % 60).padStart(2, "0");

  function reset() {
    setRunning(false);
    setMode("work");
    setLeft(WORK);
  }

  if (!open) {
    return (
      <button
        onClick={() => setOpen(true)}
        aria-label="Ouvrir le minuteur Pomodoro"
        className="fixed bottom-4 right-4 z-40 flex size-12 items-center justify-center rounded-full border bg-card text-foreground shadow-sm transition-colors hover:bg-accent"
      >
        <Timer className="size-5" />
      </button>
    );
  }

  return (
    <div className="fixed bottom-4 right-4 z-40 w-56 rounded-xl border bg-card p-4 shadow-lg">
      <div className="mb-2 flex items-center justify-between">
        <span className="text-sm font-medium">Pomodoro</span>
        <button
          onClick={() => setOpen(false)}
          aria-label="Fermer"
          className="text-muted-foreground hover:text-foreground"
        >
          <X className="size-4" />
        </button>
      </div>
      <p className={cn("text-xs", mode === "work" ? "text-primary" : "text-rate-easy")}>
        {mode === "work" ? "Travail" : "Pause"}
      </p>
      <p className="tabular my-1 text-4xl font-semibold leading-none">
        {mm}:{ss}
      </p>
      <div className="mt-3 flex gap-2">
        <Button size="sm" className="flex-1" onClick={() => setRunning((r) => !r)}>
          {running ? <Pause className="size-4" /> : <Play className="size-4" />}
          {running ? "Pause" : "Démarrer"}
        </Button>
        <Button size="sm" variant="outline" onClick={reset} aria-label="Réinitialiser">
          <RotateCcw className="size-4" />
        </Button>
      </div>
      <p className="mt-3 text-[11px] leading-tight text-muted-foreground">
        Gestion de la fatigue — pas une méthode d&apos;apprentissage en soi.
      </p>
    </div>
  );
}
