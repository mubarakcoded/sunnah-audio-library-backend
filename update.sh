#!/bin/bash

# Quick Update Script for Sunnah Audio Rust Service
# This script handles updates to the running service

set -e

SERVICE_NAME="sunnah-audio-rust-service"
SERVICE_DIR="/home/muryarsunnah/htdocs/www.muryarsunnah.com/rust-service"
BACKUP_DIR="/home/muryarsunnah/htdocs/www.muryarsunnah.com/backups"

# Colors
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m'

log_info() {
    echo -e "${GREEN}[INFO]${NC} $1"
}

log_warn() {
    echo -e "${YELLOW}[WARN]${NC} $1"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# Check if running as root
if [[ $EUID -ne 0 ]]; then
    log_error "This script must be run as root or with sudo"
    exit 1
fi

# Create backup
create_backup() {
    log_info "Creating backup of current deployment..."
    BACKUP_NAME="backup-$(date +%Y%m%d-%H%M%S)"
    mkdir -p "$BACKUP_DIR"
    
    if [ -d "$SERVICE_DIR" ] && [ "$(ls -A $SERVICE_DIR)" ]; then
        tar -czf "$BACKUP_DIR/$BACKUP_NAME.tar.gz" -C "$SERVICE_DIR" .
        log_info "Backup created: $BACKUP_NAME.tar.gz"
    else
        log_warn "No existing deployment found to backup"
    fi
}

# Stop service
stop_service() {
    log_info "Stopping service..."
    systemctl stop $SERVICE_NAME || log_warn "Service was not running"
}

# Update code
update_code() {
    log_info "Updating application code..."
    
    # Navigate to project directory
    cd "$(dirname "$0")"
    
    # Pull latest changes (if using git)
    # git pull origin main
    
    # Set environment
    export SUNNAH_AUDIO_APP_ENVIRONMENT=production
    export RUST_LOG=info
    
    # Clean and build
    log_info "Building application..."
    cargo clean
    cargo build --release
    
    log_info "Build completed"
}

# Deploy new version
deploy_new_version() {
    log_info "Deploying new version..."
    
    # Copy new binary
    cp target/release/sunnah_audio_rust_service "$SERVICE_DIR/"
    
    # Copy updated configuration if needed
    if [ -d "src/core/configurations" ]; then
        cp -r src/core/configurations "$SERVICE_DIR/"
    fi
    
    # Copy static files if they exist
    if [ -d "static" ]; then
        cp -r static "$SERVICE_DIR/"
    fi
    
    # Set proper permissions
    chown -R muryarsunnah:muryarsunnah "$SERVICE_DIR"
    chmod +x "$SERVICE_DIR/sunnah_audio_rust_service"
    
    log_info "Deployment completed"
}

# Start service
start_service() {
    log_info "Starting service..."
    systemctl start $SERVICE_NAME
    
    # Wait a moment for service to start
    sleep 3
    
    # Check if service started successfully
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
    
    # Wait for service to be ready
    sleep 5
    
    # Check if service is responding
    if curl -f http://localhost:8990/api/v1/health > /dev/null 2>&1; then
        log_info "Health check passed - service is running correctly"
    else
        log_warn "Health check failed - service may not be ready yet"
        log_info "Check logs with: journalctl -u $SERVICE_NAME -f"
    fi
}

# Rollback function
rollback() {
    log_warn "Rolling back to previous version..."
    
    # Find the most recent backup
    LATEST_BACKUP=$(ls -t "$BACKUP_DIR"/backup-*.tar.gz 2>/dev/null | head -n1)
    
    if [ -n "$LATEST_BACKUP" ]; then
        log_info "Rolling back to: $LATEST_BACKUP"
        
        # Stop service
        systemctl stop $SERVICE_NAME
        
        # Restore backup
        tar -xzf "$LATEST_BACKUP" -C "$SERVICE_DIR"
        
        # Set permissions
        chown -R muryarsunnah:muryarsunnah "$SERVICE_DIR"
        chmod +x "$SERVICE_DIR/sunnah_audio_rust_service"
        
        # Start service
        systemctl start $SERVICE_NAME
        
        log_info "Rollback completed"
    else
        log_error "No backup found for rollback"
        exit 1
    fi
}

# Main update function
main() {
    log_info "Starting update process..."
    
    create_backup
    stop_service
    update_code
    deploy_new_version
    start_service
    health_check
    
    log_info "Update completed successfully!"
    log_info "Service is running on port 8990"
    log_info "Check logs with: journalctl -u $SERVICE_NAME -f"
}

# Handle command line arguments
case "${1:-}" in
    "rollback")
        rollback
        ;;
    "backup")
        create_backup
        ;;
    "health")
        health_check
        ;;
    *)
        main
        ;;
esac
