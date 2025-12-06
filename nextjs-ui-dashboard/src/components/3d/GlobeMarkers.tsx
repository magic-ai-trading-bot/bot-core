'use client';

import { useFrame } from '@react-three/fiber';
import { useRef } from 'react';
import * as THREE from 'three';

interface MarkerProps {
  position: [number, number, number];
  pnl: number;
  symbol: string;
}

function Marker({ position, pnl, symbol }: MarkerProps) {
  const markerRef = useRef<THREE.Group>(null);

  // Color based on PnL
  const color = pnl >= 0 ? '#10b981' : '#ef4444';
  const intensity = Math.min(Math.abs(pnl) / 1000, 1);

  useFrame(({ clock }) => {
    if (!markerRef.current) return;

    // Pulse animation
    const pulse = Math.sin(clock.elapsedTime * 2) * 0.1 + 1;
    markerRef.current.scale.setScalar(pulse);
  });

  return (
    <group ref={markerRef} position={position}>
      {/* Pin base */}
      <mesh position={[0, 0.3, 0]}>
        <coneGeometry args={[0.1, 0.3, 8]} />
        <meshStandardMaterial
          color={color}
          emissive={color}
          emissiveIntensity={intensity}
        />
      </mesh>

      {/* Pin head */}
      <mesh position={[0, 0.5, 0]}>
        <sphereGeometry args={[0.15, 16, 16]} />
        <meshStandardMaterial
          color={color}
          metalness={0.8}
          roughness={0.2}
          emissive={color}
          emissiveIntensity={intensity}
        />
      </mesh>

      {/* Glow ring */}
      <mesh position={[0, 0, 0]} rotation={[Math.PI / 2, 0, 0]}>
        <ringGeometry args={[0.15, 0.2, 16]} />
        <meshBasicMaterial color={color} transparent opacity={0.5} />
      </mesh>
    </group>
  );
}

interface GlobeMarkersProps {
  markers: Array<{
    lat: number;
    lon: number;
    pnl: number;
    symbol: string;
  }>;
  radius?: number;
}

/**
 * 3D markers for portfolio positions on globe
 * Color based on PnL (green=profit, red=loss)
 */
export default function GlobeMarkers({ markers, radius = 2 }: GlobeMarkersProps) {
  // Convert lat/lon to 3D position on sphere
  const latLonToVector3 = (lat: number, lon: number, r: number): [number, number, number] => {
    const phi = (90 - lat) * (Math.PI / 180);
    const theta = (lon + 180) * (Math.PI / 180);

    const x = -(r * Math.sin(phi) * Math.cos(theta));
    const y = r * Math.cos(phi);
    const z = r * Math.sin(phi) * Math.sin(theta);

    return [x, y, z];
  };

  return (
    <group>
      {markers.map((marker, index) => {
        const position = latLonToVector3(marker.lat, marker.lon, radius);

        return (
          <Marker
            key={`marker-${index}-${marker.symbol}`}
            position={position}
            pnl={marker.pnl}
            symbol={marker.symbol}
          />
        );
      })}
    </group>
  );
}
