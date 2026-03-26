# Advanced Logging System Tests
using namespace System.IO

BeforeAll {
    # Import module
    $modulePath = Join-Path $PSScriptRoot ".." "AdvancedLogging.psd1"
    Import-Module $modulePath -Force -Verbose
    
    # Create test directory
    $script:TestDir = Join-Path $PSScriptRoot "TestLogs"
    if (Test-Path $script:TestDir) {
        Remove-Item $script:TestDir -Recurse -Force
    }
    New-Item -ItemType Directory -Path $script:TestDir -Force | Out-Null
    
    # Set test log root
    $script:LogConfig.LogRoot = $script:TestDir
}

Describe "Module Loading" {
    It "Should load the module successfully" {
        Get-Module AdvancedLogging | Should -Not -BeNullOrEmpty
    }
    
    It "Should export required functions" {
        $requiredFunctions = @(
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
        
        $exportedFunctions = (Get-Module AdvancedLogging).ExportedFunctions.Keys
        foreach ($function in $requiredFunctions) {
            $exportedFunctions | Should -Contain $function
        }
    }
}

Describe "Logging System Flow Validation" {
    BeforeEach {
        # Clean start for each test
        if (Get-Variable -Name LogConfig -Scope Script -ErrorAction SilentlyContinue) {
            Remove-LogSession
        }
    }

    Context "Initialization Flow" {
        It "Should create session with correct structure" {
            # Initialize session
            $session = Initialize-LogSession -LogRoot $TestDir
            
            # Verify directory structure
            $session.Directory | Should -Exist
            Join-Path $session.Directory "session.log" | Should -Exist
            Join-Path $session.Directory "commands" | Should -Exist
            
            # Verify initialization log entry
            $logContent = Get-Content $session.MainLog
            $logContent | Should -Match "\[\d{4}-\d{2}-\d{2} \d{2}:\d{2}:\d{2}\.\d{3}\] \[INFO\] Logging session initialized"
        }

        It "Should handle metrics initialization" {
            $session = Initialize-LogSession -LogRoot $TestDir -EnableMetrics
            
            # Verify metrics file
            Join-Path $session.Directory "metrics" "metrics.json" | Should -Exist
            
            # Verify metrics initialization log
            $logContent = Get-Content $session.MainLog
            $logContent | Should -Match "Metrics tracking enabled"
        }
    }

    Context "Basic Logging Flow" {
        BeforeEach {
            Initialize-LogSession -LogRoot $TestDir
        }

        It "Should log messages with correct format" {
            Write-Log "Test message" "INFO"
            Start-Sleep -Milliseconds 100 # Ensure file write completes
            
            $logContent = Get-Content (Get-LogConfig).CurrentSession.MainLog
            $logContent | Should -Match '^\[\d{4}-\d{2}-\d{2} \d{2}:\d{2}:\d{2}\.\d{3}\] \[INFO\] Test message$'
        }

        It "Should respect log levels" {
            Initialize-LogSession -LogRoot $TestDir -MinLogLevel "WARNING"
            
            Write-Log "Debug message" "DEBUG"
            Write-Log "Warning message" "WARNING"
            Start-Sleep -Milliseconds 100
            
            $logContent = Get-Content (Get-LogConfig).CurrentSession.MainLog
            $logContent | Should -Not -Match "\[DEBUG\]"
            $logContent | Should -Match "\[WARNING\]"
        }
    }

    Context "Command Logging Flow" {
        BeforeEach {
            Initialize-LogSession -LogRoot $TestDir
        }

        It "Should track command execution flow" {
            # Start command
            $cmdId = Write-CommandLog "test-command" @("arg1", "arg2") "/"
            $cmdId | Should -Not -BeNullOrEmpty
            
            # Write output
            Write-CommandOutput -CommandId $cmdId -Output "Test output" -ExitCode 0
            Start-Sleep -Milliseconds 100
            
            # Verify command log
            $cmdLog = Join-Path (Get-LogConfig).CurrentSession.Directory "commands" "$cmdId.log"
            $cmdContent = Get-Content $cmdLog
            $cmdContent | Should -Match "Command: test-command"
            $cmdContent | Should -Match "Arguments: arg1, arg2"
            $cmdContent | Should -Match "Test output"
            $cmdContent | Should -Match "Exit Code: 0"
        }
    }

    Context "Log Rotation Flow" {
        It "Should rotate logs when size threshold reached" {
            Initialize-LogSession -LogRoot $TestDir -MaxLogSize 1KB
            
            # Generate enough logs to trigger rotation
            1..100 | ForEach-Object {
                Write-Log ("Large log entry " * 10) "INFO"
            }
            Start-Sleep -Milliseconds 100
            
            # Verify rotation occurred
            $rotatedLogs = Get-ChildItem (Get-LogConfig).CurrentSession.Directory -Filter "session.log.*"
            $rotatedLogs | Should -Not -BeNullOrEmpty
            
            # Verify rotation log entry
            $logContent = Get-Content (Get-LogConfig).CurrentSession.MainLog
            $logContent | Should -Match "Log rotation completed"
        }
    }
}

Describe "Error Handling" {
    Context "Invalid Operations" {
        It "Should handle logging without initialization" {
            Remove-LogSession
            { Write-Log "Test" "INFO" } | Should -Throw "Logging session not initialized"
        }

        It "Should handle invalid log levels" {
            Initialize-LogSession -LogRoot $TestDir
            { Write-Log "Test" "INVALID" } | Should -Throw "Invalid log level"
        }
    }
}

Describe "PSScriptAnalyzer Compliance" {
    BeforeAll {
        $scriptFiles = Get-ChildItem -Path (Join-Path $PSScriptRoot "..") -Filter "*.ps1"
    }
    
    It "Should pass PSScriptAnalyzer rules for <_.Name>" -ForEach $scriptFiles {
        $analysis = Invoke-ScriptAnalyzer -Path $_.FullName
        $analysis | Should -BeNullOrEmpty
    }
}

# Run the tests
Invoke-Pester -Output Detailed
