'use client';

/**
 * HeroScene3D - Futuristic AI Trading Core Visualization
 *
 * Creates an impressive, high-tech AI trading visualization with:
 * - Central AI Core with pulsing energy
 * - Orbiting data rings with trading metrics
 * - Holographic floating panels
 * - Energy streams connecting elements
 * - Dynamic particle systems
 * - Futuristic grid floor
 */

import { useRef, useMemo, useState, useEffect } from 'react';
import { Canvas, useFrame, useThree } from '@react-three/fiber';
import * as THREE from 'three';
import { useTheme } from '@/contexts/ThemeContext';

// ============================================
// AI Core - Central glowing sphere with energy
// ============================================
function AICore({ color, secondaryColor }: { color: string; secondaryColor: string }) {
  const coreRef = useRef<THREE.Group>(null);
  const innerRef = useRef<THREE.Mesh>(null);
  const outerRef = useRef<THREE.Mesh>(null);
  const ringsRef = useRef<THREE.Group>(null);

  useFrame(({ clock }) => {
    if (!coreRef.current || !innerRef.current || !outerRef.current || !ringsRef.current) return;
    const t = clock.elapsedTime;

    // Rotate the entire core slowly
    coreRef.current.rotation.y = t * 0.2;

    // Pulse the inner core
    const pulse = Math.sin(t * 2) * 0.1 + 1;
    innerRef.current.scale.setScalar(pulse);

    // Counter-rotate outer shell
    outerRef.current.rotation.x = t * 0.3;
    outerRef.current.rotation.z = t * 0.2;

    // Rotate rings at different speeds
    ringsRef.current.children.forEach((ring, i) => {
      ring.rotation.x = t * (0.3 + i * 0.1);
      ring.rotation.y = t * (0.2 + i * 0.15);
    });
  });

  return (
    <group ref={coreRef} position={[0, 0, -5]}>
      {/* Inner glowing core */}
      <mesh ref={innerRef}>
        <icosahedronGeometry args={[1.2, 2]} />
        <meshBasicMaterial color={color} transparent opacity={0.9} />
      </mesh>

      {/* Outer wireframe shell */}
      <mesh ref={outerRef}>
        <icosahedronGeometry args={[1.8, 1]} />
        <meshBasicMaterial color={color} wireframe transparent opacity={0.4} />
      </mesh>

      {/* Energy rings */}
      <group ref={ringsRef}>
        {[2.2, 2.6, 3.0].map((radius, i) => (
          <mesh key={i} rotation={[Math.PI / 2 + i * 0.3, i * 0.5, 0]}>
            <torusGeometry args={[radius, 0.02, 8, 64]} />
            <meshBasicMaterial
              color={i % 2 === 0 ? color : secondaryColor}
              transparent
              opacity={0.6 - i * 0.1}
            />
          </mesh>
        ))}
      </group>

      {/* Glow effect */}
      <mesh>
        <sphereGeometry args={[2.5, 32, 32]} />
        <meshBasicMaterial
          color={color}
          transparent
          opacity={0.1}
          side={THREE.BackSide}
        />
      </mesh>
    </group>
  );
}

// ============================================
// Orbiting Data Ring - Trading metrics orbit
// ============================================
interface OrbitingRingProps {
  radius: number;
  color: string;
  speed: number;
  tilt: number;
  nodeCount: number;
}

