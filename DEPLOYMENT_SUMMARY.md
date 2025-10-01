# Sunnah Audio Rust Service - Deployment Summary

## 🚀 Quick Start Deployment

### Option 1: Automated Deployment (Recommended)

```bash
# 1. Upload your project to the server
scp -r /path/to/your/project muryarsunnah@your-server:/home/muryarsunnah/htdocs/www.muryarsunnah.com/

# 2. SSH into your server
ssh muryarsunnah@your-server

# 3. Navigate to the project directory
cd /home/muryarsunnah/htdocs/www.muryarsunnah.com/rust-service

# 4. Make deployment script executable
chmod +x deploy.sh

# 5. Run automated deployment
sudo ./deploy.sh
```

### Option 2: Docker Deployment

```bash
# 1. Upload project files
scp -r /path/to/your/project muryarsunnah@your-server:/home/muryarsunnah/htdocs/www.muryarsunnah.com/

# 2. SSH into server
ssh muryarsunnah@your-server

# 3. Navigate to project directory
cd /home/muryarsunnah/htdocs/www.muryarsunnah.com/rust-service

# 4. Start with Docker Compose
docker-compose up -d
```

## 📋 Pre-Deployment Checklist

- [ ] Server has Ubuntu 20.04+ or Debian 11+
- [ ] Domain `www.muryarsunnah.com` points to your server
- [ ] SSL certificate is available
- [ ] Database credentials are ready
- [ ] Redis server is configured
- [ ] SMTP credentials are available

## 🔧 Configuration Files

### 1. Production Configuration
File: `src/core/configurations/production.yaml`

**Important**: Update these values before deployment:
- Database credentials
- JWT secret (generate a strong one)
- SMTP settings
- File paths

### 2. Directory Structure
```
/home/muryarsunnah/htdocs/www.muryarsunnah.com/
├── web/                    # Yii2 application
│   ├── images/            # Images directory
│   └── uploads/           # Uploads directory
├── rust-service/          # Rust service
│   ├── sunnah_audio_rust_service  # Binary
│   └── configurations/    # Config files
└── backups/               # Backup directory
```

## 🛠️ Deployment Scripts

### 1. `deploy.sh` - Full Deployment
- Installs dependencies
- Builds the application
- Sets up systemd service
- Configures nginx
- Starts all services

### 2. `update.sh` - Update Existing Deployment
- Creates backup
- Updates code
- Rebuilds application
- Restarts service

Usage:
```bash
# Full update
sudo ./update.sh

# Create backup only
sudo ./update.sh backup

# Health check
sudo ./update.sh health

# Rollback
sudo ./update.sh rollback
```

## 🐳 Docker Deployment

### Files Created:
- `Dockerfile` - Multi-stage build
- `docker-compose.yml` - Service orchestration
- `nginx.conf` - Nginx configuration

### Docker Commands:
```bash
# Build and start services
docker-compose up -d

# View logs
docker-compose logs -f

# Stop services
docker-compose down

# Update services
docker-compose pull && docker-compose up -d
```

## 🔍 Service Management

### Systemd Commands:
```bash
# Service status
sudo systemctl status sunnah-audio-rust-service

# Start/Stop/Restart
sudo systemctl start sunnah-audio-rust-service
sudo systemctl stop sunnah-audio-rust-service
sudo systemctl restart sunnah-audio-rust-service

# View logs
sudo journalctl -u sunnah-audio-rust-service -f

# Enable auto-start
sudo systemctl enable sunnah-audio-rust-service
```

### Nginx Commands:
```bash
# Test configuration
sudo nginx -t

# Reload configuration
sudo systemctl reload nginx

# Restart nginx
sudo systemctl restart nginx
```

## 🔒 Security Configuration

### 1. Firewall Setup:
```bash
sudo ufw allow 22    # SSH
sudo ufw allow 80   # HTTP
sudo ufw allow 443  # HTTPS
sudo ufw enable
```

### 2. SSL Certificate:
```bash
# Using Let's Encrypt
sudo apt install certbot python3-certbot-nginx
sudo certbot --nginx -d www.muryarsunnah.com -d muryarsunnah.com
```

