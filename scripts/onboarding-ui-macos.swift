// Public AX API only. Never prompt for or alter Accessibility/TCC permissions.
import AppKit
import ApplicationServices
let pid = pid_t(CommandLine.arguments[1])!
let fixture = CommandLine.arguments[2]
let repository = CommandLine.arguments[3]
let evidence = CommandLine.arguments[4]
let started = Date()
var actions = 0
var stages: [String: Any] = [:]
if !AXIsProcessTrusted() {
    print("UNVERIFIED: macOS AX access is not authorized for this runner; no TCC bypass attempted")
    exit(77)
}
func attribute(_ element: AXUIElement, _ name: String) -> CFTypeRef? {
    var value: CFTypeRef?
    guard AXUIElementCopyAttributeValue(element, name as CFString, &value) == .success else { return nil }
    return value
}
func strings(_ element: AXUIElement) -> [String] {
    return [kAXTitleAttribute, kAXDescriptionAttribute, kAXValueAttribute].compactMap { attribute(element, $0) as? String }
}
func walk(_ element: AXUIElement, _ depth: Int = 0) -> [AXUIElement] {
    if depth > 40 { return [] }
    var result = [element]
    if let children = attribute(element, kAXChildrenAttribute) as? [AXUIElement] {
        for child in children { result += walk(child, depth + 1) }
    }
    return result
}
let app = AXUIElementCreateApplication(pid)
func find(_ name: String, _ role: String? = nil) -> AXUIElement? {
    return walk(app).first { element in
        strings(element).contains(where: { $0 == name }) &&
        (role == nil || attribute(element, kAXRoleAttribute) as? String == role)
    }
}
func wait(_ label: String, _ lookup: () -> AXUIElement?) -> AXUIElement {
    for _ in 0..<160 {
        if let element = lookup() { return element }
        Thread.sleep(forTimeInterval: 0.25)
    }
    fputs("AX timeout: \(label); available sample controls: \(walk(app).flatMap { strings($0) }.prefix(100))\n", stderr)
    exit(1)
}
func press(_ element: AXUIElement) {
    actions += 1
    guard AXUIElementPerformAction(element, kAXPressAction as CFString) == .success else {
        fputs("AXPress failed\n", stderr); exit(1)
    }
}

func click(_ name: String) {
    press(wait(name) {
        guard let element = find(name), attribute(element, kAXEnabledAttribute) as? Bool == true else { return nil }
        return element
    })
}
func fill(_ name: String, _ value: String) {
    let entry = wait(name) { find(name, kAXTextFieldRole) }
    actions += 1
    guard AXUIElementSetAttributeValue(entry, kAXValueAttribute as CFString, value as CFString) == .success else { exit(1) }
}
func mark(_ stage: String) {
    stages[stage] = ["elapsed_seconds": Date().timeIntervalSince(started), "actions": actions]
}
func key(_ code: CGKeyCode, _ flags: CGEventFlags = []) {
    guard NSWorkspace.shared.frontmostApplication?.processIdentifier == pid else { exit(1) }
    for down in [true, false] {
        let event = CGEvent(keyboardEventSource: nil, virtualKey: code, keyDown: down)!
        event.flags = flags
        event.post(tap: .cghidEventTap)
    }
    Thread.sleep(forTimeInterval: 0.3)
}
NSRunningApplication(processIdentifier: pid)?.activate(options: [])
click("Create workspace")
fill("Workspace name", "Onboarding smoke")
click("Create workspace")
click("New project")
fill("Project name", "Onboarding engine")
click("Create project")
click("Onboarding engine")
click("New issue")
fill("Title", "First approved task")
click("Create issue")
press(wait("saved issue") { walk(app).first { strings($0).contains { $0.contains("First approved task") } && attribute($0, kAXRoleAttribute) as? String == kAXButtonRole } })
click("Run with AI")
mark("setup")
click("Find installation")
click("Authorize this installation: " + fixture)
mark("authorized")
mark("signed_in")
click("Select Git folder…")
_ = wait("native chooser") { find("Connect Git repository") }
key(5, [.maskCommand, .maskShift]) // Cmd Shift G: native Go to Folder
let location = wait("Go to Folder input") {
    walk(app).first { attribute($0, kAXRoleAttribute) as? String == kAXComboBoxRole || attribute($0, kAXRoleAttribute) as? String == kAXTextFieldRole && attribute($0, kAXFocusedAttribute) as? Bool == true }
}
guard AXUIElementSetAttributeValue(location, kAXValueAttribute as CFString, repository as CFString) == .success else { exit(1) }
key(36)
click("Open")
mark("repository")
click("Preview first task")
let prompt = wait("exact prompt") { find("Task prompt (saved issue)") }
guard let value = attribute(prompt, kAXValueAttribute) as? String, value.contains("First approved task"), value.contains("UNTRUSTED TASK DATA") else { exit(1) }
mark("preview")
click("Approve & run once")
click("Inspect result")
mark("completed")
click("Accept for manual integration")
mark("reviewed")
let metrics: [String: Any] = ["schema": 1, "evidence": "simulated", "result": "passed", "actions": actions, "errors": 0, "recoveries": 0, "elapsed_seconds": Date().timeIntervalSince(started), "stages": stages]
try JSONSerialization.data(withJSONObject: metrics, options: [.prettyPrinted, .sortedKeys]).write(to: URL(fileURLWithPath: evidence))
print("PASS: native macOS onboarding and approved simulated task")
