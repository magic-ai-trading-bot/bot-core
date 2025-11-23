#!/bin/bash

# =============================================================================
# Bot Core - Viettel VPS Auto Setup Script
# =============================================================================
# This script automatically sets up a fresh Viettel VPS for Bot Core deployment
# Tested on: Ubuntu 22.04 LTS
# VPS: Viettel IDC T2.GEN 03 (8 vCPU / 8 GB RAM / 100 GB SSD)
# =============================================================================

set -e  # Exit on error

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

# Configuration
BOT_USER="botadmin"
BOT_DIR="/home/$BOT_USER/projects/bot-core"
REPO_URL="https://github.com/magic-ai-trading-bot/bot-core.git"

# =============================================================================
# Helper Functions
# =============================================================================

print_header() {
    echo -e "\n${BLUE}╔════════════════════════════════════════════════════════════════╗${NC}"
    echo -e "${BLUE}║ $1${NC}"
    echo -e "${BLUE}╚════════════════════════════════════════════════════════════════╝${NC}\n"
}

print_success() {
    echo -e "${GREEN}✅ $1${NC}"
}

print_error() {
    echo -e "${RED}❌ $1${NC}"
}

print_warning() {
    echo -e "${YELLOW}⚠️  $1${NC}"
}

print_info() {
    echo -e "${BLUE}ℹ️  $1${NC}"
}

# =============================================================================
# Setup Functions
# =============================================================================

check_requirements() {
    print_header "Checking Requirements"

    # Check if running as root
    if [ "$EUID" -ne 0 ]; then
        print_error "Please run as root (use: sudo -i)"
        exit 1
    fi

    # Check Ubuntu version
    if ! grep -q "Ubuntu 22.04" /etc/os-release; then
        print_warning "This script is optimized for Ubuntu 22.04"
        print_info "Current OS: $(cat /etc/os-release | grep PRETTY_NAME)"
        read -p "Continue anyway? (y/N): " -n 1 -r
        echo
        if [[ ! $REPLY =~ ^[Yy]$ ]]; then
            exit 1
        fi
    fi

    print_success "Requirements check passed"
}

update_system() {
    print_header "Updating System"

    apt update -y
    apt upgrade -y
    apt install -y \
        curl \
        wget \
        git \
        vim \
        htop \
        net-tools \
        ufw \
        fail2ban \
        unzip \
        build-essential \
        software-properties-common \
        ca-certificates \
        gnupg \
        lsb-release

    print_success "System updated successfully"
}

create_user() {
    print_header "Creating Bot Admin User"

    if id "$BOT_USER" &>/dev/null; then
        print_warning "User $BOT_USER already exists"
    else
        adduser --disabled-password --gecos "" $BOT_USER
        echo "$BOT_USER:botcore123" | chpasswd  # Change this password!
        usermod -aG sudo $BOT_USER
        print_success "User $BOT_USER created"
        print_warning "Default password is 'botcore123' - CHANGE IT IMMEDIATELY!"
    fi
}

setup_firewall() {
    print_header "Configuring Firewall"

    # Enable UFW
    ufw --force enable

    # Default policies
    ufw default deny incoming
    ufw default allow outgoing

    # Allow SSH
    ufw allow 22/tcp

    # Allow application ports
    ufw allow 80/tcp      # HTTP
    ufw allow 443/tcp     # HTTPS
    ufw allow 8080/tcp    # Rust API
    ufw allow 8000/tcp    # Python API
    ufw allow 3000/tcp    # Frontend

    # Reload firewall
    ufw reload

    print_success "Firewall configured"
    ufw status verbose
}

install_docker() {
    print_header "Installing Docker"

    if command -v docker &>/dev/null; then
        print_warning "Docker already installed"
        docker --version
    else
        # Add Docker's official GPG key
        install -m 0755 -d /etc/apt/keyrings
        curl -fsSL https://download.docker.com/linux/ubuntu/gpg | gpg --dearmor -o /etc/apt/keyrings/docker.gpg
        chmod a+r /etc/apt/keyrings/docker.gpg

        # Add repository
        echo \
          "deb [arch=$(dpkg --print-architecture) signed-by=/etc/apt/keyrings/docker.gpg] https://download.docker.com/linux/ubuntu \
          $(lsb_release -cs) stable" | tee /etc/apt/sources.list.d/docker.list > /dev/null

        # Install Docker
        apt update
        apt install -y docker-ce docker-ce-cli containerd.io docker-buildx-plugin docker-compose-plugin

        # Add user to docker group
        usermod -aG docker $BOT_USER

        print_success "Docker installed successfully"
        docker --version
        docker compose version
    fi
}

