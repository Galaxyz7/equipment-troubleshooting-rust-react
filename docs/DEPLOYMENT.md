# Deployment Guide

This guide explains how to deploy the Equipment Troubleshooting System with **zero hardcoded URLs** - everything is configured from a single `.env` file!

## 🎯 Key Feature: Single Configuration File

**You only need to configure ONE variable:** `FRONTEND_URL` in the root `.env` file.

The frontend automatically detects the API URL from the page it's served from, so there's no need to configure URLs separately for frontend and backend!

## 📋 Quick Deployment Steps

### 1. Build the Application

```bash
# Build frontend (URLs are NOT baked in - detected at runtime!)
cd apps/web
npm run build

# Build backend
cd ../api
cargo build --release
```

### 2. Configure ONLY the Root `.env`

Copy `.env.example` to `.env` and update **just** the `FRONTEND_URL`:

```bash
# For local network
FRONTEND_URL=http://192.168.1.100:5000

# For Tailscale
FRONTEND_URL=http://hostname.crocodile-arctic.ts.net:5000

# For HTTPS (requires SSL certificates)
FRONTEND_URL=https://yourdomain.com:5000
```

That's it! No other URL configuration needed.

### 3. Deploy Files

Copy these to your server:
```
your-server/
├── equipment-troubleshooting  (binary from target/release/)
├── ui/                        (copy from apps/web/dist/)
│   ├── index.html
│   ├── assets/
│   └── ...
└── .env                       (your configuration)
```

### 4. Run

```bash
./equipment-troubleshooting
```

## 🔧 Configuration Examples

### Example 1: Local Development

```bash
# .env
DATABASE_URL=postgresql://postgres:password@localhost:5432/equipment_troubleshooting
JWT_SECRET=your-secret-key-at-least-32-chars-long
ADMIN_USERNAME=admin@example.com
ADMIN_PASSWORD_HASH=$argon2id$v=19$m=19456,t=2,p=1$...
HOST=0.0.0.0
PORT=5000
FRONTEND_URL=http://localhost:5000
```

### Example 2: Production Server (HTTP)

```bash
# .env
DATABASE_URL=postgresql://postgres:password@db.example.com:5432/equipment_troubleshooting
JWT_SECRET=generate-with-openssl-rand-base64-32
ADMIN_USERNAME=admin@company.com
ADMIN_PASSWORD_HASH=$argon2id$v=19$m=19456,t=2,p=1$...
HOST=0.0.0.0
PORT=5000
FRONTEND_URL=http://192.168.1.100:5000
```

### Example 3: Production Server (HTTPS)

```bash
# .env
DATABASE_URL=postgresql://postgres:password@db.example.com:5432/equipment_troubleshooting
JWT_SECRET=generate-with-openssl-rand-base64-32
ADMIN_USERNAME=admin@company.com
ADMIN_PASSWORD_HASH=$argon2id$v=19$m=19456,t=2,p=1$...
HOST=0.0.0.0
PORT=5000
FRONTEND_URL=https://troubleshoot.company.com:5000
```

**Plus add SSL certificates (same directory as binary):**
```
your-server/
├── equipment-troubleshooting
├── ui/
├── .env
├── my-cert.crt         ← Any .crt file
└── my-cert.key         ← Any .key file
```

**Note:** The server automatically finds the first `.crt` and `.key` files in the directory - **no need to rename them!**

See [SSL_SETUP.md](SSL_SETUP.md) for certificate generation.

### Example 4: Tailscale Network

```bash
# .env
DATABASE_URL=postgresql://postgres:password@db.example.com:5432/equipment_troubleshooting
JWT_SECRET=generate-with-openssl-rand-base64-32
ADMIN_USERNAME=admin@company.com
ADMIN_PASSWORD_HASH=$argon2id$v=19$m=19456,t=2,p=1$...
HOST=0.0.0.0
PORT=5000
FRONTEND_URL=http://x-desktop.crocodile-arctic.ts.net:5000
```

## 🚀 Systemd Service (Linux)

Create `/etc/systemd/system/equipment-troubleshooting.service`:

```ini
[Unit]
Description=Equipment Troubleshooting System
After=network.target postgresql.service

[Service]
Type=simple
User=your-user
WorkingDirectory=/home/your-user/equipment-troubleshooting
ExecStart=/home/your-user/equipment-troubleshooting/equipment-troubleshooting
Restart=on-failure
RestartSec=5s

# Environment file
EnvironmentFile=/home/your-user/equipment-troubleshooting/.env

[Install]
WantedBy=multi-user.target
```

Enable and start:
```bash
sudo systemctl daemon-reload
sudo systemctl enable equipment-troubleshooting
sudo systemctl start equipment-troubleshooting
sudo systemctl status equipment-troubleshooting
```

