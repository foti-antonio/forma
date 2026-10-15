param([int]$AppPid, [string]$Phase, [string]$Marker)
$ErrorActionPreference = 'Stop'
Add-Type -AssemblyName UIAutomationClient
Add-Type -AssemblyName UIAutomationTypes
$A = [System.Windows.Automation.AutomationElement]
$scope = [System.Windows.Automation.TreeScope]::Descendants
$pidCondition = [System.Windows.Automation.PropertyCondition]::new($A::ProcessIdProperty, $AppPid)
function Find-Named([string]$Name, [bool]$Edit = $false) {
  $window = $A::RootElement.FindFirst([System.Windows.Automation.TreeScope]::Children, $pidCondition)
  if (!$window) { return $null }
  $condition = [System.Windows.Automation.PropertyCondition]::new($A::NameProperty, $Name)
  if ($Edit) {
    $condition = [System.Windows.Automation.AndCondition]::new($condition,
      [System.Windows.Automation.PropertyCondition]::new($A::ControlTypeProperty, [System.Windows.Automation.ControlType]::Edit))
  }
  return $window.FindFirst($scope, $condition)
}
function Wait-Node([string]$Name, [bool]$Edit = $false) {
  $end = (Get-Date).AddSeconds(40)
  do {
    $node = Find-Named $Name $Edit
    if ($node) { return $node }
    Start-Sleep -Milliseconds 250
  } while ((Get-Date) -lt $end)
  throw "UI Automation timeout: $Name"
}
if ($Phase -eq 'create') {
  $button = Wait-Node 'Create workspace'
  $button.GetCurrentPattern([System.Windows.Automation.InvokePattern]::Pattern).Invoke()
  $entry = Wait-Node 'Workspace name' $true
  $entry.GetCurrentPattern([System.Windows.Automation.ValuePattern]::Pattern).SetValue($Marker)
  $button = Wait-Node 'Create workspace'
  $button.GetCurrentPattern([System.Windows.Automation.InvokePattern]::Pattern).Invoke()
}
$null = Wait-Node $Marker
$null = Wait-Node 'Export backup'
$window = $A::RootElement.FindFirst([System.Windows.Automation.TreeScope]::Children, $pidCondition)
$window.GetCurrentPattern([System.Windows.Automation.WindowPattern]::Pattern).Close()
Write-Output "PASS: UI Automation workspace $Phase and window close requested"
