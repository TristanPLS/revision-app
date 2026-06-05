"use client";

import { Suspense, useState } from "react";
import Link from "next/link";
import { useParams, useRouter, useSearchParams } from "next/navigation";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { ArrowLeft, Plus, Loader2, ArrowRight, Check } from "lucide-react";
import { toast } from "sonner";
import { api, ApiError } from "@/lib/api/client";
import { cn } from "@/lib/utils";
import { Button, buttonVariants } from "@/components/ui/button";
import { Card, CardContent } from "@/components/ui/card";
import { Input } from "@/components/ui/input";
import { Textarea } from "@/components/ui/textarea";
import { Label } from "@/components/ui/label";
import { Skeleton } from "@/components/ui/skeleton";

function CornellInner() {
  const { id } = useParams<{ id: string }>();
  const sp = useSearchParams();
  const noteId = sp.get("note");
  const notes = useQuery({ queryKey: ["cornell", id], queryFn: () => api.cornell.list(id) });

  return (
    <div className="space-y-8">
      <Link
        href={`/subjects/${id}`}
        className="inline-flex items-center gap-1.5 text-sm text-muted-foreground hover:text-foreground"
      >
        <ArrowLeft className="size-4" /> Retour à la matière
      </Link>

      <header className="space-y-1">
        <h1 className="text-3xl font-semibold tracking-tight">Notes Cornell</h1>
        <p className="text-muted-foreground">
          Le corps à droite, tes questions de marge à gauche. Chaque question de marge se
          transforme en flashcard d&apos;un clic.
        </p>
      </header>

      <div className="grid gap-6 md:grid-cols-[220px_1fr]">
        <aside className="space-y-2">
          <Link
            href={`/subjects/${id}/cornell`}
            className={cn(buttonVariants({ variant: noteId ? "outline" : "default", size: "sm" }), "w-full")}
          >
            <Plus className="size-4" /> Nouvelle note
          </Link>
          <nav className="space-y-1">
            {notes.data?.map((n) => (
              <Link
                key={n.id}
                href={`/subjects/${id}/cornell?note=${n.id}`}
                className={cn(
                  "block truncate rounded-md px-3 py-2 text-sm transition-colors",
                  noteId === n.id
                    ? "bg-accent text-accent-foreground"
                    : "text-muted-foreground hover:bg-accent hover:text-accent-foreground"
                )}
              >
                {n.title}
              </Link>
            ))}
          </nav>
        </aside>

        <div>{noteId ? <NoteView noteId={noteId} subjectId={id} /> : <NoteEditor subjectId={id} />}</div>
      </div>
    </div>
  );
}

function NoteEditor({ subjectId }: { subjectId: string }) {
  const qc = useQueryClient();
  const router = useRouter();
  const [title, setTitle] = useState("");
  const [body, setBody] = useState("");
  const [summary, setSummary] = useState("");
  const [cues, setCues] = useState<{ question: string; answer: string }[]>([
    { question: "", answer: "" },
  ]);

  const save = useMutation({
    mutationFn: () =>
      api.cornell.create(subjectId, {
        title: title.trim(),
        body,
        summary: summary.trim() || undefined,
        cues: cues
          .filter((c) => c.question.trim())
          .map((c) => ({ question: c.question.trim(), answer: c.answer.trim() || undefined })),
      }),
    onSuccess: (note) => {
      qc.invalidateQueries({ queryKey: ["cornell", subjectId] });
      toast.success("Note créée");
      router.push(`/subjects/${subjectId}/cornell?note=${note.id}`);
    },
    onError: (e: unknown) => toast.error(e instanceof ApiError ? e.message : "Échec"),
  });

  return (
    <Card>
      <CardContent className="space-y-5 pt-6">
        <div className="space-y-2">
          <Label htmlFor="title">Titre</Label>
          <Input
            id="title"
            value={title}
            onChange={(e) => setTitle(e.target.value)}
            placeholder="Chapitre 1 — Constat"
          />
        </div>

        <div className="grid gap-4 md:grid-cols-[220px_1fr]">
          <div className="space-y-2">
            <Label>Questions de marge</Label>
            <div className="space-y-2">
              {cues.map((c, i) => (
                <div key={i} className="space-y-1 rounded-md border p-2">
                  <Input
                    value={c.question}
                    onChange={(e) =>
                      setCues((cs) => cs.map((x, j) => (j === i ? { ...x, question: e.target.value } : x)))
                    }
                    placeholder="Question…"
                    className="h-9"
                  />
                  <Input
                    value={c.answer}
                    onChange={(e) =>
                      setCues((cs) => cs.map((x, j) => (j === i ? { ...x, answer: e.target.value } : x)))
                    }
                    placeholder="Réponse (optionnel)"
                    className="h-9"
                  />
                </div>
              ))}
              <Button
                type="button"
                variant="outline"
                size="sm"
                className="w-full"
                onClick={() => setCues((cs) => [...cs, { question: "", answer: "" }])}
              >
                <Plus className="size-4" /> Question
              </Button>
            </div>
          </div>

          <div className="space-y-2">
            <Label htmlFor="body">Notes</Label>
            <Textarea
              id="body"
              value={body}
              onChange={(e) => setBody(e.target.value)}
              placeholder="Le cœur de ta prise de notes…"
              className="min-h-64"
            />
          </div>
        </div>

        <div className="space-y-2">
          <Label htmlFor="summary">Résumé (bas de page Cornell)</Label>
          <Textarea
            id="summary"
            value={summary}
            onChange={(e) => setSummary(e.target.value)}
            placeholder="En une ou deux phrases, l'essentiel…"
          />
        </div>

        <Button
          disabled={!title.trim() || !body.trim() || save.isPending}
          onClick={() => save.mutate()}
        >
          {save.isPending && <Loader2 className="animate-spin" />}
          Enregistrer
        </Button>
      </CardContent>
    </Card>
  );
}

