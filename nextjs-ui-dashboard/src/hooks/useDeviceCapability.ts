'use client';

import { useEffect, useState } from 'react';

interface DeviceCapability {
  supports3D: boolean;
  gpuTier: 'low' | 'medium' | 'high';
  isMobile: boolean;
  isTablet: boolean;
  webglVersion: 1 | 2 | null;
  maxTextureSize: number;
  supportsWebGPU: boolean;
}

/**
 * Detect device capabilities for 3D rendering
 * Returns WebGL support, GPU tier, device type
 */
export function useDeviceCapability(): DeviceCapability {
  const [capability, setCapability] = useState<DeviceCapability>({
    supports3D: true, // Optimistic default
    gpuTier: 'medium',
    isMobile: false,
    isTablet: false,
    webglVersion: null,
    maxTextureSize: 0,
    supportsWebGPU: false,
  });

  useEffect(() => {
    const detectCapabilities = () => {
      // Detect mobile/tablet
      const isMobile = /iPhone|iPod|Android.*Mobile/i.test(navigator.userAgent);
      const isTablet = /iPad|Android(?!.*Mobile)/i.test(navigator.userAgent);

      // Detect WebGL support
      let webglVersion: 1 | 2 | null = null;
      let maxTextureSize = 0;

      try {
        const canvas = document.createElement('canvas');

        // Try WebGL2 first
        const gl2 = canvas.getContext('webgl2');
        if (gl2) {
          webglVersion = 2;
          maxTextureSize = gl2.getParameter(gl2.MAX_TEXTURE_SIZE);
        } else {
          // Fallback to WebGL1
          const gl1 =
            canvas.getContext('webgl') || canvas.getContext('experimental-webgl');
          if (gl1) {
            webglVersion = 1;
            maxTextureSize = (gl1 as WebGLRenderingContext).getParameter(
              (gl1 as WebGLRenderingContext).MAX_TEXTURE_SIZE
            );
          }
        }
      } catch (e) {
        console.warn('WebGL detection failed:', e);
      }

      const supports3D = webglVersion !== null;

      // Detect GPU tier based on multiple factors
      let gpuTier: 'low' | 'medium' | 'high' = 'medium';

      if (!supports3D) {
        gpuTier = 'low';
      } else {
        // Use WebGL renderer info if available
        try {
          const canvas = document.createElement('canvas');
          const gl = canvas.getContext('webgl') || canvas.getContext('webgl2');
          if (gl) {
            const debugInfo = gl.getExtension('WEBGL_debug_renderer_info');
            if (debugInfo) {
              const renderer = gl.getParameter(debugInfo.UNMASKED_RENDERER_WEBGL);
              const rendererLower = renderer.toLowerCase();

              // High-end GPUs
              if (
                rendererLower.includes('nvidia') ||
                rendererLower.includes('radeon') ||
                rendererLower.includes('apple m1') ||
                rendererLower.includes('apple m2') ||
                rendererLower.includes('apple m3')
              ) {
                gpuTier = 'high';
              }
              // Low-end GPUs
              else if (
                rendererLower.includes('intel hd') ||
                rendererLower.includes('intel(r) hd') ||
                rendererLower.includes('mali') ||
                rendererLower.includes('adreno 3') ||
                rendererLower.includes('adreno 4')
              ) {
                gpuTier = 'low';
              }
            }
          }
        } catch (e) {
          console.warn('GPU detection failed:', e);
        }

        // Additional heuristics
        if (isMobile && maxTextureSize < 4096) {
          gpuTier = 'low';
        } else if (!isMobile && maxTextureSize >= 8192 && webglVersion === 2) {
          gpuTier = 'high';
        }
      }

      // Detect WebGPU support (future-proofing)
      const supportsWebGPU = 'gpu' in navigator;

      setCapability({
        supports3D,
        gpuTier,
        isMobile,
        isTablet,
        webglVersion,
        maxTextureSize,
        supportsWebGPU,
      });
    };

    detectCapabilities();
  }, []);

  return capability;
}

/**
 * Get recommended settings based on device capability
 */
export function useRecommendedSettings() {
  const capability = useDeviceCapability();

  return {
    // Particle count
    particleCount:
      capability.gpuTier === 'high'
        ? 5000
        : capability.gpuTier === 'medium'
          ? 3000
          : 1000,

    // Shadow quality
    enableShadows: capability.gpuTier !== 'low',

    // Antialiasing
    enableAA: capability.gpuTier !== 'low',

    // DPR (Device Pixel Ratio)
    pixelRatio:
      capability.gpuTier === 'high'
        ? 2
        : capability.gpuTier === 'medium'
          ? 1.5
          : 1,

    // Post-processing
    enablePostProcessing: capability.gpuTier === 'high',

    // Animation complexity
    animationComplexity:
      capability.gpuTier === 'high'
        ? 'full'
        : capability.gpuTier === 'medium'
          ? 'reduced'
          : 'minimal',

    // Use fallback
    useFallback: !capability.supports3D,
  };
}
