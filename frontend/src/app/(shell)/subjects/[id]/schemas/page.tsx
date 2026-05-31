"use client";

import { Suspense, useState } from "react";
import Link from "next/link";
import dynamic from "next/dynamic";
import { useParams, useRouter, useSearchParams } from "next/navigation";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { ArrowLeft, Plus, Eye, EyeOff } from "lucide-react";
import { toast } from "sonner";
import { api, ApiError } from "@/lib/api/client";
import { cn } from "@/lib/utils";
import { Button, buttonVariants } from "@/components/ui/button";
import { Card, CardContent } from "@/components/ui/card";
import { Input } from "@/components/ui/input";
import { Textarea } from "@/components/ui/textarea";
import { Label } from "@/components/ui/label";
import { Skeleton } from "@/components/ui/skeleton";

const SchemaCanvas = dynamic(() => import("@/components/app/schema-canvas"), {
  ssr: false,
  loading: () => <Skeleton className="h-[60vh]" />,
});

function SchemasInner() {
  const { id } = useParams<{ id: string }>();
  const sp = useSearchParams();
  const schemaId = sp.get("schema");
  const list = useQuery({ queryKey: ["schemas", id], queryFn: () => api.schemas.list(id) });

  return (
    <div className="space-y-8">
      <Link
        href={`/subjects/${id}`}
        className="inline-flex items-center gap-1.5 text-sm text-muted-foreground hover:text-foreground"
      >
        <ArrowLeft className="size-4" /> Retour à la matière
      </Link>

      <header className="space-y-1">
        <h1 className="text-2xl font-semibold tracking-tight sm:text-3xl">Schémas</h1>
        <p className="text-muted-foreground">
          Dessine le schéma <strong>de mémoire</strong>, puis révèle la référence pour comparer
          (dual coding).
        </p>
      </header>

      <div className="grid gap-6 md:grid-cols-[220px_1fr]">
        <aside className="space-y-2">
          <Link
            href={`/subjects/${id}/schemas`}
            className={cn(buttonVariants({ variant: schemaId ? "outline" : "default", size: "sm" }), "w-full")}
          >
            <Plus className="size-4" /> Nouveau schéma
          </Link>
          <nav className="space-y-1">
            {list.data?.map((s) => (
              <Link
                key={s.id}
                href={`/subjects/${id}/schemas?schema=${s.id}`}
                className={cn(
                  "block truncate rounded-md px-3 py-2 text-sm transition-colors",
                  schemaId === s.id
                    ? "bg-accent text-accent-foreground"
                    : "text-muted-foreground hover:bg-accent hover:text-accent-foreground"
                )}
              >
                {s.title}
              </Link>
            ))}
          </nav>
        </aside>

        <div>{schemaId ? <SchemaView schemaId={schemaId} /> : <NewSchema subjectId={id} />}</div>
      </div>
    </div>
  );
}

function NewSchema({ subjectId }: { subjectId: string }) {
  const qc = useQueryClient();
  const router = useRouter();
  const [title, setTitle] = useState("");
  const [reference, setReference] = useState("");

  const create = useMutation({
    mutationFn: () =>
      api.schemas.create(subjectId, { title: title.trim(), reference: reference.trim() || undefined }),
    onSuccess: (sch) => {
      qc.invalidateQueries({ queryKey: ["schemas", subjectId] });
      router.push(`/subjects/${subjectId}/schemas?schema=${sch.id}`);
    },
    onError: (e: unknown) => toast.error(e instanceof ApiError ? e.message : "Échec"),
  });

  return (
    <Card>
      <CardContent className="space-y-4 pt-6">
        <div className="space-y-2">
          <Label htmlFor="title">Titre du schéma</Label>
          <Input
            id="title"
            value={title}
            onChange={(e) => setTitle(e.target.value)}
            placeholder="Pyramide de biomasse 96/4"
          />
        </div>
        <div className="space-y-2">
          <Label htmlFor="ref">Référence à comparer (optionnel)</Label>
          <Textarea
            id="ref"
            value={reference}
            onChange={(e) => setReference(e.target.value)}
            placeholder="Ce que le schéma doit contenir (révélé après ton dessin)…"
          />
        </div>
        <Button disabled={!title.trim() || create.isPending} onClick={() => create.mutate()}>
          Créer & dessiner
        </Button>
      </CardContent>
    </Card>
  );
}

function SchemaView({ schemaId }: { schemaId: string }) {
  const qc = useQueryClient();
  const [showRef, setShowRef] = useState(false);
  const schema = useQuery({ queryKey: ["schema", schemaId], queryFn: () => api.schemas.get(schemaId) });

  const save = useMutation({
    mutationFn: (drawing: unknown) => api.schemas.update(schemaId, { drawing }),
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: ["schema", schemaId] });
      qc.invalidateQueries({ queryKey: ["schemas", schema.data?.subject_id] });
      toast.success("Dessin enregistré");
    },
    onError: (e: unknown) => toast.error(e instanceof ApiError ? e.message : "Échec"),
  });

  if (schema.isLoading || !schema.data) return <Skeleton className="h-[60vh]" />;

  return (
    <div className="space-y-4">
      <div className="flex flex-wrap items-center justify-between gap-2">
        <h2 className="text-lg font-semibold">{schema.data.title}</h2>
        {schema.data.reference && (
          <Button variant="outline" size="sm" onClick={() => setShowRef((v) => !v)}>
            {showRef ? <EyeOff className="size-4" /> : <Eye className="size-4" />}
            {showRef ? "Cacher la référence" : "Comparer"}
          </Button>
        )}
      </div>

      {showRef && schema.data.reference && (
        <div className="rounded-md bg-accent px-4 py-3 text-sm text-accent-foreground">
          <p className="mb-1 text-xs uppercase tracking-wide">Référence</p>
          <p className="whitespace-pre-wrap">{schema.data.reference}</p>
        </div>
      )}

      <SchemaCanvas
        initial={schema.data.drawing}
        saving={save.isPending}
        onSave={(snap) => save.mutate(snap)}
      />
    </div>
  );
}

export default function SchemasPage() {
  return (
    <Suspense fallback={null}>
      <SchemasInner />
    </Suspense>
  );
}