### 3. Database Security:
- Use strong passwords
- Limit database access
- Enable SSL connections
- Regular backups

## 📊 Monitoring

### Health Check:
```bash
# Check service health
curl http://localhost:8990/api/v1/health

# Check through nginx
curl https://www.muryarsunnah.com/api/v1/health
```

### Log Monitoring:
```bash
# Service logs
sudo journalctl -u sunnah-audio-rust-service -f

# Nginx logs
sudo tail -f /var/log/nginx/access.log
sudo tail -f /var/log/nginx/error.log
```

### Resource Monitoring:
```bash
# System resources
htop

# Service status
sudo systemctl status sunnah-audio-rust-service

# Port usage
sudo netstat -tlnp | grep 8990
```

## 🔄 Backup Strategy

### Automated Backups:
```bash
# Setup cron job for daily backups
sudo crontab -e

# Add this line for daily backups at 2 AM
0 2 * * * /usr/local/bin/backup-sunnah-service.sh
```

### Manual Backup:
```bash
# Create backup
sudo /usr/local/bin/backup-sunnah-service.sh

# List backups
ls -la /home/muryarsunnah/htdocs/www.muryarsunnah.com/backups/
```

## 🚨 Troubleshooting

### Common Issues:

1. **Service won't start**
   ```bash
   sudo journalctl -u sunnah-audio-rust-service -n 50
   ```

2. **Database connection failed**
   - Check database credentials
   - Verify database is running
   - Check network connectivity

3. **Port conflicts**
   ```bash
   sudo netstat -tlnp | grep 8990
   ```

4. **Permission issues**
   ```bash
   sudo chown -R muryarsunnah:muryarsunnah /home/muryarsunnah/htdocs/www.muryarsunnah.com/
   ```

5. **Nginx configuration errors**
   ```bash
   sudo nginx -t
   sudo systemctl status nginx
   ```

### Performance Issues:

1. **High memory usage**
   - Check for memory leaks
   - Monitor with `htop`
   - Restart service if needed

2. **Slow response times**
   - Check database performance
   - Monitor Redis usage
   - Review nginx logs

3. **High CPU usage**
   - Check for infinite loops
   - Monitor with `top`
   - Review application logs

## 📈 Performance Optimization

### 1. Nginx Optimization:
- Enable gzip compression
- Configure caching headers
- Use HTTP/2
- Optimize worker processes

### 2. Database Optimization:
- Create proper indexes
- Monitor slow queries
- Regular maintenance

### 3. Application Optimization:
- Enable connection pooling
- Use Redis for caching
- Optimize queries
- Monitor memory usage

## 🔄 Updates and Maintenance

### Regular Updates:
```bash
# Update system packages
sudo apt update && sudo apt upgrade -y

# Update Rust service
sudo ./update.sh

# Update Docker services
docker-compose pull && docker-compose up -d
```

### Security Updates:
- Regular system updates
- Monitor security advisories
- Update dependencies
- Review access logs

## 📞 Support

### Log Locations:
- Service logs: `journalctl -u sunnah-audio-rust-service`
- Nginx logs: `/var/log/nginx/`
- System logs: `/var/log/syslog`

### Useful Commands:
```bash
# Check all services
sudo systemctl status sunnah-audio-rust-service nginx postgresql redis-server

# View recent errors
sudo journalctl -p err -n 20

# Check disk space
df -h

# Check memory usage
free -h
```

---

## 🎯 Quick Reference

| Task | Command |
|------|---------|
| Deploy | `sudo ./deploy.sh` |
| Update | `sudo ./update.sh` |
| Rollback | `sudo ./update.sh rollback` |
| Health Check | `sudo ./update.sh health` |
| View Logs | `sudo journalctl -u sunnah-audio-rust-service -f` |
| Restart Service | `sudo systemctl restart sunnah-audio-rust-service` |
| Test API | `curl http://localhost:8990/api/v1/health` |

**Remember**: Always test in a staging environment before deploying to production!