function OrbitingRing({ radius, color, speed, tilt, nodeCount }: OrbitingRingProps) {
  const groupRef = useRef<THREE.Group>(null);
  const nodesRef = useRef<THREE.Group>(null);

  useFrame(({ clock }) => {
    if (!groupRef.current || !nodesRef.current) return;
    const t = clock.elapsedTime;
    groupRef.current.rotation.y = t * speed;

    // Pulse nodes
    nodesRef.current.children.forEach((node, i) => {
      const scale = 1 + Math.sin(t * 2 + i) * 0.2;
      node.scale.setScalar(scale);
    });
  });

  const nodes = useMemo(() => {
    const result: [number, number, number][] = [];
    for (let i = 0; i < nodeCount; i++) {
      const angle = (i / nodeCount) * Math.PI * 2;
      result.push([
        Math.cos(angle) * radius,
        0,
        Math.sin(angle) * radius,
      ]);
    }
    return result;
  }, [radius, nodeCount]);

  return (
    <group ref={groupRef} rotation={[tilt, 0, 0]} position={[0, 0, -5]}>
      {/* Ring path */}
      <mesh rotation={[Math.PI / 2, 0, 0]}>
        <torusGeometry args={[radius, 0.015, 8, 128]} />
        <meshBasicMaterial color={color} transparent opacity={0.3} />
      </mesh>

      {/* Orbiting nodes */}
      <group ref={nodesRef}>
        {nodes.map((pos, i) => (
          <mesh key={i} position={pos}>
            <octahedronGeometry args={[0.15, 0]} />
            <meshBasicMaterial color={color} transparent opacity={0.8} />
          </mesh>
        ))}
      </group>
    </group>
  );
}

// ============================================
// Holographic Panel - Floating data display
// ============================================
interface HoloPanelProps {
  position: [number, number, number];
  rotation: [number, number, number];
  width: number;
  height: number;
  color: string;
}

function HoloPanel({ position, rotation, width, height, color }: HoloPanelProps) {
  const panelRef = useRef<THREE.Group>(null);
  const linesRef = useRef<THREE.Group>(null);

  useFrame(({ clock }) => {
    if (!panelRef.current || !linesRef.current) return;
    const t = clock.elapsedTime;

    // Subtle floating animation
    panelRef.current.position.y = position[1] + Math.sin(t * 0.5 + position[0]) * 0.1;

    // Animate chart lines
    linesRef.current.children.forEach((line, i) => {
      const mesh = line as THREE.Mesh;
      const scale = 0.5 + Math.sin(t * 2 + i * 0.5) * 0.3;
      mesh.scale.y = scale;
    });
  });

  // Generate mini chart bars
  const bars = useMemo(() => {
    const result: { x: number; height: number; isGreen: boolean }[] = [];
    for (let i = 0; i < 8; i++) {
      result.push({
        x: -width / 2 + 0.15 + i * (width / 8),
        height: 0.3 + Math.random() * 0.4,
        isGreen: Math.random() > 0.4,
      });
    }
    return result;
  }, [width]);

  return (
    <group ref={panelRef} position={position} rotation={rotation}>
      {/* Panel background */}
      <mesh>
        <planeGeometry args={[width, height]} />
        <meshBasicMaterial color={color} transparent opacity={0.1} side={THREE.DoubleSide} />
      </mesh>

      {/* Panel border */}
      <lineSegments>
        <edgesGeometry args={[new THREE.PlaneGeometry(width, height)]} />
        <lineBasicMaterial color={color} transparent opacity={0.5} />
      </lineSegments>

      {/* Mini chart bars */}
      <group ref={linesRef} position={[0, -height / 4, 0.01]}>
        {bars.map((bar, i) => (
          <mesh key={i} position={[bar.x, bar.height / 2, 0]}>
            <boxGeometry args={[0.08, bar.height, 0.01]} />
            <meshBasicMaterial
              color={bar.isGreen ? '#22c55e' : '#ef4444'}
              transparent
              opacity={0.8}
            />
          </mesh>
        ))}
      </group>

      {/* Scan line effect */}
      <mesh position={[0, 0, 0.02]}>
        <planeGeometry args={[width, 0.02]} />
        <meshBasicMaterial color={color} transparent opacity={0.8} />
      </mesh>
    </group>
  );
}

// ============================================
// Energy Stream - Connecting lines with flow
// ============================================
interface EnergyStreamProps {
  start: [number, number, number];
  end: [number, number, number];
  color: string;
}

