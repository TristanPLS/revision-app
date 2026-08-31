"use client";

import { useState } from "react";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { CheckCircle2, ExternalLink, KeyRound, Loader2, XCircle } from "lucide-react";
import { toast } from "sonner";
import { api, ApiError, type AiSettings } from "@/lib/api/client";
import { cn } from "@/lib/utils";
import { Button } from "@/components/ui/button";
import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from "@/components/ui/card";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { Skeleton } from "@/components/ui/skeleton";

type Provider = AiSettings["provider"];

const PROVIDERS: {
  id: Provider;
  name: string;
  hint: string;
  keyHelp: string;
  keyUrl?: string;
  modelPlaceholder: string;
}[] = [
  {
    id: "gemini",
    name: "Google AI Studio",
    hint: "✅ Gratuit · ~1500 req/jour · recommandé",
    keyHelp:
      "Gemma 4, gratuit et amplement suffisant (~1500 générations/jour). Clé en 2 minutes avec un compte Google.",
    keyUrl: "https://aistudio.google.com/apikey",
    modelPlaceholder: "gemma-4-31b-it",
  },
  {
    id: "openai",
    name: "OpenAI / ChatGPT",
    hint: "Payant · ou Ollama en local (gratuit)",
    keyHelp:
      "Compte OpenAI payant à l'usage. Format compatible avec Groq, Mistral, et un serveur local Ollama/LM Studio (clé vide pour le local, 100 % gratuit et privé).",
    keyUrl: "https://platform.openai.com/api-keys",
    modelPlaceholder: "gpt-4o-mini (ou llama3.1 pour Ollama)",
  },
  {
    id: "anthropic",
    name: "Anthropic (Claude)",
    hint: "Payant à l'usage",
    keyHelp: "Compte Anthropic payant à l'usage — facturé par génération.",
    keyUrl: "https://console.anthropic.com/settings/keys",
    modelPlaceholder: "claude-opus-4-8",
  },
];

export default function SettingsPage() {
  const settings = useQuery({ queryKey: ["settings"], queryFn: api.settings.get });

  return (
    <div className="mx-auto max-w-2xl space-y-8">
      <header className="space-y-1">
        <h1 className="text-2xl font-semibold tracking-tight sm:text-3xl">Réglages</h1>
        <p className="text-muted-foreground">
          Connecte ton propre fournisseur d&apos;IA — ta clé reste sur ton serveur.
        </p>
      </header>

      {settings.data ? (
        <AiSettingsForm initial={settings.data} />
      ) : (
        <Skeleton className="h-96" />
      )}
    </div>
  );
}

