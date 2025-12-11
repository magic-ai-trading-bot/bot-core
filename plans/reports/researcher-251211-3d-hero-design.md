# 3D Hero Section Research: Bot Core Trading Platform
**Date**: 2025-12-11 | **Focus**: Three.js implementation for crypto trading UX

## Executive Summary
Crypto platforms excel with **dark cyberpunk aesthetics** + **floating elements** + **flowing data animations**. Bot Core should implement: (1) Animated neural network connecting background, (2) Floating crypto coins with rotation, (3) Data stream particles, (4) Performance-first approach (mobile-safe).

---

## 1. VISUAL CONCEPTS FOR CRYPTO PLATFORMS

### Proven Color Scheme
- **Primary**: Dark navy/black (#0a0f1f, #1a1a2e)
- **Accents**: Cyan (#00d4ff), Electric blue (#0099ff)
- **Success**: Lime green (#00ff41)
- **Data flow**: Gradient cyan→purple→pink

### Core 3D Elements (Ranked by Impact)
1. **Neural Network Grid** (background, low-opacity): Animated nodes + connecting lines, suggest AI/automation
2. **Floating Coins**: BTC/ETH rotating softly, scattered in 3D space
3. **Data Stream Particles**: Flowing lines from bottom-left to top-right (upward profit motion)
4. **Subtle Lens Flares**: Occasional light refractions (premium feel)
5. **Glow Effects**: Bloom on coins + network nodes

### Animation Philosophy
- Slow, confident movement (not jittery or overstimulating)
- Parallax effect: Background moves slower than foreground
- Interactive hover reactions (elements respond to mouse without overwhelming)
- Subtle color shifts on scroll (dark mode → slightly lighter tones)

---

## 2. THREE.JS IMPLEMENTATION PATTERNS

### Hero Section Architecture (Recommended)
```
Canvas Container (fullscreen/responsive)
├── Scene with ortho camera (for particle control)
├── Lighting: 3x soft point lights + ambient (#333)
├── Layers:
│   ├── Background: Neural network (static + subtle animation)
│   ├── Mid: Floating coins (3-5 coins, rotation + bob motion)
│   ├── Foreground: Data stream particles (continuous flow)
│   └── Effect: Post-processing (bloom, color grading)
```

### Performance Budget (Mobile-Critical)
- **Target**: 60fps on mobile (iPhone 12+), 30fps minimum
- **Max draw calls**: 50-70 (use BatchedRenderer pattern)
- **Memory**: <50MB GPU texture memory
- **Shader complexity**: Simple (no raymarching or complex calculations)

### Implementation Patterns

**1. Particle System** (1000-3000 particles)
- Use `THREE.Points` + `BufferGeometry` (NOT individual mesh objects)
- Pre-calculate particle positions in Blender or algorithm
- Use WebWorkers for physics calculations (off main thread)
- Batched rendering reduces draw calls from 3000→5

**2. Coin Floating Motion**
- Load 3D coin model (simple geometry, <1000 triangles)
- Apply shader-based animation (rotation + sine wave bob)
- Use `THREE.InstancedMesh` for multiple coins (5-10 copies)
- Fallback: 2D sprite coins on mobile

**3. Neural Network Grid**
- Procedurally generate node positions (no mesh loading needed)
- Connect nodes with `THREE.LineSegments` + texture-based glow
- Animate node opacity (pulsing effect, 0.3s cycle)
- Animate line color shifts (cyan→green)

**4. Data Streams**
- Custom shader for flowing particles along bezier curves
- Update vertex positions every frame (GPU-accelerated)
- Trail effect using depth-based transparency

### Mobile Responsiveness
- Canvas scales to window.devicePixelRatio (not oversampling)
- Reduce particle count on mobile: `particleCount = isMobile ? 1000 : 3000`
- Touch gestures: Detect swipe to pause animations (reduce GPU heat)
- WebGL fallback: Show static gradient background + CSS animations

---

## 3. BOT CORE SPECIFIC RECOMMENDATIONS

### Thematic Elements
1. **Neural Network Background**: Represents AI trading signals (primary hero asset)
2. **Crypto Coins**: BTC/ETH symbols floating, emphasize multi-asset trading
3. **Upward Data Streams**: Green particles flowing diagonally (profit narrative)
4. **Subtle Neon Grid**: Floor-like perspective (makes it feel like standing inside an engine)

### Interaction Model
- **Hover on coins**: Glow intensifies, stats popup appears (coin symbol, 24h change)
- **Scroll effect**: Camera tilts slightly, particle speed increases (momentum builds)
- **Click hero**: Scroll to dashboard demo or feature list
- **Mobile**: Auto-play animations, disable hover states

### Color Animation
- Every 8 seconds: Data stream color cycles: cyan→green→purple→cyan
- Neural network nodes pulse in sync with color shifts
- Coin glow color matches current data stream color

---

## 4. TECHNICAL BEST PRACTICES

### Performance Optimization
1. **Geometry**: Reuse geometries (instanced rendering). NOT: Create new geometry per frame.
2. **Textures**: Use GPU-compressed formats (WebP, ASTC). Limit to 2-3 textures max.
3. **Shaders**: Keep vertex/fragment shaders simple. Offload complex math to WebWorkers.
4. **Caching**: Pre-calculate all animations; use `animationMixer` for timeline-based motion.

### Loading Strategy
1. **Preload**: Coin models + textures (50KB total) before canvas creation
2. **Progressive Enhancement**: Show gradient background immediately, load 3D when ready
3. **Lazy Load**: Don't load 3D on mobile unless user scrolls past header (saves 2-3s TTL)

### Fallback Chain
```javascript
if (WebGL not supported) {
  Use CSS gradient + SVG animated background
  Disable all hover interactions
  Show simplified static header
}

if (Mobile) {
  Reduce particles by 60%
  Disable bloom post-processing
  Use 2D sprites for coins instead of 3D models
}

if (Low-end device) {
  Pause animations on scroll (requestIdleCallback)
  Show frame rate toggle (user can reduce quality)
}
```

### Code Organization
```
src/components/
├── HeroSection.tsx (wrapper, responsive canvas)
├── 3d/
│   ├── ThreeScene.ts (canvas + renderer setup)
│   ├── NeuralNetworkBackground.ts (nodes + lines)
│   ├── FloatingCoins.ts (coin models + animation)
│   ├── DataStreamParticles.ts (particle system)
│   ├── shaders/
│   │   ├── coin.vert/frag (rotation + glow)
│   │   ├── particle.vert/frag (data flow)
│   │   └── neural.vert/frag (node pulsing)
│   └── utils/
│       ├── geometryUtils.ts
│       └── animationUtils.ts
```

---

## 5. IMMEDIATE ACTION ITEMS

**Phase 1 (Week 1)**: Setup + Neural Network
- [ ] Install `three`, `@react-three/fiber`, `@react-three/postprocessing`
- [ ] Create responsive canvas container with error boundaries
- [ ] Implement neural network background (debug with visual nodes)
- [ ] Add performance monitor (show FPS + draw calls)

**Phase 2 (Week 1-2)**: Animations
- [ ] Add floating coin models (or create simple box geometry)
- [ ] Implement data stream particles (bezier curve flowing)
- [ ] Add glow/bloom post-processing
- [ ] Optimize for <50ms first paint

**Phase 3 (Week 2)**: Polish + Mobile
- [ ] Mobile responsive canvas scaling
- [ ] WebGL fallback (gradient background)
- [ ] Touch gesture handling (disable on scroll)
- [ ] Test on iPhone 12 (frame rate target: 30fps minimum)

**Phase 4 (Week 3)**: Integration
- [ ] Connect to real market data (optional: live BTC price in particle color)
- [ ] Add hero CTA button (scroll to dashboard)
- [ ] Accessibility: Add `aria-label` + keyboard fallback
- [ ] Measure Core Web Vitals (LCP <2.5s, CLS <0.1)

---

## Sources
- [Orizon 3D Crypto Landing Page](https://discourse.threejs.org/t/orizon-3d-crypto-landing-page-with-mesh-lines/66530)
- [Interactive 3D Hero Animation](https://www.matsimon.dev/blog/building-an-interactive-3d-hero-animation)
- [Three.js Neural Network Visualizer](https://github.com/marcusbuffett/Three.js-Neural-Network-Visualizer)
- [TensorSpace.js Framework](https://github.com/tensorspace-team/tensorspace)
- [Responsive WebGL Best Practices](https://blog.pixelfreestudio.com/how-to-create-responsive-3d-web-experiences-with-webgl/)
- [MDN WebGL Best Practices](https://developer.mozilla.org/en-US/docs/Web/API/WebGL_API/WebGL_best_practices)
- [Particle System Optimization](https://docs.quarks.art/docs/advanced-features/optimization)
- [WebGL Mobile Development](https://blog.pixelfreestudio.com/webgl-in-mobile-development-challenges-and-solutions/)

**Report**: `/Users/dungngo97/Documents/bot-core/plans/reports/researcher-251211-3d-hero-design.md`
