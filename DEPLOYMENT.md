# Sunnah Audio Rust Service - Deployment Guide

This guide provides step-by-step instructions for deploying the Sunnah Audio Rust Service to your production server.

## Prerequisites

- Ubuntu/Debian server with root access
- Domain: `www.muryarsunnah.com`
- Existing Yii2 application at `/home/muryarsunnah/htdocs/www.muryarsunnah.com/web`
- Database access (PostgreSQL/MySQL)
- Redis server
- SSL certificate for HTTPS

## Server Requirements

- **OS**: Ubuntu 20.04+ or Debian 11+
- **RAM**: Minimum 2GB, Recommended 4GB+
- **CPU**: 2+ cores
- **Storage**: 10GB+ free space
- **Network**: Public IP with domain configured

## Step-by-Step Deployment

### 1. Server Preparation

```bash
# Update system packages
sudo apt update && sudo apt upgrade -y

# Install essential tools
sudo apt install -y curl wget git build-essential pkg-config libssl-dev
```

### 2. Install Rust

```bash
# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
source ~/.cargo/env

# Verify installation
rustc --version
cargo --version
```

### 3. Install Database and Redis

```bash
# Install PostgreSQL
sudo apt install -y postgresql postgresql-contrib

# Install MySQL (if needed)
sudo apt install -y mysql-server

# Install Redis
sudo apt install -y redis-server

# Start services
sudo systemctl start postgresql
sudo systemctl start mysql
sudo systemctl start redis-server

# Enable services to start on boot
sudo systemctl enable postgresql
sudo systemctl enable mysql
sudo systemctl enable redis-server
```

### 4. Configure Database

```bash
# Create database and user for PostgreSQL
sudo -u postgres psql
```

In PostgreSQL shell:
```sql
CREATE DATABASE muryar_sunnah;
CREATE USER muryar_user WITH PASSWORD 'your_secure_password';
GRANT ALL PRIVILEGES ON DATABASE muryar_sunnah TO muryar_user;
\q
```

### 5. Deploy the Application

#### Option A: Automated Deployment (Recommended)

```bash
# Clone or upload your project to the server
cd /home/muryarsunnah/htdocs/www.muryarsunnah.com/

# Make the deployment script executable
chmod +x deploy.sh

# Run the automated deployment
sudo ./deploy.sh
```

#### Option B: Manual Deployment

```bash
# 1. Create service directory
sudo mkdir -p /home/muryarsunnah/htdocs/www.muryarsunnah.com/rust-service
sudo chown -R muryarsunnah:muryarsunnah /home/muryarsunnah/htdocs/www.muryarsunnah.com/rust-service

# 2. Copy project files
cp -r /path/to/your/rust/project/* /home/muryarsunnah/htdocs/www.muryarsunnah.com/rust-service/

# 3. Update production configuration
nano /home/muryarsunnah/htdocs/www.muryarsunnah.com/rust-service/src/core/configurations/production.yaml
```

### 6. Configure Production Settings

Edit the production configuration file:

```bash
nano /home/muryarsunnah/htdocs/www.muryarsunnah.com/rust-service/src/core/configurations/production.yaml
```

Update the following values:

```yaml
sunnah_audio_server_config:
  port: 8990
  host: "0.0.0.0"
  base_url: "https://www.muryarsunnah.com"

jwt_auth_config:
  secret: "YOUR_STRONG_JWT_SECRET_HERE"  # Generate a strong secret
  token_expiration_time: 7

app_paths:
  static_images: "/api/v1/static/images"
  static_uploads: "/api/v1/static/audio"
  static_audio: "/api/v1/static/audio"
  images_dir: "/home/muryarsunnah/htdocs/www.muryarsunnah.com/web/images"
  uploads_dir: "/home/muryarsunnah/htdocs/www.muryarsunnah.com/web/uploads"

postgres:
  username: "muryar_user"
  password: "your_secure_password"
  host: "localhost"
  port: 5432
  database_name: "muryar_sunnah"

mysql:
  username: "your_mysql_username"
  password: "your_mysql_password"
  host: "localhost"
  port: 3306
  database_name: "muryar_sunnah"

redis:
  host: "localhost"
  port: 6379
  password: "your_redis_password"

smtp:
  host: "your_smtp_host"
  port: 587
  username: "your_smtp_username"
  password: "your_smtp_password"
  from_email: "noreply@muryarsunnah.com"
  from_name: "Muryar Sunnah"
```

### 7. Build and Deploy

