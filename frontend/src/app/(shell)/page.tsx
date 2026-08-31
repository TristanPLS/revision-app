"use client";

import { useState } from "react";
import Link from "next/link";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import {
  BookOpenCheck,
  Flag,
  Flame,
  KeyRound,
  Landmark,
  Plus,
  Layers,
  Sparkles,
  type LucideIcon,
} from "lucide-react";
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
import { Progress } from "@/components/ui/progress";
import { Skeleton } from "@/components/ui/skeleton";
import { EmptyState } from "@/components/app/empty-state";
import type { GeoStats } from "@/lib/api/types";

export default function DashboardPage() {
  const qc = useQueryClient();
  const subjects = useQuery({ queryKey: ["subjects"], queryFn: api.subjects.list });
  const guardrails = useQuery({ queryKey: ["guardrails"], queryFn: api.guardrails });
  const settings = useQuery({ queryKey: ["settings"], queryFn: api.settings.get });
  // Mêmes clés que les pages géo : une série jouée invalide ces stats au retour.
  const flagStats = useQuery({
    queryKey: ["geo-stats", "flag"],
    queryFn: () => api.geo.stats("flag"),
  });
  const capitalStats = useQuery({
    queryKey: ["geo-stats", "capital"],
    queryFn: () => api.geo.stats("capital"),
  });
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
  const totalCards =
    subjects.data?.reduce((sum, s) => sum + s.card_count, 0) ?? 0;
  const streak = guardrails.data?.streak_days ?? 0;

  return (
    <div className="space-y-10">
      <header className="space-y-1">
        <h1 className="text-2xl font-semibold tracking-tight sm:text-3xl">Aujourd&apos;hui</h1>
        <p className="text-muted-foreground">
          {totalDue > 0
            ? "Ta file de révision t'attend — la régularité bat l'intensité."
            : "Rien à réviser pour l'instant — ajoute une matière et génère des cartes."}
        </p>
      </header>

      {/* Stats héro */}
      <section className="grid grid-cols-3 gap-2 sm:gap-4">
        <StatTile
          icon={BookOpenCheck}
          value={totalDue}
          label={`carte${totalDue > 1 ? "s" : ""} à réviser`}
          highlight={totalDue > 0}
        />
        <StatTile
          icon={Flame}
          value={streak}
          label={`jour${streak > 1 ? "s" : ""} de série`}
          highlight={streak > 0}
        />
        <StatTile icon={Layers} value={totalCards} label="cartes au total" />
      </section>

      {settings.data && !settings.data.configured && (
        <Card className="border-primary/40 bg-primary/5">
          <CardContent className="flex flex-wrap items-center justify-between gap-3 py-4">
            <div className="flex items-center gap-3">
              <span className="flex size-9 shrink-0 items-center justify-center rounded-full bg-primary/10 text-primary">
                <KeyRound className="size-4" />
              </span>
              <p className="text-sm text-muted-foreground">
                <span className="font-medium text-foreground">
                  L&apos;IA n&apos;est pas encore configurée.
                </span>{" "}
                Ajoute ta clé API (gratuite avec Google AI Studio) pour générer
                flashcards, examens et fiches depuis tes cours.
              </p>
            </div>
            <Link href="/settings" className={cn(buttonVariants({ size: "sm" }))}>
              Configurer
            </Link>
          </CardContent>
        </Card>
      )}

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
              <Card key={s.id} className="relative flex flex-col transition-colors hover:border-primary/40">
                <CardHeader>
                  <CardTitle>
                    <Link
                      href={`/subjects/${s.id}`}
                      className="hover:underline after:absolute after:inset-0"
                    >
                      {s.name}
                    </Link>
                  </CardTitle>
                  {s.description && <CardDescription>{s.description}</CardDescription>}
                </CardHeader>
                <CardContent className="mt-auto space-y-3">
                  {s.card_count > 0 && <MasteryBar due={s.due_count} total={s.card_count} />}
                  <div className="flex items-center justify-between gap-2">
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
                        className={cn(buttonVariants({ size: "sm" }), "relative z-10")}
                      >
                        Réviser
                      </Link>
                    ) : (
                      <Link
                        href={`/subjects/${s.id}`}
                        className={cn(buttonVariants({ variant: "outline", size: "sm" }), "relative z-10")}
                      >
                        Ouvrir
                      </Link>
                    )}
                  </div>
                </CardContent>
              </Card>
            ))}
          </div>
        ) : (
          <EmptyState icon={Sparkles}>
            Crée ta première matière ci-dessus, puis colle ton cours : l&apos;IA en
            générera des flashcards prêtes à réviser.
          </EmptyState>
        )}
      </section>

      <section className="space-y-4">
        <h2 className="text-sm font-medium text-muted-foreground">Géographie</h2>
        <div className="grid gap-4 sm:grid-cols-2">
          <GeoCard
            href="/geo/flags"
            icon={Flag}
            title="Devine le drapeau"
            description="Retrouve le pays parmi quatre propositions."
            stats={flagStats.data}
            loading={flagStats.isPending}
          />
          <GeoCard
            href="/geo/capitals"
            icon={Landmark}
            title="Capitales"
            description="Écris la capitale de mémoire, sans choix multiple."
            stats={capitalStats.data}
            loading={capitalStats.isPending}
          />
        </div>
      </section>
    </div>
  );
}

