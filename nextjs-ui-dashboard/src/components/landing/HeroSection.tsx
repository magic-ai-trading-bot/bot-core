import { lazy, Suspense, useState, useEffect } from "react";
import { Button } from "@/components/ui/button";
import { Badge } from "@/components/ui/badge";
import { Dialog, DialogContent, DialogHeader, DialogTitle } from "@/components/ui/dialog";
import { TrendingUp, Zap, Shield, BarChart3, Play, Users, DollarSign, Activity } from "lucide-react";
import { useTranslation } from "react-i18next";
import ErrorBoundary from "@/components/ErrorBoundary";

// Lazy load the 3D component to reduce initial bundle size
const Hero3D = lazy(() => import("./Hero3D").then(module => ({ default: module.Hero3D })));

// Animated counter hook
function useAnimatedCounter(end: number, duration: number = 2000, prefix: string = "", suffix: string = "") {
  const [count, setCount] = useState(0);

  useEffect(() => {
    let startTime: number;
    let animationFrame: number;

    const animate = (currentTime: number) => {
      if (!startTime) startTime = currentTime;
      const progress = Math.min((currentTime - startTime) / duration, 1);
      const easeOut = 1 - Math.pow(1 - progress, 3);
      setCount(Math.floor(easeOut * end));

      if (progress < 1) {
        animationFrame = requestAnimationFrame(animate);
      }
    };

    animationFrame = requestAnimationFrame(animate);
    return () => cancelAnimationFrame(animationFrame);
  }, [end, duration]);

  return `${prefix}${count.toLocaleString()}${suffix}`;
}

// Live stats component with animations
function LiveStats() {
  const activeTraders = useAnimatedCounter(2847, 2500);
  const totalVolume = useAnimatedCounter(52, 2000, "$", "M+");
  const tradesExecuted = useAnimatedCounter(15420, 3000);

  return (
    <div className="grid grid-cols-3 gap-4 max-w-2xl mx-auto mb-8">
      <div className="text-center p-3 rounded-lg bg-card/30 backdrop-blur border border-border/30">
        <div className="flex items-center justify-center gap-1 mb-1">
          <Users className="w-4 h-4 text-profit" />
          <span className="text-xl md:text-2xl font-bold text-profit">{activeTraders}</span>
        </div>
        <p className="text-xs text-muted-foreground">Active Traders</p>
      </div>
      <div className="text-center p-3 rounded-lg bg-card/30 backdrop-blur border border-border/30">
        <div className="flex items-center justify-center gap-1 mb-1">
          <DollarSign className="w-4 h-4 text-primary" />
          <span className="text-xl md:text-2xl font-bold text-primary">{totalVolume}</span>
        </div>
        <p className="text-xs text-muted-foreground">Trading Volume</p>
      </div>
      <div className="text-center p-3 rounded-lg bg-card/30 backdrop-blur border border-border/30">
        <div className="flex items-center justify-center gap-1 mb-1">
          <Activity className="w-4 h-4 text-info" />
          <span className="text-xl md:text-2xl font-bold text-info">{tradesExecuted}</span>
        </div>
        <p className="text-xs text-muted-foreground">Trades Today</p>
      </div>
    </div>
  );
}

export function HeroSection() {
  const { t } = useTranslation();
  const [isVideoOpen, setIsVideoOpen] = useState(false);

  const scrollToPricing = () => {
    const element = document.getElementById('pricing');
    if (element) {
      element.scrollIntoView({ behavior: 'smooth' });
    }
  };

  return (
    <section className="relative min-h-screen flex items-center justify-center overflow-hidden bg-gradient-to-b from-background via-background/95 to-background">
      <ErrorBoundary fallback={<div className="absolute inset-0 bg-gradient-to-br from-background/50 to-background" />}>
        <Suspense fallback={<div className="absolute inset-0" />}>
          <Hero3D />
        </Suspense>
      </ErrorBoundary>
      
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
        
        <div className="flex flex-col sm:flex-row gap-4 justify-center mb-8">
          <Button size="lg" className="bg-profit hover:bg-profit/90 text-profit-foreground px-8 py-4 text-lg" onClick={scrollToPricing}>
            {t('hero.startTrading')}
          </Button>
          <Button variant="outline" size="lg" className="px-8 py-4 text-lg group" onClick={() => setIsVideoOpen(true)}>
            <Play className="w-5 h-5 mr-2 group-hover:scale-110 transition-transform" />
            {t('hero.watchDemo')}
          </Button>
        </div>

        {/* Live Trading Stats */}
        <LiveStats />

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

      {/* Video Demo Modal */}
      <Dialog open={isVideoOpen} onOpenChange={setIsVideoOpen}>
        <DialogContent className="max-w-4xl p-0 overflow-hidden bg-card border-border">
          <DialogHeader className="p-4 pb-0">
            <DialogTitle className="flex items-center gap-2">
              <Play className="w-5 h-5 text-profit" />
              BotCore Demo - AI Trading in Action
            </DialogTitle>
          </DialogHeader>
          <div className="aspect-video bg-black/90 flex items-center justify-center">
            {/* Placeholder for demo video */}
            <div className="text-center p-8">
              <div className="w-20 h-20 rounded-full bg-profit/20 flex items-center justify-center mx-auto mb-4 animate-pulse">
                <Play className="w-10 h-10 text-profit" />
              </div>
              <h3 className="text-xl font-semibold mb-2">Demo Video Coming Soon</h3>
              <p className="text-muted-foreground text-sm max-w-md mx-auto">
                Watch our AI trading bot analyze markets, generate signals, and execute trades automatically.
                <br />
                <span className="text-profit font-medium">Average 73% accuracy rate</span>
              </p>
              <div className="flex justify-center gap-4 mt-6">
                <Badge variant="outline" className="bg-profit/10 text-profit border-profit/20">
                  Real-time Analysis
                </Badge>
                <Badge variant="outline" className="bg-info/10 text-info border-info/20">
                  Live Trading
                </Badge>
                <Badge variant="outline" className="bg-primary/10 text-primary border-primary/20">
                  Risk Management
                </Badge>
              </div>
            </div>
          </div>
        </DialogContent>
      </Dialog>
    </section>
  );
}