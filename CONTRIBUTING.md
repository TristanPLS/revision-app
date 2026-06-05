# Contribuer à revision-app

Merci de ton intérêt ! Les issues et PRs sont bienvenues, en français ou en anglais.

## Démarrage rapide (dev)

Voir le [README, section Développement](README.md#%EF%B8%8F-développement). En résumé :
Postgres dev via `docker compose -f docker-compose.dev.yml up -d` (port **5433**),
`cargo run` dans `backend/`, `pnpm dev` dans `frontend/`.

## Avant d'ouvrir une PR

La CI vérifie exactement ceci — fais-le tourner en local :

```bash
# Backend
cd backend
cargo fmt --check
cargo clippy -- -D warnings
cargo test

# Frontend
cd frontend
pnpm lint
pnpm exec tsc --noEmit
pnpm build
```

## Lignes directrices

- **Petites PRs ciblées** > grosses PRs fourre-tout. Une PR = un sujet.
- **Le positionnement produit est volontaire** : anti-bachotage, garde-fous santé,
  schémas dessinés par l'utilisateur (pas par l'IA), mono-utilisateur en v1.
  Une PR qui va contre cette philosophie sera discutée avant d'être acceptée —
  ouvre une issue d'abord.
- **Migrations** : toujours additives (`backend/migrations/NNNN_nom.sql`,
  numérotation séquentielle). Ne modifie jamais une migration existante.
- **Chaînes utilisateur** : en français, tutoiement, ton direct.
- **IA** : toute la logique provider passe par `backend/src/ai/client.rs` —
  un seul point d'entrée `generate_json(prompt, schema)`.

## Signaler un bug

Ouvre une issue avec : ce que tu as fait, ce que tu attendais, ce qui s'est passé,
ton environnement (OS, Docker ou dev natif, fournisseur IA). Les logs s'obtiennent
avec `docker compose logs backend --tail 100`.

## Failles de sécurité

Ne les signale **pas** en issue publique — voir [SECURITY.md](SECURITY.md).
