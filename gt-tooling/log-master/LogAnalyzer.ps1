# Log Analyzer Tool
using namespace System.Windows.Forms
using namespace System.Drawing

Add-Type -AssemblyName System.Windows.Forms
Add-Type -AssemblyName System.Drawing

function New-LogAnalyzerForm {
    [CmdletBinding(SupportsShouldProcess)]
    param()
    
    if (-not $PSCmdlet.ShouldProcess("Create new log analyzer form")) {
        return
    }
    
    # Create main form
    $form = New-Object Form
    $form.Text = "Log Analyzer"
    $form.Size = New-Object Size(1200, 800)
    $form.StartPosition = "CenterScreen"

    # Create controls
    $controls = Initialize-AnalyzerControl -Form $form
    
    # Event handlers
    Register-AnalyzerEvent -Controls $controls
    
    # Initial load
    if ($controls.SessionCombo.SelectedItem) {
        Update-AnalyzerView -Session $controls.SessionCombo.SelectedItem `
            -Level $controls.LevelCombo.SelectedItem `
            -SearchText $controls.SearchBox.Text `
            -LogView $controls.LogView `
            -MetricsView $controls.MetricsView
    }

    return $form
}

function Initialize-AnalyzerControl {
    [CmdletBinding()]
    param(
        [Parameter(Mandatory)]
        [System.Windows.Forms.Form]$Form
    )
    
    $controls = @{}
    
    # Session selector
    $controls.SessionLabel = New-Object Label
    $controls.SessionLabel.Location = New-Object Point(10, 10)
    $controls.SessionLabel.Size = New-Object Size(100, 20)
    $controls.SessionLabel.Text = "Log Session:"
    $Form.Controls.Add($controls.SessionLabel)

    $controls.SessionCombo = New-Object ComboBox
    $controls.SessionCombo.Location = New-Object Point(120, 10)
    $controls.SessionCombo.Size = New-Object Size(300, 20)
    $Form.Controls.Add($controls.SessionCombo)

    # Level filter
    $controls.LevelLabel = New-Object Label
    $controls.LevelLabel.Location = New-Object Point(430, 10)
    $controls.LevelLabel.Size = New-Object Size(80, 20)
    $controls.LevelLabel.Text = "Log Level:"
    $Form.Controls.Add($controls.LevelLabel)

    $controls.LevelCombo = New-Object ComboBox
    $controls.LevelCombo.Location = New-Object Point(520, 10)
    $controls.LevelCombo.Size = New-Object Size(100, 20)
    $controls.LevelCombo.Items.AddRange(@("ALL", "INFO", "WARNING", "ERROR", "COMMAND", "OUTPUT"))
    $controls.LevelCombo.SelectedIndex = 0
    $Form.Controls.Add($controls.LevelCombo)

    # Search box
    $controls.SearchLabel = New-Object Label
    $controls.SearchLabel.Location = New-Object Point(630, 10)
    $controls.SearchLabel.Size = New-Object Size(60, 20)
    $controls.SearchLabel.Text = "Search:"
    $Form.Controls.Add($controls.SearchLabel)

    $controls.SearchBox = New-Object TextBox
    $controls.SearchBox.Location = New-Object Point(700, 10)
    $controls.SearchBox.Size = New-Object Size(200, 20)
    $Form.Controls.Add($controls.SearchBox)

    # Buttons
    $controls.RefreshButton = New-Object Button
    $controls.RefreshButton.Location = New-Object Point(910, 8)
    $controls.RefreshButton.Size = New-Object Size(80, 25)
    $controls.RefreshButton.Text = "Refresh"
    $Form.Controls.Add($controls.RefreshButton)

    $controls.ExportButton = New-Object Button
    $controls.ExportButton.Location = New-Object Point(1000, 8)
    $controls.ExportButton.Size = New-Object Size(80, 25)
    $controls.ExportButton.Text = "Export"
    $Form.Controls.Add($controls.ExportButton)

    # Tab control
    $controls.TabControl = New-Object TabControl
    $controls.TabControl.Location = New-Object Point(10, 40)
    $controls.TabControl.Size = New-Object Size(1160, 710)
    $Form.Controls.Add($controls.TabControl)

    # Log View tab
    $controls.TabLogs = New-Object TabPage
    $controls.TabLogs.Text = "Log View"
    $controls.TabControl.Controls.Add($controls.TabLogs)

    $controls.LogView = New-Object ListView
    $controls.LogView.Location = New-Object Point(10, 10)
    $controls.LogView.Size = New-Object Size(1130, 660)
    $controls.LogView.View = [System.Windows.Forms.View]::Details
    $controls.LogView.FullRowSelect = $true
    $controls.LogView.GridLines = $true
    $controls.LogView.Columns.Add("Time", 150)
    $controls.LogView.Columns.Add("Level", 80)
    $controls.LogView.Columns.Add("Message", 880)
    $controls.TabLogs.Controls.Add($controls.LogView)

    # Metrics tab
    $controls.TabMetrics = New-Object TabPage
    $controls.TabMetrics.Text = "Metrics"
    $controls.TabControl.Controls.Add($controls.TabMetrics)

    $controls.MetricsView = New-Object ListView
    $controls.MetricsView.Location = New-Object Point(10, 10)
    $controls.MetricsView.Size = New-Object Size(1130, 660)
    $controls.MetricsView.View = [System.Windows.Forms.View]::Details
    $controls.MetricsView.FullRowSelect = $true
    $controls.MetricsView.GridLines = $true
    $controls.MetricsView.Columns.Add("Metric", 200)
    $controls.MetricsView.Columns.Add("Value", 400)
    $controls.TabMetrics.Controls.Add($controls.MetricsView)

    # Load sessions
    $logsPath = Join-Path -Path $PSScriptRoot -ChildPath "logs"
    $sessions = Get-ChildItem -Path $logsPath -Directory | Sort-Object CreationTime -Descending
    $controls.SessionCombo.Items.AddRange($sessions.Name)
    if ($controls.SessionCombo.Items.Count -gt 0) {
        $controls.SessionCombo.SelectedIndex = 0
    }

    return $controls
}

