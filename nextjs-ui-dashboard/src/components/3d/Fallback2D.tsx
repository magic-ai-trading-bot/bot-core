'use client';

import { useEffect, useState } from 'react';

interface Fallback2DProps {
  variant?: 'particles' | 'gradient' | 'waves';
  className?: string;
}

/**
 * Static gradient background with CSS-only animations
 * Fallback for devices that don't support WebGL/3D
 */
export default function Fallback2D({ variant = 'gradient', className = '' }: Fallback2DProps) {
  const [mounted, setMounted] = useState(false);

  useEffect(() => {
    setMounted(true);
  }, []);

  if (!mounted) {
    return (
      <div className={`absolute inset-0 bg-gradient-to-br from-slate-950 via-slate-900 to-slate-950 ${className}`} />
    );
  }

  if (variant === 'particles') {
    return (
      <div className={`absolute inset-0 overflow-hidden ${className}`}>
        {/* Animated particle dots */}
        <div className="absolute inset-0 bg-slate-950">
          <div className="absolute inset-0 opacity-30">
            {[...Array(50)].map((_, i) => (
              <div
                key={i}
                className="absolute w-1 h-1 bg-white rounded-full animate-pulse"
                style={{
                  left: `${Math.random() * 100}%`,
                  top: `${Math.random() * 100}%`,
                  animationDelay: `${Math.random() * 3}s`,
                  animationDuration: `${2 + Math.random() * 3}s`,
                }}
              />
            ))}
          </div>
        </div>
      </div>
    );
  }

  if (variant === 'waves') {
    return (
      <div className={`absolute inset-0 overflow-hidden ${className}`}>
        {/* Animated wave patterns */}
        <div className="absolute inset-0 bg-slate-950">
          <div className="absolute inset-0 opacity-20">
            <div className="absolute inset-0 bg-gradient-to-r from-transparent via-blue-500 to-transparent animate-wave-slow" />
            <div className="absolute inset-0 bg-gradient-to-r from-transparent via-purple-500 to-transparent animate-wave-medium" />
            <div className="absolute inset-0 bg-gradient-to-r from-transparent via-pink-500 to-transparent animate-wave-fast" />
          </div>
        </div>
      </div>
    );
  }

  // Default: gradient
  return (
    <div className={`absolute inset-0 ${className}`}>
      {/* Multi-layer gradient with animation */}
      <div className="absolute inset-0 bg-gradient-to-br from-slate-950 via-slate-900 to-slate-950" />

      {/* Animated gradient orbs */}
      <div className="absolute inset-0 overflow-hidden">
        <div
          className="absolute -top-1/2 -left-1/2 w-full h-full bg-gradient-radial from-blue-900/30 via-transparent to-transparent animate-pulse-slow"
          style={{ animationDuration: '8s' }}
        />
        <div
          className="absolute -bottom-1/2 -right-1/2 w-full h-full bg-gradient-radial from-purple-900/30 via-transparent to-transparent animate-pulse-slow"
          style={{ animationDuration: '10s', animationDelay: '2s' }}
        />
        <div
          className="absolute top-1/2 left-1/2 -translate-x-1/2 -translate-y-1/2 w-3/4 h-3/4 bg-gradient-radial from-pink-900/20 via-transparent to-transparent animate-pulse-slow"
          style={{ animationDuration: '12s', animationDelay: '4s' }}
        />
      </div>

      {/* Grid overlay */}
      <div
        className="absolute inset-0 opacity-10"
        style={{
          backgroundImage: `
            linear-gradient(to right, rgba(255,255,255,0.1) 1px, transparent 1px),
            linear-gradient(to bottom, rgba(255,255,255,0.1) 1px, transparent 1px)
          `,
          backgroundSize: '50px 50px',
        }}
      />

      {/* Vignette effect */}
      <div className="absolute inset-0 bg-gradient-radial from-transparent via-transparent to-slate-950/60" />
    </div>
  );
}
