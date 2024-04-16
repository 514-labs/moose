# To run the build you need to run it from the root of the repo with the following command
# docker build -f apps/moose-console/Dockerfile -t moose-console:latest .

FROM node:20-alpine AS base

# Set working directory
ENV PNPM_HOME="/pnpm"
ENV PATH="$PNPM_HOME:$PATH"

FROM base AS builder
# Check https://github.com/nodejs/docker-node/tree/b4117f9333da4138b03a546ec926ef50a31506c3#nodealpine to understand why libc6-compat might be needed.
RUN apk add --no-cache libc6-compat
RUN apk update

WORKDIR /app
RUN npm install --g turbo
COPY . .
RUN turbo prune --docker moose-console

# Add lockfile and package.json's of isolated subworkspace
FROM base AS installer
RUN apk add --no-cache libc6-compat
RUN apk update

RUN npm install --g pnpm@latest-8

WORKDIR /app

# First install the dependencies (as they change less often)
COPY .gitignore .gitignore
COPY --from=builder /app/out/json/ .
COPY --from=builder /app/apps/moose-console/gsap-bonus.tgz ./apps/moose-console/gsap-bonus.tgz

# Hack for now to get the gsap-bonus.tgz into the design-system package, need to figure out better bundling
COPY --from=builder /app/apps/moose-console/gsap-bonus.tgz /app/packages/design-system/gsap-bonus.tgz


RUN pnpm install --frozen-lockfile

# Build the project
COPY --from=builder /app/out/full/ .

# Uncomment and use build args to enable remote caching
ARG TURBO_TEAM
ENV TURBO_TEAM=$TURBO_TEAM

ARG TURBO_TOKEN
ENV TURBO_TOKEN=$TURBO_TOKEN

ARG SENTRY_AUTH_TOKEN
ENV SENTRY_AUTH_TOKEN=$SENTRY_AUTH_TOKEN

ARG RELEASE_VERSION
ENV NEXT_PUBLIC_RELEASE_VERSION=$RELEASE_VERSION

RUN pnpm run build --filter=moose-console...

FROM base AS runner
WORKDIR /app

# Don't run production as root
RUN addgroup --system --gid 1001 nodejs
RUN adduser --system --uid 1001 nextjs
USER nextjs

COPY --from=installer /app/apps/moose-console/next.config.mjs .
COPY --from=installer /app/apps/moose-console/package.json .

# Automatically leverage output traces to reduce image size
# https://nextjs.org/docs/advanced-features/output-file-tracing
COPY --from=installer --chown=nextjs:nodejs /app/apps/moose-console/.next/standalone ./
COPY --from=installer --chown=nextjs:nodejs /app/apps/moose-console/.next/static ./apps/moose-console/.next/static

# Enable when we have a public folder in the moose-console directory
# COPY --from=installer --chown=nextjs:nodejs /app/apps/moose-console/public ./apps/moose-console/public

CMD node apps/moose-console/server.js