```bash
cd /home/muryarsunnah/htdocs/www.muryarsunnah.com/rust-service

# Set environment
export SUNNAH_AUDIO_APP_ENVIRONMENT=production
export RUST_LOG=info

# Build the application
cargo build --release

# Copy the binary to service directory
cp target/release/sunnah_audio_rust_service /home/muryarsunnah/htdocs/www.muryarsunnah.com/rust-service/
```

### 8. Setup Systemd Service

Create the systemd service file:

```bash
sudo nano /etc/systemd/system/sunnah-audio-rust-service.service
```

Add the following content:

```ini
[Unit]
Description=Sunnah Audio Rust Service
After=network.target postgresql.service mysql.service redis.service

[Service]
Type=simple
User=muryarsunnah
Group=muryarsunnah
WorkingDirectory=/home/muryarsunnah/htdocs/www.muryarsunnah.com/rust-service
ExecStart=/home/muryarsunnah/htdocs/www.muryarsunnah.com/rust-service/sunnah_audio_rust_service
Restart=always
RestartSec=5
Environment=SUNNAH_AUDIO_APP_ENVIRONMENT=production
Environment=RUST_LOG=info

# Security settings
NoNewPrivileges=true
PrivateTmp=true
ProtectSystem=strict
ProtectHome=true
ReadWritePaths=/home/muryarsunnah/htdocs/www.muryarsunnah.com/rust-service /home/muryarsunnah/htdocs/www.muryarsunnah.com/web

# Logging
StandardOutput=journal
StandardError=journal
SyslogIdentifier=sunnah-audio-rust-service

[Install]
WantedBy=multi-user.target
```

Enable and start the service:

```bash
sudo systemctl daemon-reload
sudo systemctl enable sunnah-audio-rust-service
sudo systemctl start sunnah-audio-rust-service
```

### 9. Configure Nginx

Create nginx configuration:

```bash
sudo nano /etc/nginx/sites-available/sunnah-audio-rust-service
```

Add the following configuration:

```nginx
server {
    listen 80;
    server_name www.muryarsunnah.com muryarsunnah.com;
    
    # Redirect HTTP to HTTPS
    return 301 https://$server_name$request_uri;
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
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;
        
        # CORS headers
        add_header Access-Control-Allow-Origin *;
        add_header Access-Control-Allow-Methods "GET, POST, PUT, DELETE, OPTIONS";
        add_header Access-Control-Allow-Headers "Content-Type, Authorization";
        
        # Handle preflight requests
        if ($request_method = 'OPTIONS') {
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
        alias /home/muryarsunnah/htdocs/www.muryarsunnah.com/web/;
        expires 1y;
        add_header Cache-Control "public, immutable";
    }
    
    # Serve images
    location /images/ {
        alias /home/muryarsunnah/htdocs/www.muryarsunnah.com/web/images/;
        expires 1y;
        add_header Cache-Control "public, immutable";
    }
    
    # Serve uploads
    location /uploads/ {
        alias /home/muryarsunnah/htdocs/www.muryarsunnah.com/web/uploads/;
        expires 1y;
        add_header Cache-Control "public, immutable";
    }
    
    # Main Yii2 application
    location / {
        root /home/muryarsunnah/htdocs/www.muryarsunnah.com/web;
        index index.php;
        
        try_files $uri $uri/ /index.php?$args;
        
        location ~ \.php$ {
            fastcgi_pass unix:/var/run/php/php8.1-fpm.sock;
            fastcgi_index index.php;
            fastcgi_param SCRIPT_FILENAME $document_root$fastcgi_script_name;
            include fastcgi_params;
        }
    }
}
```

Enable the site:

```bash
sudo ln -s /etc/nginx/sites-available/sunnah-audio-rust-service /etc/nginx/sites-enabled/
sudo nginx -t
sudo systemctl reload nginx
```

### 10. Setup SSL Certificate

If you don't have SSL certificates, use Let's Encrypt:

```bash
# Install certbot
sudo apt install -y certbot python3-certbot-nginx

# Obtain SSL certificate
sudo certbot --nginx -d www.muryarsunnah.com -d muryarsunnah.com
```

### 11. Run Database Migrations

```bash
# Navigate to the service directory
cd /home/muryarsunnah/htdocs/www.muryarsunnah.com/rust-service

# Set environment
export SUNNAH_AUDIO_APP_ENVIRONMENT=production

# Run migrations (if you have any)
# cargo run --bin migrate
```

### 12. Verify Deployment

```bash
# Check service status
sudo systemctl status sunnah-audio-rust-service

# Check logs
sudo journalctl -u sunnah-audio-rust-service -f

# Test the API
curl http://localhost:8990/api/v1/health

# Test through nginx
curl https://www.muryarsunnah.com/api/v1/health
```

## Monitoring and Maintenance

