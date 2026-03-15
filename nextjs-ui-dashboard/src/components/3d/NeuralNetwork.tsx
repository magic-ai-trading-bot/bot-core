'use client';

/**
 * NeuralNetwork - Neural network grid visualization for hero background
 *
 * Renders a dynamic neural network with:
 * - Grid of nodes connected by animated edges
 * - Pulsing node activation effects
 * - Data flow along edges (traveling particles)
 * - Theme-aware colors (light/dark)
 */

import { useRef, useMemo } from 'react';
import { useFrame } from '@react-three/fiber';
import * as THREE from 'three';

// Network layout config
const COLS = 6;
const ROWS = 4;
const SPACING_X = 3.5;
const SPACING_Y = 2.5;
const OFFSET_X = ((COLS - 1) * SPACING_X) / 2;
const OFFSET_Y = ((ROWS - 1) * SPACING_Y) / 2;

interface NetworkNode {
  id: number;
  position: THREE.Vector3;
  layer: number;
}

interface NetworkEdge {
  from: number;
  to: number;
  points: THREE.Vector3[];
}

interface NeuralNetworkProps {
  primaryColor?: string;
  secondaryColor?: string;
  opacity?: number;
  isMobile?: boolean;
}

// ============================================
// Traveling particle along an edge
// ============================================
function EdgeParticle({
  points,
  color,
  speed,
  delay,
}: {
  points: THREE.Vector3[];
  color: string;
  speed: number;
  delay: number;
}) {
  const meshRef = useRef<THREE.Mesh>(null);
  const progress = useRef(delay);

  useFrame((_, delta) => {
    if (!meshRef.current) return;
    progress.current = (progress.current + delta * speed) % 1;
    const t = progress.current;

    // Lerp along the two-point edge
    const pos = new THREE.Vector3().lerpVectors(points[0], points[1], t);
    meshRef.current.position.copy(pos);

    // Fade in/out at endpoints
    const fade = Math.sin(t * Math.PI);
    (meshRef.current.material as THREE.MeshBasicMaterial).opacity = fade * 0.8;
  });

  return (
    <mesh ref={meshRef}>
      <sphereGeometry args={[0.06, 6, 6]} />
      <meshBasicMaterial color={color} transparent opacity={0.8} />
    </mesh>
  );
}

// ============================================
// Single network node
// ============================================
function NetworkNodeMesh({
  position,
  color,
  pulseOffset,
}: {
  position: THREE.Vector3;
  color: string;
  pulseOffset: number;
}) {
  const meshRef = useRef<THREE.Mesh>(null);
  const glowRef = useRef<THREE.Mesh>(null);

  useFrame(({ clock }) => {
    const t = clock.elapsedTime;
    if (!meshRef.current || !glowRef.current) return;

    const pulse = 0.85 + Math.sin(t * 1.5 + pulseOffset) * 0.15;
    meshRef.current.scale.setScalar(pulse);

    const glowPulse = 0.9 + Math.sin(t * 1.2 + pulseOffset) * 0.1;
    glowRef.current.scale.setScalar(glowPulse);
    (glowRef.current.material as THREE.MeshBasicMaterial).opacity =
      0.12 + Math.sin(t + pulseOffset) * 0.06;
  });

  return (
    <group position={position}>
      {/* Glow aura */}
      <mesh ref={glowRef}>
        <sphereGeometry args={[0.28, 12, 12]} />
        <meshBasicMaterial color={color} transparent opacity={0.12} />
      </mesh>
      {/* Core node */}
      <mesh ref={meshRef}>
        <octahedronGeometry args={[0.13, 0]} />
        <meshBasicMaterial color={color} transparent opacity={0.9} />
      </mesh>
    </group>
  );
}