install_mongodb() {
    print_header "Installing MongoDB"

    if command -v mongosh &>/dev/null; then
        print_warning "MongoDB already installed"
    else
        # Import MongoDB public GPG key
        curl -fsSL https://www.mongodb.org/static/pgp/server-7.0.asc | \
           gpg -o /usr/share/keyrings/mongodb-server-7.0.gpg --dearmor

        # Add MongoDB repository
        echo "deb [ arch=amd64,arm64 signed-by=/usr/share/keyrings/mongodb-server-7.0.gpg ] https://repo.mongodb.org/apt/ubuntu jammy/mongodb-org/7.0 multiverse" | \
           tee /etc/apt/sources.list.d/mongodb-org-7.0.list

        # Install MongoDB
        apt update
        apt install -y mongodb-org

        # Start and enable MongoDB
        systemctl start mongod
        systemctl enable mongod

        print_success "MongoDB installed successfully"
        systemctl status mongod --no-pager
    fi
}

configure_mongodb() {
    print_header "Configuring MongoDB"

    # Generate random password
    MONGO_PASSWORD=$(openssl rand -base64 32)

    # Wait for MongoDB to start
    sleep 5

    # Create admin user
    mongosh --eval "
    use admin;
    db.createUser({
      user: 'admin',
      pwd: '$MONGO_PASSWORD',
      roles: [ { role: 'userAdminAnyDatabase', db: 'admin' }, 'readWriteAnyDatabase' ]
    })
    " || print_warning "Admin user may already exist"

    # Create bot database user
    MONGO_BOT_PASSWORD=$(openssl rand -base64 32)

    mongosh --eval "
    use botcore_production;
    db.createUser({
      user: 'botcore_user',
      pwd: '$MONGO_BOT_PASSWORD',
      roles: [ { role: 'readWrite', db: 'botcore_production' } ]
    })
    " || print_warning "Bot user may already exist"

    # Enable authentication
    cat > /etc/mongod.conf <<EOF
# mongod.conf

storage:
  dbPath: /var/lib/mongodb

systemLog:
  destination: file
  logAppend: true
  path: /var/log/mongodb/mongod.log

net:
  port: 27017
  bindIp: 127.0.0.1

security:
  authorization: enabled

processManagement:
  timeZoneInfo: /usr/share/zoneinfo
EOF

    # Restart MongoDB
    systemctl restart mongod

    # Save credentials
    cat > /root/mongodb-credentials.txt <<EOF
MongoDB Credentials (Generated on $(date))
==========================================

Admin User:
  Username: admin
  Password: $MONGO_PASSWORD

Bot Core User:
  Username: botcore_user
  Password: $MONGO_BOT_PASSWORD
  Database: botcore_production

Connection String:
mongodb://botcore_user:$MONGO_BOT_PASSWORD@localhost:27017/botcore_production

⚠️  IMPORTANT: Keep this file secure!
⚠️  Copy these credentials before deleting this file!
EOF

    chmod 600 /root/mongodb-credentials.txt

    print_success "MongoDB configured with authentication"
    print_warning "MongoDB credentials saved to: /root/mongodb-credentials.txt"
    print_info "Connection string: mongodb://botcore_user:$MONGO_BOT_PASSWORD@localhost:27017/botcore_production"
}

clone_repository() {
    print_header "Cloning Bot Core Repository"

    # Create projects directory
    su - $BOT_USER -c "mkdir -p ~/projects"

    # Clone repository
    if [ -d "$BOT_DIR" ]; then
        print_warning "Repository already exists at $BOT_DIR"
        read -p "Do you want to update it? (y/N): " -n 1 -r
        echo
        if [[ $REPLY =~ ^[Yy]$ ]]; then
            su - $BOT_USER -c "cd $BOT_DIR && git pull"
            print_success "Repository updated"
        fi
    else
        su - $BOT_USER -c "git clone $REPO_URL ~/projects/bot-core"
        print_success "Repository cloned successfully"
    fi
}

generate_secrets() {
    print_header "Generating Secrets"

    JWT_SECRET=$(openssl rand -base64 64)
    SESSION_SECRET=$(openssl rand -base64 32)

    # Save secrets
    cat > /home/$BOT_USER/bot-secrets.txt <<EOF
Bot Core Secrets (Generated on $(date))
========================================

JWT_SECRET=$JWT_SECRET

SESSION_SECRET=$SESSION_SECRET

MongoDB Connection String:
(See /root/mongodb-credentials.txt)

⚠️  IMPORTANT: Use these values in your .env file!
⚠️  Keep this file secure!
EOF

    chown $BOT_USER:$BOT_USER /home/$BOT_USER/bot-secrets.txt
    chmod 600 /home/$BOT_USER/bot-secrets.txt

    print_success "Secrets generated and saved to /home/$BOT_USER/bot-secrets.txt"
}

