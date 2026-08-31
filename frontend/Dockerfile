# --- deps ---
FROM node:22-alpine AS deps
RUN corepack enable
WORKDIR /app
COPY package.json pnpm-lock.yaml ./
# pnpm >= 10 refuse d'exécuter les build scripts natifs (sharp, unrs-resolver…)
# sans opt-in explicite ; onlyBuiltDependencies est ignoré avec --frozen-lockfile.
RUN pnpm install --frozen-lockfile --config.dangerouslyAllowAllBuilds=true

# --- build (Next.js standalone output) ---
FROM node:22-alpine AS builder
RUN corepack enable
WORKDIR /app
COPY --from=deps /app/node_modules ./node_modules
COPY . .
ENV NEXT_TELEMETRY_DISABLED=1
# En sortie standalone, Next fige la destination du rewrite /api au moment du
# build : la variable doit exister ICI (celle du runtime dans compose ne suffit
# pas), sinon le proxy retombe sur localhost:8080 → 500 sur tout /api en prod.
ARG BACKEND_INTERNAL_URL=http://backend:8080
ENV BACKEND_INTERNAL_URL=$BACKEND_INTERNAL_URL
RUN pnpm build

# --- runtime ---
FROM node:22-alpine AS runtime
WORKDIR /app
ENV NODE_ENV=production
ENV NEXT_TELEMETRY_DISABLED=1
RUN addgroup -S nodejs && adduser -S nextjs -G nodejs
COPY --from=builder /app/.next/standalone ./
COPY --from=builder /app/.next/static ./.next/static
COPY --from=builder /app/public ./public
USER nextjs
EXPOSE 3000
ENV PORT=3000
ENV HOSTNAME=0.0.0.0
CMD ["node", "server.js"]