function AiSettingsForm({ initial }: { initial: AiSettings }) {
  const qc = useQueryClient();
  // Derniers réglages persistés (mis à jour après chaque enregistrement).
  const [saved, setSaved] = useState(initial);
  const [provider, setProvider] = useState<Provider>(initial.provider);
  const [model, setModel] = useState(initial.model);
  const [baseUrl, setBaseUrl] = useState(initial.base_url);
  const [apiKey, setApiKey] = useState("");
  const [testResult, setTestResult] = useState<{ ok: boolean; detail: string } | null>(
    null
  );

  const meta = PROVIDERS.find((p) => p.id === provider)!;

  function switchProvider(next: Provider) {
    setProvider(next);
    setTestResult(null);
    // La clé d'un autre fournisseur n'est jamais valide → on vide le champ pour
    // forcer la ressaisie (le backend efface aussi la clé au changement).
    setApiKey("");
    // Si on revient au provider enregistré, restaure ses valeurs ; sinon,
    // pré-remplit avec les défauts du nouveau provider.
    if (next === saved.provider) {
      setModel(saved.model);
      setBaseUrl(saved.base_url);
    } else {
      const defaults = saved.defaults[next];
      setModel(defaults.model ?? "");
      setBaseUrl(defaults.base_url);
    }
  }

  function buildBody() {
    return {
      provider,
      model: model.trim(),
      base_url: baseUrl.trim(),
      ...(apiKey.trim() ? { api_key: apiKey.trim() } : {}),
    };
  }

  function applySaved(data: AiSettings) {
    qc.setQueryData(["settings"], data);
    setSaved(data);
    setModel(data.model);
    setBaseUrl(data.base_url);
    setApiKey("");
  }

  const save = useMutation({
    mutationFn: () => api.settings.update(buildBody()),
    onSuccess: (data) => {
      applySaved(data);
      setTestResult(null);
      toast.success("Réglages enregistrés");
    },
    onError: (e: unknown) =>
      toast.error(e instanceof ApiError ? e.message : "Échec de l'enregistrement"),
  });

  const saveAndTest = useMutation({
    mutationFn: async () => {
      const data = await api.settings.update(buildBody());
      return { data, result: await api.settings.test() };
    },
    onSuccess: ({ data, result }) => {
      applySaved(data);
      if (result.ok) {
        setTestResult({
          ok: true,
          detail: `Connexion réussie — ${result.model} a répondu en ${
            result.latency_ms != null ? `${(result.latency_ms / 1000).toFixed(1)} s` : "?"
          }.`,
        });
      } else {
        setTestResult({ ok: false, detail: result.error ?? "Échec du test." });
      }
    },
    onError: (e: unknown) =>
      setTestResult({
        ok: false,
        detail: e instanceof ApiError ? e.message : "Échec du test.",
      }),
  });

  const busy = save.isPending || saveAndTest.isPending;

  return (
    <Card>
      <CardHeader>
        <CardTitle className="flex items-center gap-2">
          <KeyRound className="size-5" />
          Intelligence artificielle
        </CardTitle>
        <CardDescription>
          L&apos;IA transforme tes cours en flashcards, examens blancs, fiches et cartes.
          Sans clé configurée, tout le reste de l&apos;app fonctionne — seule la
          génération automatique est indisponible.
        </CardDescription>
      </CardHeader>
      <CardContent className="space-y-6">
        <div className="rounded-md border border-primary/30 bg-primary/5 px-4 py-3 text-sm">
          <p className="font-medium text-foreground">
            💡 Étudiant·e ? Reste sur <span className="text-primary">Google AI Studio</span> —
            c&apos;est gratuit.
          </p>
          <p className="mt-1 text-muted-foreground">
            Le modèle Gemma 4 offre environ <strong>1500 générations par jour</strong>,
            largement assez pour réviser. Claude (Anthropic) et ChatGPT (OpenAI) sont
            aussi disponibles, mais payants à l&apos;usage.{" "}
            <a
              href="https://aistudio.google.com/apikey"
              target="_blank"
              rel="noreferrer"
              className="inline-flex items-center gap-0.5 text-primary hover:underline"
            >
              Obtenir ta clé gratuite <ExternalLink className="size-3" />
            </a>
          </p>
        </div>

        <fieldset className="space-y-2">
          <legend className="text-sm font-medium leading-none">Fournisseur</legend>
          <div className="grid gap-2 sm:grid-cols-3">
            {PROVIDERS.map((p) => (
              <button
                key={p.id}
                type="button"
                onClick={() => switchProvider(p.id)}
                disabled={busy}
                aria-pressed={provider === p.id}
                className={cn(
                  "rounded-md border px-3 py-2.5 text-left transition-colors disabled:cursor-not-allowed disabled:opacity-60",
                  provider === p.id
                    ? "border-primary bg-primary/5 ring-1 ring-primary"
                    : "hover:border-primary/40"
                )}
              >
                <span className="block text-sm font-medium">{p.name}</span>
                <span className="block text-xs text-muted-foreground">{p.hint}</span>
              </button>
            ))}
          </div>
        </fieldset>

        <div className="space-y-2">
          <Label htmlFor="ai-key">Clé API</Label>
          <Input
            id="ai-key"
            type="password"
            autoComplete="off"
            value={apiKey}
            onChange={(e) => setApiKey(e.target.value)}
            placeholder={
              saved.api_key_set && provider === saved.provider
                ? `${saved.api_key_hint ?? "••••••••"} (enregistrée — colle une nouvelle clé pour la remplacer)`
                : meta.id === "openai"
                  ? "optionnelle pour un serveur local"
                  : "colle ta clé ici"
            }
          />
          <p className="text-xs text-muted-foreground">
            {meta.keyHelp}{" "}
            {meta.keyUrl && (
              <a
                href={meta.keyUrl}
                target="_blank"
                rel="noreferrer"
                className="inline-flex items-center gap-0.5 text-primary hover:underline"
              >
                Obtenir une clé <ExternalLink className="size-3" />
              </a>
            )}
          </p>
        </div>

        <div className="grid gap-4 sm:grid-cols-2">
          <div className="space-y-2">
            <Label htmlFor="ai-model">Modèle</Label>
            <Input
              id="ai-model"
              value={model}
              onChange={(e) => setModel(e.target.value)}
              placeholder={meta.modelPlaceholder}
              autoCapitalize="none"
              autoCorrect="off"
              spellCheck={false}
            />
          </div>
          <div className="space-y-2">
            <Label htmlFor="ai-url">URL du serveur (avancé)</Label>
            <Input
              id="ai-url"
              value={baseUrl}
              onChange={(e) => setBaseUrl(e.target.value)}
              placeholder={saved.defaults[provider].base_url}
              autoCapitalize="none"
              autoCorrect="off"
              spellCheck={false}
              inputMode="url"
            />
          </div>
        </div>

        {testResult && (
          <div
            className={cn(
              "flex items-start gap-2 rounded-md border px-3 py-2.5 text-sm",
              testResult.ok
                ? "border-green-600/30 bg-green-600/10 text-green-700 dark:text-green-400"
                : "border-destructive/30 bg-destructive/10 text-destructive"
            )}
          >
            {testResult.ok ? (
              <CheckCircle2 className="mt-0.5 size-4 shrink-0" />
            ) : (
              <XCircle className="mt-0.5 size-4 shrink-0" />
            )}
            <span className="break-words">{testResult.detail}</span>
          </div>
        )}

        <div className="flex flex-wrap gap-2">
          <Button onClick={() => save.mutate()} disabled={busy}>
            {save.isPending && <Loader2 className="animate-spin" />}
            Enregistrer
          </Button>
          <Button variant="outline" onClick={() => saveAndTest.mutate()} disabled={busy}>
            {saveAndTest.isPending && <Loader2 className="animate-spin" />}
            Enregistrer et tester
          </Button>
        </div>

        <p className="border-t pt-4 text-xs text-muted-foreground">
          <strong>Confidentialité :</strong> ta clé est stockée sur ton serveur (dans
          ta base de données) et n&apos;est envoyée à personne d&apos;autre que le fournisseur
          choisi. À chaque génération, le texte du cours est transmis à ce
          fournisseur. Avec le palier gratuit de Google AI Studio, Google peut
          utiliser ces textes pour améliorer ses produits — évite d&apos;y coller des
          données personnelles sensibles, ou utilise Ollama en local pour une
          confidentialité totale.
        </p>
      </CardContent>
    </Card>
  );
}
