FROM node:24-alpine AS builder

WORKDIR /app

COPY package.json pnpm-lock.yaml pnpm-workspace.yaml tsconfig.* /app/
COPY packages/persister /app/packages/persister/

RUN corepack enable && pnpm install
RUN pnpm --filter=@alxt/kedeng-persister run build
RUN pnpm --filter=@alxt/kedeng-persister --prod deploy /build

FROM node:24-alpine

WORKDIR /app
COPY --from=builder /build/dist .
COPY --from=builder /build/node_modules ./node_modules

ENTRYPOINT [ "node", "/app/src/index.js" ]