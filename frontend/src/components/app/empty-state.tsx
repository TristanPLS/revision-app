import type { LucideIcon } from "lucide-react";
import { cn } from "@/lib/utils";

/**
 * État vide réutilisable : icône dans une pastille, message, action optionnelle.
 * Remplace les lignes de texte nu pour donner une vraie présence aux sections
 * vides (première impression d'un nouvel utilisateur).
 */
export function EmptyState({
  icon: Icon,
  children,
  action,
  className,
}: {
  icon: LucideIcon;
  children: React.ReactNode;
  action?: React.ReactNode;
  className?: string;
}) {
  return (
    <div
      className={cn(
        "flex flex-col items-center gap-3 rounded-lg border border-dashed px-6 py-10 text-center",
        className
      )}
    >
      <span className="flex size-10 items-center justify-center rounded-full bg-accent text-accent-foreground">
        <Icon className="size-5" />
      </span>
      <p className="max-w-sm text-sm text-muted-foreground">{children}</p>
      {action}
    </div>
  );
}
