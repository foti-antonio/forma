param([int]$AppPid, [string]$Fixture, [string]$Repository, [string]$Evidence)
$started = Get-Date
$script:actions = 0
$stages = @{}
$ErrorActionPreference = 'Stop'
Add-Type -AssemblyName UIAutomationClient
Add-Type -AssemblyName UIAutomationTypes
$A = [System.Windows.Automation.AutomationElement]
$scope = [System.Windows.Automation.TreeScope]::Descendants
$pidCondition = [System.Windows.Automation.PropertyCondition]::new($A::ProcessIdProperty, $AppPid)
function Find-Named([string]$Name, [bool]$Edit = $false) {
  # Owned native dialogs may be separate UIA root windows. Search every
  # top-level window belonging to this process, never another application.
  $windows = $A::RootElement.FindAll([System.Windows.Automation.TreeScope]::Children, $pidCondition)
  $condition = [System.Windows.Automation.PropertyCondition]::new($A::NameProperty, $Name)
  if ($Edit) {
    $condition = [System.Windows.Automation.AndCondition]::new($condition,
      [System.Windows.Automation.PropertyCondition]::new($A::ControlTypeProperty, [System.Windows.Automation.ControlType]::Edit))
  }
  foreach ($window in $windows) {
    if (!$Edit -and $window.Current.Name -eq $Name) { return $window }
    $node = $window.FindFirst($scope, $condition)
    if ($node) { return $node }
  }
  return $null
}
function Wait-Node([string]$Name, [bool]$Edit = $false) {
  $end = (Get-Date).AddSeconds(40)
  do {
    $node = Find-Named $Name $Edit
    if ($node -and $node.Current.IsEnabled) { return $node }
    Start-Sleep -Milliseconds 250
  } while ((Get-Date) -lt $end)
  throw "UI Automation timeout: $Name"
}

function Click([string]$Name) {
  Write-Output ('UI action: ' + $Name)
  $node = Wait-Node $Name
  $script:actions++
  $pattern = $null
  if ($node.TryGetCurrentPattern([System.Windows.Automation.InvokePattern]::Pattern, [ref]$pattern)) { $pattern.Invoke() }
  elseif ($node.TryGetCurrentPattern([System.Windows.Automation.TogglePattern]::Pattern, [ref]$pattern)) { $pattern.Toggle() }
  else { throw "Control has no supported action: $Name" }
}
function Fill([string]$Name, [string]$Value) {
  Write-Output ('UI fill: ' + $Name)
  $script:actions++
  $node = Wait-Node $Name $true
  $node.GetCurrentPattern([System.Windows.Automation.ValuePattern]::Pattern).SetValue($Value)
}
function Mark([string]$Stage) {
  $stages[$Stage] = @{ elapsed_seconds = ((Get-Date) - $started).TotalSeconds; actions = $script:actions }
  @{ schema = 1; evidence = 'simulated'; result = 'incomplete'; actions = $script:actions; errors = 0; recoveries = 0; elapsed_seconds = ((Get-Date) - $started).TotalSeconds; stages = $stages } | ConvertTo-Json -Depth 5 | Set-Content $Evidence
}
Click 'Create workspace'
Fill 'Workspace name' 'Onboarding smoke'
Click 'Create workspace'
Click 'New project'
Fill 'Project name' 'Onboarding engine'
Click 'Create project'
Click 'Onboarding engine'
Click 'New issue'
Fill 'Title' 'First approved task'
Click 'Create issue'
$end = (Get-Date).AddSeconds(40)
do {
  $window = $A::RootElement.FindFirst([System.Windows.Automation.TreeScope]::Children, $pidCondition)
  $node = @($window.FindAll($scope, [System.Windows.Automation.Condition]::TrueCondition)) | Where-Object { $_.Current.Name.Contains('First approved task') -and $_.Current.ControlType -eq [System.Windows.Automation.ControlType]::Button } | Select-Object -First 1
  if ($node) { break }
  Start-Sleep -Milliseconds 250
} while ((Get-Date) -lt $end)
if (!$node) { throw 'Saved issue unavailable' }
$node.GetCurrentPattern([System.Windows.Automation.InvokePattern]::Pattern).Invoke()
Click 'Run with AI'
Mark 'setup'
Click 'Find installation'
Click ('Authorize this installation: ' + $Fixture)
Mark 'authorized'
Mark 'signed_in'
Click 'Select Git folder…'
# Standard Windows folder chooser. Only input into this app's foreground window.
Add-Type -AssemblyName System.Windows.Forms
Add-Type 'using System; using System.Runtime.InteropServices; public class OnboardingWindow { [DllImport("user32.dll")] public static extern IntPtr GetForegroundWindow(); [DllImport("user32.dll")] public static extern uint GetWindowThreadProcessId(IntPtr h, out uint p); }'
$null = Wait-Node 'Connect Git repository'
[uint32]$foregroundPid = 0
$null = [OnboardingWindow]::GetWindowThreadProcessId([OnboardingWindow]::GetForegroundWindow(), [ref]$foregroundPid)
if ($foregroundPid -ne $AppPid) { throw 'Refusing keyboard input outside Forma' }
[System.Windows.Forms.SendKeys]::SendWait('%d')
Start-Sleep -Milliseconds 300
$address = $A::FocusedElement
$address.GetCurrentPattern([System.Windows.Automation.ValuePattern]::Pattern).SetValue($Repository)
$null = [OnboardingWindow]::GetWindowThreadProcessId([OnboardingWindow]::GetForegroundWindow(), [ref]$foregroundPid)
if ($foregroundPid -ne $AppPid) { throw 'Refusing keyboard input outside Forma' }
[System.Windows.Forms.SendKeys]::SendWait('{ENTER}')
Click 'Select Folder'
Mark 'repository'
Click 'Preview first task'
$prompt = Wait-Node 'Task prompt (saved issue)' $true
$value = $prompt.GetCurrentPattern([System.Windows.Automation.ValuePattern]::Pattern).Current.Value
if (!$value.Contains('First approved task') -or !$value.Contains('UNTRUSTED TASK DATA')) { throw 'Exact saved prompt missing' }
Mark 'preview'
Click 'Approve & run once'
Click 'Inspect result'
Mark 'completed'
Click 'Accept for manual integration'
Mark 'reviewed'
@{ schema = 1; evidence = 'simulated'; result = 'passed'; actions = $script:actions; errors = 0; recoveries = 0; elapsed_seconds = ((Get-Date) - $started).TotalSeconds; stages = $stages } | ConvertTo-Json -Depth 5 | Set-Content $Evidence
Write-Output 'PASS: native Windows onboarding and approved simulated task'
