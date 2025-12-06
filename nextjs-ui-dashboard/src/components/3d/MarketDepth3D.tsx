'use client';

import { useFrame } from '@react-three/fiber';
import { useRef, useMemo } from 'react';
import * as THREE from 'three';
import { OrbitControls, Text } from '@react-three/drei';

interface OrderBookEntry {
  price: number;
  amount: number;
  total: number;
}

interface MarketDepth3DProps {
  bids: OrderBookEntry[];
  asks: OrderBookEntry[];
  maxBars?: number;
}

/**
 * 3D bar chart visualization of order book depth
 * Green bars for bids, red bars for asks, camera orbit
 */
export default function MarketDepth3D({ bids, asks, maxBars = 20 }: MarketDepth3DProps) {
  const groupRef = useRef<THREE.Group>(null);

  // Normalize data
  const normalizedData = useMemo(() => {
    const topBids = bids.slice(0, maxBars);
    const topAsks = asks.slice(0, maxBars);

    const maxTotal = Math.max(
      ...topBids.map((b) => b.total),
      ...topAsks.map((a) => a.total)
    );

    return {
      bids: topBids.map((bid) => ({
        ...bid,
        normalizedHeight: (bid.total / maxTotal) * 3,
      })),
      asks: topAsks.map((ask) => ({
        ...ask,
        normalizedHeight: (ask.total / maxTotal) * 3,
      })),
      maxTotal,
    };
  }, [bids, asks, maxBars]);

  useFrame(() => {
    if (!groupRef.current) return;

    // Subtle rotation animation
    groupRef.current.rotation.y += 0.001;
  });

  const barWidth = 0.15;
  const barGap = 0.05;

  return (
    <group ref={groupRef}>
      {/* Bid bars (green) */}
      {normalizedData.bids.map((bid, index) => {
        const x = -(index * (barWidth + barGap));
        const height = bid.normalizedHeight;

        return (
          <group key={`bid-${index}`}>
            {/* Bar */}
            <mesh position={[x, height / 2, 0]}>
              <boxGeometry args={[barWidth, height, barWidth]} />
              <meshStandardMaterial
                color="#10b981"
                metalness={0.3}
                roughness={0.7}
                emissive="#10b981"
                emissiveIntensity={0.2}
              />
            </mesh>

            {/* Top glow */}
            <mesh position={[x, height, 0]}>
              <boxGeometry args={[barWidth, 0.02, barWidth]} />
              <meshBasicMaterial color="#10b981" transparent opacity={0.8} />
            </mesh>

            {/* Price label (every 5th bar) */}
            {index % 5 === 0 && (
              <Text
                position={[x, -0.3, 0]}
                fontSize={0.1}
                color="#10b981"
                anchorX="center"
                anchorY="middle"
                rotation={[-Math.PI / 2, 0, 0]}
              >
                ${bid.price.toFixed(0)}
              </Text>
            )}
          </group>
        );
      })}

      {/* Ask bars (red) */}
      {normalizedData.asks.map((ask, index) => {
        const x = index * (barWidth + barGap);
        const height = ask.normalizedHeight;

        return (
          <group key={`ask-${index}`}>
            {/* Bar */}
            <mesh position={[x, height / 2, 0]}>
              <boxGeometry args={[barWidth, height, barWidth]} />
              <meshStandardMaterial
                color="#ef4444"
                metalness={0.3}
                roughness={0.7}
                emissive="#ef4444"
                emissiveIntensity={0.2}
              />
            </mesh>

            {/* Top glow */}
            <mesh position={[x, height, 0]}>
              <boxGeometry args={[barWidth, 0.02, barWidth]} />
              <meshBasicMaterial color="#ef4444" transparent opacity={0.8} />
            </mesh>

            {/* Price label (every 5th bar) */}
            {index % 5 === 0 && (
              <Text
                position={[x, -0.3, 0]}
                fontSize={0.1}
                color="#ef4444"
                anchorX="center"
                anchorY="middle"
                rotation={[-Math.PI / 2, 0, 0]}
              >
                ${ask.price.toFixed(0)}
              </Text>
            )}
          </group>
        );
      })}

      {/* Base grid */}
      <mesh rotation={[-Math.PI / 2, 0, 0]} position={[0, -0.01, 0]}>
        <planeGeometry args={[maxBars * (barWidth + barGap), 2]} />
        <meshBasicMaterial
          color="#1e293b"
          transparent
          opacity={0.3}
          side={THREE.DoubleSide}
        />
      </mesh>

      {/* Center line */}
      <mesh position={[0, 1.5, 0]}>
        <boxGeometry args={[0.02, 3, 0.02]} />
        <meshBasicMaterial color="#8b5cf6" />
      </mesh>

      {/* Labels */}
      <Text
        position={[-2, 3.5, 0]}
        fontSize={0.15}
        color="#10b981"
        anchorX="center"
        anchorY="middle"
      >
        BIDS
      </Text>

      <Text
        position={[2, 3.5, 0]}
        fontSize={0.15}
        color="#ef4444"
        anchorX="center"
        anchorY="middle"
      >
        ASKS
      </Text>

      {/* Orbit controls */}
      <OrbitControls
        enableZoom
        enablePan
        minDistance={5}
        maxDistance={15}
        maxPolarAngle={Math.PI / 2}
      />
    </group>
  );
}
