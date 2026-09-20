# scripts/bump_version.ps1
# Automates updating the version in all Cargo.toml files and pushing a Git tag to trigger the CI release.

param (
    [Parameter(Mandatory=$true)]
    [string]$NewVersion
)

$ErrorActionPreference = "Stop"

# Navigate to the workspace root
$ScriptDir = Split-Path -Parent $MyInvocation.MyCommand.Path
$WorkspaceRoot = (Resolve-Path "$ScriptDir\..").Path
Set-Location $WorkspaceRoot

Write-Host "Bumping version to $NewVersion..." -ForegroundColor Cyan

# 1. Ensure working directory is clean
$GitStatus = git status --porcelain
if ($GitStatus) {
    Write-Host "[ERROR] Git working directory is not clean. Commit or stash your changes first." -ForegroundColor Red
    exit 1
}

# 2. Update version in all Cargo.toml files
$TomlFiles = Get-ChildItem -Recurse -Filter "Cargo.toml" -File

foreach ($TomlFile in $TomlFiles) {
    $Content = Get-Content $TomlFile.FullName
    $UpdatedContent = $Content -replace '^version\s*=\s*".*"', "version = `"$NewVersion`""
    Set-Content -Path $TomlFile.FullName -Value $UpdatedContent
}

Write-Host "Updated $($TomlFiles.Count) Cargo.toml files." -ForegroundColor Green

# 3. Update Cargo.lock
cargo check

# 4. Commit and Tag
git add .
git commit -m "Bump version to $NewVersion"
git tag "v$NewVersion"

Write-Host "`nVersion bumped successfully!" -ForegroundColor Green
Write-Host "To release, push the commit and tag to GitHub:" -ForegroundColor Yellow
Write-Host "  git push origin main" -ForegroundColor Yellow
Write-Host "  git push origin v$NewVersion" -ForegroundColor Yellow
Write-Host "`nThe GitHub Action will automatically build and distribute the release once the tag is pushed." -ForegroundColor Cyan
