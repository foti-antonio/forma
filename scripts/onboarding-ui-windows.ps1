param([int]$AppPid, [string]$Fixture, [string]$Repository, [string]$Evidence)
$started = Get-Date
$script:actions = 0
$stages = @{}
# Windows PowerShell 5 reads BOM-less files as ANSI. Build this Unicode label explicitly.
$folderControl = 'Select Git folder' + [char]0x2026
$ErrorActionPreference = 'Stop'
Add-Type -AssemblyName UIAutomationClient
Add-Type -AssemblyName UIAutomationTypes
Add-Type -AssemblyName System.Windows.Forms
Add-Type @'
using System;
using System.Runtime.InteropServices;
[StructLayout(LayoutKind.Sequential)] public struct PickerPoint { public int X; public int Y; public PickerPoint(int x, int y) { X=x; Y=y; } }
public class OnboardingWindow {
  [DllImport("user32.dll")] public static extern IntPtr GetForegroundWindow();
  [DllImport("user32.dll")] public static extern uint GetWindowThreadProcessId(IntPtr h, out uint p);
  [DllImport("user32.dll")] public static extern IntPtr WindowFromPoint(PickerPoint p);
  [DllImport("user32.dll", SetLastError=true)] public static extern bool SetCursorPos(int x, int y);
  [DllImport("user32.dll")] public static extern void mouse_event(uint flags, uint x, uint y, uint data, UIntPtr extra);
}
'@
$A = [System.Windows.Automation.AutomationElement]
$scope = [System.Windows.Automation.TreeScope]::Descendants
$pidCondition = [System.Windows.Automation.PropertyCondition]::new($A::ProcessIdProperty, $AppPid)
function Find-Named([string]$Name, [bool]$Edit = $false, [bool]$Pressable = $false) {
  # Owned native dialogs may be separate UIA root windows. Search every
  # top-level window belonging to this process, never another application.
  $windows = $A::RootElement.FindAll([System.Windows.Automation.TreeScope]::Children, $pidCondition)
  $condition = [System.Windows.Automation.PropertyCondition]::new($A::NameProperty, $Name)
  if ($Edit) {
    $condition = [System.Windows.Automation.AndCondition]::new($condition,
      [System.Windows.Automation.PropertyCondition]::new($A::ControlTypeProperty, [System.Windows.Automation.ControlType]::Edit))
  }
  if ($Pressable) {
    $button = [System.Windows.Automation.PropertyCondition]::new($A::ControlTypeProperty, [System.Windows.Automation.ControlType]::Button)
    $split = [System.Windows.Automation.PropertyCondition]::new($A::ControlTypeProperty, [System.Windows.Automation.ControlType]::SplitButton)
    $check = [System.Windows.Automation.PropertyCondition]::new($A::ControlTypeProperty, [System.Windows.Automation.ControlType]::CheckBox)
    $roles = [System.Windows.Automation.OrCondition]::new([System.Windows.Automation.OrCondition]::new($button, $split), $check)
    $condition = [System.Windows.Automation.AndCondition]::new($condition, $roles)
  }
  for ($j = 0; $j -lt $windows.Count; $j++) {
    $window = $windows[$j]
    if (!$Edit -and !$Pressable -and $window.Current.Name -eq $Name) { return $window }
    $node = $window.FindFirst($scope, $condition)
    if ($node) { return $node }
  }
  return $null
}
function Wait-Node([string]$Name, [bool]$Edit = $false, [bool]$Pressable = $false) {
  $end = (Get-Date).AddSeconds(40)
  do {
    $node = Find-Named $Name $Edit $Pressable
    if ($node -and $node.Current.IsEnabled) { return $node }
    Start-Sleep -Milliseconds 250
  } while ((Get-Date) -lt $end)
  # Sample-only runner data. Capture names/roles (never Value/Text content) to
  # diagnose a missing native control without silently skipping the test.
  $windows = $A::RootElement.FindAll([System.Windows.Automation.TreeScope]::Children, $pidCondition)
  for ($w = 0; $w -lt $windows.Count; $w++) {
    $controls = $windows[$w].FindAll($scope, [System.Windows.Automation.Condition]::TrueCondition)
    for ($i = 0; $i -lt [Math]::Min(60, $controls.Count); $i++) {
      try {
        $info = $controls[$i].Current
        if (!$info.IsOffscreen) {
          $label = [string]$info.Name
          if ($label.Length -gt 120) { $label = $label.Substring(0, 120) }
          Write-Output ('UIA diagnostic: ' + $info.ControlType.ProgrammaticName + ' / ' + $info.AutomationId + ' / ' + $label)
        }
      } catch [System.Windows.Automation.ElementNotAvailableException] { }
    }
  }
  throw "UI Automation timeout: $Name"
}

function Click([string]$Name) {
  Write-Output ('UI action: ' + $Name)
  $node = Wait-Node $Name $false $true
  $script:actions++
  $pattern = $null
  if ($node.TryGetCurrentPattern([System.Windows.Automation.InvokePattern]::Pattern, [ref]$pattern)) { $pattern.Invoke() }
  elseif ($node.TryGetCurrentPattern([System.Windows.Automation.TogglePattern]::Pattern, [ref]$pattern)) { $pattern.Toggle() }
  elseif ($Name -eq 'Select Folder') {
    $foreground = [OnboardingWindow]::GetForegroundWindow()
    [uint32]$owner = 0
    $null = [OnboardingWindow]::GetWindowThreadProcessId($foreground, [ref]$owner)
    if ($owner -ne $AppPid -or $A::FromHandle($foreground).Current.Name -ne 'Connect Git repository') { throw 'Refusing mouse input outside the native Forma folder picker' }
    $bounds = $node.Current.BoundingRectangle
    if ($node.Current.IsOffscreen -or $bounds.Width -le 0 -or $bounds.Height -le 0) { throw 'Native Select Folder button is not visible' }
    $point = [PickerPoint]::new([int]($bounds.X + $bounds.Width / 2), [int]($bounds.Y + $bounds.Height / 2))
    [uint32]$pointOwner = 0
    $null = [OnboardingWindow]::GetWindowThreadProcessId([OnboardingWindow]::WindowFromPoint($point), [ref]$pointOwner)
    if ($pointOwner -ne $AppPid) { throw 'Refusing mouse input over another process' }
    if (![OnboardingWindow]::SetCursorPos($point.X, $point.Y)) { throw 'Native picker cursor placement failed' }
    Write-Output 'Native folder selection: scoped button mouse click'
    [OnboardingWindow]::mouse_event(0x0002, 0, 0, 0, [UIntPtr]::Zero)
    [OnboardingWindow]::mouse_event(0x0004, 0, 0, 0, [UIntPtr]::Zero)
  }
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
$null = Wait-Node 'Select Folder' $false $true
$folder = Find-Named 'Folder:' $true
if (!$folder) { $folder = Find-Named 'File name:' $true }
$folderPattern = $null
if ($folder -and $folder.TryGetCurrentPattern([System.Windows.Automation.ValuePattern]::Pattern, [ref]$folderPattern)) {
  Write-Output 'Native folder selection: labelled path field'
  $folderPattern.SetValue($Repository)
} else {
  # Some Windows builds expose no writable UIA pattern in this dialog.
  # Use normal address-bar keys, scoped to the exact native picker window.
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
