# Multi-stage build for optimized React application with Bun
FROM oven/bun:1.1.45 AS builder

# Set working directory
WORKDIR /app

# Copy package files
COPY package.json bun.lockb* ./

# Install dependencies with all optional dependencies
RUN bun install --frozen-lockfile

# Update dependencies to get security patches
RUN bun update esbuild || true

# Add missing rollup dependencies for ARM64
RUN bun add -D @rollup/rollup-linux-arm64-gnu @rollup/rollup-linux-arm64-musl || true

# Copy source code
COPY . .

# Build the application
RUN bun run build

# Production stage
FROM oven/bun:1.1.45-alpine

# Update system packages for security patches and install curl for health checks
RUN apk update && apk upgrade && apk add --no-cache curl

# Create app user
RUN addgroup -g 1001 -S appuser && \
    adduser -S appuser -u 1001

# Set working directory
WORKDIR /app

# Copy built application from builder stage
COPY --from=builder /app/dist ./dist
COPY --from=builder /app/package.json ./
COPY --from=builder /app/node_modules ./node_modules

# Set ownership
RUN chown -R appuser:appuser /app

# Switch to app user
USER appuser

# Set environment variables
ENV NODE_ENV=production

# Expose port
EXPOSE 3000

# Health check
HEALTHCHECK --interval=30s --timeout=10s --start-period=30s --retries=3 \
    CMD curl -f http://localhost:3000/ || exit 1

# Serve the built application using Vite preview
CMD ["bun", "run", "preview", "--host", "0.0.0.0", "--port", "3000"] 