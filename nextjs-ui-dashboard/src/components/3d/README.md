# 3D Components Documentation

Premium Three.js/React Three Fiber components for enhanced visual experience.

## Installation

Dependencies already installed:
- `three`
- `@react-three/fiber`
- `@react-three/drei`

## Components Overview

### Canvas3D
Shared React Three Fiber Canvas wrapper with performance monitoring and common settings.

```tsx
import { Canvas3D, FloatingCoins } from '@/components/3d';

<Canvas3D camera={{ position: [0, 0, 5], fov: 75 }} shadows>
  <FloatingCoins />
</Canvas3D>
```

**Features:**
- Automatic DPR adjustment based on FPS
- Performance monitoring (targets 60fps)
- Tab visibility pause
- Built-in lighting setup
- Shadow support

**Props:**
- `camera` - Camera configuration
- `shadows` - Enable/disable shadows
- `className` - CSS classes

---

### FloatingCoins
Animated cryptocurrency coins (BTC, ETH, SOL) with hover effects.

```tsx
import { FloatingCoins } from '@/components/3d';

<FloatingCoins
  coins={[
    { symbol: 'BTC', position: [-2, 0, 0] },
    { symbol: 'ETH', position: [0, 0, 0] },
    { symbol: 'SOL', position: [2, 0, 0] },
  ]}
/>
```

**Features:**
- Float animation (sin wave)
- Rotation animation
- Hover highlight with scale
- Metallic material with emissive glow
- Glow ring on hover

**Props:**
- `coins` - Array of coin configs (optional, defaults to BTC/ETH/SOL)

---

### GlowOrb
Shader-based glowing orb with pulse animation and mode-aware colors.

```tsx
import { GlowOrb } from '@/components/3d';

<GlowOrb
  position={[0, 0, 0]}
  mode="profit" // 'profit' | 'loss' | 'neutral'
  pulseSpeed={1}
  size={1}
/>
```

