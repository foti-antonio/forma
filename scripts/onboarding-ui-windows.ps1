param([int]$AppPid, [string]$Fixture, [string]$Repository, [string]$Evidence)
$started = Get-Date
$script:actions = 0
$stages = @{}
# Windows PowerShell 5 reads BOM-less files as ANSI. Build this Unicode label explicitly.
$folderControl = 'Select Git folder' + [char]0x2026
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
  for ($j = 0; $j -lt $windows.Count; $j++) {
    $window = $windows[$j]
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
  $node = $null
  if ($window) {
    $buttons = $window.FindAll($scope, [System.Windows.Automation.PropertyCondition]::new($A::ControlTypeProperty, [System.Windows.Automation.ControlType]::Button))
    for ($j = 0; $j -lt $buttons.Count; $j++) {
      $button = $buttons[$j]
      if ($button.Current.Name -like '*First approved task*') { $node = $button; break }
    }
  }
  if ($node) { break }
  Start-Sleep -Milliseconds 250
} while ((Get-Date) -lt $end)
if (!$node) { throw 'Saved issue unavailable' }
$node.GetCurrentPattern([System.Windows.Automation.InvokePattern]::Pattern).Invoke()
Click 'Run with AI'
Mark 'setup'
Click 'Find installation'
Click ('Authorize this installation: ' + $Fixture)
$null = Wait-Node $folderControl
Mark 'authorized'
Mark 'signed_in'
Click $folderControl
# Standard Windows folder chooser: prefer its labelled Folder field. This
# avoids depending on whichever element Windows reports as keyboard-focused.
$picker = Wait-Node 'Connect Git repository'
$folder = Find-Named 'Folder:' $true
if (!$folder) { $folder = Find-Named 'File name:' $true }
$folderPattern = $null
if ($folder -and $folder.TryGetCurrentPattern([System.Windows.Automation.ValuePattern]::Pattern, [ref]$folderPattern)) {
  Write-Output 'Native folder selection: labelled path field'
  $folderPattern.SetValue($Repository)
} else {
  # Some Windows builds expose no writable UIA pattern in this dialog.
  # Use normal address-bar keys, scoped to the exact native picker window.
  Add-Type -AssemblyName System.Windows.Forms
  Add-Type 'using System; using System.Runtime.InteropServices; public class OnboardingWindow { [DllImport("user32.dll")] public static extern IntPtr GetForegroundWindow(); [DllImport("user32.dll")] public static extern uint GetWindowThreadProcessId(IntPtr h, out uint p); }'
  function Picker-Keys([string]$Keys) {
    [uint32]$foregroundPid = 0
    $foreground = [OnboardingWindow]::GetForegroundWindow()
    $null = [OnboardingWindow]::GetWindowThreadProcessId($foreground, [ref]$foregroundPid)
    $active = $A::FromHandle($foreground)
    if ($foregroundPid -ne $AppPid -or $active.Current.Name -ne 'Connect Git repository') {
      throw 'Refusing keyboard input outside the native Forma folder picker'
    }
    [System.Windows.Forms.SendKeys]::SendWait($Keys)
  }
  # Some native accessibility providers report a Pane as focused even while the
  # address edit has keyboard focus. Type through the public keyboard interface;
  # escape SendKeys metacharacters (including the CI profile's RUNNER~1 path).
  Picker-Keys '%d'
  Start-Sleep -Milliseconds 500
  $escapedPath = -join ($Repository.ToCharArray() | ForEach-Object {
    if ('+^%~()[]{}'.Contains([string]$_)) { '{' + $_ + '}' } else { [string]$_ }
  })
  Write-Output 'Native folder selection: scoped address-bar keys'
  Picker-Keys '^a'
  Picker-Keys $escapedPath
  Picker-Keys '{ENTER}'
  Start-Sleep -Milliseconds 750

}
Click 'Select Folder'
$null = Wait-Node 'Preview first task'
Mark 'repository'
Click 'Preview first task'
$prompt = Wait-Node 'Task prompt (saved issue)' $true
$pattern = $null
if ($prompt.TryGetCurrentPattern([System.Windows.Automation.ValuePattern]::Pattern, [ref]$pattern)) { $value = $pattern.Current.Value }
elseif ($prompt.TryGetCurrentPattern([System.Windows.Automation.TextPattern]::Pattern, [ref]$pattern)) { $value = $pattern.DocumentRange.GetText(-1) }
else { throw 'Exact prompt has no readable UI Automation text pattern' }
if (!$value.Contains('First approved task') -or !$value.Contains('UNTRUSTED TASK DATA')) { throw 'Exact saved prompt missing' }
Mark 'preview'
Click 'Approve & run once'
Click 'Inspect result'
Mark 'completed'
Click 'Accept for manual integration'
Mark 'reviewed'
@{ schema = 1; evidence = 'simulated'; result = 'passed'; actions = $script:actions; errors = 0; recoveries = 0; elapsed_seconds = ((Get-Date) - $started).TotalSeconds; stages = $stages } | ConvertTo-Json -Depth 5 | Set-Content $Evidence
Write-Output 'PASS: native Windows onboarding and approved simulated task'
