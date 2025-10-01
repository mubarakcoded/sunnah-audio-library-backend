#!/bin/bash

# Sunnah Audio Rust Service Deployment Script
# This script automates the deployment process

set -e  # Exit on any error

# Configuration
SERVICE_NAME="sunnah-audio-rust-service"
SERVICE_USER="muryarsunnah"
SERVICE_DIR="/home/muryarsunnah/htdocs/www.muryarsunnah.com"
RUST_SERVICE_DIR="$SERVICE_DIR/rust-service"
BACKUP_DIR="$SERVICE_DIR/backups"
LOG_DIR="/var/log/$SERVICE_NAME"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Functions
log_info() {
    echo -e "${GREEN}[INFO]${NC} $1"
}

log_warn() {
    echo -e "${YELLOW}[WARN]${NC} $1"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# Check if running as root or with sudo
check_permissions() {
    if [[ $EUID -ne 0 ]]; then
        log_error "This script must be run as root or with sudo"
        exit 1
    fi
}

# Install system dependencies
install_dependencies() {
    log_info "Installing system dependencies..."
    
    # Update package list
    apt update
    
    # Install required packages
    apt install -y curl build-essential pkg-config libssl-dev \
        postgresql-client mysql-client redis-tools nginx \
        supervisor systemd
    
    # Install Rust if not present
    if ! command -v cargo &> /dev/null; then
        log_info "Installing Rust..."
        curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
        source ~/.cargo/env
    fi
}

# Create necessary directories
create_directories() {
    log_info "Creating necessary directories..."
    
    # Create service directories
    mkdir -p "$RUST_SERVICE_DIR"
    mkdir -p "$BACKUP_DIR"
    mkdir -p "$LOG_DIR"
    mkdir -p "$SERVICE_DIR/web/images"
    mkdir -p "$SERVICE_DIR/web/uploads"
    
    # Set proper permissions
    chown -R "$SERVICE_USER:$SERVICE_USER" "$SERVICE_DIR"
    chmod -R 755 "$SERVICE_DIR"
}

# Build the Rust application
build_application() {
    log_info "Building Rust application..."
    
    # Set environment for production build
    export SUNNAH_AUDIO_APP_ENVIRONMENT=production
    export RUST_LOG=info
    
    # Navigate to project directory
    cd "$(dirname "$0")"
    
    # Clean previous build
    cargo clean
    
    # Build in release mode
    cargo build --release
    
    log_info "Build completed successfully"
}

# Deploy the application
deploy_application() {
    log_info "Deploying application..."
    
    # Create backup of current deployment
    if [ -d "$RUST_SERVICE_DIR" ] && [ "$(ls -A $RUST_SERVICE_DIR)" ]; then
        log_info "Creating backup of current deployment..."
        BACKUP_NAME="backup-$(date +%Y%m%d-%H%M%S)"
        cp -r "$RUST_SERVICE_DIR" "$BACKUP_DIR/$BACKUP_NAME"
    fi
    
    # Copy built binary
    cp target/release/sunnah_audio_rust_service "$RUST_SERVICE_DIR/"
    
    # Copy configuration files
    cp -r src/core/configurations "$RUST_SERVICE_DIR/"
    
    # Copy static files if they exist
    if [ -d "static" ]; then
        cp -r static "$RUST_SERVICE_DIR/"
    fi
    
    # Set proper permissions
    chown -R "$SERVICE_USER:$SERVICE_USER" "$RUST_SERVICE_DIR"
    chmod +x "$RUST_SERVICE_DIR/sunnah_audio_rust_service"
    
    log_info "Application deployed successfully"
}

# Setup systemd service
setup_systemd_service() {
    log_info "Setting up systemd service..."
    
    cat > /etc/systemd/system/$SERVICE_NAME.service << EOF
[Unit]
Description=Sunnah Audio Rust Service
After=network.target postgresql.service mysql.service redis.service

[Service]
Type=simple
User=$SERVICE_USER
Group=$SERVICE_USER
WorkingDirectory=$RUST_SERVICE_DIR
ExecStart=$RUST_SERVICE_DIR/sunnah_audio_rust_service
Restart=always
RestartSec=5
Environment=SUNNAH_AUDIO_APP_ENVIRONMENT=production
Environment=RUST_LOG=info

# Security settings
NoNewPrivileges=true
PrivateTmp=true
ProtectSystem=strict
ProtectHome=true
ReadWritePaths=$RUST_SERVICE_DIR $SERVICE_DIR/web

# Logging
StandardOutput=journal
StandardError=journal
SyslogIdentifier=$SERVICE_NAME

[Install]
WantedBy=multi-user.target
EOF

    # Reload systemd and enable service
    systemctl daemon-reload
    systemctl enable $SERVICE_NAME
    
    log_info "Systemd service configured"
}

# Setup nginx reverse proxy
setup_nginx() {
    log_info "Setting up nginx reverse proxy..."
    
    cat > /etc/nginx/sites-available/$SERVICE_NAME << EOF
server {
    listen 80;
    server_name www.muryarsunnah.com muryarsunnah.com;
    
    # Redirect HTTP to HTTPS
    return 301 https://\$server_name\$request_uri;
}

server {
    listen 443 ssl http2;
    server_name www.muryarsunnah.com muryarsunnah.com;
    
    # SSL configuration (update with your certificate paths)
    ssl_certificate /path/to/your/certificate.crt;
    ssl_certificate_key /path/to/your/private.key;
    
    # Security headers
    add_header X-Frame-Options DENY;
    add_header X-Content-Type-Options nosniff;
    add_header X-XSS-Protection "1; mode=block";
    
    # API routes to Rust service
    location /api/v1/ {
        proxy_pass http://127.0.0.1:8990;
        proxy_set_header Host \$host;
        proxy_set_header X-Real-IP \$remote_addr;
        proxy_set_header X-Forwarded-For \$proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto \$scheme;
        
        # CORS headers
        add_header Access-Control-Allow-Origin *;
        add_header Access-Control-Allow-Methods "GET, POST, PUT, DELETE, OPTIONS";
        add_header Access-Control-Allow-Headers "Content-Type, Authorization";
        
        # Handle preflight requests
        if (\$request_method = 'OPTIONS') {
            add_header Access-Control-Allow-Origin *;
            add_header Access-Control-Allow-Methods "GET, POST, PUT, DELETE, OPTIONS";
            add_header Access-Control-Allow-Headers "Content-Type, Authorization";
            add_header Access-Control-Max-Age 1728000;
            add_header Content-Type text/plain;
            add_header Content-Length 0;
            return 204;
        }
    }
    
    # Serve static files directly from nginx
    location /static/ {
        alias $SERVICE_DIR/web/;
        expires 1y;
        add_header Cache-Control "public, immutable";
    }
    
    # Serve images
    location /images/ {
        alias $SERVICE_DIR/web/images/;
        expires 1y;
        add_header Cache-Control "public, immutable";
    }
    
    # Serve uploads
    location /uploads/ {
        alias $SERVICE_DIR/web/uploads/;
        expires 1y;
        add_header Cache-Control "public, immutable";
    }
    
    # Main Yii2 application
    location / {
        root $SERVICE_DIR/web;
        index index.php;
        
        try_files \$uri \$uri/ /index.php?\$args;
        
        location ~ \.php$ {
            fastcgi_pass unix:/var/run/php/php8.1-fpm.sock;
            fastcgi_index index.php;
            fastcgi_param SCRIPT_FILENAME \$document_root\$fastcgi_script_name;
            include fastcgi_params;
        }
    }
}
EOF

    # Enable the site
    ln -sf /etc/nginx/sites-available/$SERVICE_NAME /etc/nginx/sites-enabled/
    
    # Test nginx configuration
    nginx -t
    
    log_info "Nginx configuration created"
}

# Start services
start_services() {
    log_info "Starting services..."
    
    # Start the Rust service
    systemctl start $SERVICE_NAME
    
    # Reload nginx
    systemctl reload nginx
    
    # Check service status
    if systemctl is-active --quiet $SERVICE_NAME; then
        log_info "Service started successfully"
    else
        log_error "Failed to start service"
        systemctl status $SERVICE_NAME
        exit 1
    fi
}

# Health check
health_check() {
    log_info "Performing health check..."
    
    # Wait a moment for service to start
    sleep 5
    
    # Check if service is responding
    if curl -f http://localhost:8990/api/v1/health > /dev/null 2>&1; then
        log_info "Health check passed"
    else
        log_warn "Health check failed - service may not be ready yet"
    fi
}

# Main deployment function
main() {
    log_info "Starting deployment of $SERVICE_NAME..."
    
    check_permissions
    install_dependencies
    create_directories
    build_application
    deploy_application
    setup_systemd_service
    setup_nginx
    start_services
    health_check
    
    log_info "Deployment completed successfully!"
    log_info "Service is running on port 8990"
    log_info "Check logs with: journalctl -u $SERVICE_NAME -f"
}

# Run main function
main "$@"
