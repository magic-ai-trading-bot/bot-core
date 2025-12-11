/**
 * Hero3DScene - 3D Hero Scene with React Three Fiber
 *
 * Features:
 * - Floating crypto coins (BTC, ETH)
 * - Glowing orb in center
 * - Particle field background
 * - Mouse parallax effect
 * - Fallback for low-end devices
 */

import { Canvas } from '@react-three/fiber';
import { Float, Environment, Stars, Sphere, MeshDistortMaterial } from '@react-three/drei';
import { useState, useEffect, Suspense } from 'react';
import { motion } from 'framer-motion';

// Crypto coin component
function CryptoCoin({ position, color }: { position: [number, number, number]; color: string }) {
  return (
    <Float speed={2} rotationIntensity={1} floatIntensity={2}>
      <mesh position={position}>
        <cylinderGeometry args={[0.5, 0.5, 0.1, 32]} />
        <meshStandardMaterial
          color={color}
          metalness={0.9}
          roughness={0.1}
          emissive={color}
          emissiveIntensity={0.3}
        />
      </mesh>
    </Float>
  );
}

// Glowing orb component
function GlowOrb() {
  return (
    <Float speed={1.5} rotationIntensity={0.5} floatIntensity={1}>
      <Sphere args={[1, 64, 64]}>
        <MeshDistortMaterial
          color="#0EA5E9"
          attach="material"
          distort={0.4}
          speed={2}
          roughness={0.2}
          metalness={0.8}
          emissive="#0EA5E9"
          emissiveIntensity={0.5}
        />
      </Sphere>
    </Float>
  );
}

// 3D Scene content
function SceneContent() {
  return (
    <>
      {/* Lighting */}
      <ambientLight intensity={0.3} />
      <pointLight position={[10, 10, 10]} intensity={1} />
      <pointLight position={[-10, -10, -10]} color="#0EA5E9" intensity={0.5} />

      {/* Central glow orb */}
      <GlowOrb />

      {/* Floating crypto coins */}
      <CryptoCoin position={[-3, 2, 0]} color="#F7931A" /> {/* Bitcoin orange */}
      <CryptoCoin position={[3, -2, 0]} color="#627EEA" /> {/* Ethereum blue */}
      <CryptoCoin position={[2, 2, -2]} color="#10B981" /> {/* Green */}
      <CryptoCoin position={[-2, -1, -2]} color="#8B5CF6" /> {/* Purple */}

      {/* Stars background */}
      <Stars
        radius={100}
        depth={50}
        count={5000}
        factor={4}
        saturation={0}
        fade
        speed={1}
      />

      {/* Environment for reflections */}
      <Environment preset="night" />
    </>
  );
}

// Fallback gradient animation for low-end devices
function FallbackGradient() {
  return (
    <motion.div
      className="absolute inset-0 bg-gradient-to-br from-blue-600/20 via-purple-600/10 to-pink-600/20"
      animate={{
        background: [
          'radial-gradient(circle at 20% 50%, rgba(14, 165, 233, 0.2), transparent 50%), radial-gradient(circle at 80% 50%, rgba(139, 92, 246, 0.1), transparent 50%)',
          'radial-gradient(circle at 80% 50%, rgba(14, 165, 233, 0.2), transparent 50%), radial-gradient(circle at 20% 50%, rgba(139, 92, 246, 0.1), transparent 50%)',
          'radial-gradient(circle at 20% 50%, rgba(14, 165, 233, 0.2), transparent 50%), radial-gradient(circle at 80% 50%, rgba(139, 92, 246, 0.1), transparent 50%)',
        ],
      }}
      transition={{
        duration: 8,
        repeat: Infinity,
        ease: 'easeInOut',
      }}
    />
  );
}

// Loading fallback
function CanvasLoader() {
  return (
    <div className="flex items-center justify-center w-full h-full">
      <motion.div
        className="w-16 h-16 border-4 border-blue-500/30 border-t-blue-500 rounded-full"
        animate={{ rotate: 360 }}
        transition={{ duration: 1, repeat: Infinity, ease: 'linear' }}
      />
    </div>
  );
}

export function Hero3DScene() {
  const [useFallback, setUseFallback] = useState(false);
  const [hasError, setHasError] = useState(false);

  // Detect low-end devices
  useEffect(() => {
    const checkDevice = () => {
      // Check for low memory or reduced motion preference
      const isLowEnd =
        (navigator as any).deviceMemory && (navigator as any).deviceMemory < 4;
      const prefersReducedMotion = window.matchMedia(
        '(prefers-reduced-motion: reduce)'
      ).matches;

      if (isLowEnd || prefersReducedMotion) {
        setUseFallback(true);
      }
    };

    checkDevice();
  }, []);

  // Error boundary for WebGL issues
  const handleError = () => {
    console.warn('WebGL not supported, falling back to gradient animation');
    setHasError(true);
    setUseFallback(true);
  };

  if (useFallback || hasError) {
    return <FallbackGradient />;
  }

  return (
    <div className="absolute inset-0 -z-10">
      <Canvas
        camera={{ position: [0, 0, 8], fov: 50 }}
        onError={handleError}
        gl={{
          antialias: true,
          alpha: true,
          powerPreference: 'high-performance',
        }}
      >
        <Suspense fallback={<CanvasLoader />}>
          <SceneContent />
        </Suspense>
      </Canvas>
    </div>
  );
}