// ============================================
// Edge line between two nodes
// ============================================
function EdgeLine({
  from,
  to,
  color,
}: {
  from: THREE.Vector3;
  to: THREE.Vector3;
  color: string;
}) {
  const lineRef = useRef<THREE.Line>(null);
  const baseOpacity = 0.12 + Math.random() * 0.1;
  const phaseRef = useRef(Math.random() * Math.PI * 2);

  useFrame(({ clock }) => {
    if (!lineRef.current) return;
    const t = clock.elapsedTime;
    const opacity = baseOpacity + Math.sin(t * 0.8 + phaseRef.current) * 0.06;
    (lineRef.current.material as THREE.LineBasicMaterial).opacity = opacity;
  });

  const points = useMemo(
    () => new Float32Array([from.x, from.y, from.z, to.x, to.y, to.z]),
    [from, to]
  );

  return (
    <line ref={lineRef}>
      <bufferGeometry>
        <bufferAttribute attach="attributes-position" count={2} array={points} itemSize={3} />
      </bufferGeometry>
      <lineBasicMaterial color={color} transparent opacity={baseOpacity} />
    </line>
  );
}

// ============================================
// Main NeuralNetwork component
// ============================================
export default function NeuralNetwork({
  primaryColor = '#00D9FF',
  secondaryColor = '#22c55e',
  opacity = 1,
  isMobile = false,
}: NeuralNetworkProps) {
  const groupRef = useRef<THREE.Group>(null);

  // Reduce complexity on mobile
  const cols = isMobile ? 4 : COLS;
  const rows = isMobile ? 3 : ROWS;
  const offsetX = ((cols - 1) * SPACING_X) / 2;
  const offsetY = ((rows - 1) * SPACING_Y) / 2;

  const { nodes, edges } = useMemo(() => {
    const nodes: NetworkNode[] = [];
    let id = 0;

    for (let row = 0; row < rows; row++) {
      for (let col = 0; col < cols; col++) {
        // Add slight jitter for organic look
        const jitterX = (Math.random() - 0.5) * 0.3;
        const jitterY = (Math.random() - 0.5) * 0.3;

        nodes.push({
          id: id++,
          position: new THREE.Vector3(
            col * SPACING_X - offsetX + jitterX,
            row * SPACING_Y - offsetY + jitterY,
            -8
          ),
          layer: col,
        });
      }
    }

    // Connect nodes to adjacent columns (feed-forward style)
    const edges: NetworkEdge[] = [];
    for (let row = 0; row < rows; row++) {
      for (let col = 0; col < cols - 1; col++) {
        const fromIdx = row * cols + col;
        // Connect to 1-2 nodes in next column
        const connections = isMobile ? 1 : Math.floor(Math.random() * 2) + 1;
        for (let c = 0; c < connections; c++) {
          const toRow = Math.min(rows - 1, Math.max(0, row + Math.round((Math.random() - 0.5) * 2)));
          const toIdx = toRow * cols + (col + 1);
          if (fromIdx !== toIdx) {
            edges.push({
              from: fromIdx,
              to: toIdx,
              points: [nodes[fromIdx].position, nodes[toIdx].position],
            });
          }
        }
      }
    }

    return { nodes, edges };
  }, [cols, rows, offsetX, offsetY, isMobile]);

  // Slow drift animation for the whole network
  useFrame(({ clock }) => {
    if (!groupRef.current) return;
    const t = clock.elapsedTime;
    groupRef.current.rotation.y = Math.sin(t * 0.05) * 0.08;
    groupRef.current.rotation.x = Math.cos(t * 0.04) * 0.04;
  });

  // Alternate node colors by layer
  const nodeColor = (layer: number) =>
    layer % 2 === 0 ? primaryColor : secondaryColor;

  return (
    <group ref={groupRef} visible={opacity > 0}>
      {/* Nodes */}
      {nodes.map((node) => (
        <NetworkNodeMesh
          key={node.id}
          position={node.position}
          color={nodeColor(node.layer)}
          pulseOffset={node.id * 0.7}
        />
      ))}

      {/* Edges */}
      {edges.map((edge, i) => (
        <EdgeLine
          key={i}
          from={edge.points[0]}
          to={edge.points[1]}
          color={primaryColor}
        />
      ))}

      {/* Traveling particles along select edges */}
      {edges
        .filter((_, i) => i % 3 === 0) // Only every 3rd edge for performance
        .map((edge, i) => (
          <EdgeParticle
            key={`particle-${i}`}
            points={edge.points}
            color={i % 2 === 0 ? primaryColor : secondaryColor}
            speed={0.15 + Math.random() * 0.2}
            delay={Math.random()}
          />
        ))}
    </group>
  );
}
