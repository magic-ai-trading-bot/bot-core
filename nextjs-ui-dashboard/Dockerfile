# Multi-stage build for optimized React application with Bun
FROM oven/bun:1 AS builder

# Set working directory
WORKDIR /app

# Copy package files
COPY package.json bun.lockb* ./

# Install dependencies with all optional dependencies
RUN bun install --frozen-lockfile

# Add missing rollup dependencies for ARM64
RUN bun add -D @rollup/rollup-linux-arm64-gnu @rollup/rollup-linux-arm64-musl || true

# Copy source code
COPY . .

# Build the application
RUN bun run build

# Production stage
FROM oven/bun:1-alpine

# Install serve for serving static files
RUN bun add -g serve

# Create app user
RUN addgroup -g 1001 -S appuser && \
    adduser -S appuser -u 1001

# Set working directory
WORKDIR /app

# Copy built application from builder stage
COPY --from=builder /app/dist ./dist
COPY --from=builder /app/package.json ./

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

# Serve the built application
CMD ["serve", "-s", "dist", "-l", "3000"] 