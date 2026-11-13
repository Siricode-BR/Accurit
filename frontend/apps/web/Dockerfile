# syntax=docker/dockerfile:1

# Build context: this file is referenced as `dockerfile: apps/web/Dockerfile`
# with `context: ./frontend` in the root docker-compose.yml — the pnpm/Turbo
# workspace must be available to build @accurit/*@workspace deps.

ARG NODE_VERSION=24

FROM node:${NODE_VERSION}-alpine AS base
RUN corepack enable && corepack prepare pnpm@12.3.4 --activate
WORKDIR /repo

# --- deps: install workspace dependencies (cached on manifest changes) ---
FROM base AS deps
COPY package.json pnpm-lock.yaml pnpm-workspace.yaml .npmrc /repo/
COPY packages/ts-config/package.json packages/ts-config/
COPY packages/shared/package.json packages/shared/
COPY packages/core/package.json packages/core/
COPY packages/ui/package.json packages/ui/
COPY packages/eslint-config/package.json packages/eslint-config/
COPY apps/web/package.json apps/web/
COPY apps/admin/package.json apps/admin/
COPY apps/client/package.json apps/client/
RUN pnpm install --frozen-lockfile

# --- build: Next.js production build (standalone output) ---
FROM deps AS build
ENV NEXT_OUTPUT=standalone
COPY . .
RUN pnpm --filter web... build \
    && mkdir -p /repo/apps/web/public

# --- runner: minimal server (standalone) ---
FROM node:${NODE_VERSION}-alpine AS runtime
ENV NODE_ENV=production PORT=3000 HOSTNAME=0.0.0.0
WORKDIR /app
COPY --from=build /repo/apps/web/.next/standalone ./
COPY --from=build /repo/apps/web/.next/static apps/web/.next/static
COPY --from=build /repo/apps/web/public apps/web/public
USER node
EXPOSE 3000
CMD ["node", "apps/web/server.js"]
