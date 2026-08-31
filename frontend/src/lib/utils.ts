import { clsx, type ClassValue } from "clsx";
import { twMerge } from "tailwind-merge";

/** Merge class names, de-duplicating conflicting Tailwind utilities. */
export function cn(...inputs: ClassValue[]) {
  return twMerge(clsx(inputs));
}

/**
 * ISO2 → flag emoji, via the Unicode regional indicators (0x1F1E6 + letter).
 * Ships no glyph on Windows/Chrome, which has no flag font: the pair of letters
 * ("FR") shows through instead. Acceptable — the target is the phone (iOS and
 * Android both render these), the desktop is only a fallback screen.
 * Returns "" on anything that is not two ASCII letters.
 */
export function flagEmoji(iso2: string): string {
  const code = iso2.trim().toUpperCase();
  if (!/^[A-Z]{2}$/.test(code)) return "";
  return String.fromCodePoint(
    ...[...code].map((c) => 0x1f1e6 + c.charCodeAt(0) - 65)
  );
}
