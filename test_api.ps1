# RustFlow-AI API Test Script

Write-Host "Testing RustFlow-AI API Endpoints..." -ForegroundColor Cyan
Write-Host "Make sure the server is running first!" -ForegroundColor Yellow
Write-Host ""

$baseUrl = "http://localhost:8080"

$endpoints = @(
    @{Name="Root"; Url="/"},
    @{Name="Health Check"; Url="/api/v1/health"},
    @{Name="Topology"; Url="/api/v1/topology"},
    @{Name="Switches"; Url="/api/v1/switches"},
    @{Name="Flows"; Url="/api/v1/flows"},
    @{Name="Metrics"; Url="/api/v1/metrics"},
    @{Name="Prometheus Metrics"; Url="/metrics"}
)

foreach ($endpoint in $endpoints) {
    $url = $baseUrl + $endpoint.Url
    Write-Host "Testing: $($endpoint.Name)" -ForegroundColor Cyan
    Write-Host "URL: $url" -ForegroundColor Gray
    
    try {
        $response = Invoke-WebRequest -Uri $url -UseBasicParsing
        Write-Host "✓ Status: $($response.StatusCode) OK" -ForegroundColor Green
        
        if ($response.Content.Length -lt 200) {
            Write-Host "Response: $($response.Content)" -ForegroundColor White
        } else {
            Write-Host "Response: [Large response - $($response.Content.Length) bytes]" -ForegroundColor White
        }
    } catch {
        Write-Host "✗ Error: $($_.Exception.Message)" -ForegroundColor Red
    }
    
    Write-Host ""
}

Write-Host "Testing complete!" -ForegroundColor Green
