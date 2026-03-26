@{
    ModuleVersion     = '1.0.0'
    GUID              = 'f8dbc21d-3f4a-4c3e-9f2b-8d4f8d8e8c7a'
    Author            = 'Grigor T'
    CompanyName       = 'Grigor T'
    Copyright         = ''
    Description       = 'LogMaster - Advanced PowerShell Logging System'
    PowerShellVersion = '7.0'
    
    # Script module or binary module file associated with this manifest
    RootModule        = 'LogMaster.psm1'
    
    # Functions to export from this module
    FunctionsToExport = @(
        'Initialize-LogSession',
        'Write-Log',
        'Write-CommandLog',
        'Write-CommandOutput',
        'Update-LogMetric',
        'Start-LogRotation',
        'Remove-LogSession',
        'Compress-LogSession',
        'Expand-LogArchive',
        'Start-LogArchiveManager'
    )
    
    # Cmdlets to export from this module
    CmdletsToExport   = @()
    
    # Variables to export from this module
    VariablesToExport = @()
    
    # Aliases to export from this module
    AliasesToExport   = @()
    
    # Private data to pass to the module specified in RootModule/ModuleToProcess
    PrivateData       = @{
        PSData = @{
            # Tags applied to this module for module discovery
            Tags         = @('Logging', 'PowerShell', 'Core', 'Metrics', 'Archival')
            
            # License URI for this module
            LicenseUri   = ''
            
            # Project URI for this module
            ProjectUri   = ''
            
            # Icon URI for this module
            IconUri      = ''
            
            # Release notes for this module
            ReleaseNotes = 'Initial release of Advanced PowerShell Logging System'
        }
    }
}
