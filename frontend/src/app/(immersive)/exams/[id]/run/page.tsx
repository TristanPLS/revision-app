"use client";

import { useEffect, useRef, useState } from "react";
import Link from "next/link";
import { useParams, useRouter } from "next/navigation";
import { useMutation, useQuery } from "@tanstack/react-query";
import { X, Loader2, Send } from "lucide-react";
import { toast } from "sonner";
import { api, ApiError } from "@/lib/api/client";
import { cn } from "@/lib/utils";
import { Button } from "@/components/ui/button";
import { Card, CardContent } from "@/components/ui/card";
import { Textarea } from "@/components/ui/textarea";
import { Input } from "@/components/ui/input";
import { Badge } from "@/components/ui/badge";
import { Skeleton } from "@/components/ui/skeleton";
import type { QuestionPublic } from "@/lib/api/types";

export default function ExamRunPage() {
  const { id } = useParams<{ id: string }>();
  const router = useRouter();
  const [answers, setAnswers] = useState<Record<string, string>>({});
  const [remaining, setRemaining] = useState<number | null>(null);
  const [hasFocus, setHasFocus] = useState(false);
  const [confirmArmed, setConfirmArmed] = useState(false);
  const submittedRef = useRef(false);
  const confirmTimer = useRef<ReturnType<typeof setTimeout> | null>(null);

  const exam = useQuery({ queryKey: ["exam", id], queryFn: () => api.exams.get(id) });
  const attempt = useQuery({
    queryKey: ["attempt-start", id],
    queryFn: () => api.exams.startAttempt(id),
    enabled: !!exam.data,
    staleTime: Infinity,
    refetchOnMount: false,
    refetchOnWindowFocus: false,
  });

  // Refs so the timer's auto-submit reads current values (no stale closures).
  // Updated after each commit (a bare effect runs on every render).
  const answersRef = useRef(answers);
  const examRef = useRef(exam.data);
  const attemptRef = useRef(attempt.data);
  useEffect(() => {
    answersRef.current = answers;
    examRef.current = exam.data;
    attemptRef.current = attempt.data;
  });

  const submit = useMutation({
    mutationFn: () => {
      const qs = examRef.current?.questions ?? [];
      const arr = qs.map((q) => ({
        question_id: q.id,
        response: answersRef.current[q.id] ?? null,
      }));
      return api.exams.submit(attemptRef.current!.attempt_id, arr);
    },
    onSuccess: () =>
      router.push(`/exams/${id}/results?attempt=${attemptRef.current!.attempt_id}`),
    onError: (e: unknown) => {
      submittedRef.current = false;
      toast.error(e instanceof ApiError ? e.message : "Échec de l'envoi");
    },
  });

  // Countdown from the server's started_at + time_limit; auto-submit at 0.
  useEffect(() => {
    const a = attempt.data;
    if (!a?.started_at || !a.time_limit_s) return;
    const deadline = new Date(a.started_at).getTime() + a.time_limit_s * 1000;
    const tick = () => {
      const rem = Math.max(0, Math.round((deadline - Date.now()) / 1000));
      setRemaining(rem);
      if (rem <= 0 && !submittedRef.current) {
        submittedRef.current = true;
        submit.mutate();
      }
    };
    tick();
    const t = setInterval(tick, 1000);
    return () => clearInterval(t);
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [attempt.data?.started_at, attempt.data?.time_limit_s]);

  const total = exam.data?.questions.length ?? 0;
  const answered = exam.data
    ? exam.data.questions.filter((q) => answers[q.id]?.trim()).length
    : 0;

  const mm = remaining != null ? String(Math.floor(remaining / 60)).padStart(2, "0") : "--";
  const ss = remaining != null ? String(remaining % 60).padStart(2, "0") : "--";
  const lowTime = remaining != null && remaining <= 60;

  function doSubmit() {
    if (submittedRef.current) return;
    submittedRef.current = true;
    submit.mutate();
  }

  const unanswered = total - answered;

  useEffect(
    () => () => {
      if (confirmTimer.current) clearTimeout(confirmTimer.current);
    },
    []
  );

  function handleFinish() {
    if (unanswered > 0 && !confirmArmed) {
      setConfirmArmed(true);
      if (confirmTimer.current) clearTimeout(confirmTimer.current);
      confirmTimer.current = setTimeout(() => setConfirmArmed(false), 4000);
      return;
    }
    if (confirmTimer.current) clearTimeout(confirmTimer.current);
    setConfirmArmed(false);
    doSubmit();
  }

  const armed = confirmArmed && unanswered > 0;

  return (
    <div className="mx-auto max-w-2xl px-4 py-6">
      <div className="sticky top-0 z-10 -mx-4 flex items-center gap-4 border-b bg-background/95 px-4 py-3 backdrop-blur">
        <Link
          href={exam.data ? `/subjects/${exam.data.subject_id}` : "/"}
          className="-m-3 flex size-11 items-center justify-center text-muted-foreground hover:text-foreground"
          aria-label="Quitter l'examen"
        >
          <X className="size-5" />
        </Link>
        <p className="flex-1 truncate text-sm font-medium">{exam.data?.title ?? "Examen"}</p>
        <span className="tabular text-sm text-muted-foreground">
          {answered}/{total}
        </span>
        <span
          className={cn(
            "tabular rounded-md px-2 py-1 text-sm font-medium",
            lowTime ? "bg-rate-again text-rate-foreground" : "bg-muted text-muted-foreground"
          )}
        >
          {mm}:{ss}
        </span>
      </div>

      {exam.isLoading || !exam.data ? (
        <div className="mt-6 space-y-4">
          <Skeleton className="h-40" />
          <Skeleton className="h-40" />
        </div>
      ) : (
        <>
          <p className="mt-6 rounded-md bg-accent px-3 py-2 text-center text-sm text-accent-foreground">
            Conditions réelles : cours fermé, à toi de jouer.
          </p>
          <div
            className="mt-4 space-y-4"
            onFocus={(e) => {
              if (isTextField(e.target)) setHasFocus(true);
            }}
            onBlur={(e) => {
              if (!isTextField(e.relatedTarget)) setHasFocus(false);
            }}
          >
            {exam.data.questions.map((q, i) => (
              <QuestionCard
                key={q.id}
                index={i + 1}
                q={q}
                value={answers[q.id] ?? ""}
                onChange={(v) => setAnswers((a) => ({ ...a, [q.id]: v }))}
              />
            ))}
          </div>

          <div
            className={cn(
              "sticky bottom-0 -mx-4 mt-6 flex items-center justify-between gap-3 border-t bg-background/95 px-4 pt-3 pb-[max(0.75rem,env(safe-area-inset-bottom))] backdrop-blur",
              hasFocus && "max-md:hidden"
            )}
          >
            {!armed && (
              <span className="text-sm text-muted-foreground tabular">
                {answered}/{total} répondu{answered > 1 ? "es" : ""}
              </span>
            )}
            <Button
              onClick={handleFinish}
              disabled={submit.isPending || !attempt.data}
              className={cn(armed && "ml-auto")}
            >
              {submit.isPending ? <Loader2 className="animate-spin" /> : armed ? null : <Send />}
              {armed
                ? `${unanswered} sans réponse — terminer quand même ?`
                : "Terminer & corriger"}
            </Button>
          </div>
        </>
      )}
    </div>
  );
}

// Champs qui ouvrent le clavier virtuel (les radios QCM n'en font pas partie).
function isTextField(el: EventTarget | null): boolean {
  return (
    el instanceof HTMLTextAreaElement ||
    (el instanceof HTMLInputElement && el.type !== "radio")
  );
}

function QuestionCard({
  index,
  q,
  value,
  onChange,
}: {
  index: number;
  q: QuestionPublic;
  value: string;
  onChange: (v: string) => void;
}) {
  return (
    <Card>
      <CardContent className="space-y-3 pt-6">
        <div className="flex items-start justify-between gap-3">
          <p className="font-medium">
            <span className="text-muted-foreground tabular">{index}. </span>
            {q.prompt}
          </p>
          <Badge variant="muted" className="shrink-0 tabular">
            {q.points} pt{q.points > 1 ? "s" : ""}
          </Badge>
        </div>

        {q.qtype === "mcq" && q.options && (
          <div className="space-y-2">
            {q.options.map((o) => (
              <label
                key={o.key}
                className={cn(
                  "flex min-h-11 cursor-pointer items-center gap-3 rounded-md border px-3 py-3 text-sm transition-colors",
                  value === o.key ? "border-primary bg-primary/10" : "hover:bg-accent"
                )}
              >
                <input
                  type="radio"
                  name={q.id}
                  checked={value === o.key}
                  onChange={() => onChange(o.key)}
                  className="size-5 accent-[var(--primary)]"
                />
                <span>{o.text}</span>
              </label>
            ))}
          </div>
        )}

        {q.qtype === "true_false" && (
          <div className="flex gap-2">
            {[
              { v: "true", label: "Vrai" },
              { v: "false", label: "Faux" },
            ].map((opt) => (
              <button
                key={opt.v}
                type="button"
                onClick={() => onChange(opt.v)}
                className={cn(
                  "h-11 flex-1 rounded-md border text-sm font-medium transition-colors focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring",
                  value === opt.v ? "border-primary bg-primary/10" : "hover:bg-accent"
                )}
              >
                {opt.label}
              </button>
            ))}
          </div>
        )}

        {q.qtype === "short_answer" && (
          <Input
            value={value}
            onChange={(e) => onChange(e.target.value)}
            placeholder="Ta réponse…"
          />
        )}

        {q.qtype === "open_ended" && (
          <Textarea
            value={value}
            onChange={(e) => onChange(e.target.value)}
            placeholder="Développe ta réponse…"
            className="min-h-28"
          />
        )}
      </CardContent>
    </Card>
  );
}
