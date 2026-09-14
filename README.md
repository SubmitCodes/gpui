# gpui

[Sonora](https://github.com/sonorahq/sonora)'s fork of [GPUI](https://github.com/zed-industries/zed), the GPU-accelerated UI framework behind
Zed.

This fork extends upstream GPUI with additional rendering, animation, and debugging capabilities:

- **Visual effects**
  - `Styled::text_blur` — Gaussian blur baked into glyph rasterization and cached in the atlas
  - `Styled::backdrop_blur` — CSS-style `backdrop-filter: blur()`
  - `Styled::blur` — blur an element and its subtree as a composited layer
  - Edge and side fading for composited layers

- **Paint-only transforms**
  - `Styled::layer_scale` — scale an element and its subtree without affecting layout
  - `Styled::layer_scale_origin` — choose the origin of a layer scale
  - `Styled::layer_translate` — translate a composited layer without affecting layout or hitboxes
  - Fractional/subpixel transforms suitable for smooth spring animations

- **Subpixel rendering**
  - Optional pixel-snapping control for subtrees via `Window::with_pixel_snapping`
  - Smooth fractional positioning without layout positions being rounded to device pixels

- **Text rendering**
  - Proper font-weight handling in the `cosmic-text` backend
  - Variable-font `wght` axis support
  - Weight-aware font fallback
  - Improved colour emoji font detection and rendering

- **Profiling and debugging**
  - Extended frame profiler with layout, prepaint, and paint timings
  - View render/cache/dirty statistics
  - Visual surface repaint highlighting
  - Notify/re-render diagnostics
  - Environment-controlled profiler and repaint overlays

- **Platform and rendering fixes**
  - Layer-filter support across WGPU, Metal, and DirectX renderers
  - Improved Wayland fractional-scaling behaviour
  - Fixes for half-device-pixel window edges and backdrop artefacts
  - Additional renderer fixes for filtered and transformed layers

## Licence

Apache-2.0, inherited from Zed. See `LICENSE-APACHE`.
