# Vulkan iOS example project
This is an example project to run Vulkan application on iOS using Rust language.

---
## How it works
It is designed in such a way that the iOS application written in Objective-C calls the render framework written in Rust.

When an iOS application passes a UIView handle to the render framework, Rust's Vulkan wrapper vulkano initializes the Vulkan(MoltenVk) API. then iOS application continuously calls the render framework update function to draw the screen.

---
### Todo list
- Reduce draw calls. (Improve the render process)
- Apply texture.
- etc...