function Register-AnalyzerEvent {
    [CmdletBinding()]
    param(
        [Parameter(Mandatory)]
        [hashtable]$Controls
    )
    
    $Controls.RefreshButton.Add_Click({
            Update-AnalyzerView -Session $Controls.SessionCombo.SelectedItem `
                -Level $Controls.LevelCombo.SelectedItem `
                -SearchText $Controls.SearchBox.Text `
                -LogView $Controls.LogView `
                -MetricsView $Controls.MetricsView
        })

    $Controls.ExportButton.Add_Click({
            Export-AnalyzerLog -Session $Controls.SessionCombo.SelectedItem `
                -Level $Controls.LevelCombo.SelectedItem `
                -SearchText $Controls.SearchBox.Text
        })

    $Controls.SessionCombo.Add_SelectedIndexChanged({
            Update-AnalyzerView -Session $Controls.SessionCombo.SelectedItem `
                -Level $Controls.LevelCombo.SelectedItem `
                -SearchText $Controls.SearchBox.Text `
                -LogView $Controls.LogView `
                -MetricsView $Controls.MetricsView
        })

    $Controls.LevelCombo.Add_SelectedIndexChanged({
            Update-AnalyzerView -Session $Controls.SessionCombo.SelectedItem `
                -Level $Controls.LevelCombo.SelectedItem `
                -SearchText $Controls.SearchBox.Text `
                -LogView $Controls.LogView `
                -MetricsView $Controls.MetricsView
        })

    $Controls.SearchBox.Add_TextChanged({
            Update-AnalyzerView -Session $Controls.SessionCombo.SelectedItem `
                -Level $Controls.LevelCombo.SelectedItem `
                -SearchText $Controls.SearchBox.Text `
                -LogView $Controls.LogView `
                -MetricsView $Controls.MetricsView
        })
}

