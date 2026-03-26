# Installation and Validation Script for Advanced Logging System
[CmdletBinding()]
param(
    [Parameter()]
    [string]$InstallPath = $PSScriptRoot,
    
    [Parameter()]
    [switch]$Force,
    
    [Parameter()]
    [switch]$SkipTests,
    
    [Parameter()]
    [switch]$RegisterModule
)

# Initialize logging
$timestamp = Get-Date -Format "yyyy-MM-dd_HH-mm-ss"
$logDir = Join-Path -Path $PSScriptRoot -ChildPath "logs" | Join-Path -ChildPath "install_$timestamp"
$logFile = Join-Path -Path $logDir -ChildPath "install.log"

if (-not (Test-Path -Path $logDir)) {
    New-Item -ItemType Directory -Path $logDir -Force | Out-Null
}

function Write-InstallLog {
    param(
        [Parameter(Mandatory)]
        [string]$Message,
        
        [Parameter()]
        [ValidateSet('INFO', 'WARNING', 'ERROR', 'DEBUG')]
        [string]$Level = 'INFO'
    )
    
    $timestamp = Get-Date -Format "yyyy-MM-dd HH:mm:ss"
    $logMessage = "[$timestamp] [$Level] $Message"
    Add-Content -Path $logFile -Value $logMessage
    
    switch ($Level) {
        'WARNING' { Write-Warning $Message }
        'ERROR' { Write-Error $Message }
        'DEBUG' { Write-Verbose $Message }
        default { Write-Host $Message }
    }
}

Write-InstallLog "Starting installation process"
Write-InstallLog "Installation path: $InstallPath"
Write-InstallLog "Log file: $logFile"

# Verify PowerShell Core
Write-InstallLog "Checking PowerShell version" "DEBUG"
if ($PSVersionTable.PSEdition -ne 'Core') {
    Write-InstallLog "This script requires PowerShell Core (pwsh). Please run it using 'pwsh' instead of 'powershell'." "ERROR"
    exit 1
}
Write-InstallLog "PowerShell Core verified: $($PSVersionTable.PSVersion)" "DEBUG"

# Import required modules
$requiredModules = @(
    @{
        Name           = 'Pester'
        MinimumVersion = '5.0.0'
    },
    @{
        Name           = 'PSScriptAnalyzer'
        MinimumVersion = '1.20.0'
    }
)

foreach ($module in $requiredModules) {
    Write-InstallLog "Checking module: $($module.Name) (>= $($module.MinimumVersion))" "DEBUG"
    if (-not (Get-Module -ListAvailable -Name $module.Name | Where-Object { $_.Version -ge $module.MinimumVersion })) {
        Write-InstallLog "Installing module: $($module.Name)"
        Install-Module -Name $module.Name -Force -Scope CurrentUser -MinimumVersion $module.MinimumVersion
    }
    Write-InstallLog "Importing module: $($module.Name)" "DEBUG"
    Import-Module $module.Name -Force -MinimumVersion $module.MinimumVersion
}

# Skip file copy if installing to same directory
$skipFileCopy = $InstallPath -eq $PSScriptRoot
Write-InstallLog "Skip file copy: $skipFileCopy" "DEBUG"

if (-not $skipFileCopy) {
    Write-InstallLog "Validating installation path" "DEBUG"
    if (-not (Test-Path -Path $InstallPath)) {
        Write-InstallLog "Creating installation directory: $InstallPath"
        New-Item -ItemType Directory -Path $InstallPath -Force | Out-Null
    }

    # Copy module files
    $moduleFiles = @(
        'AdvancedLogging.psd1',
        'AdvancedLogging.psm1',
        'logger.ps1',
        'LogAnalyzer.ps1',
        'LogArchiver.ps1'
    )

    foreach ($file in $moduleFiles) {
        $sourcePath = Join-Path -Path $PSScriptRoot -ChildPath $file
        $targetPath = Join-Path -Path $InstallPath -ChildPath $file
        
        Write-InstallLog "Processing file: $file" "DEBUG"
        if (-not (Test-Path -Path $sourcePath)) {
            Write-InstallLog "Required file not found: $file" "ERROR"
            continue
        }
        
        try {
            Write-InstallLog "Copying file: $sourcePath -> $targetPath" "DEBUG"
            Copy-Item -Path $sourcePath -Destination $targetPath -Force
        }
        catch {
            Write-InstallLog "Error copying file $file : $_" "ERROR"
        }
    }
}

# Create required directories
$directories = @(
    'logs',
    'Tests',
    'docs'
)

foreach ($dir in $directories) {
    $path = Join-Path -Path $InstallPath -ChildPath $dir
    Write-InstallLog "Checking directory: $dir" "DEBUG"
    if (-not (Test-Path -Path $path)) {
        Write-InstallLog "Creating directory: $path"
        New-Item -ItemType Directory -Path $path -Force | Out-Null
    }
}

