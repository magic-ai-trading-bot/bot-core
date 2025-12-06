'use client';

import { useFrame } from '@react-three/fiber';
import { useRef, useMemo } from 'react';
import * as THREE from 'three';

interface ParticleFieldProps {
  count?: number;
  size?: number;
  spread?: number;
  color?: string;
  speed?: number;
}

/**
 * Star field background with slow drift animation
 * 1000-5000 particles for depth and atmosphere
 */
export default function ParticleField({
  count = 3000,
  size = 0.02,
  spread = 50,
  color = '#ffffff',
  speed = 0.1,
}: ParticleFieldProps) {
  const pointsRef = useRef<THREE.Points>(null);

  // Generate random particle positions
  const particles = useMemo(() => {
    const positions = new Float32Array(count * 3);
    const velocities = new Float32Array(count * 3);
    const sizes = new Float32Array(count);

    for (let i = 0; i < count; i++) {
      // Random position in sphere
      const radius = Math.random() * spread;
      const theta = Math.random() * Math.PI * 2;
      const phi = Math.acos(Math.random() * 2 - 1);

      positions[i * 3] = radius * Math.sin(phi) * Math.cos(theta);
      positions[i * 3 + 1] = radius * Math.sin(phi) * Math.sin(theta);
      positions[i * 3 + 2] = radius * Math.cos(phi);

      // Random velocity
      velocities[i * 3] = (Math.random() - 0.5) * 0.01;
      velocities[i * 3 + 1] = (Math.random() - 0.5) * 0.01;
      velocities[i * 3 + 2] = (Math.random() - 0.5) * 0.01;

      // Random size variation
      sizes[i] = Math.random() * size + size * 0.5;
    }

    return { positions, velocities, sizes };
  }, [count, spread, size]);

  useFrame(() => {
    if (!pointsRef.current) return;

    const positions = pointsRef.current.geometry.attributes.position
      .array as Float32Array;

    // Slow drift animation
    for (let i = 0; i < count; i++) {
      positions[i * 3] += particles.velocities[i * 3] * speed;
      positions[i * 3 + 1] += particles.velocities[i * 3 + 1] * speed;
      positions[i * 3 + 2] += particles.velocities[i * 3 + 2] * speed;

      // Wrap around when particles go too far
      if (Math.abs(positions[i * 3]) > spread) {
        positions[i * 3] *= -1;
      }
      if (Math.abs(positions[i * 3 + 1]) > spread) {
        positions[i * 3 + 1] *= -1;
      }
      if (Math.abs(positions[i * 3 + 2]) > spread) {
        positions[i * 3 + 2] *= -1;
      }
    }

    pointsRef.current.geometry.attributes.position.needsUpdate = true;
  });

  return (
    <points ref={pointsRef}>
      <bufferGeometry>
        <bufferAttribute
          attach="attributes-position"
          count={count}
          array={particles.positions}
          itemSize={3}
        />
        <bufferAttribute
          attach="attributes-size"
          count={count}
          array={particles.sizes}
          itemSize={1}
        />
      </bufferGeometry>
      <pointsMaterial
        size={size}
        color={color}
        transparent
        opacity={0.6}
        sizeAttenuation
        blending={THREE.AdditiveBlending}
        depthWrite={false}
      />
    </points>
  );
}