**Features:**
- Custom shader for fresnel glow effect
- Pulse animation
- Color changes based on mode:
  - Profit: Blue (#3b82f6)
  - Loss: Red (#ef4444)
  - Neutral: Purple (#8b5cf6)
- Additive blending for glow

**Props:**
- `position` - 3D position [x, y, z]
- `color` - Override color (optional)
- `mode` - Profit/loss/neutral
- `pulseSpeed` - Animation speed
- `size` - Orb radius

---

### ParticleField
Star field background with 1000-5000 particles and slow drift.

```tsx
import { ParticleField } from '@/components/3d';

<ParticleField
  count={3000}
  size={0.02}
  spread={50}
  color="#ffffff"
  speed={0.1}
/>
```

**Features:**
- Random particle positions in sphere
- Slow drift animation
- Wrap-around when particles go too far
- Size variation for depth
- Additive blending

**Props:**
- `count` - Number of particles (1000-5000)
- `size` - Particle size
- `spread` - Distribution radius
- `color` - Particle color
- `speed` - Drift speed

---

### PortfolioGlobe
Earth sphere with portfolio position markers and connection lines.

```tsx
import { PortfolioGlobe } from '@/components/3d';

<PortfolioGlobe
  positions={[
    { lat: 40.7128, lon: -74.0060, pnl: 1500, symbol: 'BTC' }, // NYC
    { lat: 51.5074, lon: -0.1278, pnl: -500, symbol: 'ETH' },  // London
  ]}
  radius={2}
  autoRotate
/>
```

**Features:**
- Earth-like texture (procedural gradient)
- 3D pin markers at lat/lon
- Curved connection lines between positions
- Atmosphere glow effect
- Drag rotation (OrbitControls)
- Auto-rotate when not dragging

**Props:**
- `positions` - Array of { lat, lon, pnl, symbol }
- `radius` - Globe radius
- `autoRotate` - Enable auto-rotation

---

### GlobeMarkers
3D markers for positions on globe, color-coded by PnL.

```tsx
import { GlobeMarkers } from '@/components/3d';

<GlobeMarkers
  markers={[
    { lat: 40.7128, lon: -74.0060, pnl: 1500, symbol: 'BTC' },
  ]}
  radius={2}
/>
```

**Features:**
- Pin-style markers (cone + sphere)
- Green for profit, red for loss
- Pulse animation
- Intensity based on PnL amount
- Glow ring at base

**Props:**
- `markers` - Array of { lat, lon, pnl, symbol }
- `radius` - Globe radius (match parent)

---

### PriceTicker3D
3D carousel of cryptocurrency prices with color-coded changes.

```tsx
import { PriceTicker3D } from '@/components/3d';

<PriceTicker3D
  coins={[
    { symbol: 'BTC', price: 45000, change24h: 2.5 },
    { symbol: 'ETH', price: 3000, change24h: -1.2 },
    { symbol: 'SOL', price: 100, change24h: 5.8 },
  ]}
  radius={4}
  rotationSpeed={0.2}
/>
```

**Features:**
- Rotating carousel layout
- 3D coin meshes
- Price and change labels
- Glow ring based on volatility
- Color-coded by coin (BTC=gold, ETH=blue, SOL=green)
- Change color (green=up, red=down)

**Props:**
- `coins` - Array of { symbol, price, change24h }
- `radius` - Carousel radius
- `rotationSpeed` - Rotation speed

---

### MarketDepth3D
3D bar chart visualization of order book depth.

```tsx
import { MarketDepth3D } from '@/components/3d';

<MarketDepth3D
  bids={[
    { price: 44900, amount: 0.5, total: 5 },
    { price: 44800, amount: 1.2, total: 6.2 },
  ]}
  asks={[
    { price: 45100, amount: 0.8, total: 4 },
    { price: 45200, amount: 1.5, total: 5.5 },
  ]}
  maxBars={20}
/>
```

**Features:**
- 3D bars for bids (green) and asks (red)
- Height represents cumulative volume
- Top glow on each bar
- Price labels every 5th bar
- OrbitControls for camera
- Center dividing line

**Props:**
- `bids` - Array of { price, amount, total }
- `asks` - Array of { price, amount, total }
- `maxBars` - Max bars per side (default 20)

---

### Fallback2D
Static gradient background with CSS-only animations for low-end devices.

```tsx
import { Fallback2D } from '@/components/3d';

<Fallback2D variant="gradient" />
// or
<Fallback2D variant="particles" />
// or
<Fallback2D variant="waves" />
```

**Features:**
- CSS-only animations (no JS)
- Three variants:
  - `gradient` - Animated gradient orbs
  - `particles` - Pulsing dots
  - `waves` - Moving wave patterns
- Grid overlay
- Vignette effect

**Props:**
- `variant` - Animation style
- `className` - CSS classes

---

### useDeviceCapability Hook
Detect device capabilities for 3D rendering.

```tsx
import { useDeviceCapability, useRecommendedSettings } from '@/hooks/useDeviceCapability';

const capability = useDeviceCapability();
// {
//   supports3D: true,
//   gpuTier: 'high',
//   isMobile: false,
//   isTablet: false,
//   webglVersion: 2,
//   maxTextureSize: 16384,
//   supportsWebGPU: false,
// }

const settings = useRecommendedSettings();
// {
//   particleCount: 5000,
//   enableShadows: true,
//   enableAA: true,
//   pixelRatio: 2,
//   enablePostProcessing: true,
//   animationComplexity: 'full',
//   useFallback: false,
// }
```

**Returns:**
- `supports3D` - WebGL available
- `gpuTier` - 'low' | 'medium' | 'high'
- `isMobile` - Mobile device
- `isTablet` - Tablet device
- `webglVersion` - 1 | 2 | null
- `maxTextureSize` - Max texture size
- `supportsWebGPU` - WebGPU available

---

## Complete Example

```tsx
'use client';

import { Suspense } from 'react';
import {
  Canvas3D,
  FloatingCoins,
  GlowOrb,
  ParticleField,
  Fallback2D,
} from '@/components/3d';
import { useDeviceCapability } from '@/hooks/useDeviceCapability';

export default function HeroSection() {
  const { supports3D, gpuTier } = useDeviceCapability();

  if (!supports3D) {
    return <Fallback2D variant="gradient" />;
  }

  return (
    <div className="relative h-screen w-full">
      <Suspense fallback={<Fallback2D variant="gradient" />}>
        <Canvas3D camera={{ position: [0, 0, 8], fov: 75 }} shadows>
          {/* Background particles */}
          <ParticleField
            count={gpuTier === 'high' ? 5000 : 3000}
            spread={50}
            speed={0.1}
          />

          {/* Floating coins */}
          <FloatingCoins />

          {/* Center orb */}
          <GlowOrb position={[0, 0, 0]} mode="profit" size={1} />
        </Canvas3D>
      </Suspense>

      {/* Overlay content */}
      <div className="absolute inset-0 flex items-center justify-center">
        <h1 className="text-6xl font-bold text-white">
          Premium 3D Experience
        </h1>
      </div>
    </div>
  );
}
```

## Performance Tips

1. **Lazy Load**: Use `React.lazy()` for 3D components
2. **Suspense**: Wrap in `<Suspense>` with fallback
3. **Device Detection**: Use `useDeviceCapability()` to adjust quality
4. **Instancing**: Use for repeated objects (particles)
5. **LOD**: Use Level of Detail for complex meshes
6. **Pause**: Components automatically pause when tab hidden

## Bundle Size

- Three.js: ~600KB (lazy loaded)
- React Three Fiber: ~100KB
- Drei: ~200KB
- **Total: ~900KB** (only loaded when 3D components used)

## Browser Support

- ✅ Chrome 90+
- ✅ Firefox 88+
- ✅ Safari 15+
- ✅ Edge 90+
- ❌ IE (not supported)

Falls back to `Fallback2D` on older browsers.

---

## Troubleshooting

### "WebGL not supported"
- Use `Fallback2D` component
- Check `useDeviceCapability().supports3D`

### Low FPS
- Reduce particle count
- Disable shadows
- Lower `pixelRatio`
- Use `useRecommendedSettings()`

### Black screen
- Check console for errors
- Verify Three.js installed
- Check camera position
- Add lighting

### Text not rendering
- Install `@react-three/drei`
- Check font loading
- Use `<Text>` from drei
