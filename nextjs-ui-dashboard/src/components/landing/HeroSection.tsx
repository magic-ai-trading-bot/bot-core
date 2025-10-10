import { lazy, Suspense } from "react";
import { Button } from "@/components/ui/button";
import { Badge } from "@/components/ui/badge";
import { TrendingUp, Zap, Shield, BarChart3 } from "lucide-react";
import { useTranslation } from "react-i18next";

// Lazy load the 3D component to reduce initial bundle size
const Hero3D = lazy(() => import("./Hero3D").then(module => ({ default: module.Hero3D })));

export function HeroSection() {
  const { t } = useTranslation();

  const scrollToPricing = () => {
    const element = document.getElementById('pricing');
    if (element) {
      element.scrollIntoView({ behavior: 'smooth' });
    }
  };

  return (
    <section className="relative min-h-screen flex items-center justify-center overflow-hidden bg-gradient-to-b from-background via-background/95 to-background">
      <Suspense fallback={<div className="absolute inset-0" />}>
        <Hero3D />
      </Suspense>
      
      <div className="relative z-10 container mx-auto px-4 text-center">
        <Badge variant="outline" className="mb-6 bg-primary/10 text-primary border-primary/20">
          <Zap className="w-3 h-3 mr-1" />
          {t('hero.badge')}
        </Badge>
        
        <h1 className="text-4xl md:text-6xl lg:text-7xl font-bold mb-6 bg-gradient-to-r from-foreground via-primary to-accent bg-clip-text text-transparent">
          {t('hero.title')}
          <br />
          <span className="text-profit">{t('hero.subtitle')}</span>
        </h1>
        
        <p className="text-lg md:text-xl text-muted-foreground mb-8 max-w-3xl mx-auto leading-relaxed">
          {t('hero.description')}
          <span className="text-primary font-semibold">{t('hero.highlight')}</span>
        </p>
        
        <div className="flex flex-col sm:flex-row gap-4 justify-center mb-12">
          <Button size="lg" className="bg-profit hover:bg-profit/90 text-profit-foreground px-8 py-4 text-lg" onClick={scrollToPricing}>
            {t('hero.startTrading')}
          </Button>
          <Button variant="outline" size="lg" className="px-8 py-4 text-lg">
            {t('hero.watchDemo')}
          </Button>
        </div>
        
        {/* Key Features Preview */}
        <div className="grid grid-cols-1 md:grid-cols-3 gap-6 max-w-4xl mx-auto">
          <div className="flex items-center justify-center gap-3 p-4 rounded-lg bg-card/50 backdrop-blur border border-border/50">
            <BarChart3 className="w-5 h-5 text-primary" />
            <span className="text-sm font-medium">{t('hero.aiChart')}</span>
          </div>
          <div className="flex items-center justify-center gap-3 p-4 rounded-lg bg-card/50 backdrop-blur border border-border/50">
            <TrendingUp className="w-5 h-5 text-profit" />
            <span className="text-sm font-medium">{t('hero.autoTrading')}</span>
          </div>
          <div className="flex items-center justify-center gap-3 p-4 rounded-lg bg-card/50 backdrop-blur border border-border/50">
            <Shield className="w-5 h-5 text-info" />
            <span className="text-sm font-medium">{t('hero.riskManagement')}</span>
          </div>
        </div>
      </div>
      
      {/* Scroll indicator */}
      <div className="absolute bottom-8 left-1/2 transform -translate-x-1/2 animate-bounce">
        <div className="w-6 h-10 border-2 border-muted-foreground/30 rounded-full flex justify-center">
          <div className="w-1 h-2 bg-muted-foreground/50 rounded-full mt-2 animate-pulse"></div>
        </div>
      </div>
    </section>
  );
}