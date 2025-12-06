# Phase 3: Landing Page Redesign

## Context
- **Parent Plan**: [plan.md](./plan.md)
- **Dependencies**: Phase 1 (Design System), Phase 2 (Navigation)
- **Research**: [Landing & Navigation](./research/researcher-03-landing-navigation.md), [3D Visualization](./research/researcher-02-3d-visualization-design.md)

## Overview
| Field | Value |
|-------|-------|
| Priority | P0 - Critical |
| Status | Pending |
| Est. Time | 3-4 days |
| Description | Award-winning landing page with 3D hero, feature sections, trust signals, and CTAs |

## Key Insights
- Hero: 3D animated element (trading chart/globe) with gradient background
- Above-fold: Value prop + CTA + trust signals
- Social proof: Trading volume, users count, performance stats
- Sticky CTA on scroll
- Skeleton loading for perceived performance

## Requirements

### Functional
- Animated 3D hero section
- Feature showcase with icons/animations
- Live trading stats (volume, users)
- Testimonials/social proof
- Pricing section (if applicable)
- Footer with links
- Mobile-optimized

### Non-Functional
- First paint < 1.5s
- Interactive < 3s
- 3D fallback for low-end devices
- SEO optimized

## Architecture

```
LandingPage/
├── HeroSection/
│   ├── Hero3DScene (Three.js)
│   ├── HeroContent (headline, subtext, CTA)
│   └── HeroStats (live numbers)
├── FeaturesSection/
│   ├── FeatureCard (icon, title, description)
│   └── FeatureGrid
├── HowItWorksSection/
│   └── StepCard (numbered steps)
├── StatsSection/
│   └── AnimatedCounter
├── TestimonialsSection/
│   └── TestimonialCarousel
├── CTASection/
│   └── FinalCTA
└── Footer/
    └── FooterLinks
```

## Related Code Files

### Create
- `/Users/dungngo97/Documents/bot-core/nextjs-ui-dashboard/src/pages/Landing.tsx` (rewrite)
- `/Users/dungngo97/Documents/bot-core/nextjs-ui-dashboard/src/components/landing/HeroSection.tsx`
- `/Users/dungngo97/Documents/bot-core/nextjs-ui-dashboard/src/components/landing/Hero3DScene.tsx`
- `/Users/dungngo97/Documents/bot-core/nextjs-ui-dashboard/src/components/landing/FeaturesSection.tsx`
- `/Users/dungngo97/Documents/bot-core/nextjs-ui-dashboard/src/components/landing/FeatureCard.tsx`
- `/Users/dungngo97/Documents/bot-core/nextjs-ui-dashboard/src/components/landing/StatsSection.tsx`
- `/Users/dungngo97/Documents/bot-core/nextjs-ui-dashboard/src/components/landing/AnimatedCounter.tsx`
- `/Users/dungngo97/Documents/bot-core/nextjs-ui-dashboard/src/components/landing/TestimonialsSection.tsx`
- `/Users/dungngo97/Documents/bot-core/nextjs-ui-dashboard/src/components/landing/CTASection.tsx`
- `/Users/dungngo97/Documents/bot-core/nextjs-ui-dashboard/src/components/landing/Footer.tsx`

### Modify
- `/Users/dungngo97/Documents/bot-core/nextjs-ui-dashboard/src/App.tsx` - Update landing route

## Implementation Steps

1. **Create Hero3DScene**
   ```tsx
   // React Three Fiber scene
   // Options: Floating coins, trading chart 3D, globe with connections
   // Fallback: Gradient animation for low-end devices
   ```

2. **Create HeroSection**
   - Headline: "Trade Smarter with AI-Powered Insights"
   - Subtext: Value proposition
   - CTA buttons: "Start Trading" / "Try Paper Trading"
   - Live stats badges

3. **Create FeaturesSection**
   - Grid of 4-6 feature cards
   - Icons with hover animations
   - Features: AI Signals, Paper Trading, Real-time Data, Risk Management

4. **Create StatsSection**
   - Animated counters (Framer Motion)
   - Stats: Total volume, Active users, Accuracy rate, Uptime

5. **Create TestimonialsSection**
   - Carousel of testimonials
   - Avatar, name, quote
   - Auto-rotate with pause on hover

6. **Create CTASection**
   - Final conversion push
   - "Ready to start?" + CTA button

7. **Create Footer**
   - Links: About, Docs, Support, Legal
   - Social media icons
   - Copyright

8. **Optimize Performance**
   - Lazy load 3D scene
   - Skeleton screens
   - Image optimization

## Todo List

- [ ] Design hero section mockup
- [ ] Create Hero3DScene with Three.js
- [ ] Create HeroSection with CTAs
- [ ] Create FeatureCard component
- [ ] Create FeaturesSection grid
- [ ] Create AnimatedCounter component
- [ ] Create StatsSection
- [ ] Create TestimonialsSection carousel
- [ ] Create CTASection
- [ ] Create Footer
- [ ] Assemble Landing page
- [ ] Add scroll animations (Framer Motion)
- [ ] Optimize for mobile
- [ ] Add 3D fallback for low-end devices
- [ ] Performance testing (Lighthouse)
- [ ] Write tests

## Success Criteria

- [ ] Lighthouse Performance > 90
- [ ] 3D scene renders without blocking UI
- [ ] All sections responsive on mobile
- [ ] CTAs clearly visible above fold
- [ ] Animations smooth (60fps)
- [ ] Fallback works on low-end devices

## Risk Assessment

| Risk | Impact | Mitigation |
|------|--------|------------|
| 3D performance on mobile | High | Progressive enhancement, fallback |
| Large bundle size | Medium | Lazy load Three.js |
| SEO impact from SPA | Medium | Meta tags, structured data |

## Security Considerations
- No sensitive data on landing page
- External links open in new tab with rel="noopener"

## Next Steps
→ Phase 4: Dashboard Redesign
