"use client";

import { cn } from "@/lib/utils";

const LABELS = ["À apprendre", "Récent", "En cours", "Solide", "Maîtrisé"];

export function LeitnerBoard({
  byBox,
  highlight,
}: {
  byBox: number[];
  highlight?: number; // 1..5
}) {
  return (
    <div className="grid grid-cols-5 gap-2">
      {byBox.map((count, i) => {
        const isHi = highlight === i + 1;
        return (
          <div
            key={i}
            className={cn(
              "flex flex-col items-center gap-1 rounded-lg border p-3 text-center transition-colors",
              isHi ? "border-primary bg-primary/10" : "bg-card"
            )}
          >
            <span className="text-xs text-muted-foreground">Boîte {i + 1}</span>
            <span className="tabular text-xl font-semibold leading-none">{count}</span>
            <span className="text-[10px] leading-tight text-muted-foreground">
              {LABELS[i]}
            </span>
          </div>
        );
      })}
    </div>
  );
}
