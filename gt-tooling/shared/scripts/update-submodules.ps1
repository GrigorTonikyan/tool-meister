#!/usr/bin/env pwsh

param(
    [switch]$Init,
    [switch]$Force
)

$ErrorActionPreference = 'Stop'

function Write-Status {
    param([string]$Message)
    Write-Host "==> $Message" -ForegroundColor Cyan
}

try {
    Write-Status "Updating GT Tooling submodules..."

    # Ensure we're in the repository root
    $scriptPath = $PSScriptRoot
    if (-not $scriptPath) {
        $scriptPath = Split-Path -Parent $MyInvocation.MyCommand.Path
    }
    $repoRoot = Split-Path -Parent (Split-Path -Parent $scriptPath)
    
    # Normalize path for the current OS
    if ($IsWindows) {
        $repoRoot = $repoRoot.Replace('/', '\')
    } else {
        $repoRoot = $repoRoot.Replace('\', '/')
    }
    
    Push-Location $repoRoot
    
    if ($Init) {
        Write-Status "Initializing submodules..."
        git submodule update --init --recursive
        if ($LASTEXITCODE -ne 0) {
            throw "Failed to initialize submodules"
        }
    }

    # Update all submodules to latest version
    Write-Status "Updating submodules to latest version..."
    git submodule update --remote --merge --recursive
    if ($LASTEXITCODE -ne 0) {
        throw "Failed to update submodules"
    }

    if ($Force) {
        Write-Status "Force resetting submodules to remote state..."
        git submodule foreach --recursive 'git reset --hard origin/$(git rev-parse --abbrev-ref HEAD)'
        if ($LASTEXITCODE -ne 0) {
            throw "Failed to force reset submodules"
        }
    }

    Write-Status "Submodules updated successfully!"
}
catch {
    Write-Error "Error updating submodules: $_"
    exit 1
}
finally {
    Pop-Location
}
