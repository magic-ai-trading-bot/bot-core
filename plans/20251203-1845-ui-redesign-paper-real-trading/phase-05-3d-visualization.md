# Phase 5: 3D Visualization (Optional Enhancement)

**Status**: Pending | **Priority**: P2 (Optional) | **Est. Time**: 2-3 days

---

## Context

- [Main Plan](./plan.md)
- [3D Visualization Research](./research/researcher-02-3d-visualization-design.md)

## Overview

Add optional 3D visualizations using Three.js + React Three Fiber: interactive portfolio pie chart, performance timeline, and hero section elements. Focus on <10k data points for optimal performance.

## Key Insights

1. **React Three Fiber for React Teams** - Component-based 3D, hot reloading, easier learning curve
2. **<10k Points Threshold** - Standard mesh approach; use InstancedMesh for 100k+
3. **WebGL Performance** - 60fps target with proper optimization
4. **Progressive Enhancement** - 3D is enhancement, 2D fallback required

## Requirements

### Functional
- [ ] 3D Portfolio pie chart (interactive, rotatable)
- [ ] 3D Performance timeline (line chart with depth)
- [ ] Hero section 3D elements (optional)
- [ ] Fallback to 2D charts on low-end devices

### Non-Functional
- [ ] 60fps on modern hardware
- [ ] <3s initial load time
- [ ] <50MB GPU memory usage
- [ ] Works without WebGL (fallback)

## Architecture

### 3D Components Structure

```
src/components/3d/
├── Portfolio3D.tsx           # Interactive pie chart
├── PerformanceTimeline3D.tsx # 3D line chart
├── Hero3DScene.tsx           # Landing page 3D
├── shared/
│   ├── Canvas3D.tsx          # Wrapper with error boundary
│   ├── controls.tsx          # Camera controls
│   └── lighting.tsx          # Standard lighting setup
└── hooks/
    ├── use3DEnabled.ts       # WebGL detection
    └── usePortfolioData.ts   # Data transformer for 3D
```

### Technology Stack

```bash
# Install dependencies
npm install three @react-three/fiber @react-three/drei

# Optional for advanced effects
npm install @react-three/postprocessing
```

### Performance Optimization Strategy

| Data Points | Approach | Expected FPS |
|-------------|----------|--------------|
| <1,000 | Standard Mesh | 60fps |
| 1,000-10,000 | Optimized Geometry | 45-60fps |
| 10,000-100,000 | InstancedMesh | 30-60fps |
| >100,000 | WebGPU or 2D fallback | Varies |

## Related Files

| File | Path | Action |
|------|------|--------|
| Portfolio3D | `/nextjs-ui-dashboard/src/components/3d/Portfolio3D.tsx` | Create |
| PerformanceTimeline3D | `/nextjs-ui-dashboard/src/components/3d/PerformanceTimeline3D.tsx` | Create |
| Canvas3D | `/nextjs-ui-dashboard/src/components/3d/shared/Canvas3D.tsx` | Create |
| use3DEnabled | `/nextjs-ui-dashboard/src/hooks/use3DEnabled.ts` | Create |

## Implementation Steps

### Step 1: Create Canvas3D Wrapper

```typescript
// src/components/3d/shared/Canvas3D.tsx
import { Canvas } from '@react-three/fiber'
import { OrbitControls, Environment, PerspectiveCamera } from '@react-three/drei'
import { Suspense } from 'react'
import { ErrorBoundary } from '@/components/ErrorBoundary'

interface Props {
  children: React.ReactNode
  className?: string
  enableControls?: boolean
  cameraPosition?: [number, number, number]
  fallback?: React.ReactNode
}

export function Canvas3D({
  children,
  className,
  enableControls = true,
  cameraPosition = [0, 0, 5],
  fallback
}: Props) {
  return (
    <ErrorBoundary fallback={fallback}>
      <Canvas
        className={className}
        gl={{ antialias: true, alpha: true }}
        dpr={[1, 2]} // Device pixel ratio
      >
        <PerspectiveCamera makeDefault position={cameraPosition} />
        <ambientLight intensity={0.5} />
        <directionalLight position={[10, 10, 5]} intensity={1} />

        <Suspense fallback={null}>
          {children}
        </Suspense>

        {enableControls && (
          <OrbitControls
            enableZoom={false}
            enablePan={false}
            autoRotate
            autoRotateSpeed={0.5}
          />
        )}
      </Canvas>
    </ErrorBoundary>
  )
}
```

### Step 2: Create Portfolio3D Component

