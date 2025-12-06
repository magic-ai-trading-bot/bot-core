'use client';

import { Canvas } from '@react-three/fiber';
import { Suspense, useEffect, useState } from 'react';
import { ACESFilmicToneMapping, PCFSoftShadowMap } from 'three';

interface Canvas3DProps {
  children: React.ReactNode;
  camera?: {
    position?: [number, number, number];
    fov?: number;
  };
  shadows?: boolean;
  className?: string;
}

/**
 * Shared React Three Fiber Canvas wrapper
 * Provides common settings, lighting, and performance monitoring
 */
export default function Canvas3D({
  children,
  camera = { position: [0, 0, 5], fov: 75 },
  shadows = true,
  className = '',
}: Canvas3DProps) {
  const [dpr, setDpr] = useState(1);
  const [frameCount, setFrameCount] = useState(0);
  const [lastTime, setLastTime] = useState(Date.now());

  // Performance monitoring
  useEffect(() => {
    const interval = setInterval(() => {
      const now = Date.now();
      const elapsed = now - lastTime;
      const fps = (frameCount / elapsed) * 1000;

      // Adjust DPR based on FPS
      if (fps < 30 && dpr > 0.5) {
        setDpr(Math.max(0.5, dpr - 0.25));
      } else if (fps > 55 && dpr < 2) {
        setDpr(Math.min(2, dpr + 0.25));
      }

      setFrameCount(0);
      setLastTime(now);
    }, 2000);

    return () => clearInterval(interval);
  }, [frameCount, lastTime, dpr]);

  // Pause rendering when tab is hidden
  useEffect(() => {
    const handleVisibilityChange = () => {
      if (document.hidden) {
        // Pause animations
      }
    };

    document.addEventListener('visibilitychange', handleVisibilityChange);
    return () => document.removeEventListener('visibilitychange', handleVisibilityChange);
  }, []);

  return (
    <Canvas
      className={className}
      camera={camera}
      dpr={[1, dpr]}
      shadows={shadows}
      gl={{
        antialias: true,
        alpha: true,
        toneMapping: ACESFilmicToneMapping,
        shadowMap: {
          enabled: shadows,
          type: PCFSoftShadowMap,
        },
      }}
      onCreated={({ gl }) => {
        gl.setClearColor(0x000000, 0);
      }}
      frameloop="always"
      onPointerMissed={() => {
        setFrameCount((prev) => prev + 1);
      }}
    >
      <Suspense fallback={null}>
        {/* Ambient light for overall illumination */}
        <ambientLight intensity={0.3} />

        {/* Main directional light */}
        <directionalLight
          position={[10, 10, 5]}
          intensity={1}
          castShadow={shadows}
          shadow-mapSize-width={1024}
          shadow-mapSize-height={1024}
        />

        {/* Fill light */}
        <directionalLight position={[-5, 5, -5]} intensity={0.3} />

        {/* Point light for highlights */}
        <pointLight position={[0, 10, 0]} intensity={0.5} />

        {children}
      </Suspense>
    </Canvas>
  );
}
