# scripts/build_release.ps1
# Automates testing, building, and packaging for Windows, Linux, and macOS.

$ErrorActionPreference = "Stop"

# Navigate to the workspace root (one directory up from the script location)
$ScriptDir = Split-Path -Parent $MyInvocation.MyCommand.Path
$WorkspaceRoot = (Resolve-Path "$ScriptDir\..").Path
Set-Location $WorkspaceRoot

Write-Host "Viyal Release Builder" -ForegroundColor Cyan
Write-Host "---------------------" -ForegroundColor Cyan

# 1. Parse the version from tools\cli\Cargo.toml
$CliCargoToml = Get-Content "$WorkspaceRoot\tools\cli\Cargo.toml"
$VersionLine = $CliCargoToml | Where-Object { $_ -match '^version\s*=\s*"(.*)"' } | Select-Object -First 1
if (-not $VersionLine) {
    Write-Host "[ERROR] Could not find version in tools\cli\Cargo.toml" -ForegroundColor Red
    exit 1
}
$Version = $matches[1]
Write-Host "Preparing release for version: v$Version" -ForegroundColor Green

# 2. Check for dependencies
Write-Host "`n[1/5] Checking dependencies..." -ForegroundColor Yellow
$ZigInstalled = Get-Command zig -ErrorAction SilentlyContinue
if (-not $ZigInstalled) {
    Write-Host "[ERROR] zig is not installed." -ForegroundColor Red
    Write-Host "Please install it using: winget install zig.zig" -ForegroundColor Yellow
    exit 1
}

$CargoZigbuildInstalled = Get-Command cargo-zigbuild -ErrorAction SilentlyContinue
if (-not $CargoZigbuildInstalled) {
    Write-Host "[ERROR] cargo-zigbuild is not installed." -ForegroundColor Red
    Write-Host "Please install it using: cargo install cargo-zigbuild" -ForegroundColor Yellow
    exit 1
}

# 3. Run tests
Write-Host "`n[2/5] Running workspace tests..." -ForegroundColor Yellow
cargo test --workspace
if ($LASTEXITCODE -ne 0) {
    Write-Host "[ERROR] Tests failed! Aborting release." -ForegroundColor Red
    exit 1
}
Write-Host "All tests passed!" -ForegroundColor Green

# 4. Define targets
$Targets = @(
    @{ Name = "windows"; Triple = "x86_64-pc-windows-msvc"; IsUnix = $false; Executable = "viyal.exe" },
    @{ Name = "linux"; Triple = "x86_64-unknown-linux-gnu"; IsUnix = $true; Executable = "viyal" },
    @{ Name = "macos-intel"; Triple = "x86_64-apple-darwin"; IsUnix = $true; Executable = "viyal" },
    @{ Name = "macos-silicon"; Triple = "aarch64-apple-darwin"; IsUnix = $true; Executable = "viyal" }
)

# Create release output directory
$ReleaseOut = "$WorkspaceRoot\target\release-assets"
if (Test-Path $ReleaseOut) {
    Remove-Item -Recurse -Force $ReleaseOut
}
New-Item -ItemType Directory -Path $ReleaseOut | Out-Null

# 5. Build and Package
Write-Host "`n[3/5] Compiling and Packaging..." -ForegroundColor Yellow

foreach ($target in $Targets) {
    Write-Host "`nBuilding for $($target.Name) ($($target.Triple))..." -ForegroundColor Cyan
    
    # We must explicitly add the target to rustup first
    rustup target add $target.Triple | Out-Null

    if ($target.Name -eq "windows") {
        # Use standard cargo for the native host platform
        cargo build --release --bin viyal --target $target.Triple
    } else {
        # Use cargo-zigbuild for cross-compilation
        cargo zigbuild --release --bin viyal --target $target.Triple
    }

    if ($LASTEXITCODE -ne 0) {
        Write-Host "[ERROR] Build failed for $($target.Name)." -ForegroundColor Red
        exit 1
    }

    # Assemble the package directory
    $PackageName = "viyal-v$Version-$($target.Name)"
    $PackageDir = "$ReleaseOut\$PackageName"
    New-Item -ItemType Directory -Path $PackageDir | Out-Null

    # Copy executable
    $BinPath = "$WorkspaceRoot\target\$($target.Triple)\release\$($target.Executable)"
    Copy-Item $BinPath -Destination "$PackageDir\$($target.Executable)"

    # Copy stdlib
    Copy-Item -Recurse "$WorkspaceRoot\stdlib" -Destination "$PackageDir\stdlib"

    # Copy README
    if (Test-Path "$WorkspaceRoot\README.md") {
        Copy-Item "$WorkspaceRoot\README.md" -Destination "$PackageDir\README.md"
    }

    # Archive
    Write-Host "Packaging $PackageName..." -ForegroundColor Cyan
    Set-Location $ReleaseOut
    
    if ($target.IsUnix) {
        # Use built-in tar on modern Windows 10/11
        tar -czf "${PackageName}.tar.gz" $PackageName
        Write-Host "Created ${PackageName}.tar.gz" -ForegroundColor Green
    } else {
        # Use Compress-Archive for Windows zip
        Compress-Archive -Path $PackageDir -DestinationPath "${PackageName}.zip" -Force
        Write-Host "Created ${PackageName}.zip" -ForegroundColor Green
    }
    
    # Clean up the unarchived folder
    Remove-Item -Recurse -Force $PackageDir
    Set-Location $WorkspaceRoot
}

Write-Host "`n[5/5] Release process complete!" -ForegroundColor Green
Write-Host "Assets are available in: $ReleaseOut" -ForegroundColor Cyan
