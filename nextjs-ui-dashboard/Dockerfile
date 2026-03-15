# Multi-stage build for optimized React application with Bun
FROM oven/bun:1.2-alpine AS builder

# Set working directory
WORKDIR /app

# Copy package files and root lockfile for deterministic builds
COPY nextjs-ui-dashboard/package.json ./
COPY bun.lock ./

# Update system packages for security patches
RUN apk upgrade --no-cache

# Install dependencies with frozen lockfile
RUN bun install --frozen-lockfile || bun install

# Copy source code
COPY nextjs-ui-dashboard/ .

# Build the application
RUN bun run build

# Production stage
FROM oven/bun:1.2-alpine

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
