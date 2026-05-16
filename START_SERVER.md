# 🚀 How to Start RustFlow-AI Server

## Quick Start (Choose One Method)

### Method 1: Double-Click (Easiest)
1. Find `run.bat` in your project folder
2. Double-click it
3. Server will start in a new window

### Method 2: PowerShell
```bash
cargo run --bin dashboard_api
```

### Method 3: Run the compiled binary
```bash
.\target\release\dashboard_api.exe
```

---

## What You'll See

When the server starts successfully:
```
Starting RustFlow-AI Dashboard API
Initializing Prometheus metrics
Listening on 0.0.0.0:8080
```

**Keep this window open!** The server is running.

---

## Test the Server

### Open your browser:
- http://localhost:8080/api/v1/health

You should see:
```json
{"status":"healthy","version":"0.1.0"}
```

### Or use the test script:
```powershell
.\test_api.ps1
```

---

## Stop the Server

Press `Ctrl + C` in the terminal window

---

## Troubleshooting

**Port 8080 already in use?**
1. Edit `.env` file
2. Change `API_PORT=8080` to `API_PORT=8081`
3. Restart server

**Server won't start?**
- Make sure you're in the project directory
- Try rebuilding: `cargo build --release`
