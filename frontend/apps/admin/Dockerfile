# syntax=docker/dockerfile:1

# Accurit admin (SaaS backoffice). Runtime: nginx serving static Angular SPA.
# Build context: ./frontend (root docker-compose.yml) — workspace needed
# to build @accurit/ui and friends.

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

# --- build: admin production bundle with the edge base-href (/admin/) ---
FROM deps AS build
ARG BASE_HREF=/admin/
COPY . .
RUN pnpm --filter admin^... build \
    && pnpm --filter admin exec ng build --configuration production --base-href "${BASE_HREF}"

# --- runtime: static nginx SPA server ---
FROM nginx:stable-alpine AS runtime
COPY --from=build /repo/apps/admin/dist/admin/browser/ /usr/share/nginx/html/
COPY apps/admin/nginx.conf /etc/nginx/conf.d/default.conf
EXPOSE 80
HEALTHCHECK --interval=15s --timeout=5s --retries=5 \
    CMD wget -qO- http://127.0.0.1/healthz || exit 1
