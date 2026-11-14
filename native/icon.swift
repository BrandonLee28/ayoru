import AppKit

let image = NSImage(size: NSSize(width: 1024, height: 1024))
image.lockFocus()
let frame = NSRect(x: 72, y: 72, width: 880, height: 880)
let background = NSBezierPath(roundedRect: frame, xRadius: 204, yRadius: 204)
NSGradient(starting: NSColor(srgbRed: 0.22, green: 0.17, blue: 0.29, alpha: 1),
           ending: NSColor(srgbRed: 0.08, green: 0.06, blue: 0.11, alpha: 1))!.draw(in: background, angle: -70)
let moon = NSBezierPath(ovalIn: NSRect(x: 243, y: 242, width: 540, height: 540))
moon.appendOval(in: NSRect(x: 405, y: 362, width: 478, height: 478))
moon.windingRule = .evenOdd
NSGraphicsContext.saveGraphicsState()
background.addClip()
NSBezierPath(ovalIn: NSRect(x: 243, y: 242, width: 540, height: 540)).addClip()
NSColor(srgbRed: 0.91, green: 0.85, blue: 0.98, alpha: 1).setFill()
moon.fill()
NSGraphicsContext.restoreGraphicsState()
let play = NSBezierPath()
play.move(to: NSPoint(x: 490, y: 365))
play.line(to: NSPoint(x: 490, y: 535))
play.line(to: NSPoint(x: 635, y: 450))
play.close()
NSColor(srgbRed: 0.745, green: 0.627, blue: 0.898, alpha: 1).setFill()
play.fill()
image.unlockFocus()
let bitmap = NSBitmapImageRep(data: image.tiffRepresentation!)!
try bitmap.representation(using: .png, properties: [:])!.write(to: URL(fileURLWithPath: CommandLine.arguments[1]))
