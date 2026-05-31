# revision-app

Environnement de révision **sain** (anti-bachotage), bâti sur les méthodes validées
scientifiquement : active recall, répétition espacée (**FSRS**), interleaving, Feynman,
dual coding, anti-fluence et garde-fous sommeil/charge.

Générique (n'importe quelle matière) : l'IA **Gemma** (via Google AI Studio) transforme un
cours brut en blocs, flashcards, examens, menu Feynman et cartes conceptuelles.

## Stack
- **Backend** : Rust + Axum + sqlx (PostgreSQL). Scheduler **FSRS-5** implémenté en Rust pur.
- **Frontend** : Next.js (App Router) + TypeScript + shadcn/ui + Tailwind v4, standards **Impeccable**.
- **BDD** : PostgreSQL (Docker).
- **IA** : Google AI Studio (endpoint Gemini REST servant Gemma), sortie JSON structurée.
- **Réseau** : Tailscale (`tailscale serve`) pour l'accès privé PC / laptop / téléphone.

## Démarrage (dev, Windows/PowerShell)

```powershell
Copy-Item .env.example .env   # puis éditer (DATABASE_URL host=localhost)

# 1) Postgres en Docker (uniquement la BDD)
docker compose -f docker-compose.dev.yml --env-file .env up -d

# 2) Backend (les migrations s'appliquent au démarrage via sqlx::migrate!)
cd backend
cargo run                     # http://localhost:8080  (GET /api/health)

# 3) Frontend
cd ..\frontend
pnpm install
pnpm dev                      # http://localhost:3000
```

Le frontend appelle l'API en **même origine** (`/api/*`) via les rewrites Next.js → aucun CORS.

## Production

```powershell
docker compose --env-file .env up -d --build
tailscale serve --bg --https=443 http://127.0.0.1:3000
# → https://revision.<ton-tailnet>.ts.net depuis n'importe quel appareil du tailnet
```

## Structure
- `backend/` — API Axum, `migrations/`, `src/{ai,srs,models,routes}`
- `frontend/` — app Next.js
- `docker-compose.yml` (prod) · `docker-compose.dev.yml` (postgres seul)