function EnergyStream({ start, end, color }: EnergyStreamProps) {
  const lineRef = useRef<THREE.Line>(null);
  const particlesRef = useRef<THREE.Points>(null);

  const curve = useMemo(() => {
    const midPoint = [
      (start[0] + end[0]) / 2,
      (start[1] + end[1]) / 2 + 2,
      (start[2] + end[2]) / 2,
    ];
    return new THREE.QuadraticBezierCurve3(
      new THREE.Vector3(...start),
      new THREE.Vector3(midPoint[0], midPoint[1], midPoint[2]),
      new THREE.Vector3(...end)
    );
  }, [start, end]);

  const points = useMemo(() => curve.getPoints(50), [curve]);

  useFrame(({ clock }) => {
    if (!lineRef.current) return;
    const t = clock.elapsedTime;
    const opacity = 0.3 + Math.sin(t * 3) * 0.2;
    (lineRef.current.material as THREE.LineBasicMaterial).opacity = opacity;
  });

  return (
    <group>
      <line ref={lineRef}>
        <bufferGeometry>
          <bufferAttribute
            attach="attributes-position"
            count={points.length}
            array={new Float32Array(points.flatMap(p => [p.x, p.y, p.z]))}
            itemSize={3}
          />
        </bufferGeometry>
        <lineBasicMaterial color={color} transparent opacity={0.4} />
      </line>
    </group>
  );
}

// ============================================
// Particle Field - Dynamic background particles
// ============================================
interface ParticleFieldProps {
  count: number;
  color: string;
  secondaryColor: string;
}

function ParticleField({ count, color, secondaryColor }: ParticleFieldProps) {
  const pointsRef = useRef<THREE.Points>(null);

  const { positions, colors, velocities, phases } = useMemo(() => {
    const positions = new Float32Array(count * 3);
    const colors = new Float32Array(count * 3);
    const velocities = new Float32Array(count * 3);
    const phases = new Float32Array(count);

    const primaryColor = new THREE.Color(color);
    const secondary = new THREE.Color(secondaryColor);

    for (let i = 0; i < count; i++) {
      // Spread in a sphere around the center
      const theta = Math.random() * Math.PI * 2;
      const phi = Math.acos(2 * Math.random() - 1);
      const r = 8 + Math.random() * 25;

      positions[i * 3] = r * Math.sin(phi) * Math.cos(theta);
      positions[i * 3 + 1] = r * Math.sin(phi) * Math.sin(theta) * 0.5; // Flatten vertically
      positions[i * 3 + 2] = r * Math.cos(phi) - 10;

      const c = Math.random() > 0.5 ? primaryColor : secondary;
      colors[i * 3] = c.r;
      colors[i * 3 + 1] = c.g;
      colors[i * 3 + 2] = c.b;

      velocities[i * 3] = (Math.random() - 0.5) * 0.01;
      velocities[i * 3 + 1] = (Math.random() - 0.5) * 0.01;
      velocities[i * 3 + 2] = (Math.random() - 0.5) * 0.01;

      phases[i] = Math.random() * Math.PI * 2;
    }

    return { positions, colors, velocities, phases };
  }, [count, color, secondaryColor]);

  useFrame(({ clock }) => {
    if (!pointsRef.current) return;
    const t = clock.elapsedTime;
    const posArray = pointsRef.current.geometry.attributes.position.array as Float32Array;

    for (let i = 0; i < count; i++) {
      // Gentle orbital motion
      const phase = phases[i];
      posArray[i * 3] += Math.sin(t * 0.2 + phase) * 0.01;
      posArray[i * 3 + 1] += Math.cos(t * 0.15 + phase) * 0.005;
      posArray[i * 3 + 2] += velocities[i * 3 + 2];
    }

    pointsRef.current.geometry.attributes.position.needsUpdate = true;
  });

  return (
    <points ref={pointsRef}>
      <bufferGeometry>
        <bufferAttribute
          attach="attributes-position"
          count={count}
          array={positions}
          itemSize={3}
        />
        <bufferAttribute
          attach="attributes-color"
          count={count}
          array={colors}
          itemSize={3}
        />
      </bufferGeometry>
      <pointsMaterial
        size={0.08}
        vertexColors
        transparent
        opacity={0.7}
        sizeAttenuation
        blending={THREE.AdditiveBlending}
        depthWrite={false}
      />
    </points>
  );
}

