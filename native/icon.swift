import AppKit

// Use explicit pixel dimensions so icon generation is independent of display scale.
let bitmap = NSBitmapImageRep(bitmapDataPlanes: nil, pixelsWide: 1024, pixelsHigh: 1024,
    bitsPerSample: 8, samplesPerPixel: 4, hasAlpha: true, isPlanar: false,
    colorSpaceName: .deviceRGB, bytesPerRow: 0, bitsPerPixel: 0)!
NSGraphicsContext.saveGraphicsState()
NSGraphicsContext.current = NSGraphicsContext(bitmapImageRep: bitmap)
let frame = NSRect(x: 72, y: 72, width: 880, height: 880)
let background = NSBezierPath(roundedRect: frame, xRadius: 196, yRadius: 196)
NSGradient(starting: NSColor(srgbRed: 0.16, green: 0.16, blue: 0.17, alpha: 1),
           ending: NSColor(srgbRed: 0.07, green: 0.07, blue: 0.08, alpha: 1))!
    .draw(in: background, angle: -90)
NSColor(white: 1, alpha: 0.12).setStroke()
background.lineWidth = 2
background.stroke()
let attributes: [NSAttributedString.Key: Any] = [
    .font: NSFont(name: "Menlo-Bold", size: 430) ?? NSFont.monospacedSystemFont(ofSize: 430, weight: .bold),
    .foregroundColor: NSColor(srgbRed: 0.92, green: 0.92, blue: 0.92, alpha: 1)
]
let text = "Ay" as NSString
let size = text.size(withAttributes: attributes)
text.draw(at: NSPoint(x: (1024 - size.width) / 2, y: (1024 - size.height) / 2 + 12),
          withAttributes: attributes)
NSGraphicsContext.restoreGraphicsState()
try bitmap.representation(using: .png, properties: [:])!
    .write(to: URL(fileURLWithPath: CommandLine.arguments[1]))
