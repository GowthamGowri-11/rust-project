# PowerShell script for Docker-based cargo check (fast syntax validation)

Write-Host "🐳 Checking code in Docker..." -ForegroundColor Cyan

docker run --rm `
  -v "${PWD}:/workspace" `
  -w /workspace `
  rust:1.75 `
  cargo check --all

if ($LASTEXITCODE -eq 0) {
    Write-Host "`n✅ Code check passed!" -ForegroundColor Green
} else {
    Write-Host "`n❌ Code check failed!" -ForegroundColor Red
    exit 1
}
