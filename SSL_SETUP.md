# SSL/HTTPS Setup Guide

The application automatically enables HTTPS based on your `.env` configuration.

## 🎯 Quick Setup (2 Steps)

1. **Update `.env` to use HTTPS:**
   ```bash
   # Change FRONTEND_URL from http:// to https://
   FRONTEND_URL=https://x-desktop.crocodile-arctic.ts.net:5000
   ```

2. **Add SSL certificate files to project root:**
   - `server.crt` - SSL certificate file
   - `server.key` - Private key file

That's it! The server automatically detects the `https://` in `FRONTEND_URL` and enables HTTPS mode.

## 📋 How It Works

- Server checks `FRONTEND_URL` in `.env`
- If it starts with `https://` → Enables HTTPS and requires certificates
- If it starts with `http://` → Uses HTTP mode (no certificates needed)
- Missing certificates with `https://` → Server won't start (with helpful error)

## 🔧 Option 1: Self-Signed Certificates (Development/Testing)

### Using OpenSSL

Generate a self-signed certificate for local testing:

```bash
# Navigate to project root
cd /path/to/equipment-troubleshooting-rust-react

# Generate private key and certificate (valid for 365 days)
openssl req -x509 -newkey rsa:4096 -keyout server.key -out server.crt -days 365 -nodes \
  -subj "/CN=x-desktop.crocodile-arctic.ts.net" \
  -addext "subjectAltName=DNS:x-desktop.crocodile-arctic.ts.net,DNS:localhost,IP:127.0.0.1"
```

**Note:** Self-signed certificates will show browser warnings. You'll need to accept the security warning to proceed.

### Using mkcert (Recommended for Development)

[mkcert](https://github.com/FiloSottile/mkcert) creates locally-trusted certificates without warnings:

```bash
# Install mkcert (macOS)
brew install mkcert
brew install nss # for Firefox

# Or install mkcert (Linux)
# See: https://github.com/FiloSottile/mkcert#installation

# Install local CA
mkcert -install

# Generate certificate
cd /path/to/equipment-troubleshooting-rust-react
mkcert -key-file server.key -cert-file server.crt \
  x-desktop.crocodile-arctic.ts.net localhost 127.0.0.1 ::1
```

## 🌐 Option 2: Let's Encrypt (Production)

For production deployment with a real domain:

### Using Certbot

```bash
# Install certbot
sudo apt-get update
sudo apt-get install certbot

# Get certificate (requires domain to point to your server)
sudo certbot certonly --standalone -d yourdomain.com

# Copy certificates to project root
sudo cp /etc/letsencrypt/live/yourdomain.com/fullchain.pem ./server.crt
sudo cp /etc/letsencrypt/live/yourdomain.com/privkey.pem ./server.key

# Set proper permissions
sudo chown $USER:$USER server.crt server.key
chmod 644 server.crt
chmod 600 server.key
```

### Automatic Renewal

Let's Encrypt certificates expire every 90 days. Set up auto-renewal:

```bash
# Test renewal
sudo certbot renew --dry-run

# Create renewal script
cat > renew-certs.sh << 'EOF'
#!/bin/bash
sudo certbot renew --quiet
sudo cp /etc/letsencrypt/live/yourdomain.com/fullchain.pem /path/to/project/server.crt
sudo cp /etc/letsencrypt/live/yourdomain.com/privkey.pem /path/to/project/server.key
sudo chown $USER:$USER /path/to/project/server.crt /path/to/project/server.key
# Restart your service here
EOF

chmod +x renew-certs.sh

# Add to crontab (runs daily at 2am)
(crontab -l 2>/dev/null; echo "0 2 * * * /path/to/renew-certs.sh") | crontab -
```

## 🔒 Option 3: Tailscale HTTPS (Easiest for Tailnet)

If you're using Tailscale, you can get automatic HTTPS certificates:

```bash
# Enable HTTPS on your Tailscale machine
tailscale cert x-desktop.crocodile-arctic.ts.net

# This creates:
# - /var/lib/tailscale/certs/x-desktop.crocodile-arctic.ts.net.crt
# - /var/lib/tailscale/certs/x-desktop.crocodile-arctic.ts.net.key

# Copy to project root
sudo cp /var/lib/tailscale/certs/x-desktop.crocodile-arctic.ts.net.crt ./server.crt
sudo cp /var/lib/tailscale/certs/x-desktop.crocodile-arctic.ts.net.key ./server.key
sudo chown $USER:$USER server.crt server.key
chmod 644 server.crt
chmod 600 server.key
```

**Benefits:**
- ✅ Automatically trusted by all devices on your Tailnet
- ✅ No browser warnings
- ✅ Certificates auto-renew
- ✅ Works across your entire Tailscale network

## 🚀 Starting the Server

Once certificates are in place:

```bash
# Build and run
cd apps/api
cargo build --release

# Server automatically detects certificates
cargo run --release --bin equipment-troubleshooting
```

You'll see:
```
🔒 SSL certificates detected - starting HTTPS server
📡 Server listening on https://0.0.0.0:5000
🌐 Frontend & API available at https://0.0.0.0:5000
```

## 🔍 Troubleshooting

### Certificates not detected

Make sure files are in the **project root**, not the api directory:
```
equipment-troubleshooting-rust-react/
├── server.crt          ← Here
├── server.key          ← Here
├── apps/
│   ├── api/
│   └── web/
└── .env
```

### Permission denied errors

```bash
chmod 644 server.crt
chmod 600 server.key
```

### Browser security warnings

- Self-signed certs will always show warnings
- Use mkcert for local development
- Use Let's Encrypt or Tailscale for production

## 🔄 Enabling HTTPS (Complete Steps)

1. **Generate certificates** (choose one option above)

2. **Update root `.env`:**
   ```bash
   FRONTEND_URL=https://x-desktop.crocodile-arctic.ts.net:5000
   ```

3. **Update frontend `.env`:**
   ```bash
   # apps/web/.env
   VITE_API_URL=https://x-desktop.crocodile-arctic.ts.net:5000
   ```

4. **Rebuild frontend:**
   ```bash
   cd apps/web
   npm run build
   ```

5. **Restart server** - it will automatically detect HTTPS from `.env`

## 📝 Notes

- **No code changes needed** - just update `.env` and add certificates
- Server detects HTTPS mode from `FRONTEND_URL` environment variable
- Missing certificates with `https://` URL will prevent server startup (with clear error)
- For development, HTTP is fine; for production, always use HTTPS
- Both `.env` files should match (root and `apps/web/.env`)
