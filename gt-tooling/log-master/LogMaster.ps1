# Advanced logging module for script actions
using namespace System.IO
using namespace System.Security.Cryptography

# Configuration
$script:LogConfig = @{
    LogRoot          = Join-Path -Path $PSScriptRoot -ChildPath "logs"
    CurrentSession   = $null
    LogLevels        = @{
        TRACE    = "TRACE"
        DEBUG    = "DEBUG"
        INFO     = "INFO"
        WARNING  = "WARNING"
        ERROR    = "ERROR"
        CRITICAL = "CRITICAL"
        COMMAND  = "COMMAND"
        OUTPUT   = "OUTPUT"
        METRIC   = "METRIC"
    }
    TimeFormat       = "yyyy-MM-dd HH:mm:ss.fff"
    RetentionDays    = 30
    CompressionAge   = 7  # Days before compressing logs
    MaxLogSize       = 10MB  # Size before rotating
    EnableMetrics    = $true
    EnableStackTrace = $true
    LogLevel         = "INFO"  # Minimum level to log
}

# Performance metrics
$script:Metrics = @{
    CommandCount = 0
    ErrorCount   = 0
    WarningCount = 0
    StartTime    = $null
    LogSize      = 0
    LastFlush    = $null
}

function Initialize-LogSession {
    [CmdletBinding(SupportsShouldProcess)]
    param(
        [Parameter()]
        [string]$LogRoot = $script:LogConfig.LogRoot,
        
        [Parameter()]
        [switch]$EnableCompression,
        
        [Parameter()]
        [switch]$EnableMetrics,
        
        [Parameter()]
        [ValidateSet("TRACE", "DEBUG", "INFO", "WARNING", "ERROR", "CRITICAL")]
        [string]$MinLogLevel = "INFO",
        
        [Parameter()]
        [ValidateRange(1MB, 1GB)]
        [long]$MaxLogSize = 10MB
    )
    
    try {
        # Update configuration
        $script:LogConfig.LogRoot = $LogRoot
        $script:LogConfig.EnableMetrics = $EnableMetrics
        $script:LogConfig.LogLevel = $MinLogLevel
        $script:LogConfig.MaxLogSize = $MaxLogSize
        
        # Create session directory
        $sessionId = [Guid]::NewGuid().ToString()
        $sessionDir = Join-Path $LogRoot $sessionId
        New-Item -ItemType Directory -Path $sessionDir -Force | Out-Null
        
        # Create command log directory
        $commandDir = Join-Path $sessionDir "commands"
        New-Item -ItemType Directory -Path $commandDir -Force | Out-Null
        
        # Create main log file
        $mainLog = Join-Path $sessionDir "session.log"
        New-Item -ItemType File -Path $mainLog -Force | Out-Null
        
        # Create metrics directory if enabled
        if ($EnableMetrics) {
            $metricsDir = Join-Path $sessionDir "metrics"
            New-Item -ItemType Directory -Path $metricsDir -Force | Out-Null
            $metricsFile = Join-Path $metricsDir "metrics.json"
            New-Item -ItemType File -Path $metricsFile -Force | Out-Null
            
            # Initialize metrics
            $script:Metrics.StartTime = Get-Date
            $script:Metrics.LastFlush = Get-Date
            $script:Metrics | ConvertTo-Json | Set-Content $metricsFile
        }
        
        # Create session object
        $session = @{
            Id           = $sessionId
            Directory    = $sessionDir
            CommandDir   = $commandDir
            MainLog      = $mainLog
            MetricsFile  = if ($EnableMetrics) { $metricsFile } else { $null }
            StartTime    = Get-Date
            LastRotation = Get-Date
        }
        
        # Store session
        $script:LogConfig.CurrentSession = $session
        
        # Log initialization
        Write-Log "Logging session initialized" "INFO"
        if ($EnableMetrics) {
            Write-Log "Metrics tracking enabled" "INFO"
        }
        
        return $session
    }
    catch {
        Write-Error "Failed to initialize logging session: $_"
        throw
    }
}

function Write-Log {
    [CmdletBinding()]
    param(
        [Parameter(Mandatory)]
        [string]$Message,
        
        [Parameter()]
        [ValidateSet("TRACE", "DEBUG", "INFO", "WARNING", "ERROR", "CRITICAL", "COMMAND", "OUTPUT", "METRIC")]
        [string]$Level = "INFO",
        
        [Parameter()]
        [switch]$NoRotate
    )
    
    try {
        if (-not $script:LogConfig.CurrentSession) {
            throw "Logging session not initialized"
        }
        
        # Check if level is valid
        if (-not $script:LogConfig.LogLevels.ContainsKey($Level)) {
            throw "Invalid log level: $Level"
        }
        
        # Format log entry
        $timestamp = Get-Date -Format $script:LogConfig.TimeFormat
        $logEntry = "[$timestamp] [$Level] $Message"
        
        # Write to log file
        Add-Content -Path $script:LogConfig.CurrentSession.MainLog -Value $logEntry
        
        # Check if rotation needed
        if (-not $NoRotate) {
            $logFile = Get-Item $script:LogConfig.CurrentSession.MainLog
            if ($logFile.Length -gt $script:LogConfig.MaxLogSize) {
                Start-LogRotation
            }
        }
    }
    catch {
        Write-Error "Failed to write log: $_"
        throw
    }
}

