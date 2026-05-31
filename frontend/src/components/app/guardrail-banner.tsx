"use client";

import { useQuery } from "@tanstack/react-query";
import { Moon } from "lucide-react";
import { api } from "@/lib/api/client";

export function GuardrailBanner() {
  const { data } = useQuery({
    queryKey: ["guardrails"],
    queryFn: api.guardrails,
    refetchInterval: 60_000,
  });

  if (!data || data.nudges.length === 0) return null;

  return (
    <div className="mb-6 space-y-2">
      {data.nudges.map((nudge, i) => (
        <div
          key={i}
          className="flex items-center gap-2.5 rounded-lg border bg-accent px-4 py-2.5 text-sm text-accent-foreground"
        >
          <Moon className="size-4 shrink-0" />
          <span>{nudge}</span>
        </div>
      ))}
    </div>
  );
}
