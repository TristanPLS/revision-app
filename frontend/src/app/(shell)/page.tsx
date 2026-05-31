"use client";

import { useState } from "react";
import Link from "next/link";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { Flame, Plus, Layers, Sparkles } from "lucide-react";
import { toast } from "sonner";
import { api, ApiError } from "@/lib/api/client";
import { cn } from "@/lib/utils";
import { Button, buttonVariants } from "@/components/ui/button";
import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from "@/components/ui/card";
import { Input } from "@/components/ui/input";
import { Badge } from "@/components/ui/badge";
import { Skeleton } from "@/components/ui/skeleton";

export default function DashboardPage() {
  const qc = useQueryClient();
  const subjects = useQuery({ queryKey: ["subjects"], queryFn: api.subjects.list });
  const guardrails = useQuery({ queryKey: ["guardrails"], queryFn: api.guardrails });
  const [name, setName] = useState("");

  const createSubject = useMutation({
    mutationFn: () => api.subjects.create({ name: name.trim() }),
    onSuccess: () => {
      setName("");
      qc.invalidateQueries({ queryKey: ["subjects"] });
      toast.success("Matière créée");
    },
    onError: (e: unknown) =>
      toast.error(e instanceof ApiError ? e.message : "Échec de la création"),
  });

  const totalDue =
    subjects.data?.reduce((sum, s) => sum + s.due_count, 0) ?? 0;
  const streak = guardrails.data?.streak_days ?? 0;

  return (
    <div className="space-y-10">
      <header className="flex flex-wrap items-end justify-between gap-4">
        <div className="space-y-1">
          <h1 className="text-2xl font-semibold tracking-tight sm:text-3xl">Aujourd&apos;hui</h1>
          <p className="text-muted-foreground">
            {totalDue > 0 ? (
              <>
                <span className="tabular font-medium text-foreground">{totalDue}</span>{" "}
                carte{totalDue > 1 ? "s" : ""} à réviser, toutes matières confondues.
              </>
            ) : (
              "Rien à réviser pour l'instant — ajoute une matière et génère des cartes."
            )}
          </p>
        </div>
        {streak > 0 && (
          <Badge className="gap-1.5 px-3 py-1 text-sm">
            <Flame className="size-4" />
            <span className="tabular">{streak} j</span> de série
          </Badge>
        )}
      </header>

      <section className="space-y-3">
        <form
          className="flex gap-2"
          onSubmit={(e) => {
            e.preventDefault();
            if (name.trim() && !createSubject.isPending) createSubject.mutate();
          }}
        >
          <Input
            value={name}
            onChange={(e) => setName(e.target.value)}
            placeholder="Nouvelle matière (ex. Biodiversité)"
            aria-label="Nom de la matière"
          />
          <Button type="submit" disabled={!name.trim() || createSubject.isPending}>
            <Plus /> Ajouter
          </Button>
        </form>
      </section>

      <section className="space-y-4">
        <h2 className="text-sm font-medium text-muted-foreground">Mes matières</h2>

        {subjects.isLoading ? (
          <div className="grid gap-4 sm:grid-cols-2">
            <Skeleton className="h-36" />
            <Skeleton className="h-36" />
          </div>
        ) : subjects.data && subjects.data.length > 0 ? (
          <div className="grid gap-4 sm:grid-cols-2">
            {subjects.data.map((s) => (
              <Card key={s.id} className="flex flex-col transition-colors hover:border-primary/40">
                <CardHeader>
                  <CardTitle>
                    <Link href={`/subjects/${s.id}`} className="hover:underline">
                      {s.name}
                    </Link>
                  </CardTitle>
                  {s.description && <CardDescription>{s.description}</CardDescription>}
                </CardHeader>
                <CardContent className="mt-auto flex items-center justify-between gap-2">
                  <div className="flex flex-wrap gap-2">
                    <Badge variant="muted" className="gap-1">
                      <Layers className="size-3.5" />
                      <span className="tabular">{s.card_count}</span> cartes
                    </Badge>
                    {s.due_count > 0 && (
                      <Badge className="tabular">{s.due_count} à réviser</Badge>
                    )}
                  </div>
                  {s.due_count > 0 ? (
                    <Link
                      href={`/review?subject=${s.id}`}
                      className={cn(buttonVariants({ size: "sm" }))}
                    >
                      Réviser
                    </Link>
                  ) : (
                    <Link
                      href={`/subjects/${s.id}`}
                      className={cn(buttonVariants({ variant: "outline", size: "sm" }))}
                    >
                      Ouvrir
                    </Link>
                  )}
                </CardContent>
              </Card>
            ))}
          </div>
        ) : (
          <Card>
            <CardContent className="flex flex-col items-center gap-3 py-12 text-center">
              <span className="flex size-12 items-center justify-center rounded-full bg-accent text-accent-foreground">
                <Sparkles className="size-6" />
              </span>
              <p className="max-w-sm text-sm text-muted-foreground">
                Crée ta première matière ci-dessus, puis colle ton cours : l&apos;IA en
                générera des flashcards prêtes à réviser.
              </p>
            </CardContent>
          </Card>
        )}
      </section>
    </div>
  );
}
