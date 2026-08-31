import { clsx, type ClassValue } from "clsx";
import { twMerge } from "tailwind-merge";

/** Merge class names, de-duplicating conflicting Tailwind utilities. */
export function cn(...inputs: ClassValue[]) {
  return twMerge(clsx(inputs));
}

/**
 * ISO2 → path of the flag served from public/flags. SVG files rather than the
 * Unicode regional indicators, which Windows renders as a bare letter pair —
 * i.e. the answer, on the very screen that asks for it.
 * Returns "" on anything that is not two ASCII letters.
 */
export function flagSrc(iso2: string): string {
  const code = iso2.trim().toLowerCase();
  if (!/^[a-z]{2}$/.test(code)) return "";
  return `/flags/${code}.svg`;
}
