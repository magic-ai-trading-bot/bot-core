import { Canvas } from '@react-three/fiber';
import { OrbitControls, Float, Text, MeshDistortMaterial } from '@react-three/drei';
import { useRef, useMemo } from 'react';
import { useFrame } from '@react-three/fiber';
import * as THREE from 'three';

function FloatingChart() {
  const meshRef = useRef<THREE.Mesh>(null);
  
  useFrame((state) => {
    if (meshRef.current) {
      meshRef.current.rotation.y = state.clock.elapsedTime * 0.5;
      meshRef.current.rotation.x = Math.sin(state.clock.elapsedTime) * 0.1;
      meshRef.current.position.y = Math.sin(state.clock.elapsedTime * 1.5) * 0.3;
    }
  });

  return (
    <Float speed={2} rotationIntensity={1} floatIntensity={2}>
      <mesh ref={meshRef} position={[3, 0, -1]}>
        <icosahedronGeometry args={[1.2, 1]} />
        <MeshDistortMaterial
          color="#22c55e"
          metalness={0.9}
          roughness={0.1}
          distort={0.3}
          speed={2}
          transparent
          opacity={0.8}
        />
      </mesh>
    </Float>
  );
}

function FloatingTorus() {
  const meshRef = useRef<THREE.Mesh>(null);
  
  useFrame((state) => {
    if (meshRef.current) {
      meshRef.current.rotation.x = state.clock.elapsedTime * 0.3;
      meshRef.current.rotation.z = Math.sin(state.clock.elapsedTime) * 0.2;
    }
  });

  return (
    <Float speed={1.8} rotationIntensity={0.8} floatIntensity={1.5}>
      <mesh ref={meshRef} position={[-3, 1, 1]}>
        <torusGeometry args={[1, 0.3, 16, 100]} />
        <meshStandardMaterial 
          color="#fbbf24"
          metalness={0.8}
          roughness={0.2}
          transparent
          opacity={0.7}
        />
      </mesh>
    </Float>
  );
}

function AIBrain() {
  const sphereRef = useRef<THREE.Mesh>(null);
  
  useFrame((state) => {
    if (sphereRef.current) {
      sphereRef.current.rotation.y = state.clock.elapsedTime * 0.3;
      sphereRef.current.position.y = Math.sin(state.clock.elapsedTime * 2) * 0.3;
      sphereRef.current.position.x = -2 + Math.sin(state.clock.elapsedTime * 0.5) * 0.5;
    }
  });

  return (
    <Float speed={1.5} rotationIntensity={0.5} floatIntensity={1}>
      <mesh ref={sphereRef} position={[-2, 1, 0]}>
        <sphereGeometry args={[1, 32, 32]} />
        <MeshDistortMaterial
          color="#3b82f6"
          metalness={0.9}
          roughness={0.1}
          distort={0.4}
          speed={1.5}
          transparent
          opacity={0.9}
        />
      </mesh>
    </Float>
  );
}

function CryptoRings() {
  const groupRef = useRef<THREE.Group>(null);
  
  useFrame((state) => {
    if (groupRef.current) {
      groupRef.current.rotation.y = state.clock.elapsedTime * 0.2;
      groupRef.current.rotation.x = Math.sin(state.clock.elapsedTime * 0.5) * 0.1;
    }
  });

  return (
    <group ref={groupRef} position={[0, -1, 2]}>
      <Float speed={1} rotationIntensity={0.3} floatIntensity={0.5}>
        <mesh>
          <torusGeometry args={[2, 0.1, 8, 32]} />
          <meshStandardMaterial color="#e879f9" transparent opacity={0.6} />
        </mesh>
      </Float>
      <Float speed={1.2} rotationIntensity={0.4} floatIntensity={0.7}>
        <mesh rotation={[Math.PI / 2, 0, 0]}>
          <torusGeometry args={[2.5, 0.08, 8, 32]} />
          <meshStandardMaterial color="#06b6d4" transparent opacity={0.5} />
        </mesh>
      </Float>
    </group>
  );
}

function ParticleField() {
  const particlesRef = useRef<THREE.Points>(null);
  
  useFrame((state) => {
    if (particlesRef.current) {
      particlesRef.current.rotation.y = state.clock.elapsedTime * 0.1;
      particlesRef.current.rotation.x = Math.sin(state.clock.elapsedTime * 0.2) * 0.05;
    }
  });

  const particleCount = 200;
  const positions = useMemo(() => {
    const pos = new Float32Array(particleCount * 3);
    for (let i = 0; i < particleCount; i++) {
      pos[i * 3] = (Math.random() - 0.5) * 15;
      pos[i * 3 + 1] = (Math.random() - 0.5) * 15;
      pos[i * 3 + 2] = (Math.random() - 0.5) * 15;
    }
    return pos;
  }, []);

  return (
    <points ref={particlesRef}>
      <bufferGeometry>
        <bufferAttribute
          attach="attributes-position"
          count={particleCount}
          array={positions}
          itemSize={3}
        />
      </bufferGeometry>
      <pointsMaterial size={0.03} color="#fbbf24" transparent opacity={0.8} />
    </points>
  );
}

function FloatingElements() {
  return (
    <>
      {Array.from({ length: 8 }).map((_, i) => (
        <Float key={i} speed={1 + i * 0.2} rotationIntensity={0.5 + i * 0.1} floatIntensity={1 + i * 0.2}>
          <mesh position={[
            (Math.random() - 0.5) * 8,
            (Math.random() - 0.5) * 6,
            (Math.random() - 0.5) * 8
          ]}>
            <sphereGeometry args={[0.1 + Math.random() * 0.2, 8, 8]} />
            <meshStandardMaterial 
              color={`hsl(${Math.random() * 360}, 70%, 60%)`}
              transparent 
              opacity={0.6 + Math.random() * 0.4}
            />
          </mesh>
        </Float>
      ))}
    </>
  );
}

export function Hero3D() {
  return (
    <div className="absolute inset-0 z-0">
      <Canvas camera={{ position: [0, 0, 8], fov: 75 }}>
        <ambientLight intensity={0.2} />
        <directionalLight position={[10, 10, 5]} intensity={1.2} />
        <pointLight position={[-10, -10, -5]} intensity={0.8} color="#3b82f6" />
        <pointLight position={[5, 5, 5]} intensity={0.6} color="#22c55e" />
        <spotLight position={[0, 10, 0]} intensity={0.5} color="#fbbf24" />
        
        <FloatingChart />
        <FloatingTorus />
        <AIBrain />
        <CryptoRings />
        <ParticleField />
        <FloatingElements />
        
        <Text
          position={[0, -3, 0]}
          fontSize={0.25}
          color="#22c55e"
          anchorX="center"
          anchorY="middle"
          font="https://fonts.gstatic.com/s/orbitron/v29/yMJWMIlzdpvBhQQL_Qq7dys.woff"
        >
          AI POWERED TRADING
        </Text>
        
        <OrbitControls 
          enableZoom={false} 
          enablePan={false} 
          autoRotate 
          autoRotateSpeed={0.3}
          maxPolarAngle={Math.PI / 1.8}
          minPolarAngle={Math.PI / 2.5}
        />
      </Canvas>
    </div>
  );
}