function Update-AnalyzerView {
    [CmdletBinding(SupportsShouldProcess)]
    param(
        [Parameter(Mandatory)]
        [string]$Session,
        
        [Parameter(Mandatory)]
        [string]$Level,
        
        [Parameter(Mandatory)]
        [string]$SearchText,
        
        [Parameter(Mandatory)]
        [System.Windows.Forms.ListView]$LogView,
        
        [Parameter(Mandatory)]
        [System.Windows.Forms.ListView]$MetricsView
    )
    
    if (-not $PSCmdlet.ShouldProcess("Update analyzer view")) {
        return
    }

    $LogView.Items.Clear()
    $MetricsView.Items.Clear()

    if (-not $Session) { return }

    $sessionPath = Join-Path -Path $PSScriptRoot -ChildPath "logs" -AdditionalPath $Session
    $logFile = Join-Path -Path $sessionPath -ChildPath "session.log"
    $metricsFile = Join-Path -Path $sessionPath -ChildPath "metrics" -AdditionalPath "metrics.json"

    # Update log view
    $logs = Get-Content -Path $logFile
    foreach ($log in $logs) {
        if ($log -match "\[(.*?)\] \[(.*?)\] (.*)") {
            $time = $matches[1]
            $logLevel = $matches[2]
            $message = $matches[3]

            if (($Level -eq "ALL" -or $logLevel -eq $Level) -and
                (-not $SearchText -or $message -like "*$SearchText*")) {
                $item = New-Object ListViewItem($time)
                $item.SubItems.Add($logLevel)
                $item.SubItems.Add($message)
                $LogView.Items.Add($item)
            }
        }
    }

    # Update metrics view
    if (Test-Path -Path $metricsFile) {
        $metrics = Get-Content -Path $metricsFile | ConvertFrom-Json
        $metrics.PSObject.Properties | ForEach-Object {
            $item = New-Object ListViewItem($_.Name)
            $item.SubItems.Add($_.Value)
            $MetricsView.Items.Add($item)
        }
    }
}

function Export-AnalyzerLog {
    [CmdletBinding(SupportsShouldProcess)]
    param(
        [Parameter(Mandatory)]
        [string]$Session,
        
        [Parameter(Mandatory)]
        [string]$Level,
        
        [Parameter(Mandatory)]
        [string]$SearchText
    )
    
    if (-not $PSCmdlet.ShouldProcess("Export analyzer log")) {
        return
    }

    if (-not $Session) { return }

    $saveDialog = New-Object SaveFileDialog
    $saveDialog.Filter = "CSV files (*.csv)|*.csv|All files (*.*)|*.*"
    $saveDialog.DefaultExt = "csv"
    $saveDialog.AddExtension = $true

    if ($saveDialog.ShowDialog() -eq [System.Windows.Forms.DialogResult]::OK) {
        $sessionPath = Join-Path -Path $PSScriptRoot -ChildPath "logs" -AdditionalPath $Session
        $logFile = Join-Path -Path $sessionPath -ChildPath "session.log"
        
        $logs = Get-Content -Path $logFile | ForEach-Object {
            if ($_ -match "\[(.*?)\] \[(.*?)\] (.*)") {
                $time = $matches[1]
                $logLevel = $matches[2]
                $message = $matches[3]

                if (($Level -eq "ALL" -or $logLevel -eq $Level) -and
                    (-not $SearchText -or $message -like "*$SearchText*")) {
                    [PSCustomObject]@{
                        Time    = $time
                        Level   = $logLevel
                        Message = $message
                    }
                }
            }
        }

        $logs | Export-Csv -Path $saveDialog.FileName -NoTypeInformation
        [System.Windows.Forms.MessageBox]::Show(
            "Logs exported successfully to $($saveDialog.FileName)",
            "Export Complete",
            [System.Windows.Forms.MessageBoxButtons]::OK,
            [System.Windows.Forms.MessageBoxIcon]::Information
        )
    }
}

# Show the analyzer
$form = New-LogAnalyzerForm
$form.ShowDialog()
