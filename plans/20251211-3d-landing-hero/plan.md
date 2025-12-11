# 3D Landing Page Hero - Bot Core

**Created**: 2025-12-11
**Status**: In Progress
**Priority**: High

## Overview

Add immersive 3D elements to the Bot Core landing page hero section to create a stunning, modern crypto trading platform experience.

## Concept

**Theme**: AI-Powered Neural Network Trading
- Neural network grid background representing AI trading
- Floating data particles showing market flow
- Glowing orbs representing trade opportunities
- Interactive elements responding to mouse movement

## Phases

| Phase | Name | Status | Progress |
|-------|------|--------|----------|
| 01 | [Hero Background 3D](./phase-01-hero-background.md) | In Progress | 0% |
| 02 | [Particle System](./phase-02-particle-system.md) | Pending | 0% |
| 03 | [Integration & Polish](./phase-03-integration.md) | Pending | 0% |

## Technical Stack

- Three.js + React Three Fiber (already installed)
- Custom GLSL shaders for effects
- Framer Motion for scroll animations
- useThemeColors() for theme support

## Design Principles

1. **Performance First**: Target 60fps desktop, 30fps mobile
2. **Theme Aware**: Support light/dark mode seamlessly
3. **Non-Intrusive**: 3D enhances, doesn't distract from content
4. **Mobile Optimized**: Graceful degradation on low-end devices

## Files to Create/Modify

- `src/components/3d/HeroScene3D.tsx` - Main 3D scene
- `src/components/3d/NeuralNetwork.tsx` - Neural network grid
- `src/components/3d/DataParticles.tsx` - Particle system
- `src/pages/Index.tsx` - Integration point

## Success Criteria

- [ ] 3D scene loads in under 2s
- [ ] Maintains 60fps on desktop
- [ ] Works in both light/dark themes
- [ ] Graceful fallback for WebGL issues
- [ ] Mobile responsive with reduced effects
