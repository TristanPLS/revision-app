"use client";

import { useMemo } from "react";
import Link from "next/link";
import { useParams } from "next/navigation";
import { useQuery } from "@tanstack/react-query";
import { ReactFlow, Background, Controls, type Node, type Edge } from "@xyflow/react";
import "@xyflow/react/dist/style.css";
import { X } from "lucide-react";
import { api } from "@/lib/api/client";
import { Skeleton } from "@/components/ui/skeleton";
import type { ConceptMapDetail } from "@/lib/api/types";

const nodeStyle: React.CSSProperties = {
  background: "var(--card)",
  color: "var(--card-foreground)",
  border: "1px solid var(--border)",
  borderRadius: 8,
  fontSize: 12,
  padding: "8px 12px",
  maxWidth: 200,
};

function layout(map: ConceptMapDetail): { nodes: Node[]; edges: Edge[] } {
  const byId = new Map(map.nodes.map((n) => [n.id, n]));
  const children = new Map<string, string[]>();
  const roots: string[] = [];
  for (const n of map.nodes) {
    if (n.parent_id && byId.has(n.parent_id)) {
      (children.get(n.parent_id) ?? children.set(n.parent_id, []).get(n.parent_id)!).push(n.id);
    } else {
      roots.push(n.id);
    }
  }
  const levels: string[][] = [];
  const visited = new Set<string>();
  const queue = roots.map((id) => ({ id, d: 0 }));
  while (queue.length) {
    const { id, d } = queue.shift()!;
    if (visited.has(id)) continue;
    visited.add(id);
    (levels[d] ??= []).push(id);
    for (const c of children.get(id) ?? []) queue.push({ id: c, d: d + 1 });
  }
  for (const n of map.nodes) {
    if (!visited.has(n.id)) (levels[0] ??= []).push(n.id);
  }
  const pos = new Map<string, { x: number; y: number }>();
  levels.forEach((ids, d) => ids.forEach((id, i) => pos.set(id, { x: i * 230, y: d * 130 })));

  const nodes: Node[] = map.nodes.map((n) => ({
    id: n.id,
    data: { label: n.label },
    position: pos.get(n.id) ?? { x: 0, y: 0 },
    style: nodeStyle,
  }));
  const edges: Edge[] = [
    ...map.nodes
      .filter((n) => n.parent_id && byId.has(n.parent_id))
      .map((n) => ({ id: `p-${n.id}`, source: n.parent_id as string, target: n.id })),
    ...map.edges.map((e) => ({
      id: e.id,
      source: e.from_node,
      target: e.to_node,
      label: e.label ?? undefined,
      animated: true,
    })),
  ];
  return { nodes, edges };
}

export default function ConceptMapPage() {
  const { id } = useParams<{ id: string }>();
  const map = useQuery({ queryKey: ["map", id], queryFn: () => api.maps.get(id) });
  const flow = useMemo(() => (map.data ? layout(map.data) : null), [map.data]);

  return (
    <div className="flex h-screen flex-col">
      <div className="flex items-center gap-4 border-b px-4 py-3">
        <Link
          href={map.data ? `/subjects/${map.data.subject_id}` : "/"}
          className="text-muted-foreground hover:text-foreground"
          aria-label="Fermer"
        >
          <X className="size-5" />
        </Link>
        <p className="flex-1 truncate text-sm font-medium">{map.data?.title ?? "Carte"}</p>
      </div>
      <div className="relative flex-1">
        {flow ? (
          <ReactFlow nodes={flow.nodes} edges={flow.edges} fitView minZoom={0.2}>
            <Background />
            <Controls />
          </ReactFlow>
        ) : (
          <div className="p-6">
            <Skeleton className="h-full min-h-96" />
          </div>
        )}
      </div>
    </div>
  );
}
