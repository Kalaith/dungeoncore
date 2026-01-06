# Standard Game Publishing Script
# Builds WebGL package and deploys to preview/production servers

param(
    [switch]$SkipBuild = $false,
    [switch]$WindowsOnly = $false,
    [switch]$WebGLOnly = $false,
    [switch]$DeployOnly = $false,
    [Alias('p')] [switch]$Production = $false,
    [switch]$DryRun = $false
)

$ErrorActionPreference = "Stop"
$ProjectRoot = $PSScriptRoot
$DistDir = Join-Path $ProjectRoot "dist"
$CargoToml = Join-Path $ProjectRoot "Cargo.toml"

# Deployment paths
$PreviewRoot = "H:\xampp\htdocs"
$ProductionRoot = "F:\WebHatchery"

# Parse project name from Cargo.toml
if (-not (Test-Path $CargoToml)) {
    Write-Error "Cargo.toml not found at: $CargoToml"
    exit 1
}

$CargoContent = Get-Content $CargoToml -Raw
if ($CargoContent -match 'name\s*=\s*"([^"]+)"') {
    $ProjectName = $matches[1]
} else {
    Write-Error "Could not parse project name from Cargo.toml"
    exit 1
}

$ProjectTitle = ($ProjectName -replace '_', ' ').ToUpper()

Write-Host "=== $ProjectTitle Publisher ===" -ForegroundColor Cyan
Write-Host "Project: $ProjectName"
Write-Host ""

# Determine deployment target
$DeployRoot = $PreviewRoot
$Environment = "Preview"
if ($Production) {
    $DeployRoot = $ProductionRoot
    $Environment = "Production"
}
$DeployDir = Join-Path $DeployRoot "games\$ProjectName"

Write-Host "Target: $Environment ($DeployDir)" -ForegroundColor Magenta
Write-Host ""

if ($DeployOnly) {
    Write-Host "Deploy-only mode: Skipping build, deploying existing files..." -ForegroundColor Yellow
    $SkipBuild = $true
}

$buildWindows = -not $WebGLOnly -and -not $DeployOnly
$buildWebGL = -not $WindowsOnly
$totalSteps = 3
if ($buildWindows) { $totalSteps += 2 }
if ($buildWebGL) { $totalSteps += 2 }
$currentStep = 0

if (-not $DeployOnly) {
    $currentStep++
    Write-Host "[$currentStep/$totalSteps] Preparing dist folder..." -ForegroundColor Yellow
    if (Test-Path $DistDir) { Remove-Item $DistDir -Recurse -Force }
    New-Item -ItemType Directory -Path $DistDir -Force | Out-Null
}

if ($buildWindows) {
    $currentStep++
    if (-not $SkipBuild) {
        Write-Host "[$currentStep/$totalSteps] Building Windows release..." -ForegroundColor Yellow
        cargo build --release
        if ($LASTEXITCODE -ne 0) { Write-Error "Windows build failed!"; exit 1 }
        Write-Host "Windows build complete!" -ForegroundColor Green
    } else {
        Write-Host "[$currentStep/$totalSteps] Skipping Windows build" -ForegroundColor Gray
    }

    $currentStep++
    Write-Host "[$currentStep/$totalSteps] Packaging Windows build..." -ForegroundColor Yellow
    $WindowsPackageDir = Join-Path $DistDir "windows"
    New-Item -ItemType Directory -Path $WindowsPackageDir -Force | Out-Null
    $ExePath = Join-Path $ProjectRoot "target\release\$ProjectName.exe"
    if (-not (Test-Path $ExePath)) { Write-Error "Executable not found: $ExePath"; exit 1 }
    Copy-Item $ExePath $WindowsPackageDir
    $AssetsPath = Join-Path $ProjectRoot "assets"
    if (Test-Path $AssetsPath) { Copy-Item $AssetsPath -Destination $WindowsPackageDir -Recurse }
    $WindowsZipPath = Join-Path $DistDir "${ProjectName}_windows.zip"
    Compress-Archive -Path "$WindowsPackageDir\*" -DestinationPath $WindowsZipPath -CompressionLevel Optimal
    Write-Host "Windows package created!" -ForegroundColor Green
}