```typescript
// src/components/3d/Portfolio3D.tsx
import { useRef, useMemo } from 'react'
import { useFrame } from '@react-three/fiber'
import { Text, RoundedBox } from '@react-three/drei'
import * as THREE from 'three'
import { Canvas3D } from './shared/Canvas3D'
import { PortfolioStats } from '@/components/trading/shared/PortfolioStats'

interface Asset {
  symbol: string
  value: number
  percentage: number
  pnl: number
}

interface Props {
  assets: Asset[]
  totalValue: number
}

// 3D Pie slice component
function PieSlice({
  startAngle,
  endAngle,
  color,
  label,
  percentage,
  radius = 2,
  height = 0.3
}: {
  startAngle: number
  endAngle: number
  color: string
  label: string
  percentage: number
  radius?: number
  height?: number
}) {
  const meshRef = useRef<THREE.Mesh>(null)

  const geometry = useMemo(() => {
    const shape = new THREE.Shape()
    shape.moveTo(0, 0)
    shape.arc(0, 0, radius, startAngle, endAngle, false)
    shape.lineTo(0, 0)

    return new THREE.ExtrudeGeometry(shape, {
      depth: height,
      bevelEnabled: true,
      bevelThickness: 0.02,
      bevelSize: 0.02,
    })
  }, [startAngle, endAngle, radius, height])

  // Hover animation
  const [hovered, setHovered] = useState(false)

  useFrame(() => {
    if (meshRef.current) {
      const targetScale = hovered ? 1.05 : 1
      meshRef.current.scale.lerp(
        new THREE.Vector3(targetScale, targetScale, targetScale),
        0.1
      )
    }
  })

  // Label position at center of slice
  const midAngle = (startAngle + endAngle) / 2
  const labelRadius = radius * 0.6
  const labelPosition: [number, number, number] = [
    Math.cos(midAngle) * labelRadius,
    Math.sin(midAngle) * labelRadius,
    height + 0.1
  ]

  return (
    <group>
      <mesh
        ref={meshRef}
        geometry={geometry}
        rotation={[-Math.PI / 2, 0, 0]}
        onPointerOver={() => setHovered(true)}
        onPointerOut={() => setHovered(false)}
      >
        <meshStandardMaterial color={color} />
      </mesh>

      {percentage > 5 && (
        <Text
          position={labelPosition}
          fontSize={0.15}
          color="white"
          anchorX="center"
          anchorY="middle"
        >
          {label}
          {'\n'}
          {percentage.toFixed(1)}%
        </Text>
      )}
    </group>
  )
}

export function Portfolio3D({ assets, totalValue }: Props) {
  const colors = ['#10B981', '#0EA5E9', '#F59E0B', '#EF4444', '#8B5CF6', '#EC4899']

  const slices = useMemo(() => {
    let currentAngle = 0
    return assets.map((asset, index) => {
      const startAngle = currentAngle
      const sliceAngle = (asset.percentage / 100) * Math.PI * 2
      currentAngle += sliceAngle

      return {
        ...asset,
        startAngle,
        endAngle: currentAngle,
        color: colors[index % colors.length]
      }
    })
  }, [assets])

  return (
    <div className="relative h-[300px]">
      <Canvas3D cameraPosition={[0, 3, 4]}>
        <group rotation={[0.2, 0, 0]}>
          {slices.map((slice, index) => (
            <PieSlice
              key={slice.symbol}
              startAngle={slice.startAngle}
              endAngle={slice.endAngle}
              color={slice.color}
              label={slice.symbol}
              percentage={slice.percentage}
            />
          ))}
        </group>
      </Canvas3D>

      {/* Center value overlay */}
      <div className="absolute inset-0 flex items-center justify-center pointer-events-none">
        <div className="text-center">
          <p className="text-sm text-slate-400">Total Value</p>
          <p className="text-2xl font-bold">${totalValue.toLocaleString()}</p>
        </div>
      </div>
    </div>
  )
}
```

### Step 3: Create use3DEnabled Hook

```typescript
// src/hooks/use3DEnabled.ts
import { useState, useEffect } from 'react'

export function use3DEnabled(): boolean {
  const [enabled, setEnabled] = useState(true)

  useEffect(() => {
    // Check WebGL support
    const canvas = document.createElement('canvas')
    const gl = canvas.getContext('webgl') || canvas.getContext('experimental-webgl')

    if (!gl) {
      setEnabled(false)
      return
    }

    // Check for low-end device
    const renderer = (gl as WebGLRenderingContext).getParameter(
      (gl as WebGLRenderingContext).RENDERER
    )

    // Disable on known low-performance GPUs
    const lowEndGPUs = ['Mali-4', 'Adreno 3', 'PowerVR SGX']
    if (lowEndGPUs.some(gpu => renderer.includes(gpu))) {
      setEnabled(false)
      return
    }

    // Check user preference
    const prefersReduced = window.matchMedia('(prefers-reduced-motion: reduce)').matches
    if (prefersReduced) {
      setEnabled(false)
    }
  }, [])

  return enabled
}
```