// ============================================
// Grid Floor - Futuristic perspective grid
// ============================================
function GridFloor({ color, opacity }: { color: string; opacity: number }) {
  const gridRef = useRef<THREE.Group>(null);

  useFrame(({ clock }) => {
    if (!gridRef.current) return;
    const t = clock.elapsedTime;
    // Subtle wave animation
    gridRef.current.children.forEach((line, i) => {
      if (line instanceof THREE.Line) {
        line.position.z = -15 + Math.sin(t * 0.5 + i * 0.1) * 0.2;
      }
    });
  });

  const lines = useMemo(() => {
    const result: THREE.Vector3[][] = [];

    // Horizontal lines
    for (let z = 0; z <= 20; z += 2) {
      result.push([
        new THREE.Vector3(-40, -8, -z - 10),
        new THREE.Vector3(40, -8, -z - 10),
      ]);
    }

    // Vertical lines (perspective)
    for (let x = -40; x <= 40; x += 4) {
      result.push([
        new THREE.Vector3(x, -8, -10),
        new THREE.Vector3(x * 0.3, -8, -30),
      ]);
    }

    return result;
  }, []);

  return (
    <group ref={gridRef}>
      {lines.map((points, i) => (
        <line key={i}>
          <bufferGeometry>
            <bufferAttribute
              attach="attributes-position"
              count={2}
              array={new Float32Array(points.flatMap(p => [p.x, p.y, p.z]))}
              itemSize={3}
            />
          </bufferGeometry>
          <lineBasicMaterial color={color} transparent opacity={opacity * (1 - i * 0.02)} />
        </line>
      ))}
    </group>
  );
}

// ============================================
// Floating Metrics - Trading stats
// ============================================
function FloatingMetrics({ color }: { color: string }) {
  const groupRef = useRef<THREE.Group>(null);

  useFrame(({ clock }) => {
    if (!groupRef.current) return;
    const t = clock.elapsedTime;
    groupRef.current.children.forEach((child, i) => {
      child.position.y += Math.sin(t + i) * 0.002;
    });
  });

  const positions: [number, number, number][] = [
    [-12, 6, -3],
    [12, 5, -4],
    [-10, -4, -2],
    [11, -3, -3],
    [-15, 1, -5],
    [14, 0, -4],
  ];

  return (
    <group ref={groupRef}>
      {positions.map((pos, i) => (
        <mesh key={i} position={pos}>
          <boxGeometry args={[0.8, 0.4, 0.05]} />
          <meshBasicMaterial color={color} transparent opacity={0.3} />
        </mesh>
      ))}
    </group>
  );
}

// ============================================
// Main Scene
// ============================================
interface SceneProps {
  primaryColor: string;
  secondaryColor: string;
  accentColor: string;
  isDark: boolean;
}

