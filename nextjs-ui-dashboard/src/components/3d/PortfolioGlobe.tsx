'use client';

import { useFrame, useThree } from '@react-three/fiber';
import { useRef, useState, useMemo } from 'react';
import * as THREE from 'three';
import { OrbitControls } from '@react-three/drei';
import GlobeMarkers from './GlobeMarkers';

interface PortfolioGlobeProps {
  positions?: Array<{
    lat: number;
    lon: number;
    pnl: number;
    symbol: string;
  }>;
  radius?: number;
  autoRotate?: boolean;
}

/**
 * Earth sphere with portfolio position markers
 * Features: texture mapping, 3D pins, connection lines, drag rotation
 */
export default function PortfolioGlobe({
  positions = [],
  radius = 2,
  autoRotate = true,
}: PortfolioGlobeProps) {
  const globeRef = useRef<THREE.Mesh>(null);
  const groupRef = useRef<THREE.Group>(null);
  const [isDragging, setIsDragging] = useState(false);
  const { camera } = useThree();

  // Create earth-like texture (simplified gradient)
  const globeTexture = useMemo(() => {
    const canvas = document.createElement('canvas');
    canvas.width = 1024;
    canvas.height = 512;
    const ctx = canvas.getContext('2d');

    if (ctx) {
      // Ocean gradient
      const gradient = ctx.createLinearGradient(0, 0, 0, 512);
      gradient.addColorStop(0, '#1e3a8a');
      gradient.addColorStop(0.5, '#1e40af');
      gradient.addColorStop(1, '#1e3a8a');
      ctx.fillStyle = gradient;
      ctx.fillRect(0, 0, 1024, 512);

      // Add some "land" patterns
      ctx.fillStyle = '#059669';
      for (let i = 0; i < 50; i++) {
        const x = Math.random() * 1024;
        const y = Math.random() * 512;
        const size = Math.random() * 50 + 20;
        ctx.beginPath();
        ctx.arc(x, y, size, 0, Math.PI * 2);
        ctx.fill();
      }
    }

    return new THREE.CanvasTexture(canvas);
  }, []);

  useFrame(({ clock }) => {
    if (!groupRef.current) return;

    // Auto-rotate when not dragging
    if (autoRotate && !isDragging) {
      groupRef.current.rotation.y += 0.001;
    }

    // Pulse atmosphere
    if (globeRef.current) {
      const pulse = Math.sin(clock.elapsedTime * 0.5) * 0.02 + 1;
      globeRef.current.scale.setScalar(pulse);
    }
  });

  // Generate connection lines between markers
  const connectionLines = useMemo(() => {
    if (positions.length < 2) return null;

    const lines: THREE.Line[] = [];

    for (let i = 0; i < positions.length - 1; i++) {
      const start = positions[i];
      const end = positions[i + 1];

      // Convert lat/lon to 3D
      const phi1 = (90 - start.lat) * (Math.PI / 180);
      const theta1 = (start.lon + 180) * (Math.PI / 180);
      const phi2 = (90 - end.lat) * (Math.PI / 180);
      const theta2 = (end.lon + 180) * (Math.PI / 180);

      const startVec = new THREE.Vector3(
        -(radius * Math.sin(phi1) * Math.cos(theta1)),
        radius * Math.cos(phi1),
        radius * Math.sin(phi1) * Math.sin(theta1)
      );

      const endVec = new THREE.Vector3(
        -(radius * Math.sin(phi2) * Math.cos(theta2)),
        radius * Math.cos(phi2),
        radius * Math.sin(phi2) * Math.sin(theta2)
      );

      // Create curved line using quadratic bezier
      const curve = new THREE.QuadraticBezierCurve3(
        startVec,
        startVec.clone().add(endVec).multiplyScalar(0.6),
        endVec
      );

      const points = curve.getPoints(50);
      const geometry = new THREE.BufferGeometry().setFromPoints(points);

      lines.push(
        new THREE.Line(
          geometry,
          new THREE.LineBasicMaterial({
            color: '#8b5cf6',
            transparent: true,
            opacity: 0.5,
          })
        )
      );
    }

    return lines;
  }, [positions, radius]);

  return (
    <group ref={groupRef}>
      {/* Main globe */}
      <mesh
        ref={globeRef}
        onPointerDown={() => setIsDragging(true)}
        onPointerUp={() => setIsDragging(false)}
        onPointerLeave={() => setIsDragging(false)}
      >
        <sphereGeometry args={[radius, 64, 64]} />
        <meshStandardMaterial
          map={globeTexture}
          metalness={0.1}
          roughness={0.8}
          emissive="#1e3a8a"
          emissiveIntensity={0.2}
        />
      </mesh>

      {/* Atmosphere glow */}
      <mesh scale={1.05}>
        <sphereGeometry args={[radius, 64, 64]} />
        <meshBasicMaterial
          color="#3b82f6"
          transparent
          opacity={0.1}
          side={THREE.BackSide}
          blending={THREE.AdditiveBlending}
        />
      </mesh>

      {/* Portfolio markers */}
      {positions.length > 0 && <GlobeMarkers markers={positions} radius={radius} />}

      {/* Connection lines */}
      {connectionLines &&
        connectionLines.map((line, index) => <primitive key={`line-${index}`} object={line} />)}

      {/* Orbit controls for drag rotation */}
      <OrbitControls
        enableZoom
        enablePan={false}
        minDistance={radius + 1}
        maxDistance={radius + 10}
        camera={camera}
      />
    </group>
  );
}
