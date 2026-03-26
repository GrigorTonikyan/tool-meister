# Automated tests for logging system
using namespace System.IO

BeforeAll {
    # Import logging module
    $modulePath = Join-Path $PSScriptRoot ".." "AdvancedLogging.psd1"
    Import-Module $modulePath -Force
    
    # Create test directory
    $script:TestDir = Join-Path $PSScriptRoot "TestLogs"
    if (Test-Path $script:TestDir) {
        Remove-Item $script:TestDir -Recurse -Force
    }
    New-Item -ItemType Directory -Path $script:TestDir -Force | Out-Null
    
    # Set test log root
    $script:LogConfig.LogRoot = $script:TestDir
}

Describe "Logger Tests" {
    BeforeEach {
        Initialize-LogSession -EnableMetrics -EnableCompression
    }
    
    AfterEach {
        if ($script:LogConfig.CurrentSession) {
            Remove-Item $script:LogConfig.CurrentSession.Directory -Recurse -Force -ErrorAction SilentlyContinue
        }
    }
    
    Context "Basic Logging" {
        It "Should create log session directory" {
            $script:LogConfig.CurrentSession.Directory | Should -Exist
        }
        
        It "Should create main log file" {
            $script:LogConfig.CurrentSession.MainLog | Should -Exist
        }
        
        It "Should write log entries correctly" {
            Write-Log "Test message" "INFO"
            $content = Get-Content $script:LogConfig.CurrentSession.MainLog
            $content | Should -Match "\[\d{4}-\d{2}-\d{2} \d{2}:\d{2}:\d{2}\.\d{3}\] \[INFO\] Test message"
        }
    }
    
    Context "Log Levels" {
        It "Should respect minimum log level" {
            Initialize-LogSession -MinLogLevel "WARNING"
            Write-Log "Debug message" "DEBUG"
            Write-Log "Warning message" "WARNING"
            $content = Get-Content $script:LogConfig.CurrentSession.MainLog
            $content | Should -Not -Match "\[DEBUG\] Debug message"
            $content | Should -Match "\[WARNING\] Warning message"
        }
    }
    
    Context "Command Logging" {
        It "Should log command execution" {
            $cmdId = Write-CommandLog "test-command" @("arg1", "arg2") "/"
            $cmdId | Should -Not -BeNullOrEmpty
            
            Write-CommandOutput -CommandId $cmdId -Output "Test output" -Error "Test error" -ExitCode 0
            
            $cmdLog = Join-Path $script:LogConfig.CurrentSession.Directory "commands" "$cmdId.log"
            $cmdLog | Should -Exist
            
            $content = Get-Content $cmdLog
            $content | Should -Match "test-command"
            $content | Should -Match "Test output"
            $content | Should -Match "Test error"
        }
    }
}

# Run the tests
Invoke-Pester -Output Detailed