if ($buildWebGL) {
    $currentStep++
    if (-not $SkipBuild) {
        Write-Host "[$currentStep/$totalSteps] Building WebGL release..." -ForegroundColor Yellow
        $targets = rustup target list --installed
        if ($targets -notcontains "wasm32-unknown-unknown") { rustup target add wasm32-unknown-unknown }
        cargo build --release --target wasm32-unknown-unknown
        if ($LASTEXITCODE -ne 0) { Write-Error "WebGL build failed!"; exit 1 }
        Write-Host "WebGL build complete!" -ForegroundColor Green
    } else {
        Write-Host "[$currentStep/$totalSteps] Skipping WebGL build" -ForegroundColor Gray
    }

    $currentStep++
    Write-Host "[$currentStep/$totalSteps] Packaging WebGL build..." -ForegroundColor Yellow
    $WebGLPackageDir = Join-Path $DistDir "webgl"
    New-Item -ItemType Directory -Path $WebGLPackageDir -Force | Out-Null
    $WasmPath = Join-Path $ProjectRoot "target\wasm32-unknown-unknown\release\$ProjectName.wasm"
    if (-not (Test-Path $WasmPath)) { Write-Error "WASM not found: $WasmPath"; exit 1 }
    Copy-Item $WasmPath $WebGLPackageDir
    $AssetsPath = Join-Path $ProjectRoot "assets"
    if (Test-Path $AssetsPath) { Copy-Item $AssetsPath -Destination $WebGLPackageDir -Recurse }
    $JsBundlePath = Join-Path $WebGLPackageDir "mq_js_bundle.js"
    try { Invoke-WebRequest -Uri "https://not-fl3.github.io/miniquad-samples/mq_js_bundle.js" -OutFile $JsBundlePath } catch { Write-Warning "Could not download mq_js_bundle.js" }
    $WebGLZipPath = Join-Path $DistDir "${ProjectName}_webgl.zip"
    Compress-Archive -Path "$WebGLPackageDir\*" -DestinationPath $WebGLZipPath -CompressionLevel Optimal
    Write-Host "WebGL package created!" -ForegroundColor Green
}

$currentStep++
Write-Host ""
Write-Host "[$currentStep/$totalSteps] Deploying to $Environment..." -ForegroundColor Yellow

if ($DryRun) {
    Write-Host "[DRY-RUN] Would deploy to: $DeployDir" -ForegroundColor DarkYellow
} else {
    if (-not (Test-Path $DeployDir)) { New-Item -ItemType Directory -Path $DeployDir -Force | Out-Null }
    $IndexPath = Join-Path $ProjectRoot "index.html"
    if (Test-Path $IndexPath) { Copy-Item $IndexPath $DeployDir -Force; Write-Host "  Copied: index.html" -ForegroundColor Gray }
    $WebGLSourceDir = Join-Path $DistDir "webgl"
    if (Test-Path $WebGLSourceDir) {
        $wasmFile = Join-Path $WebGLSourceDir "$ProjectName.wasm"
        if (Test-Path $wasmFile) { Copy-Item $wasmFile $DeployDir -Force; Write-Host "  Copied: $ProjectName.wasm" -ForegroundColor Gray }
        $jsBundle = Join-Path $WebGLSourceDir "mq_js_bundle.js"
        if (Test-Path $jsBundle) { Copy-Item $jsBundle $DeployDir -Force; Write-Host "  Copied: mq_js_bundle.js" -ForegroundColor Gray }
        $assetsDir = Join-Path $WebGLSourceDir "assets"
        if (Test-Path $assetsDir) {
            $destAssets = Join-Path $DeployDir "assets"
            if (Test-Path $destAssets) { Remove-Item $destAssets -Recurse -Force }
            Copy-Item $assetsDir -Destination $DeployDir -Recurse
            Write-Host "  Copied: assets/" -ForegroundColor Gray
        }
    }
    Write-Host "Deployed to: $DeployDir" -ForegroundColor Green
}

Write-Host ""
Write-Host "=== Complete ===" -ForegroundColor Cyan
Write-Host "Deploy: $DeployDir" -ForegroundColor Green
Write-Host ""
Write-Host "Options: -SkipBuild, -WebGLOnly, -WindowsOnly, -DeployOnly, -Production (-p), -DryRun" -ForegroundColor Yellow