# Run PSScriptAnalyzer
Write-InstallLog "Running PSScriptAnalyzer..."
$analysis = Get-ChildItem -Path $InstallPath -Filter "*.ps1" | ForEach-Object {
    Write-InstallLog "Analyzing file: $($_.Name)" "DEBUG"
    $results = Invoke-ScriptAnalyzer -Path $_.FullName -Settings PSGallery
    if ($results) {
        [PSCustomObject]@{
            File   = $_.Name
            Issues = $results
        }
    }
}

if ($analysis) {
    Write-InstallLog "PSScriptAnalyzer found issues:" "WARNING"
    $analysis | ForEach-Object {
        Write-InstallLog "File: $($_.File)" "WARNING"
        $_.Issues | ForEach-Object {
            Write-InstallLog "  Line $($_.Line): $($_.Message)" "WARNING"
        }
    }
}
else {
    Write-InstallLog "PSScriptAnalyzer: All files pass analysis."
}

# Run tests if not skipped
if (-not $SkipTests) {
    Write-InstallLog "Running Pester tests..."
    try {
        Write-InstallLog "Configuring Pester" "DEBUG"
        $config = [PesterConfiguration]::Default
        $config.Run.Path = Join-Path -Path $InstallPath -ChildPath "Tests"
        $config.Run.PassThru = $true
        $config.Run.SkipRemainingOnFailure = 'None'
        $config.TestResult.Enabled = $true
        $config.TestResult.OutputPath = Join-Path -Path $logDir -ChildPath "pester_results.xml"
        $config.Output.Verbosity = 'Detailed'
        $config.Debug.ShowFullErrors = $true
        $config.CodeCoverage.Enabled = $false
        $config.TestRegistry.Enabled = $false
        
        Write-InstallLog "Starting Pester tests" "DEBUG"
        $testResults = Invoke-Pester -Configuration $config
        
        Write-InstallLog "Tests completed. Passed: $($testResults.PassedCount), Failed: $($testResults.FailedCount)" "DEBUG"
        if ($testResults.FailedCount -gt 0) {
            Write-InstallLog "Some tests failed. See $($config.TestResult.OutputPath) for details." "WARNING"
        }
        else {
            Write-InstallLog "All tests passed successfully!"
        }
    }
    catch {
        Write-InstallLog "Error running tests: $_" "ERROR"
    }
}

# Register module if requested
if ($RegisterModule) {
    Write-InstallLog "Registering module..."
    try {
        $userModulePath = if ($IsWindows) {
            Join-Path -Path ([Environment]::GetFolderPath('MyDocuments')) -ChildPath "PowerShell\Modules"
        }
        else {
            "~/.local/share/powershell/Modules"
        }
        
        Write-InstallLog "Module path: $userModulePath" "DEBUG"
        $moduleInstallPath = Join-Path -Path $userModulePath -ChildPath "AdvancedLogging"
        
        if (-not (Test-Path -Path $moduleInstallPath)) {
            Write-InstallLog "Creating module directory: $moduleInstallPath" "DEBUG"
            New-Item -ItemType Directory -Path $moduleInstallPath -Force | Out-Null
        }
        
        Write-InstallLog "Copying module files to: $moduleInstallPath" "DEBUG"
        Copy-Item -Path (Join-Path -Path $InstallPath -ChildPath "*") -Destination $moduleInstallPath -Recurse -Force
        Write-InstallLog "Module installed to: $moduleInstallPath"
    }
    catch {
        Write-InstallLog "Error registering module: $_" "ERROR"
    }
}

# Validate installation
try {
    Write-InstallLog "Validating installation..."
    Import-Module (Join-Path -Path $InstallPath -ChildPath "AdvancedLogging.psd1") -Force
    Initialize-LogSession -EnableMetrics
    Write-Log -Message "Installation test" -Level "INFO"
    Write-InstallLog "Installation validated successfully!"
}
catch {
    Write-InstallLog "Installation validation failed: $_" "ERROR"
}

Write-InstallLog @"

Installation Summary
-------------------
Location: $InstallPath
PSScriptAnalyzer: $($analysis ? 'Issues Found' : 'Passed')
Tests: $($SkipTests ? 'Skipped' : ($testResults.FailedCount -eq 0 ? 'Passed' : 'Failed'))
Module Registration: $($RegisterModule ? 'Completed' : 'Skipped')

Next Steps:
1. Review any warnings or errors above
2. Import the module: Import-Module AdvancedLogging
3. Initialize logging: Initialize-LogSession
4. Start using the logging functions!

Documentation can be found in: $(Join-Path $InstallPath "docs")
"@

Write-InstallLog "Installation process completed"
