# Log Archiver Tool
using namespace System.IO.Compression

Add-Type -AssemblyName System.IO.Compression.FileSystem

# Import logging module
. (Join-Path $PSScriptRoot "logger.ps1")

function Compress-LogSession {
    [CmdletBinding(SupportsShouldProcess)]
    param(
        [Parameter(Mandatory)]
        [string]$SessionPath,
        
        [Parameter()]
        [string]$ArchivePath,
        
        [Parameter()]
        [switch]$RemoveSource,
        
        [Parameter()]
        [switch]$CreateChecksum
    )
    
    if (-not $PSCmdlet.ShouldProcess($SessionPath, "Compress log session")) {
        return $false
    }
    
    try {
        Write-Log -Message "Starting compression of session: $SessionPath" -Level "INFO"
        
        # Generate archive path if not provided
        if (-not $ArchivePath) {
            $sessionName = Split-Path -Path $SessionPath -Leaf
            $ArchivePath = Join-Path -Path (Split-Path -Path $SessionPath) -ChildPath "archive" -AdditionalPath "$sessionName.zip"
        }
        
        # Ensure archive directory exists
        $archiveDir = Split-Path -Path $ArchivePath -Parent
        if (-not (Test-Path -Path $archiveDir)) {
            New-Item -ItemType Directory -Path $archiveDir -Force | Out-Null
        }
        
        # Create temporary directory for organizing files
        $tempDir = Join-Path -Path $env:TEMP -ChildPath "LogArchive_$(Get-Random)"
        New-Item -ItemType Directory -Path $tempDir -Force | Out-Null
        
        try {
            # Copy files to temp directory with organized structure
            Write-Log -Message "Organizing files for archival" -Level "INFO"
            
            # Copy main log file
            Copy-Item -Path (Join-Path -Path $SessionPath -ChildPath "session.log") -Destination $tempDir
            
            # Copy and organize other logs
            $dirs = @("commands", "output", "errors", "metrics")
            foreach ($dir in $dirs) {
                $sourcePath = Join-Path -Path $SessionPath -ChildPath $dir
                if (Test-Path -Path $sourcePath) {
                    $targetPath = Join-Path -Path $tempDir -ChildPath $dir
                    Copy-Item -Path $sourcePath -Destination $targetPath -Recurse
                }
            }
            
            # Create manifest
            $manifest = @{
                ArchiveDate = Get-Date
                SessionPath = $SessionPath
                Files       = Get-ChildItem -Path $tempDir -Recurse | ForEach-Object {
                    @{
                        Path          = $_.FullName.Substring($tempDir.Length + 1)
                        Size          = if ($_.PSIsContainer) { 0 } else { $_.Length }
                        LastWriteTime = $_.LastWriteTime
                    }
                }
            }
            
            $manifest | ConvertTo-Json -Depth 10 | Set-Content -Path (Join-Path -Path $tempDir -ChildPath "manifest.json")
            
            # Compress the directory
            Write-Log -Message "Creating archive: $ArchivePath" -Level "INFO"
            if (Test-Path -Path $ArchivePath) {
                Remove-Item -Path $ArchivePath -Force
            }
            [ZipFile]::CreateFromDirectory($tempDir, $ArchivePath, [CompressionLevel]::Optimal, $false)
            
            # Create checksum file if requested
            if ($CreateChecksum) {
                $hash = Get-FileHash -Path $ArchivePath -Algorithm SHA256
                $checksumFile = "$ArchivePath.sha256"
                "$($hash.Hash)  $(Split-Path -Path $ArchivePath -Leaf)" | Set-Content -Path $checksumFile
                Write-Log -Message "Created checksum file: $checksumFile" -Level "INFO"
            }
            
            # Remove source if requested
            if ($RemoveSource) {
                Write-Log -Message "Removing source directory: $SessionPath" -Level "INFO"
                Remove-Item -Path $SessionPath -Recurse -Force
            }
            
            Write-Log -Message "Archive created successfully: $ArchivePath" -Level "INFO"
            return $true
        }
        finally {
            # Clean up temp directory
            if (Test-Path -Path $tempDir) {
                Remove-Item -Path $tempDir -Recurse -Force
            }
        }
    }
    catch {
        Write-Log -Message "Error creating archive: $_" -Level "ERROR"
        return $false
    }
}