create_env_file() {
    print_header "Creating Environment File"

    # Get MongoDB password from credentials file
    MONGO_BOT_PASSWORD=$(grep "Password:" /root/mongodb-credentials.txt | tail -1 | awk '{print $2}')

    # Create .env file
    su - $BOT_USER -c "cd ~/projects/bot-core && cp .env.production.example .env"

    # Update MongoDB connection string
    sed -i "s|mongodb://botcore_user:CHANGE_THIS_PASSWORD@localhost:27017/botcore_production|mongodb://botcore_user:$MONGO_BOT_PASSWORD@localhost:27017/botcore_production|g" /home/$BOT_USER/projects/bot-core/.env

    # Update secrets
    JWT_SECRET=$(grep "JWT_SECRET=" /home/$BOT_USER/bot-secrets.txt | cut -d'=' -f2)
    SESSION_SECRET=$(grep "SESSION_SECRET=" /home/$BOT_USER/bot-secrets.txt | cut -d'=' -f2)

    sed -i "s|JWT_SECRET=GENERATE_WITH_openssl_rand_-base64_64|JWT_SECRET=$JWT_SECRET|g" /home/$BOT_USER/projects/bot-core/.env
    sed -i "s|SESSION_SECRET=GENERATE_WITH_openssl_rand_-base64_32|SESSION_SECRET=$SESSION_SECRET|g" /home/$BOT_USER/projects/bot-core/.env

    chown $BOT_USER:$BOT_USER /home/$BOT_USER/projects/bot-core/.env
    chmod 600 /home/$BOT_USER/projects/bot-core/.env

    print_success ".env file created"
    print_warning "You MUST edit .env file and add:"
    print_info "  - BINANCE_API_KEY"
    print_info "  - BINANCE_SECRET_KEY"
    print_info "  - OPENAI_API_KEY"
}

print_summary() {
    print_header "Setup Complete!"

    echo -e "${GREEN}✅ Bot Core VPS setup completed successfully!${NC}\n"

    echo "📝 Next Steps:"
    echo "=============="
    echo "1. Switch to bot user:"
    echo "   ${BLUE}su - $BOT_USER${NC}"
    echo ""
    echo "2. Edit environment file:"
    echo "   ${BLUE}cd ~/projects/bot-core${NC}"
    echo "   ${BLUE}nano .env${NC}"
    echo ""
    echo "   Add these values:"
    echo "   - BINANCE_API_KEY=your_key"
    echo "   - BINANCE_SECRET_KEY=your_secret"
    echo "   - OPENAI_API_KEY=sk-your_key"
    echo ""
    echo "3. Deploy services:"
    echo "   ${BLUE}docker compose up -d${NC}"
    echo ""
    echo "4. Check status:"
    echo "   ${BLUE}docker compose ps${NC}"
    echo ""
    echo "5. Access dashboard:"
    echo "   ${BLUE}http://$(hostname -I | awk '{print $1}'):3000${NC}"
    echo ""
    echo "📂 Important Files:"
    echo "==================="
    echo "Secrets:     /home/$BOT_USER/bot-secrets.txt"
    echo "MongoDB:     /root/mongodb-credentials.txt"
    echo "Bot Directory: $BOT_DIR"
    echo ""
    echo "⚠️  Security Reminders:"
    echo "======================"
    echo "1. Change password for user $BOT_USER"
    echo "2. Setup SSH key authentication"
    echo "3. Disable root SSH login (optional)"
    echo "4. Keep TRADING_ENABLED=false until fully tested"
    echo "5. Always start with BINANCE_TESTNET=true"
    echo ""
    echo -e "${GREEN}Happy Trading! 🚀${NC}"
}

# =============================================================================
# Main Function
# =============================================================================

main() {
    clear

    echo -e "${BLUE}"
    echo "╔════════════════════════════════════════════════════════════════╗"
    echo "║                                                                ║"
    echo "║            Bot Core - Viettel VPS Auto Setup                  ║"
    echo "║                                                                ║"
    echo "║         Automated deployment for Viettel IDC T2.GEN 03        ║"
    echo "║            (8 vCPU / 8 GB RAM / 100 GB SSD)                   ║"
    echo "║                                                                ║"
    echo "╚════════════════════════════════════════════════════════════════╝"
    echo -e "${NC}\n"

    print_warning "This script will:"
    echo "  1. Update system packages"
    echo "  2. Create bot admin user"
    echo "  3. Configure firewall"
    echo "  4. Install Docker & Docker Compose"
    echo "  5. Install and configure MongoDB"
    echo "  6. Clone bot-core repository"
    echo "  7. Generate secrets"
    echo "  8. Create .env file"
    echo ""
    read -p "Continue? (y/N): " -n 1 -r
    echo
    if [[ ! $REPLY =~ ^[Yy]$ ]]; then
        print_info "Setup cancelled"
        exit 0
    fi

    check_requirements
    update_system
    create_user
    setup_firewall
    install_docker
    install_mongodb
    configure_mongodb
    clone_repository
    generate_secrets
    create_env_file
    print_summary
}

# Run main function
main
