// Public AX API only. Never prompt for or alter Accessibility/TCC permissions.
import AppKit
import ApplicationServices
let pid = pid_t(CommandLine.arguments[1])!
let phase = CommandLine.arguments[2]
let marker = CommandLine.arguments[3]
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
    guard AXUIElementPerformAction(element, kAXPressAction as CFString) == .success else {
        fputs("AXPress failed\n", stderr); exit(1)
    }
}
if phase == "create" {
    press(wait("welcome button") { find("Create workspace", kAXButtonRole) })
    let entry = wait("workspace input") { find("Workspace name", kAXTextFieldRole) }
    guard AXUIElementSetAttributeValue(entry, kAXValueAttribute as CFString, marker as CFString) == .success else {
        fputs("AX text input failed\n", stderr); exit(1)
    }
    let submit = wait("enabled submit") {
        guard let button = find("Create workspace", kAXButtonRole),
              attribute(button, kAXEnabledAttribute) as? Bool == true else { return nil }
        return button
    }
    press(submit)
}
_ = wait("persisted workspace") { find(marker) }
press(wait("export backup") { find("Export backup", kAXButtonRole) })
// This Cancel belongs to the isolated app's native save panel, after the form closed.
press(wait("native save panel Cancel") { find("Cancel", kAXButtonRole) })
_ = wait("workspace after dialog cancellation") { find(marker) }
print("PASS: macOS AX workspace \(phase), rendered persistence and native export dialog/cancellation")