function Write-CommandLog {
    [CmdletBinding()]
    param(
        [Parameter(Mandatory)]
        [string]$Command,
        
        [Parameter()]
        [string[]]$Arguments,
        
        [Parameter()]
        [string]$WorkingDirectory = $PWD
    )
    
    try {
        if (-not $script:LogConfig.CurrentSession) {
            throw "Logging session not initialized"
        }
        
        # Generate command ID
        $commandId = [Guid]::NewGuid().ToString()
        
        # Create command log file
        $logFile = Join-Path $script:LogConfig.CurrentSession.CommandDir "$commandId.log"
        
        # Log command details
        $details = @"
Command: $Command
Arguments: $($Arguments -join ', ')
Working Directory: $WorkingDirectory
Start Time: $(Get-Date -Format $script:LogConfig.TimeFormat)
"@
        Set-Content -Path $logFile -Value $details
        
        # Update metrics
        if ($script:LogConfig.EnableMetrics) {
            $script:Metrics.CommandCount++
        }
        
        # Log to main log
        Write-Log "Command started: $Command (ID: $commandId)" "COMMAND"
        
        return $commandId
    }
    catch {
        Write-Error "Failed to log command: $_"
        throw
    }
}

function Write-CommandOutput {
    [CmdletBinding()]
    param(
        [Parameter(Mandatory)]
        [string]$CommandId,
        
        [Parameter()]
        [string]$Output,
        
        [Parameter()]
        [int]$ExitCode = 0
    )
    
    try {
        if (-not $script:LogConfig.CurrentSession) {
            throw "Logging session not initialized"
        }
        
        # Get command log file
        $logFile = Join-Path $script:LogConfig.CurrentSession.CommandDir "$CommandId.log"
        if (-not (Test-Path $logFile)) {
            throw "Command log not found: $CommandId"
        }
        
        # Append output and exit code
        Add-Content -Path $logFile -Value "`nOutput:`n$Output`n`nExit Code: $ExitCode"
        
        # Log to main log
        Write-Log "Command completed: $CommandId (Exit Code: $ExitCode)" "COMMAND"
    }
    catch {
        Write-Error "Failed to write command output: $_"
        throw
    }
}

function Get-LogConfig {
    return $script:LogConfig
}

function Start-LogRotation {
    [CmdletBinding(SupportsShouldProcess)]
    param(
        [Parameter(Mandatory)]
        [string]$LogFile
    )
    
    if (-not $PSCmdlet.ShouldProcess($LogFile, "Rotate log file")) {
        return $false
    }
    
    if ((Get-Item -Path $LogFile).Length -gt $script:LogConfig.MaxLogSize) {
        $timestamp = Get-Date -Format "yyyyMMddHHmmss"
        $rotatedFile = "$LogFile.$timestamp"
        Move-Item -Path $LogFile -Destination $rotatedFile -Force
        New-Item -ItemType File -Path $LogFile -Force | Out-Null
        
        if ($script:LogConfig.CurrentSession.EnableCompression) {
            Compress-Archive -Path $rotatedFile -DestinationPath "$rotatedFile.zip" -Force
            Remove-Item -Path $rotatedFile -Force
        }
        
        return $true
    }
    
    return $false
}

function Update-LogMetric {
    [CmdletBinding(SupportsShouldProcess)]
    param()
    
    if (-not $PSCmdlet.ShouldProcess("Update log metrics")) {
        return
    }
    
    if (-not $script:LogConfig.CurrentSession.EnableMetrics) {
        return
    }
    
    $metrics = @{
        Timestamp       = Get-Date
        CommandCount    = $script:Metrics.CommandCount
        ErrorCount      = $script:Metrics.ErrorCount
        WarningCount    = $script:Metrics.WarningCount
        SessionDuration = (Get-Date) - $script:Metrics.StartTime
        LogSize         = $script:Metrics.LogSize
    }
    
    $metricsPath = Join-Path -Path $script:LogConfig.CurrentSession.Directory -ChildPath "metrics/metrics.json"
    $metrics | ConvertTo-Json | Set-Content -Path $metricsPath
    
    $script:Metrics.LastFlush = Get-Date
    Write-Log -Message "Metrics updated" -Level "METRIC"
}

function Remove-LogSession {
    [CmdletBinding(SupportsShouldProcess)]
    param()
    
    if (-not $PSCmdlet.ShouldProcess("Remove old log sessions")) {
        return
    }
    
    $cutoffDate = (Get-Date).AddDays(-$script:LogConfig.RetentionDays)
    Get-ChildItem -Path $script:LogConfig.LogRoot -Directory |
    Where-Object { $_.CreationTime -lt $cutoffDate } |
    Remove-Item -Recurse -Force
}

Export-ModuleMember -Function @(
    'Initialize-LogSession',
    'Write-Log',
    'Write-CommandLog',
    'Write-CommandOutput',
    'Get-LogConfig',
    'Start-LogRotation',
    'Update-LogMetric',
    'Remove-LogSession'
)
