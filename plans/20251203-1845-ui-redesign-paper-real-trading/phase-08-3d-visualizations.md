# Phase 8: 3D Visualizations

## Context
- **Parent Plan**: [plan.md](./plan.md)
- **Dependencies**: Phase 1-7
- **Research**: [3D Visualization](./research/researcher-02-3d-visualization-design.md)

## Overview
| Field | Value |
|-------|-------|
| Priority | P2 - Medium |
| Status | Pending |
| Est. Time | 3-4 days |
| Description | Premium 3D elements using Three.js/React Three Fiber for landing page and portfolio visualization |

## Key Insights
- 3D enhances premium feel but must not hurt performance
- Progressive enhancement: 2D fallback for low-end devices
- Key areas: Hero section, portfolio globe, price ticker 3D
- Use cases: Landing hero, portfolio distribution, market visualization

## Requirements

### Functional
- 3D hero scene (landing page)
- Portfolio globe/sphere visualization
- 3D price ticker (floating coins)
- Interactive market depth 3D chart
- Performance metrics 3D visualization

### Non-Functional
- 60fps on mid-range devices
- < 500KB additional bundle (lazy loaded)
- Graceful degradation to 2D
- Touch/mouse interaction support

## Architecture

```
3DComponents/
├── Hero3DScene/
│   ├── FloatingCoins.tsx
│   ├── GlowOrb.tsx
│   ├── ParticleField.tsx
│   └── CameraController.tsx
│
├── PortfolioGlobe/
│   ├── GlobeScene.tsx
│   ├── GlobeMarkers.tsx
│   ├── ConnectionLines.tsx
│   └── GlobeTooltip.tsx
│
├── PriceTicker3D/
│   ├── CoinMesh.tsx
│   ├── PriceLabel.tsx
│   └── TickerAnimation.tsx
│
├── MarketDepth3D/
│   ├── DepthBars.tsx
│   ├── DepthSurface.tsx
│   └── DepthCamera.tsx
│
└── Shared/
    ├── Canvas3D.tsx (R3F wrapper)
    ├── PerformanceMonitor.tsx
    ├── Fallback2D.tsx
    └── useDeviceCapability.ts
```

## Related Code Files

### Create
- `/Users/dungngo97/Documents/bot-core/nextjs-ui-dashboard/src/components/3d/Canvas3D.tsx`
- `/Users/dungngo97/Documents/bot-core/nextjs-ui-dashboard/src/components/3d/Hero3DScene.tsx`
- `/Users/dungngo97/Documents/bot-core/nextjs-ui-dashboard/src/components/3d/FloatingCoins.tsx`
- `/Users/dungngo97/Documents/bot-core/nextjs-ui-dashboard/src/components/3d/GlowOrb.tsx`
- `/Users/dungngo97/Documents/bot-core/nextjs-ui-dashboard/src/components/3d/ParticleField.tsx`
- `/Users/dungngo97/Documents/bot-core/nextjs-ui-dashboard/src/components/3d/PortfolioGlobe.tsx`
- `/Users/dungngo97/Documents/bot-core/nextjs-ui-dashboard/src/components/3d/GlobeMarkers.tsx`
- `/Users/dungngo97/Documents/bot-core/nextjs-ui-dashboard/src/components/3d/PriceTicker3D.tsx`
- `/Users/dungngo97/Documents/bot-core/nextjs-ui-dashboard/src/components/3d/MarketDepth3D.tsx`
- `/Users/dungngo97/Documents/bot-core/nextjs-ui-dashboard/src/components/3d/Fallback2D.tsx`
- `/Users/dungngo97/Documents/bot-core/nextjs-ui-dashboard/src/hooks/useDeviceCapability.ts`

### Modify
- Landing page to include Hero3DScene
- Portfolio page to include PortfolioGlobe (optional)
- Dashboard to include PriceTicker3D (optional)

## Implementation Steps

1. **Create Canvas3D Wrapper**
   ```tsx
   // Shared R3F Canvas with common settings
   // Performance monitoring
   // Resize handling
   // Touch/mouse controls
   ```