function Scene({ primaryColor, secondaryColor, accentColor, isDark }: SceneProps) {
  return (
    <>
      {/* Ambient Light */}
      <ambientLight intensity={isDark ? 0.2 : 0.4} />

      {/* Point light at core */}
      <pointLight position={[0, 0, -5]} intensity={1} color={primaryColor} distance={20} />

      {/* Central AI Core */}
      <AICore color={primaryColor} secondaryColor={secondaryColor} />

      {/* Orbiting Data Rings */}
      <OrbitingRing radius={5} color={primaryColor} speed={0.3} tilt={0.3} nodeCount={8} />
      <OrbitingRing radius={6.5} color={secondaryColor} speed={-0.2} tilt={-0.5} nodeCount={12} />
      <OrbitingRing radius={8} color={accentColor} speed={0.15} tilt={0.8} nodeCount={6} />

      {/* Holographic Panels */}
      <HoloPanel position={[-8, 3, -2]} rotation={[0, 0.3, 0]} width={2.5} height={1.5} color={primaryColor} />
      <HoloPanel position={[8, 2, -3]} rotation={[0, -0.3, 0]} width={2.2} height={1.3} color={secondaryColor} />
      <HoloPanel position={[-6, -3, -1]} rotation={[0, 0.2, 0]} width={2} height={1.2} color={accentColor} />
      <HoloPanel position={[7, -2, -2]} rotation={[0, -0.25, 0]} width={2.3} height={1.4} color={primaryColor} />

      {/* Energy Streams */}
      <EnergyStream start={[0, 0, -5]} end={[-8, 3, -2]} color={primaryColor} />
      <EnergyStream start={[0, 0, -5]} end={[8, 2, -3]} color={secondaryColor} />
      <EnergyStream start={[0, 0, -5]} end={[-6, -3, -1]} color={accentColor} />
      <EnergyStream start={[0, 0, -5]} end={[7, -2, -2]} color={primaryColor} />

      {/* Particle Field */}
      <ParticleField count={400} color={primaryColor} secondaryColor={secondaryColor} />

      {/* Grid Floor */}
      <GridFloor color={primaryColor} opacity={isDark ? 0.25 : 0.15} />

      {/* Floating Metrics */}
      <FloatingMetrics color={secondaryColor} />
    </>
  );
}

// ============================================
// Camera Animation
// ============================================
function CameraAnimation() {
  const { camera } = useThree();

  useFrame(({ clock }) => {
    const t = clock.elapsedTime;
    // Subtle camera sway
    camera.position.x = Math.sin(t * 0.1) * 0.5;
    camera.position.y = Math.cos(t * 0.15) * 0.3;
    camera.lookAt(0, 0, -5);
  });

  return null;
}

// ============================================
// Exported Component
// ============================================
interface HeroScene3DProps {
  className?: string;
}

export default function HeroScene3D({ className = '' }: HeroScene3DProps) {
  const { resolvedTheme } = useTheme();
  const isDark = resolvedTheme === 'dark';
  const [isMounted, setIsMounted] = useState(false);
  const [isLowPerf, setIsLowPerf] = useState(false);

  useEffect(() => {
    setIsMounted(true);
    const isMobile = /iPhone|iPad|iPod|Android/i.test(navigator.userAgent);
    const isLowEnd = navigator.hardwareConcurrency && navigator.hardwareConcurrency < 4;
    setIsLowPerf(isMobile || isLowEnd || false);
  }, []);

  // Theme-aware colors
  const colors = {
    primary: isDark ? '#00D9FF' : '#0891b2',     // Cyan
    secondary: isDark ? '#22c55e' : '#16a34a',   // Green
    accent: isDark ? '#8B5CF6' : '#7c3aed',      // Purple
  };

  if (!isMounted) {
    return null;
  }

  return (
    <div className={`absolute inset-0 ${className}`}>
      <Canvas
        camera={{ position: [0, 0, 12], fov: 60 }}
        dpr={isLowPerf ? [1, 1] : [1, 2]}
        gl={{
          antialias: !isLowPerf,
          alpha: true,
          powerPreference: 'high-performance',
        }}
        style={{ background: 'transparent' }}
      >
        <CameraAnimation />
        <Scene
          primaryColor={colors.primary}
          secondaryColor={colors.secondary}
          accentColor={colors.accent}
          isDark={isDark}
        />
      </Canvas>
    </div>
  );
}
