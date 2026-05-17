# PowerShell script for Docker-based Rust builds on Windows

Write-Host "🐳 Building RustFlow-AI in Docker..." -ForegroundColor Cyan

# Build the project
Write-Host "`n📦 Building project..." -ForegroundColor Yellow
docker run --rm `
  -v "${PWD}:/workspace" `
  -w /workspace `
  rust:1.75 `
  cargo build --release

if ($LASTEXITCODE -eq 0) {
    Write-Host "✅ Build successful!" -ForegroundColor Green
} else {
    Write-Host "❌ Build failed!" -ForegroundColor Red
    exit 1
}

# Run tests
Write-Host "`n🧪 Running tests..." -ForegroundColor Yellow
docker run --rm `
  -v "${PWD}:/workspace" `
  -w /workspace `
  rust:1.75 `
  cargo test --all

if ($LASTEXITCODE -eq 0) {
    Write-Host "✅ All tests passed!" -ForegroundColor Green
} else {
    Write-Host "⚠️ Some tests failed!" -ForegroundColor Yellow
}

Write-Host "`n✨ Docker build complete!" -ForegroundColor Cyan
Write-Host "Binaries available in: target/release/" -ForegroundColor Gray
