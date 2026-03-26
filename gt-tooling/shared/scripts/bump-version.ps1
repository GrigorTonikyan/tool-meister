#!/usr/bin/env pwsh

param(
    [Parameter(Mandatory=$true)]
    [ValidateSet('major', 'minor', 'patch')]
    [string]$VersionType,
    
    [Parameter(Mandatory=$false)]
    [string]$Submodule
)

$ErrorActionPreference = 'Stop'

function Update-Version {
    param(
        [string]$Version,
        [string]$Type
    )
    
    $parts = $Version.Split('.')
    switch ($Type) {
        'major' { 
            $parts[0] = [int]$parts[0] + 1
            $parts[1] = 0
            $parts[2] = 0
        }
        'minor' { 
            $parts[1] = [int]$parts[1] + 1
            $parts[2] = 0
        }
        'patch' { 
            $parts[2] = [int]$parts[2] + 1
        }
    }
    return $parts -join '.'
}

function Update-PowerShellModule {
    param([string]$ModulePath)
    
    $manifestPath = Get-ChildItem -Path $ModulePath -Filter "*.psd1" | Select-Object -First 1
    if ($manifestPath) {
        $manifest = Import-PowerShellDataFile $manifestPath.FullName
        $newVersion = Update-Version -Version $manifest.ModuleVersion -Type $VersionType
        
        $content = Get-Content $manifestPath.FullName -Raw
        $content = $content -replace "ModuleVersion\s*=\s*'[^']*'", "ModuleVersion = '$newVersion'"
        $content | Set-Content -Path $manifestPath.FullName -NoNewline
        
        Write-Host "Updated PowerShell module version to $newVersion"
        return $true
    }
    return $false
}

function Update-NodePackage {
    param([string]$PackagePath)
    
    $packageJsonPath = Join-Path $PackagePath "package.json"
    if (Test-Path $packageJsonPath) {
        $packageJson = Get-Content $packageJsonPath -Raw | ConvertFrom-Json
        $newVersion = Update-Version -Version $packageJson.version -Type $VersionType
        
        $packageJson.version = $newVersion
        $packageJson | ConvertTo-Json -Depth 100 | Set-Content -Path $packageJsonPath -NoNewline
        
        Write-Host "Updated Node.js package version to $newVersion"
        return $true
    }
    return $false
}

# Normalize path separators for cross-platform compatibility
function Get-NormalizedPath {
    param([string]$Path)
    if ($IsWindows) {
        return $Path.Replace('/', '\')
    }
    return $Path.Replace('\', '/')
}

# Main execution
try {
    $scriptPath = $PSScriptRoot
    if (-not $scriptPath) {
        $scriptPath = Split-Path -Parent $MyInvocation.MyCommand.Path
    }
    
    if ($Submodule) {
        $submodulePath = Join-Path (Split-Path -Parent $scriptPath) $Submodule
        $submodulePath = Get-NormalizedPath $submodulePath
        
        if (-not (Test-Path $submodulePath)) {
            throw "Submodule path not found: $submodulePath"
        }
        
        $updated = $false
        $updated = Update-PowerShellModule -ModulePath $submodulePath
        if (-not $updated) {
            $updated = Update-NodePackage -PackagePath $submodulePath
        }
        
        if (-not $updated) {
            throw "No version files found in submodule: $Submodule"
        }
    }
    else {
        # Update all submodules
        $rootPath = Split-Path -Parent $scriptPath
        Get-ChildItem -Path $rootPath -Directory |
            Where-Object { $_.Name -ne "shared" } |
            ForEach-Object {
                Write-Host "Processing $($_.Name)..."
                $updated = $false
                $updated = Update-PowerShellModule -ModulePath $_.FullName
                if (-not $updated) {
                    $updated = Update-NodePackage -PackagePath $_.FullName
                }
                if (-not $updated) {
                    Write-Warning "No version files found in: $($_.Name)"
                }
            }
    }
}
catch {
    Write-Error "Error updating versions: $_"
    exit 1
}
