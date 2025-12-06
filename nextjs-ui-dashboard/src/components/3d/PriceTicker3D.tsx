'use client';

import { useFrame } from '@react-three/fiber';
import { useRef, useMemo } from 'react';
import * as THREE from 'three';
import { Text } from '@react-three/drei';

interface CoinData {
  symbol: string;
  price: number;
  change24h: number;
}

interface CoinTickerProps extends CoinData {
  position: [number, number, number];
  rotation: number;
}

function CoinTicker({ symbol, price, change24h, position, rotation }: CoinTickerProps) {
  const groupRef = useRef<THREE.Group>(null);
  const glowRef = useRef<THREE.Mesh>(null);

  // Color based on price change
  const color = change24h >= 0 ? '#10b981' : '#ef4444';
  const intensity = Math.min(Math.abs(change24h) / 10, 1);

  useFrame(({ clock }) => {
    if (!groupRef.current) return;

    // Rotate to face camera
    groupRef.current.rotation.y = rotation;

    // Pulse glow based on volatility
    if (glowRef.current) {
      const pulse = Math.sin(clock.elapsedTime * 3) * 0.3 + 0.7;
      glowRef.current.scale.setScalar(pulse);
    }
  });

  // Get coin color
  const coinColor = useMemo(() => {
    const colors: Record<string, string> = {
      BTC: '#F7931A',
      ETH: '#627EEA',
      SOL: '#14F195',
      BNB: '#F3BA2F',
      ADA: '#0033AD',
      DOT: '#E6007A',
    };
    return colors[symbol] || '#8b5cf6';
  }, [symbol]);

  return (
    <group ref={groupRef} position={position}>
      {/* Coin */}
      <mesh>
        <cylinderGeometry args={[0.4, 0.4, 0.08, 32]} />
        <meshStandardMaterial
          color={coinColor}
          metalness={0.9}
          roughness={0.1}
          emissive={coinColor}
          emissiveIntensity={0.3}
        />
      </mesh>

      {/* Glow ring */}
      <mesh ref={glowRef} rotation={[Math.PI / 2, 0, 0]}>
        <torusGeometry args={[0.5, 0.05, 16, 32]} />
        <meshBasicMaterial color={color} transparent opacity={intensity} />
      </mesh>

      {/* Symbol text */}
      <Text
        position={[0, 0.8, 0]}
        fontSize={0.2}
        color="#ffffff"
        anchorX="center"
        anchorY="middle"
        outlineWidth={0.02}
        outlineColor="#000000"
      >
        {symbol}
      </Text>

      {/* Price text */}
      <Text
        position={[0, 0.5, 0]}
        fontSize={0.15}
        color="#ffffff"
        anchorX="center"
        anchorY="middle"
      >
        ${price.toLocaleString()}
      </Text>

      {/* Change text */}
      <Text
        position={[0, 0.3, 0]}
        fontSize={0.12}
        color={color}
        anchorX="center"
        anchorY="middle"
      >
        {change24h >= 0 ? '+' : ''}
        {change24h.toFixed(2)}%
      </Text>
    </group>
  );
}

interface PriceTicker3DProps {
  coins: CoinData[];
  radius?: number;
  rotationSpeed?: number;
}

/**
 * 3D carousel of cryptocurrency prices
 * Features: rotating coins with price labels, color-coded by change
 */
export default function PriceTicker3D({
  coins,
  radius = 4,
  rotationSpeed = 0.2,
}: PriceTicker3DProps) {
  const carouselRef = useRef<THREE.Group>(null);

  useFrame(({ clock }) => {
    if (!carouselRef.current) return;

    // Carousel rotation
    carouselRef.current.rotation.y = clock.elapsedTime * rotationSpeed;
  });

  // Calculate positions in circle
  const coinPositions = useMemo(() => {
    return coins.map((_, index) => {
      const angle = (index / coins.length) * Math.PI * 2;
      const x = Math.cos(angle) * radius;
      const z = Math.sin(angle) * radius;
      return [x, 0, z] as [number, number, number];
    });
  }, [coins.length, radius]);

  return (
    <group ref={carouselRef}>
      {coins.map((coin, index) => (
        <CoinTicker
          key={`${coin.symbol}-${index}`}
          {...coin}
          position={coinPositions[index]}
          rotation={-(index / coins.length) * Math.PI * 2}
        />
      ))}

      {/* Center platform */}
      <mesh rotation={[Math.PI / 2, 0, 0]} position={[0, -0.5, 0]}>
        <ringGeometry args={[radius - 0.5, radius + 0.5, 64]} />
        <meshBasicMaterial
          color="#8b5cf6"
          transparent
          opacity={0.1}
          side={THREE.DoubleSide}
        />
      </mesh>
    </group>
  );
}
