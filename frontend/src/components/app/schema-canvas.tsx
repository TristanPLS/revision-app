"use client";

import { useRef } from "react";
import { Tldraw, getSnapshot, loadSnapshot, type Editor } from "tldraw";
import "tldraw/tldraw.css";
import { Save, Loader2 } from "lucide-react";
import { Button } from "@/components/ui/button";

export default function SchemaCanvas({
  initial,
  onSave,
  saving,
}: {
  initial: unknown;
  onSave: (snapshot: unknown) => void;
  saving?: boolean;
}) {
  const ref = useRef<Editor | null>(null);
  return (
    <div className="space-y-2">
      <div className="flex justify-end">
        <Button
          size="sm"
          disabled={saving}
          onClick={() => {
            if (ref.current) onSave(getSnapshot(ref.current.store));
          }}
        >
          {saving ? <Loader2 className="size-4 animate-spin" /> : <Save className="size-4" />}
          Enregistrer le dessin
        </Button>
      </div>
      <div className="relative h-[60vh] overflow-hidden rounded-lg border">
        <Tldraw
          onMount={(editor) => {
            ref.current = editor;
            if (initial) {
              try {
                loadSnapshot(editor.store, initial as Parameters<typeof loadSnapshot>[1]);
              } catch {
                /* ignore a malformed/old snapshot */
              }
            }
          }}
        />
      </div>
    </div>
  );
}
