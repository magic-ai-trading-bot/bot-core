#!/bin/bash

# Script to reorganize bot-core folder structure
# This is OPTIONAL - only run if you want the new structure

set -e

echo "🔄 Bot Core Folder Reorganization Script"
echo "========================================"
echo ""
echo "This will reorganize the folder structure to:"
echo "- Group infrastructure configs in infrastructure/"
echo "- Centralize tests in tests/"
echo ""
read -p "Do you want to proceed? (y/N): " -n 1 -r
echo ""

if [[ ! $REPLY =~ ^[Yy]$ ]]; then
    echo "❌ Reorganization cancelled"
    exit 0
fi

# Create new directories
echo "📁 Creating new directory structure..."
mkdir -p infrastructure/{docker,kubernetes,terraform,nginx,kong,rabbitmq,mongodb,monitoring}
mkdir -p tests/{e2e,integration,performance}

# Move Docker files
echo "🐳 Moving Docker files..."
if [ -f "docker-compose.yml" ]; then
    cp docker-compose.yml infrastructure/docker/
    echo "  ✓ Copied docker-compose.yml"
fi
if [ -f "docker-compose.prod.yml" ]; then
    cp docker-compose.prod.yml infrastructure/docker/
    echo "  ✓ Copied docker-compose.prod.yml"
fi

# Move infrastructure configs
echo "🔧 Moving infrastructure configs..."
if [ -d "istio" ]; then
    cp -r istio/* infrastructure/kubernetes/ 2>/dev/null || true
    echo "  ✓ Copied istio configs"
fi
if [ -d "nginx" ]; then
    cp -r nginx/* infrastructure/nginx/ 2>/dev/null || true
    echo "  ✓ Copied nginx configs"
fi
if [ -d "kong" ]; then
    cp -r kong/* infrastructure/kong/ 2>/dev/null || true
    echo "  ✓ Copied kong configs"
fi
if [ -d "rabbitmq" ]; then
    cp -r rabbitmq/* infrastructure/rabbitmq/ 2>/dev/null || true
    echo "  ✓ Copied rabbitmq configs"
fi
if [ -d "mongodb" ]; then
    cp -r mongodb/* infrastructure/mongodb/ 2>/dev/null || true
    echo "  ✓ Copied mongodb configs"
fi
if [ -d "monitoring" ]; then
    cp -r monitoring/* infrastructure/monitoring/ 2>/dev/null || true
    echo "  ✓ Copied monitoring configs"
fi
if [ -d "terraform" ]; then
    cp -r terraform/* infrastructure/terraform/ 2>/dev/null || true
    echo "  ✓ Copied terraform configs"
fi

# Move tests
echo "🧪 Moving test files..."
if [ -d "e2e" ]; then
    cp -r e2e/* tests/e2e/ 2>/dev/null || true
    echo "  ✓ Copied e2e tests"
fi

# Create symlinks for backward compatibility
echo "🔗 Creating symlinks for backward compatibility..."
ln -sf infrastructure/docker/docker-compose.yml docker-compose.yml 2>/dev/null || true
ln -sf infrastructure/docker/docker-compose.prod.yml docker-compose.prod.yml 2>/dev/null || true

# Update paths in key files
echo "📝 Updating paths in configuration files..."

# Update Makefile
if [ -f "Makefile" ]; then
    sed -i.bak 's|docker-compose\.yml|infrastructure/docker/docker-compose.yml|g' Makefile
    sed -i.bak 's|docker-compose\.prod\.yml|infrastructure/docker/docker-compose.prod.yml|g' Makefile
    echo "  ✓ Updated Makefile"
fi

# Update bot.sh
if [ -f "scripts/bot.sh" ]; then
    sed -i.bak 's|docker-compose\.yml|infrastructure/docker/docker-compose.yml|g' scripts/bot.sh
    sed -i.bak 's|docker-compose\.prod\.yml|infrastructure/docker/docker-compose.prod.yml|g' scripts/bot.sh
    echo "  ✓ Updated bot.sh"
fi

echo ""
echo "✅ Reorganization complete!"
echo ""
echo "📋 Next steps:"
echo "1. Review the new structure in infrastructure/ and tests/"
echo "2. Test that everything still works with: ./scripts/bot.sh verify"
echo "3. If happy, remove old directories:"
echo "   rm -rf istio nginx kong rabbitmq mongodb monitoring terraform e2e"
echo "4. Update any CI/CD pipelines that reference old paths"
echo ""
echo "⚠️  Note: Symlinks created for docker-compose files for compatibility"