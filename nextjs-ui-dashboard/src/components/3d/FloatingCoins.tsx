'use client';

import { useFrame } from '@react-three/fiber';
import { useRef, useState } from 'react';
import * as THREE from 'three';

interface CoinProps {
  position: [number, number, number];
  symbol: 'BTC' | 'ETH' | 'SOL';
  color: string;
  delay?: number;
}

function Coin({ position, symbol, color, delay = 0 }: CoinProps) {
  const meshRef = useRef<THREE.Mesh>(null);
  const [hovered, setHovered] = useState(false);

  useFrame(({ clock }) => {
    if (!meshRef.current) return;

    const time = clock.elapsedTime + delay;

    // Float animation (sin wave)
    meshRef.current.position.y = position[1] + Math.sin(time * 0.5) * 0.3;

    // Rotation animation
    meshRef.current.rotation.y += 0.01;
    meshRef.current.rotation.x = Math.sin(time * 0.3) * 0.1;

    // Hover effect
    const targetScale = hovered ? 1.2 : 1;
    meshRef.current.scale.lerp(
      new THREE.Vector3(targetScale, targetScale, targetScale),
      0.1
    );
  });

  return (
    <group position={position}>
      <mesh
        ref={meshRef}
        onPointerOver={() => setHovered(true)}
        onPointerOut={() => setHovered(false)}
        castShadow
        receiveShadow
      >
        {/* Coin cylinder */}
        <cylinderGeometry args={[0.5, 0.5, 0.1, 32]} />
        <meshStandardMaterial
          color={color}
          metalness={0.8}
          roughness={0.2}
          emissive={color}
          emissiveIntensity={hovered ? 0.5 : 0.2}
        />
      </mesh>

      {/* Symbol text (simplified - use sprite in production) */}
      <mesh position={[0, 0, 0.06]}>
        <circleGeometry args={[0.3, 32]} />
        <meshBasicMaterial color="#000000" opacity={0.5} transparent />
      </mesh>

      {/* Glow ring when hovered */}
      {hovered && (
        <mesh rotation={[Math.PI / 2, 0, 0]}>
          <torusGeometry args={[0.6, 0.05, 16, 32]} />
          <meshBasicMaterial color={color} transparent opacity={0.6} />
        </mesh>
      )}
    </group>
  );
}

interface FloatingCoinsProps {
  coins?: Array<{
    symbol: 'BTC' | 'ETH' | 'SOL';
    position?: [number, number, number];
  }>;
}

/**
 * Floating cryptocurrency coins with animations
 * BTC (gold), ETH (blue), SOL (purple)
 */
export default function FloatingCoins({ coins }: FloatingCoinsProps) {
  const defaultCoins = [
    { symbol: 'BTC' as const, position: [-2, 0, 0] as [number, number, number], color: '#F7931A' },
    { symbol: 'ETH' as const, position: [0, 0, 0] as [number, number, number], color: '#627EEA' },
    { symbol: 'SOL' as const, position: [2, 0, 0] as [number, number, number], color: '#14F195' },
  ];

  const coinsToRender = coins
    ? coins.map((coin, index) => ({
        ...coin,
        position: coin.position || defaultCoins[index]?.position || [0, 0, 0],
        color: defaultCoins.find((c) => c.symbol === coin.symbol)?.color || '#FFFFFF',
      }))
    : defaultCoins;

  return (
    <group>
      {coinsToRender.map((coin, index) => (
        <Coin
          key={`${coin.symbol}-${index}`}
          position={coin.position}
          symbol={coin.symbol}
          color={coin.color}
          delay={index * 0.5}
        />
      ))}
    </group>
  );
}