2. **Create useDeviceCapability Hook**
   ```tsx
   // Detect WebGL support
   // Detect GPU tier (low/mid/high)
   // Detect mobile/desktop
   // Return { supports3D, gpuTier, isMobile }
   ```

3. **Create Hero3DScene**
   - Floating crypto coin meshes
   - Animated glow orb (center)
   - Particle field background
   - Mouse parallax effect
   - Auto-rotate when idle

4. **Create FloatingCoins**
   ```tsx
   // BTC, ETH, SOL coin textures
   // Float animation (sin wave)
   // Rotation animation
   // Hover highlight
   ```

5. **Create GlowOrb**
   - Shader-based glow effect
   - Pulse animation
   - Color matches mode (blue/red)

6. **Create PortfolioGlobe**
   - Earth sphere with custom texture
   - Portfolio markers as 3D pins
   - Connection lines between positions
   - Rotate on drag
   - Tooltip on marker hover

7. **Create PriceTicker3D**
   - 3D coins with price labels
   - Scroll/carousel animation
   - Green/red glow based on change
   - Click to navigate

8. **Create MarketDepth3D**
   - 3D bar chart for order book
   - Bid/ask sides
   - Animated updates
   - Camera orbit controls

9. **Create Fallback2D**
   - Static gradient background
   - 2D animated elements (CSS)
   - Same visual theme, no WebGL

10. **Performance Optimization**
    - Lazy load Three.js bundle
    - Use instancing for repeated objects
    - LOD (Level of Detail) for complex meshes
    - Frame rate limiter for battery

## Todo List

- [ ] Install Three.js and React Three Fiber
- [ ] Create Canvas3D wrapper component
- [ ] Create useDeviceCapability hook
- [ ] Create Hero3DScene for landing
- [ ] Create FloatingCoins component
- [ ] Create GlowOrb with shader
- [ ] Create ParticleField background
- [ ] Create PortfolioGlobe
- [ ] Create PriceTicker3D
- [ ] Create MarketDepth3D
- [ ] Create Fallback2D components
- [ ] Integrate into landing page
- [ ] Optimize bundle size (< 500KB)
- [ ] Test on low-end devices
- [ ] Add touch controls
- [ ] Performance profiling
- [ ] Write component tests

## Success Criteria

- [ ] Hero 3D renders at 60fps on mid-range devices
- [ ] Fallback activates on low-end devices
- [ ] Three.js bundle lazy loaded
- [ ] Touch controls work on mobile
- [ ] No visual glitches on resize
- [ ] Battery-conscious (reduce fps when tab hidden)

## Risk Assessment

| Risk | Impact | Mitigation |
|------|--------|------------|
| Performance on mobile | High | Fallback2D, reduced complexity |
| Large bundle size | Medium | Lazy load, code split |
| WebGL crashes | Medium | Error boundary, fallback |
| Battery drain | Medium | Pause when not visible |

## Security Considerations
- No user data in 3D visualizations
- Textures from local assets only
- No external 3D model loading

## Technical References

### React Three Fiber Setup
```tsx
import { Canvas } from '@react-three/fiber';
import { OrbitControls, Environment } from '@react-three/drei';

function Scene() {
  return (
    <Canvas camera={{ position: [0, 0, 5] }}>
      <ambientLight intensity={0.5} />
      <pointLight position={[10, 10, 10]} />
      <FloatingCoins />
      <GlowOrb />
      <OrbitControls enableZoom={false} />
    </Canvas>
  );
}
```

### Floating Animation
```tsx
import { useFrame } from '@react-three/fiber';

function FloatingCoin({ position }) {
  const meshRef = useRef();

  useFrame(({ clock }) => {
    meshRef.current.position.y = position[1] + Math.sin(clock.elapsedTime) * 0.2;
    meshRef.current.rotation.y += 0.01;
  });

  return <mesh ref={meshRef} position={position}>...</mesh>;
}
```

## Next Steps
→ Phase 9: Polish & Testing
