param([int]$AppPid, [string]$Phase, [string]$Marker)
$ErrorActionPreference = 'Stop'
Add-Type -AssemblyName UIAutomationClient
Add-Type -AssemblyName UIAutomationTypes
$A = [System.Windows.Automation.AutomationElement]
$scope = [System.Windows.Automation.TreeScope]::Descendants
$pidCondition = [System.Windows.Automation.PropertyCondition]::new($A::ProcessIdProperty, $AppPid)
function Find-Named([string]$Name) {
  $window = $A::RootElement.FindFirst([System.Windows.Automation.TreeScope]::Children, $pidCondition)
  if (!$window) { return $null }
  return $window.FindFirst($scope, [System.Windows.Automation.PropertyCondition]::new($A::NameProperty, $Name))
}
function Wait-Node([string]$Name) {
  $end = (Get-Date).AddSeconds(40)
  do {
    $node = Find-Named $Name
    if ($node) { return $node }
    Start-Sleep -Milliseconds 250
  } while ((Get-Date) -lt $end)
  throw "UI Automation timeout: $Name"
}
if ($Phase -eq 'create') {
  $button = Wait-Node 'Create workspace'
  $button.GetCurrentPattern([System.Windows.Automation.InvokePattern]::Pattern).Invoke()
  $entry = Wait-Node 'Workspace name'
  $entry.GetCurrentPattern([System.Windows.Automation.ValuePattern]::Pattern).SetValue($Marker)
  $button = Wait-Node 'Create workspace'
  $button.GetCurrentPattern([System.Windows.Automation.InvokePattern]::Pattern).Invoke()
} else {
  $null = Wait-Node $Marker
}
$null = Wait-Node 'Export backup'
$window = $A::RootElement.FindFirst([System.Windows.Automation.TreeScope]::Children, $pidCondition)
$window.GetCurrentPattern([System.Windows.Automation.WindowPattern]::Pattern).Close()
Write-Output "PASS: UI Automation workspace $Phase and window close requested"
