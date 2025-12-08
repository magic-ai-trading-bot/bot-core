#!/bin/bash
# =============================================================================
# DOCKER CLEANUP SCRIPT
# =============================================================================
# Automatically cleans up Docker resources to prevent disk full issues
# Run via cron: 0 3 * * 0 /path/to/docker-cleanup.sh >> /var/log/docker-cleanup.log 2>&1
# =============================================================================

set -e

echo "=============================================="
echo "Docker Cleanup - $(date '+%Y-%m-%d %H:%M:%S')"
echo "=============================================="

# Show disk usage before cleanup
echo ""
echo "Disk usage BEFORE cleanup:"
df -h / | tail -1

# Show Docker disk usage before
echo ""
echo "Docker disk usage BEFORE cleanup:"
docker system df 2>/dev/null || echo "Docker not running"

# Remove stopped containers
echo ""
echo "Removing stopped containers..."
docker container prune -f 2>/dev/null || true

# Remove unused images (not just dangling)
echo ""
echo "Removing unused images..."
docker image prune -af 2>/dev/null || true

# Remove unused volumes
echo ""
echo "Removing unused volumes..."
docker volume prune -f 2>/dev/null || true

# Remove unused networks
echo ""
echo "Removing unused networks..."
docker network prune -f 2>/dev/null || true

# Remove build cache
echo ""
echo "Removing build cache..."
docker builder prune -af 2>/dev/null || true

# Show disk usage after cleanup
echo ""
echo "Disk usage AFTER cleanup:"
df -h / | tail -1

# Show Docker disk usage after
echo ""
echo "Docker disk usage AFTER cleanup:"
docker system df 2>/dev/null || echo "Docker not running"

echo ""
echo "Cleanup completed at $(date '+%Y-%m-%d %H:%M:%S')"
echo "=============================================="