function NoteView({ noteId, subjectId }: { noteId: string; subjectId: string }) {
  const qc = useQueryClient();
  const note = useQuery({ queryKey: ["cornell-note", noteId], queryFn: () => api.cornell.get(noteId) });

  const toFlashcard = useMutation({
    mutationFn: (cueId: string) => api.cornell.cueToFlashcard(cueId),
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: ["cornell-note", noteId] });
      qc.invalidateQueries({ queryKey: ["flashcards", subjectId] });
      qc.invalidateQueries({ queryKey: ["stats", subjectId] });
      toast.success("Flashcard créée");
    },
    onError: (e: unknown) => toast.error(e instanceof ApiError ? e.message : "Échec"),
  });

  if (note.isLoading || !note.data) return <Skeleton className="h-64" />;

  return (
    <Card>
      <CardContent className="space-y-5 pt-6">
        <h2 className="text-xl font-semibold">{note.data.title}</h2>

        <div className="grid gap-4 md:grid-cols-[220px_1fr]">
          <div className="space-y-2 md:border-r md:pr-4">
            <p className="text-xs uppercase tracking-wide text-muted-foreground">Questions</p>
            {note.data.cues.length === 0 && (
              <p className="text-sm text-muted-foreground">Aucune.</p>
            )}
            {note.data.cues.map((c) => (
              <div key={c.id} className="space-y-1 rounded-md border p-2">
                <p className="text-sm font-medium">{c.question}</p>
                {c.answer && <p className="text-sm text-muted-foreground">{c.answer}</p>}
                {c.flashcard_id ? (
                  <span className="inline-flex items-center gap-1 text-xs text-rate-good">
                    <Check className="size-3.5" /> Flashcard créée
                  </span>
                ) : (
                  <Button
                    variant="ghost"
                    size="sm"
                    className="h-8 px-2 text-xs"
                    disabled={toFlashcard.isPending}
                    onClick={() => toFlashcard.mutate(c.id)}
                  >
                    <ArrowRight className="size-3.5" /> Flashcard
                  </Button>
                )}
              </div>
            ))}
          </div>

          <div className="space-y-4">
            <p className="whitespace-pre-wrap text-sm leading-relaxed">{note.data.body}</p>
          </div>
        </div>

        {note.data.summary && (
          <div className="rounded-md bg-muted px-4 py-3">
            <p className="text-xs uppercase tracking-wide text-muted-foreground">Résumé</p>
            <p className="text-sm">{note.data.summary}</p>
          </div>
        )}
      </CardContent>
    </Card>
  );
}

export default function CornellPage() {
  return (
    <Suspense fallback={null}>
      <CornellInner />
    </Suspense>
  );
}