/** Tuile de stat héro : grand chiffre en display, libellé discret. */
function StatTile({
  icon: Icon,
  value,
  label,
  highlight = false,
}: {
  icon: typeof Flame;
  value: number;
  label: string;
  highlight?: boolean;
}) {
  return (
    <Card className={cn(highlight && "border-primary/40")}>
      <CardContent className="flex flex-col gap-1 px-3 py-4 sm:px-6 sm:py-5">
        <Icon
          className={cn("size-4", highlight ? "text-primary" : "text-muted-foreground")}
        />
        <span className="tabular font-display text-3xl font-semibold leading-none sm:text-4xl">
          {value}
        </span>
        <span className="text-xs text-muted-foreground sm:text-sm">{label}</span>
      </CardContent>
    </Card>
  );
}

/** Carte d'entrée d'un mode géo : même trame que les cartes matières. */
function GeoCard({
  href,
  icon: Icon,
  title,
  description,
  stats,
  loading,
}: {
  href: string;
  icon: LucideIcon;
  title: string;
  description: string;
  stats?: GeoStats;
  loading: boolean;
}) {
  const pct =
    stats && stats.total_cards > 0
      ? Math.round((stats.mastered / stats.total_cards) * 100)
      : 0;
  return (
    <Card className="relative flex flex-col transition-colors hover:border-primary/40">
      <CardHeader>
        <CardTitle className="flex items-center gap-2">
          <Icon className="size-4 shrink-0 text-muted-foreground" />
          <Link href={href} className="hover:underline after:absolute after:inset-0">
            {title}
          </Link>
        </CardTitle>
        <CardDescription>{description}</CardDescription>
      </CardHeader>
      <CardContent className="mt-auto space-y-3">
        {loading ? (
          <Skeleton className="h-10" />
        ) : stats ? (
          <>
            <Progress
              value={pct}
              className="h-1.5"
              aria-label={`${pct} % de cartes maîtrisées`}
            />
            <div className="flex flex-wrap gap-2">
              <Badge variant="muted" className="tabular">
                {stats.mastered}/{stats.total_cards} maîtrisées
              </Badge>
              {stats.due_now > 0 && (
                <Badge className="tabular">{stats.due_now} à réviser</Badge>
              )}
            </div>
          </>
        ) : null}
      </CardContent>
    </Card>
  );
}

/** Barre fine « à jour » : part des cartes qui ne sont pas dues. */
function MasteryBar({ due, total }: { due: number; total: number }) {
  const pct = Math.round(((total - due) / total) * 100);
  return (
    <div className="space-y-1">
      <div className="h-1.5 overflow-hidden rounded-full bg-secondary">
        <div
          className="h-full rounded-full bg-primary transition-[width] duration-500"
          style={{ width: `${pct}%` }}
        />
      </div>
      <p className="tabular text-xs text-muted-foreground">à jour à {pct} %</p>
    </div>
  );
}
