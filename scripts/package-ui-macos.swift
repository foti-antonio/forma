// Public AX API probe. Never prompt for or alter Accessibility/TCC permissions.
import AppKit
import ApplicationServices
let pid = pid_t(CommandLine.arguments[1])!
if !AXIsProcessTrusted() {
    print("UNVERIFIED: macOS AX access is not authorized for this runner; no TCC bypass attempted")
    exit(77)
}
func walk(_ element: AXUIElement, _ depth: Int = 0) -> [String] {
    if depth > 40 { return [] }
    var value: CFTypeRef?
    var result: [String] = []
    for attr in [kAXTitleAttribute, kAXDescriptionAttribute, kAXValueAttribute] {
        if AXUIElementCopyAttributeValue(element, attr as CFString, &value) == .success,
           let text = value as? String { result.append(text) }
    }
    if AXUIElementCopyAttributeValue(element, kAXChildrenAttribute as CFString, &value) == .success,
       let children = value as? [AXUIElement] {
        for child in children { result += walk(child, depth + 1) }
    }
    return result
}
let app = AXUIElementCreateApplication(pid)
for _ in 0..<80 {
    if walk(app).contains(where: { $0.contains("New workspace") }) {
        print("PASS: macOS AX rendered workspace control (read-only probe)")
        exit(0)
    }
    Thread.sleep(forTimeInterval: 0.5)
}
fputs("macOS AX authorized but workspace control unavailable\n", stderr)
exit(1)
