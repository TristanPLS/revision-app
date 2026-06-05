## Quoi / pourquoi


## Checklist

- [ ] `cargo fmt --check && cargo clippy -- -D warnings && cargo test` (backend)
- [ ] `pnpm lint && pnpm exec tsc --noEmit && pnpm build` (frontend)
- [ ] Migrations : additives uniquement, numérotation séquentielle
- [ ] Pas de secret / donnée personnelle dans le diff
