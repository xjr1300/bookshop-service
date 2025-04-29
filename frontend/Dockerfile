FROM node:23-bookworm-slim AS builder
WORKDIR /app
COPY package.json package-lock.json ./
RUN npm install
COPY public ./public
COPY src ./src
COPY index.html ./
COPY tsconfig.* ./
COPY vite.config.ts ./
RUN npm run build

FROM nginx:alpine
COPY --from=builder /app/dist /usr/share/nginx/html