function Expand-LogArchive {
    [CmdletBinding(SupportsShouldProcess)]
    param(
        [Parameter(Mandatory)]
        [string]$ArchivePath,
        
        [Parameter()]
        [string]$DestinationPath,
        
        [Parameter()]
        [switch]$ValidateChecksum
    )
    
    if (-not $PSCmdlet.ShouldProcess($ArchivePath, "Expand log archive")) {
        return $false
    }
    
    try {
        Write-Log -Message "Starting extraction of archive: $ArchivePath" -Level "INFO"
        
        # Validate archive exists
        if (-not (Test-Path -Path $ArchivePath)) {
            Write-Log -Message "Archive not found: $ArchivePath" -Level "ERROR"
            return $false
        }
        
        # Validate checksum if requested
        if ($ValidateChecksum) {
            $checksumFile = "$ArchivePath.sha256"
            if (Test-Path -Path $checksumFile) {
                $storedHash = (Get-Content -Path $checksumFile).Split()[0]
                $actualHash = (Get-FileHash -Path $ArchivePath -Algorithm SHA256).Hash
                if ($storedHash -ne $actualHash) {
                    Write-Log -Message "Checksum validation failed" -Level "ERROR"
                    return $false
                }
                Write-Log -Message "Checksum validation successful" -Level "INFO"
            }
        }
        
        # Generate destination path if not provided
        if (-not $DestinationPath) {
            $archiveName = [Path]::GetFileNameWithoutExtension($ArchivePath)
            $DestinationPath = Join-Path -Path (Split-Path -Path $ArchivePath) -ChildPath $archiveName
        }
        
        # Create destination directory
        if (-not (Test-Path -Path $DestinationPath)) {
            New-Item -ItemType Directory -Path $DestinationPath -Force | Out-Null
        }
        
        # Extract archive
        Write-Log -Message "Extracting to: $DestinationPath" -Level "INFO"
        [ZipFile]::ExtractToDirectory($ArchivePath, $DestinationPath, $true)
        
        # Validate manifest
        $manifestPath = Join-Path -Path $DestinationPath -ChildPath "manifest.json"
        if (Test-Path -Path $manifestPath) {
            $manifest = Get-Content -Path $manifestPath | ConvertFrom-Json
            Write-Log -Message "Archive created on: $($manifest.ArchiveDate)" -Level "INFO"
            Write-Log -Message "Original session path: $($manifest.SessionPath)" -Level "INFO"
        }
        
        Write-Log -Message "Archive extracted successfully" -Level "INFO"
        return $true
    }
    catch {
        Write-Log -Message "Error extracting archive: $_" -Level "ERROR"
        return $false
    }
}

function Start-LogArchiveManager {
    [CmdletBinding(SupportsShouldProcess)]
    param(
        [Parameter()]
        [int]$RetentionDays = 30,
        
        [Parameter()]
        [int]$CompressionAgeDays = 7,
        
        [Parameter()]
        [switch]$CreateChecksums
    )
    
    if (-not $PSCmdlet.ShouldProcess("Manage log archives")) {
        return $false
    }
    
    try {
        Write-Log -Message "Starting Log Archive Manager" -Level "INFO"
        
        # Get all log sessions
        $logsPath = $script:LogConfig.LogRoot
        $sessions = Get-ChildItem -Path $logsPath -Directory |
        Where-Object { -not $_.Name.EndsWith(".zip") }
        
        foreach ($session in $sessions) {
            $age = (Get-Date) - $session.CreationTime
            
            # Archive old sessions
            if ($age.TotalDays -gt $CompressionAgeDays) {
                Write-Log -Message "Archiving session: $($session.Name)" -Level "INFO"
                Compress-LogSession -SessionPath $session.FullName -RemoveSource -CreateChecksum:$CreateChecksums
            }
        }
        
        # Clean up old archives
        $archives = Get-ChildItem -Path $logsPath -Filter "*.zip" -Recurse
        foreach ($archive in $archives) {
            $age = (Get-Date) - $archive.CreationTime
            if ($age.TotalDays -gt $RetentionDays) {
                Write-Log -Message "Removing old archive: $($archive.Name)" -Level "INFO"
                Remove-Item -Path $archive.FullName -Force
                
                # Remove associated checksum file if it exists
                $checksumFile = "$($archive.FullName).sha256"
                if (Test-Path -Path $checksumFile) {
                    Remove-Item -Path $checksumFile -Force
                }
            }
        }
        
        Write-Log -Message "Log Archive Manager completed successfully" -Level "INFO"
        return $true
    }
    catch {
        Write-Log -Message "Error in Log Archive Manager: $_" -Level "ERROR"
        return $false
    }
}

# Export functions
Export-ModuleMember -Function @(
    'Compress-LogSession',
    'Expand-LogArchive',
    'Start-LogArchiveManager'
)
