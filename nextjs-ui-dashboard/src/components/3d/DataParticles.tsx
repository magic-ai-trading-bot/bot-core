'use client';

/**
 * DataParticles - Interactive floating data particles with mouse tracking
 *
 * Features:
 * - Floating particles representing market data flow
 * - Mouse repulsion/attraction interaction
 * - Theme-aware colors
 * - Performance-scaled particle count
 */

import { useRef, useMemo, useEffect } from 'react';
import { useFrame, useThree } from '@react-three/fiber';
import * as THREE from 'three';

interface DataParticlesProps {
  count?: number;
  primaryColor?: string;
  secondaryColor?: string;
  accentColor?: string;
  mouseInfluence?: number;
  isMobile?: boolean;
}

// ============================================
// Mouse tracker (reads from canvas events)
// ============================================
function useMousePosition() {
  const mouse = useRef(new THREE.Vector2(0, 0));
  const { gl } = useThree();

  useEffect(() => {
    const canvas = gl.domElement;

    const onMove = (e: MouseEvent) => {
      const rect = canvas.getBoundingClientRect();
      // Normalize to [-1, 1]
      mouse.current.x = ((e.clientX - rect.left) / rect.width) * 2 - 1;
      mouse.current.y = -((e.clientY - rect.top) / rect.height) * 2 + 1;
    };

    const onLeave = () => {
      mouse.current.set(0, 0);
    };

    canvas.addEventListener('mousemove', onMove);
    canvas.addEventListener('mouseleave', onLeave);
    return () => {
      canvas.removeEventListener('mousemove', onMove);
      canvas.removeEventListener('mouseleave', onLeave);
    };
  }, [gl]);

  return mouse;
}

// ============================================
// Main DataParticles component
// ============================================
export default function DataParticles({
  count = 300,
  primaryColor = '#00D9FF',
  secondaryColor = '#22c55e',
  accentColor = '#8b5cf6',
  mouseInfluence = 2.5,
  isMobile = false,
}: DataParticlesProps) {
  const pointsRef = useRef<THREE.Points>(null);
  const mouse = useMousePosition();
  const { camera } = useThree();

  // Scale down on mobile
  const particleCount = isMobile ? Math.floor(count * 0.4) : count;

  const { positions, colors, velocities, phases, sizes } = useMemo(() => {
    const positions = new Float32Array(particleCount * 3);
    const colors = new Float32Array(particleCount * 3);
    const velocities = new Float32Array(particleCount * 3);
    const phases = new Float32Array(particleCount);
    const sizes = new Float32Array(particleCount);

    const colorPalette = [
      new THREE.Color(primaryColor),
      new THREE.Color(secondaryColor),
      new THREE.Color(accentColor),
    ];

    for (let i = 0; i < particleCount; i++) {
      // Spread particles across a wide plane in front of camera
      positions[i * 3] = (Math.random() - 0.5) * 40;
      positions[i * 3 + 1] = (Math.random() - 0.5) * 20;
      positions[i * 3 + 2] = -5 + (Math.random() - 0.5) * 8;

      const c = colorPalette[Math.floor(Math.random() * colorPalette.length)];
      colors[i * 3] = c.r;
      colors[i * 3 + 1] = c.g;
      colors[i * 3 + 2] = c.b;

      // Gentle drift velocities
      velocities[i * 3] = (Math.random() - 0.5) * 0.005;
      velocities[i * 3 + 1] = (Math.random() - 0.5) * 0.003;
      velocities[i * 3 + 2] = 0;

      phases[i] = Math.random() * Math.PI * 2;

      // Vary sizes for depth perception
      sizes[i] = 0.05 + Math.random() * 0.1;
    }

    return { positions, colors, velocities, phases, sizes };
  }, [particleCount, primaryColor, secondaryColor, accentColor]);

  // Reusable vectors to avoid GC pressure
  const worldMouse = useRef(new THREE.Vector3());
  const toParticle = useRef(new THREE.Vector3());

  useFrame(({ clock }) => {
    if (!pointsRef.current) return;
    const t = clock.elapsedTime;
    const pos = pointsRef.current.geometry.attributes.position.array as Float32Array;

    // Project mouse to world space at z=-1 plane
    worldMouse.current.set(mouse.current.x * 20, mouse.current.y * 10, -1);

    for (let i = 0; i < particleCount; i++) {
      const ix = i * 3;

      // Floating drift
      pos[ix] += velocities[ix] + Math.sin(t * 0.3 + phases[i]) * 0.002;
      pos[ix + 1] += velocities[ix + 1] + Math.cos(t * 0.25 + phases[i]) * 0.001;

      // Mouse repulsion
      toParticle.current.set(pos[ix], pos[ix + 1], pos[ix + 2]);
      toParticle.current.sub(worldMouse.current);
      const dist = toParticle.current.length();

      if (dist < mouseInfluence && dist > 0.01) {
        const force = (mouseInfluence - dist) / mouseInfluence;
        const pushX = (toParticle.current.x / dist) * force * 0.04;
        const pushY = (toParticle.current.y / dist) * force * 0.04;
        pos[ix] += pushX;
        pos[ix + 1] += pushY;
      }

      // Wrap particles that drift too far
      if (pos[ix] > 22) pos[ix] = -22;
      if (pos[ix] < -22) pos[ix] = 22;
      if (pos[ix + 1] > 12) pos[ix + 1] = -12;
      if (pos[ix + 1] < -12) pos[ix + 1] = 12;
    }

    pointsRef.current.geometry.attributes.position.needsUpdate = true;
  });

  return (
    <points ref={pointsRef}>
      <bufferGeometry>
        <bufferAttribute
          attach="attributes-position"
          count={particleCount}
          array={positions}
          itemSize={3}
        />
        <bufferAttribute
          attach="attributes-color"
          count={particleCount}
          array={colors}
          itemSize={3}
        />
      </bufferGeometry>
      <pointsMaterial
        size={isMobile ? 0.08 : 0.1}
        vertexColors
        transparent
        opacity={0.65}
        sizeAttenuation
        blending={THREE.AdditiveBlending}
        depthWrite={false}
      />
    </points>
  );
}
