'use client';

import { useFrame } from '@react-three/fiber';
import { useRef, useMemo } from 'react';
import * as THREE from 'three';

interface GlowOrbProps {
  position?: [number, number, number];
  color?: string;
  mode?: 'profit' | 'loss' | 'neutral';
  pulseSpeed?: number;
  size?: number;
}

/**
 * Shader-based glowing orb with pulse animation
 * Color changes based on mode (profit=blue, loss=red, neutral=purple)
 */
export default function GlowOrb({
  position = [0, 0, 0],
  color,
  mode = 'neutral',
  pulseSpeed = 1,
  size = 1,
}: GlowOrbProps) {
  const orbRef = useRef<THREE.Mesh>(null);
  const glowRef = useRef<THREE.Mesh>(null);

  // Mode-aware colors
  const modeColor = useMemo(() => {
    if (color) return color;
    switch (mode) {
      case 'profit':
        return '#3b82f6'; // Blue
      case 'loss':
        return '#ef4444'; // Red
      default:
        return '#8b5cf6'; // Purple
    }
  }, [color, mode]);

  // Custom shader for glow effect
  const glowMaterial = useMemo(
    () =>
      new THREE.ShaderMaterial({
        uniforms: {
          time: { value: 0 },
          color: { value: new THREE.Color(modeColor) },
          intensity: { value: 1.0 },
        },
        vertexShader: `
          varying vec3 vNormal;
          varying vec3 vPosition;

          void main() {
            vNormal = normalize(normalMatrix * normal);
            vPosition = position;
            gl_Position = projectionMatrix * modelViewMatrix * vec4(position, 1.0);
          }
        `,
        fragmentShader: `
          uniform float time;
          uniform vec3 color;
          uniform float intensity;

          varying vec3 vNormal;
          varying vec3 vPosition;

          void main() {
            // Fresnel effect for glow
            float fresnel = pow(1.0 - abs(dot(vNormal, vec3(0.0, 0.0, 1.0))), 3.0);

            // Pulse effect
            float pulse = sin(time * 2.0) * 0.3 + 0.7;

            // Combine effects
            float alpha = fresnel * pulse * intensity;

            gl_FragColor = vec4(color, alpha);
          }
        `,
        transparent: true,
        side: THREE.BackSide,
        blending: THREE.AdditiveBlending,
      }),
    [modeColor]
  );

  useFrame(({ clock }) => {
    if (!orbRef.current || !glowRef.current) return;

    const time = clock.elapsedTime * pulseSpeed;

    // Update shader time
    glowMaterial.uniforms.time.value = time;

    // Pulse animation
    const pulse = Math.sin(time * 2) * 0.1 + 1;
    orbRef.current.scale.setScalar(pulse);
    glowRef.current.scale.setScalar(pulse * 1.2);

    // Slow rotation
    orbRef.current.rotation.y += 0.005;
    orbRef.current.rotation.x += 0.003;
  });

  return (
    <group position={position}>
      {/* Core orb */}
      <mesh ref={orbRef}>
        <sphereGeometry args={[size, 32, 32]} />
        <meshStandardMaterial
          color={modeColor}
          metalness={0.9}
          roughness={0.1}
          emissive={modeColor}
          emissiveIntensity={0.5}
        />
      </mesh>

      {/* Glow layer with shader */}
      <mesh ref={glowRef} material={glowMaterial}>
        <sphereGeometry args={[size * 1.2, 32, 32]} />
      </mesh>

      {/* Additional glow ring */}
      <mesh rotation={[Math.PI / 2, 0, 0]}>
        <torusGeometry args={[size * 1.5, 0.05, 16, 32]} />
        <meshBasicMaterial color={modeColor} transparent opacity={0.3} />
      </mesh>
    </group>
  );
}