### Service Management

```bash
# Start service
sudo systemctl start sunnah-audio-rust-service

# Stop service
sudo systemctl stop sunnah-audio-rust-service

# Restart service
sudo systemctl restart sunnah-audio-rust-service

# Check status
sudo systemctl status sunnah-audio-rust-service

# View logs
sudo journalctl -u sunnah-audio-rust-service -f
```

### Log Management

```bash
# View recent logs
sudo journalctl -u sunnah-audio-rust-service --since "1 hour ago"

# View logs with timestamps
sudo journalctl -u sunnah-audio-rust-service -o short-iso

# Clear old logs
sudo journalctl --vacuum-time=7d
```

### Backup Strategy

```bash
# Create backup script
sudo nano /usr/local/bin/backup-sunnah-service.sh
```

Add the following content:

```bash
#!/bin/bash
BACKUP_DIR="/home/muryarsunnah/htdocs/www.muryarsunnah.com/backups"
SERVICE_DIR="/home/muryarsunnah/htdocs/www.muryarsunnah.com/rust-service"
DATE=$(date +%Y%m%d-%H%M%S)

# Create backup
tar -czf "$BACKUP_DIR/sunnah-service-$DATE.tar.gz" -C "$SERVICE_DIR" .

# Keep only last 7 backups
find "$BACKUP_DIR" -name "sunnah-service-*.tar.gz" -mtime +7 -delete

echo "Backup completed: sunnah-service-$DATE.tar.gz"
```

Make it executable:

```bash
sudo chmod +x /usr/local/bin/backup-sunnah-service.sh
```

Setup cron job for daily backups:

```bash
sudo crontab -e
```

Add this line:

```
0 2 * * * /usr/local/bin/backup-sunnah-service.sh
```

## Troubleshooting

### Common Issues

1. **Service won't start**
   ```bash
   sudo journalctl -u sunnah-audio-rust-service -n 50
   ```

2. **Database connection issues**
   - Check database credentials in production.yaml
   - Verify database is running: `sudo systemctl status postgresql`

3. **Port conflicts**
   ```bash
   sudo netstat -tlnp | grep 8990
   ```

4. **Permission issues**
   ```bash
   sudo chown -R muryarsunnah:muryarsunnah /home/muryarsunnah/htdocs/www.muryarsunnah.com/
   ```

5. **Nginx configuration issues**
   ```bash
   sudo nginx -t
   sudo systemctl status nginx
   ```

### Performance Optimization

1. **Enable gzip compression in nginx**
2. **Setup Redis for caching**
3. **Configure log rotation**
4. **Monitor resource usage**

## Security Considerations

1. **Firewall setup**
   ```bash
   sudo ufw allow 22
   sudo ufw allow 80
   sudo ufw allow 443
   sudo ufw enable
   ```

2. **Regular security updates**
   ```bash
   sudo apt update && sudo apt upgrade -y
   ```

3. **Database security**
   - Use strong passwords
   - Limit database access
   - Regular backups

4. **SSL/TLS configuration**
   - Use strong cipher suites
   - Enable HSTS
   - Regular certificate renewal

## Updates and Maintenance

### Updating the Service

```bash
# Stop the service
sudo systemctl stop sunnah-audio-rust-service

# Backup current version
sudo /usr/local/bin/backup-sunnah-service.sh

# Update code
cd /home/muryarsunnah/htdocs/www.muryarsunnah.com/rust-service
git pull origin main

# Rebuild
cargo build --release

# Restart service
sudo systemctl start sunnah-audio-rust-service
```

### Health Monitoring

Create a health check script:

```bash
sudo nano /usr/local/bin/health-check.sh
```

```bash
#!/bin/bash
HEALTH_URL="http://localhost:8990/api/v1/health"

if curl -f "$HEALTH_URL" > /dev/null 2>&1; then
    echo "Service is healthy"
    exit 0
else
    echo "Service is unhealthy"
    # Restart service
    sudo systemctl restart sunnah-audio-rust-service
    exit 1
fi
```

Make it executable and setup monitoring:

```bash
sudo chmod +x /usr/local/bin/health-check.sh

# Add to crontab for monitoring every 5 minutes
sudo crontab -e
```

Add this line:

```
*/5 * * * * /usr/local/bin/health-check.sh
```

## Support

For issues or questions:

1. Check service logs: `sudo journalctl -u sunnah-audio-rust-service -f`
2. Verify configuration: `sudo nginx -t`
3. Test database connectivity
4. Check firewall rules: `sudo ufw status`

---

**Note**: Remember to update all placeholder values (passwords, secrets, paths) with your actual production values before deployment.