### Step 4: Create PerformanceTimeline3D

```typescript
// src/components/3d/PerformanceTimeline3D.tsx
import { useMemo, useRef } from 'react'
import { useFrame } from '@react-three/fiber'
import { Line } from '@react-three/drei'
import * as THREE from 'three'
import { Canvas3D } from './shared/Canvas3D'

interface DataPoint {
  timestamp: Date
  value: number
}

interface Props {
  data: DataPoint[]
  height?: number
  width?: number
}

export function PerformanceTimeline3D({ data, height = 2, width = 4 }: Props) {
  const lineRef = useRef<THREE.Line>(null)

  const points = useMemo(() => {
    if (data.length === 0) return []

    const minValue = Math.min(...data.map(d => d.value))
    const maxValue = Math.max(...data.map(d => d.value))
    const range = maxValue - minValue || 1

    return data.map((point, index) => {
      const x = (index / (data.length - 1)) * width - width / 2
      const y = ((point.value - minValue) / range) * height - height / 2
      const z = 0
      return new THREE.Vector3(x, y, z)
    })
  }, [data, height, width])

  // Subtle animation
  useFrame(({ clock }) => {
    if (lineRef.current) {
      lineRef.current.rotation.y = Math.sin(clock.elapsedTime * 0.2) * 0.1
    }
  })

  const isProfitable = data.length > 1 && data[data.length - 1].value >= data[0].value

  return (
    <Canvas3D cameraPosition={[0, 0, 5]} enableControls={false}>
      <group ref={lineRef}>
        <Line
          points={points}
          color={isProfitable ? '#10B981' : '#EF4444'}
          lineWidth={3}
        />

        {/* Grid lines */}
        <gridHelper
          args={[width, 10, '#1E293B', '#1E293B']}
          rotation={[Math.PI / 2, 0, 0]}
          position={[0, -height / 2, -0.1]}
        />
      </group>
    </Canvas3D>
  )
}
```

### Step 5: Integrate with Portfolio Overview

```typescript
// Update PortfolioOverview.tsx to use 3D when available
import { use3DEnabled } from '@/hooks/use3DEnabled'
import { Portfolio3D } from '@/components/3d/Portfolio3D'
import { PieChart } from '@/components/charts/PieChart' // 2D fallback

export function PortfolioOverview({ assets, totalValue, ...props }) {
  const is3DEnabled = use3DEnabled()

  return (
    <div>
      {is3DEnabled ? (
        <Portfolio3D assets={assets} totalValue={totalValue} />
      ) : (
        <PieChart assets={assets} totalValue={totalValue} />
      )}
      {/* Rest of portfolio overview */}
    </div>
  )
}
```

## Todo List

- [ ] Install three, @react-three/fiber, @react-three/drei
- [ ] Create Canvas3D wrapper component
- [ ] Create Portfolio3D pie chart
- [ ] Create PerformanceTimeline3D line chart
- [ ] Create use3DEnabled hook with WebGL detection
- [ ] Implement 2D fallback for each 3D component
- [ ] Add hover interactions to 3D elements
- [ ] Optimize for 60fps performance
- [ ] Test on mobile devices
- [ ] Add loading states for 3D scenes
- [ ] Test reduced motion preference
- [ ] Profile GPU memory usage

## Success Criteria

1. 3D renders at 60fps on modern browsers
2. Graceful fallback on unsupported devices
3. Interactive hover/click on 3D elements
4. <3s load time for 3D scenes
5. Reduced motion preference respected

## Risk Assessment

| Risk | Probability | Impact | Mitigation |
|------|-------------|--------|------------|
| WebGL not supported | Low | Medium | 2D fallback |
| Performance issues | Medium | Medium | Detection + disable |
| Mobile GPU limits | Medium | Low | Simpler mobile scenes |
| Bundle size increase | Medium | Low | Dynamic import |

## Security Considerations

- No security concerns for visualization
- WebGL context isolated from main thread
- No user data exposed in 3D rendering

## Performance Optimization Tips

```typescript
// 1. Memoize geometry
const geometry = useMemo(() => new THREE.BoxGeometry(1, 1, 1), [])

// 2. Dispose properly
useEffect(() => {
  return () => {
    geometry.dispose()
    material.dispose()
  }
}, [])

// 3. Use instancing for many objects
<instancedMesh args={[geometry, material, count]}>
  {/* Instance matrix updates */}
</instancedMesh>

// 4. Limit draw calls
// Merge meshes when possible

// 5. Use dynamic import
const Portfolio3D = lazy(() => import('./3d/Portfolio3D'))
```

## Next Steps

After Phase 5 completion (or skip if time-constrained), proceed to [Phase 6: Polish & Testing](./phase-06-polish-testing.md)
