# Advanced Logging System Module
using namespace System.IO
using namespace System.IO.Compression
using namespace System.Security.Cryptography

# Import all component scripts
$components = @(
    'logger.ps1',
    'LogArchiver.ps1'
)

foreach ($component in $components) {
    $componentPath = Join-Path $PSScriptRoot $component
    if (Test-Path $componentPath) {
        . $componentPath
    }
    else {
        throw "Required component not found: $component"
    }
}

# Module initialization
$script:ModuleConfig = @{
    Version        = '1.0.0'
    DefaultLogRoot = Join-Path $PSScriptRoot "logs"
}

# Module startup
if (-not (Test-Path $script:ModuleConfig.DefaultLogRoot)) {
    New-Item -ItemType Directory -Path $script:ModuleConfig.DefaultLogRoot -Force | Out-Null
}

# Export public functions and variables
Export-ModuleMember -Function @(
    'Initialize-LogSession',
    'Write-Log',
    'Write-CommandLog',
    'Write-CommandOutput',
    'Update-Metrics',
    'Remove-OldLogs',
    'Compress-LogSession',
    'Expand-LogArchive',
    'Start-LogArchiveManager'
) -Variable @('LogConfig')
