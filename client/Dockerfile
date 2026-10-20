# syntax=docker/dockerfile:1

FROM node:22-alpine AS deps

WORKDIR /app

COPY package.json package-lock.json ./

RUN npm ci

FROM node:22-alpine AS builder

WORKDIR /app

ARG BASE_HREF=/app/

COPY --from=deps /app/node_modules ./node_modules
COPY . .

RUN npm run build -- --configuration production --base-href="$BASE_HREF"

FROM nginx:1.29-alpine AS runtime

COPY nginx.conf /etc/nginx/conf.d/default.conf
COPY --from=builder /app/dist/client/browser /usr/share/nginx/html

EXPOSE 80
