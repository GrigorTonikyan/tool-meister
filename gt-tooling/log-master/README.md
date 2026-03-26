# LogMaster - Advanced PowerShell Logging System

A comprehensive logging system for PowerShell scripts with advanced features including log rotation, compression, metrics tracking, and GUI-based analysis.

## Features

- **Multi-level Logging**: TRACE, DEBUG, INFO, WARNING, ERROR, CRITICAL
- **Metrics Tracking**: Command counts, error rates, performance metrics
- **Log Rotation**: Automatic file rotation based on size
- **Compression**: Automatic archival with SHA256 checksums
- **GUI Analysis**: Interactive log viewer and analyzer
- **Command Logging**: Detailed command execution tracking
- **Stack Traces**: Optional stack trace capture for debugging

## Components

### Core Module (LogMaster.ps1)

The main logging engine providing core functionality:

```powershell
# Initialize a new logging session
Initialize-LogSession -EnableMetrics -EnableCompression -MinLogLevel "INFO"

# Write logs at different levels
Write-Log "Operation completed" "INFO"
Write-Log "Configuration error" "ERROR" -IncludeStack

# Track command execution
$cmdId = Write-CommandLog "git" @("status") "/"
Write-CommandOutput $cmdId $output $error 0
```

### Log Analyzer (LogAnalyzer.ps1)

GUI-based tool for viewing and analyzing logs:

- Session selection
- Level filtering
- Full-text search
- Metrics visualization
- CSV export

### Log Archiver (LogArchiver.ps1)

Handles log compression and archival:

```powershell
# Compress a log session
Compress-LogSession -SessionPath "path/to/logs" -CreateChecksum

# Extract an archive
Expand-LogArchive -ArchivePath "logs.zip" -ValidateChecksum

# Start automatic archive management
Start-LogArchiveManager -RetentionDays 30 -CompressionAgeDays 7
```

## Installation

1. Clone the repository
2. Import the modules:

```powershell
Import-Module .\LogMaster.ps1
Import-Module .\LogArchiver.ps1
```

## Configuration

### Log Levels

- TRACE: Detailed debugging information
- DEBUG: Debugging information
- INFO: General information
- WARNING: Warning messages
- ERROR: Error messages
- CRITICAL: Critical errors
- COMMAND: Command execution
- OUTPUT: Command output
- METRIC: Performance metrics

### Retention Settings

- Default retention: 30 days
- Default compression age: 7 days
- Maximum log size: 10MB

## Usage Examples

### Basic Logging

```powershell
# Initialize logging
Initialize-LogSession

# Write logs
Write-Log "Starting application" "INFO"
Write-Log "Configuration loaded" "DEBUG"
Write-Log "Invalid input" "ERROR" -IncludeStack
```

### Command Logging

```powershell
# Log command execution
$cmdId = Write-CommandLog "npm" @("install") $workingDir
try {
    $output = npm install
    Write-CommandOutput $cmdId $output $null 0
}
catch {
    Write-CommandOutput $cmdId $null $_.Exception.Message 1
}
```

### Log Analysis

```powershell
# Start the GUI analyzer
& .\LogAnalyzer.ps1

# Export logs
Export-Logs -Session "2023-11-15_12-00-00" -Level "ERROR"
```

### Log Archival

```powershell
# Archive old logs
Start-LogArchiveManager -RetentionDays 30 -CreateChecksums

# Extract specific archive
Expand-LogArchive -ArchivePath "logs_2023-11.zip" -ValidateChecksum
```

## Best Practices

1. **Log Level Selection**
   - Use TRACE for detailed debugging
   - Use INFO for general operation tracking
   - Use ERROR for application errors
   - Use CRITICAL for system-level issues

2. **Performance Considerations**
   - Enable compression for old logs
   - Set appropriate retention periods
   - Use log rotation for large files

3. **Security**
   - Avoid logging sensitive information
   - Use appropriate file permissions
   - Validate checksums when extracting archives

## Testing

Run the test suite:

```powershell
Invoke-Pester .\Tests\Test-Logger.ps1 -Output Detailed
```

## Contributing

1. Follow PowerShell best practices
2. Run PSScriptAnalyzer before submitting changes
3. Include tests for new features
4. Update documentation as needed

## License

MIT License - See LICENSE file for details