## 🐳 Docker Deployment

Create `Dockerfile`:

```dockerfile
FROM rust:1.75-slim as builder

# Install build dependencies
RUN apt-get update && apt-get install -y pkg-config libssl-dev

# Build backend
WORKDIR /build
COPY apps/api ./apps/api
COPY Cargo.* ./
RUN cargo build --release --bin equipment-troubleshooting

# Runtime stage
FROM debian:bookworm-slim

# Install runtime dependencies
RUN apt-get update && apt-get install -y ca-certificates libssl3 && rm -rf /var/lib/apt/lists/*

# Copy binary and frontend
COPY --from=builder /build/target/release/equipment-troubleshooting /app/
COPY apps/web/dist /app/ui

WORKDIR /app

# Expose port
EXPOSE 5000

CMD ["./equipment-troubleshooting"]
```

Run with:
```bash
docker build -t equipment-troubleshooting .
docker run -d \
  --name equipment-troubleshooting \
  -p 5000:5000 \
  --env-file .env \
  equipment-troubleshooting
```

## 📦 Environment Variables Reference

### Required Variables

| Variable | Description | Example |
|----------|-------------|---------|
| `DATABASE_URL` | PostgreSQL connection string | `postgresql://user:pass@host:5432/db` |
| `JWT_SECRET` | Secret for JWT tokens (min 32 chars) | Generate with `openssl rand -base64 32` |
| `ADMIN_USERNAME` | Admin email | `admin@example.com` |
| `ADMIN_PASSWORD_HASH` | Argon2 password hash | Generate with `cargo run --bin hash_password` |
| `FRONTEND_URL` | **THE ONLY URL TO CONFIGURE!** | `http://your-domain.com:5000` |

### Optional Variables

| Variable | Default | Description |
|----------|---------|-------------|
| `HOST` | `0.0.0.0` | IP to bind to |
| `PORT` | `5000` | Port to listen on |
| `ENVIRONMENT` | `development` | `development` or `production` |
| `JWT_EXPIRATION_HOURS` | `24` | JWT token lifetime |
| `RUST_LOG` | `info` | Logging level |

## ✅ How It Works

### Single Source of Truth

1. You configure `FRONTEND_URL` in root `.env`
2. Backend reads it for CORS and HTTPS detection
3. Frontend auto-detects API URL from `window.location`
4. **Result:** No hardcoded URLs, fully portable builds!

### URL Auto-Detection Flow

```
User visits: http://192.168.1.100:5000
             ↓
Frontend loads from server
             ↓
JavaScript detects: window.location = "http://192.168.1.100:5000"
             ↓
API calls go to: http://192.168.1.100:5000/api/*
             ✓ No configuration needed!
```

### HTTPS Auto-Enable

```
FRONTEND_URL=https://domain.com:5000
             ↓
Server checks for server.crt + server.key
             ↓
    Found?  → Start HTTPS ✓
    Missing → Error with helpful message ✗
```

## 🔍 Troubleshooting

### Frontend can't connect to API

**Check:** Are you accessing the app via the same URL as `FRONTEND_URL` in `.env`?

❌ **Wrong:**
```bash
FRONTEND_URL=http://localhost:5000
# But accessing via: http://192.168.1.100:5000
```

✅ **Correct:**
```bash
FRONTEND_URL=http://192.168.1.100:5000
# Accessing via: http://192.168.1.100:5000
```

### HTTPS not working

1. Check `.env` has `FRONTEND_URL=https://...`
2. Ensure `server.crt` and `server.key` exist in project root
3. Check permissions: `chmod 644 server.crt && chmod 600 server.key`

### Database connection failed

Update `DATABASE_URL` in `.env` with correct credentials.

## 📝 Migration Guide

If you're updating from an old version with multiple URL configs:

**Old way (❌ deprecated):**
- `.env` had `FRONTEND_URL`
- `apps/web/.env` had `VITE_API_URL`
- Both needed to match

**New way (✅ automatic):**
- Only configure `FRONTEND_URL` in root `.env`
- Delete or comment out `VITE_API_URL` in `apps/web/.env`
- Rebuild frontend - it auto-detects!

## 🎉 Benefits

- ✅ **Single configuration point** - just `FRONTEND_URL`
- ✅ **Portable builds** - no hardcoded URLs
- ✅ **Zero config for end users** - just edit `.env`
- ✅ **Works anywhere** - localhost, LAN, domain, Tailscale
- ✅ **HTTPS auto-detection** - just change `http://` to `https://`

## 📖 Additional Documentation

- [SSL_SETUP.md](SSL_SETUP.md) - HTTPS/SSL certificate setup
- [README.md](README.md) - Full project documentation
- [.env.example](.env.example) - Configuration template
