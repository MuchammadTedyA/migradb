# Build script for Windows

Write-Host "=== Building MigrDB ===" -ForegroundColor Cyan

# Build core Rust library
Write-Host "Building Rust core library..." -ForegroundColor Yellow
cargo build --release
if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }

# Build Go bindings
Write-Host "Building Go bindings..." -ForegroundColor Yellow
Set-Location go
go build ./...
Set-Location ..
if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }

# Build Node.js bindings
Write-Host "Building Node.js bindings..." -ForegroundColor Yellow
Set-Location npm
npm install
npm run build
Set-Location ..
if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }

# Build Python bindings
Write-Host "Building Python bindings..." -ForegroundColor Yellow
Set-Location python
pip install maturin
maturin develop
Set-Location ..
if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }

Write-Host "=== Build Complete ===" -ForegroundColor Green
Write-Host ""
Write-Host "Run examples:" -ForegroundColor Cyan
Write-Host "  Go:     go run go/example/main.go" -ForegroundColor White
Write-Host "  Node:   node npm/example.js" -ForegroundColor White
Write-Host "  Python: python python/example.py" -ForegroundColor White
