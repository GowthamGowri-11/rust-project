# PowerShell script for Docker-based testing

param(
    [string]$Package = "",
    [string]$Test = ""
)

Write-Host "🐳 Running tests in Docker..." -ForegroundColor Cyan

$testCmd = "cargo test"

if ($Package -ne "") {
    $testCmd += " --package $Package"
    Write-Host "📦 Testing package: $Package" -ForegroundColor Yellow
}

if ($Test -ne "") {
    $testCmd += " $Test"
    Write-Host "🎯 Running test: $Test" -ForegroundColor Yellow
}

docker run --rm `
  -v "${PWD}:/workspace" `
  -w /workspace `
  rust:1.75 `
  bash -c "$testCmd"

if ($LASTEXITCODE -eq 0) {
    Write-Host "`n✅ Tests passed!" -ForegroundColor Green
} else {
    Write-Host "`n❌ Tests failed!" -ForegroundColor Red
    exit 1
}
