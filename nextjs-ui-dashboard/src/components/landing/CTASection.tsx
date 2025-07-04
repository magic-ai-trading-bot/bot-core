import { Button } from "@/components/ui/button";
import { Badge } from "@/components/ui/badge";
import { Rocket, ArrowRight, Shield, Zap } from "lucide-react";
import { useTranslation } from "react-i18next";

export function CTASection() {
  const { t } = useTranslation();

  const scrollToPricing = () => {
    const element = document.getElementById('pricing');
    if (element) {
      element.scrollIntoView({ behavior: 'smooth' });
    }
  };

  return (
    <section className="py-24 bg-gradient-to-r from-primary/10 via-accent/10 to-profit/10 relative overflow-hidden">
      {/* Background decorative elements */}
      <div className="absolute inset-0 bg-[radial-gradient(ellipse_at_center,_var(--tw-gradient-stops))] from-primary/5 via-transparent to-transparent"></div>
      
      <div className="container mx-auto px-4 text-center relative z-10">
        <Badge variant="outline" className="mb-6 bg-profit/10 text-profit border-profit/20">
          <Rocket className="w-3 h-3 mr-1" />
          {t('cta.badge')}
        </Badge>
        
        <h2 className="text-3xl md:text-5xl font-bold mb-6">
          {t('cta.title')} <span className="text-profit">{t('cta.subtitle')}</span> {t('cta.todayText')}
        </h2>
        
        <p className="text-lg text-muted-foreground mb-8 max-w-2xl mx-auto">
          {t('cta.description')}
        </p>
        
        <div className="flex flex-col sm:flex-row gap-4 justify-center mb-12">
          <Button size="lg" className="bg-profit hover:bg-profit/90 text-profit-foreground px-8 py-4 text-lg group" onClick={scrollToPricing}>
            {t('cta.startTrial')}
            <ArrowRight className="w-5 h-5 ml-2 group-hover:translate-x-1 transition-transform" />
          </Button>
          <Button variant="outline" size="lg" className="px-8 py-4 text-lg">
            {t('cta.scheduleDemo')}
          </Button>
        </div>
        
        {/* Trust indicators */}
        <div className="grid grid-cols-1 md:grid-cols-3 gap-6 max-w-3xl mx-auto">
          <div className="flex items-center justify-center gap-3 p-4">
            <Shield className="w-5 h-5 text-info" />
            <span className="text-sm">{t('cta.bankSecurity')}</span>
          </div>
          <div className="flex items-center justify-center gap-3 p-4">
            <Zap className="w-5 h-5 text-warning" />
            <span className="text-sm">{t('cta.quickSetup')}</span>
          </div>
          <div className="flex items-center justify-center gap-3 p-4">
            <Rocket className="w-5 h-5 text-profit" />
            <span className="text-sm">{t('cta.noCommitment')}</span>
          </div>
        </div>
        
        <div className="mt-8 text-sm text-muted-foreground">
          <p>{t('cta.footer')}</p>
        </div>
      </div>
    </section>
  );
}