# revision-app

Environnement de révision **sain** (anti-bachotage), bâti sur les méthodes validées
scientifiquement : active recall, répétition espacée (**FSRS-5**), interleaving,
technique Feynman, fiches Cornell, dual coding, anti-fluence et garde-fous
sommeil/charge.

Colle ton cours (n'importe quelle matière) : l'IA en génère un plan de révision
complet — flashcards, examen blanc chronométré, concepts à expliquer à voix haute,
carte conceptuelle, fiche Cornell et schémas à dessiner. Tu valides, tu révises.

> 🇫🇷 L'interface est en français. Le contenu généré (cartes, questions…) suit la
> langue de ton cours.

## ✨ Fonctionnalités

- **Flashcards FSRS-5** — répétition espacée moderne (scheduler implémenté en Rust pur),
  files de révision par matière, interleaving entre blocs.
- **« Tout générer »** — l'IA lit ton cours, propose un plan (blocs + quantités) que tu
  peux ajuster, puis génère tous les supports d'un coup.
- **Examens blancs** — QCM, vrai/faux, réponses courtes et questions ouvertes,
  chronométrés ; les réponses libres sont corrigées par l'IA avec feedback.
- **Feynman** — une liste de mécanismes à savoir expliquer « comme à un enfant »,
  avec auto-évaluation.
- **Fiches Cornell** — notes structurées + questions de rappel en marge,
  convertibles en flashcards.
- **Cartes conceptuelles** — hiérarchie + liens transversaux, rendu interactif.
- **Schémas (dual coding)** — l'IA te dit *quoi* dessiner et ce que le schéma doit
  contenir ; **c'est toi qui dessines, de mémoire**, puis tu compares. C'est voulu :
  un schéma déjà dessiné ne ferait rien apprendre.
- **Garde-fous santé** — nudge sommeil après 22 h, jour de repos, plafonds de charge,
  suivi de série (streak). C'est un outil d'apprentissage durable, pas de cramming.

## 🤖 L'IA : ta clé, ton choix (BYOK)

L'app fonctionne avec **ta propre clé API**, configurée directement dans l'interface
(page **Réglages**) — rien à éditer à la main. Trois fournisseurs supportés :

| Fournisseur | Coût | Notes |
|---|---|---|
| **Google AI Studio** (Gemma) | **Gratuit** (limites/jour) | Recommandé pour démarrer — [obtenir une clé](https://aistudio.google.com/apikey) en 2 min |
| **OpenAI-compatible** | Variable | Couvre OpenAI, **Ollama (100 % local et privé)**, LM Studio, Groq, Mistral… |
| **Anthropic** (Claude) | Payant à l'usage | [Console Anthropic](https://console.anthropic.com/settings/keys) |

Sans clé configurée, tout le reste fonctionne (création manuelle de cartes, fiches,
examens, révision FSRS) — seule la génération automatique est indisponible, et
l'app te l'indique clairement.

## 🚀 Installation (5 minutes)

**Prérequis : [Docker Desktop](https://www.docker.com/products/docker-desktop/)**
(Windows/Mac) ou Docker Engine + le plugin compose (Linux). C'est tout.

1. **Récupère le code** — soit avec git, soit en
   [téléchargeant le ZIP](https://github.com/TristanPLS/revision-app/archive/refs/heads/main.zip) :

   ```bash
   git clone https://github.com/TristanPLS/revision-app.git
   cd revision-app
   ```

2. **Lance l'application** (aucun fichier à éditer — les images pré-construites
   sont téléchargées, compte 2 à 5 minutes la première fois) :

   ```bash
   docker compose up -d
   ```

3. **Ouvre http://localhost:3000**, va dans **Réglages**, colle ta clé API
   (gratuite avec Google AI Studio), clique « Enregistrer et tester ». ✅

Comment savoir que ça marche : la page d'accueil s'affiche, et le test de
connexion dans Réglages répond « Connexion réussie ».

**Mise à jour** : `docker compose pull && docker compose up -d`. Tes données sont
conservées (volume Docker `pgdata`).

### Accéder depuis ton téléphone / un autre appareil

- **Réseau local (Wi-Fi de la maison)** : crée un fichier `.env` contenant
  `BIND_ADDR=0.0.0.0`, relance `docker compose up -d`, puis ouvre
  `http://IP-DE-TON-PC:3000`. ⚠️ Lis d'abord l'encadré Sécurité.
- **Depuis n'importe où (recommandé)** : installe [Tailscale](https://tailscale.com)
  (VPN privé gratuit) sur le serveur et tes appareils, puis :
  `tailscale serve --bg --https=443 http://127.0.0.1:3000`
  → `https://ton-serveur.ton-tailnet.ts.net`, chiffré et accessible uniquement
  par tes appareils.

## 🔒 Sécurité — à lire avant d'exposer quoi que ce soit

> **Cette application est mono-utilisateur et n'a AUCUNE authentification.**
> Quiconque peut ouvrir la page peut lire, modifier et supprimer toutes tes
> données, et consommer ton quota/ta clé IA.
>
> - ✅ OK : `localhost` (défaut), réseau domestique de confiance, Tailscale/VPN.
> - ❌ JAMAIS : exposition directe sur internet (port-forwarding, VPS avec port
>   ouvert, reverse-proxy public sans authentification).
>
> Par défaut, seul le frontend est publié et uniquement en local (`127.0.0.1`) ;
> la base de données et le backend ne sont pas joignables depuis l'extérieur.
> La clé API est stockée **en clair** dans la base de données locale.

## 🔐 Confidentialité

- **Tout reste chez toi** (base de données locale), **sauf** les appels IA : à
  chaque génération, le texte du cours est envoyé au fournisseur que tu as choisi.
- **Palier gratuit Google AI Studio** : Google peut utiliser les textes envoyés
  pour améliorer ses produits (relecture humaine possible). Évite d'y coller des
  données personnelles sensibles ; le palier payant n'a pas cette clause.
- **Confidentialité totale** : utilise **Ollama** en local (fournisseur
  « OpenAI-compatible », URL `http://host.docker.internal:11434/v1`, clé vide) —
  rien ne quitte ta machine.

## 💾 Sauvegarde

Tes données (cartes, historique FSRS, examens…) vivent dans le volume Docker
`pgdata`. Pour les sauvegarder / restaurer :

```bash
# Sauvegarde → fichier revision-backup.sql
docker compose exec postgres pg_dump -U revision revision > revision-backup.sql

# Restauration (base vide)
docker compose exec -T postgres psql -U revision revision < revision-backup.sql
```

⚠️ `docker compose down -v` **détruit le volume et tout ton historique** —
n'utilise jamais `-v` sans sauvegarde.

## ⚙️ Configuration avancée (optionnelle)

Tout a une valeur par défaut ; un fichier `.env` (copié depuis
[`.env.example`](.env.example)) permet d'ajuster :

| Variable | Défaut | Rôle |
|---|---|---|
| `FRONTEND_PORT` | `3000` | Port de l'interface |
| `BIND_ADDR` | `127.0.0.1` | `0.0.0.0` = accessible depuis le réseau local |
| `TZ` | `Europe/Paris` | Fuseau (streak, garde-fou sommeil 22 h–5 h) |
| `FSRS_RETENTION` | `0.9` | Rétention cible FSRS (0.7–0.97) |
| `AI_MAX_SOURCE_CHARS` | `16000` | Taille max du cours injecté dans les prompts |
| `GEMINI_API_KEY`, `AI_PROVIDER`, `AI_MODEL`, `GEMINI_BASE_URL` | — | Valeurs initiales IA ; la page Réglages a priorité |
| `POSTGRES_PASSWORD` | `revision-local-only` | La BDD n'est pas exposée hors du réseau Docker |

## 🛠️ Développement

Stack : **Rust** (Axum + sqlx, scheduler FSRS-5 maison) · **Next.js** (App Router,
TypeScript, Tailwind v4, shadcn/ui) · **PostgreSQL**.

```bash
cp .env.example .env        # décommenter DATABASE_URL (localhost:5433)

# 1) Postgres seul (publié sur localhost:5433)
docker compose -f docker-compose.dev.yml up -d

# 2) Backend (migrations auto au démarrage)
cd backend && cargo run     # http://localhost:8080  (GET /api/health)

# 3) Frontend
cd frontend && pnpm install && pnpm dev   # http://localhost:3000
```

Le frontend appelle l'API en même origine (`/api/*`) via les rewrites Next.js →
aucun CORS. Avant de proposer une PR : `cargo fmt && cargo clippy && cargo test`
côté backend, `pnpm lint && pnpm exec tsc --noEmit` côté frontend (c'est ce que
vérifie la CI).

### Structure

- `backend/` — API Axum, `migrations/`, `src/{ai,srs,models,routes}`
  - `src/ai/client.rs` — client multi-provider (Gemini / OpenAI-compat / Anthropic)
  - `src/srs.rs` — FSRS-5 pur Rust (testé)
- `frontend/` — app Next.js (`src/app`, `src/components`, `src/lib/api`)
- `docker-compose.yml` (prod, images GHCR) · `docker-compose.dev.yml` (Postgres seul)

## 🧭 Philosophie (et ce que l'app ne fera pas)

- Pas de mode « bachotage » : l'app t'arrête après tes plafonds de charge et te
  pousse à dormir après 22 h. Le dimanche est un jour de repos.
- L'IA **ne dessine pas les schémas à ta place** et ne te fait pas relire
  passivement : tout est conçu pour te faire **produire** (recall, explication,
  dessin) — c'est là que la mémoire se construit.
- Mono-utilisateur par design : 1 instance = 1 personne. Le multi-utilisateurs
  n'est pas un objectif de la v1.

## 📄 Licence

[MIT](LICENSE) — fais-en bon usage. Les contributions sont bienvenues
(issues et PRs).
