import { Button } from "@/components/ui/button";
import { Badge } from "@/components/ui/badge";
import { useState } from "react";
import { Menu, X } from "lucide-react";
import { useTranslation } from "react-i18next";
import { LanguageSelector } from "@/components/LanguageSelector";
import { useNavigate } from "react-router-dom";

export function LandingHeader() {
  const [isMenuOpen, setIsMenuOpen] = useState(false);
  const { t } = useTranslation();
  const navigate = useNavigate();

  const scrollToSection = (sectionId: string) => {
    const element = document.getElementById(sectionId);
    if (element) {
      element.scrollIntoView({ behavior: 'smooth' });
    }
    setIsMenuOpen(false);
  };

  const handleSignIn = () => {
    navigate('/login');
  };

  const handleStartTrial = () => {
    scrollToSection('pricing');
  };

  return (
    <header className="fixed top-0 w-full z-50 bg-background/80 backdrop-blur-md border-b border-border/50">
      <div className="container mx-auto px-4">
        <div className="flex items-center justify-between h-16">
          {/* Logo */}
          <div className="flex items-center gap-2">
            <div className="w-8 h-8 bg-gradient-to-br from-primary to-accent rounded-lg flex items-center justify-center">
              <span className="text-primary-foreground font-bold text-sm">BT</span>
            </div>
            <div>
              <h1 className="text-lg font-bold">CryptoBot AI</h1>
              <p className="text-xs text-muted-foreground hidden sm:block">AI-Powered Trading</p>
            </div>
          </div>

          {/* Desktop Navigation */}
          <nav className="hidden md:flex items-center gap-6">
            <button 
              onClick={() => scrollToSection('features')}
              className="text-sm text-muted-foreground hover:text-foreground transition-colors"
            >
              {t('nav.features')}
            </button>
            <button 
              onClick={() => scrollToSection('pricing')}
              className="text-sm text-muted-foreground hover:text-foreground transition-colors"
            >
              {t('nav.pricing')}
            </button>
            <button 
              onClick={() => scrollToSection('testimonials')}
              className="text-sm text-muted-foreground hover:text-foreground transition-colors"
            >
              {t('nav.reviews')}
            </button>
            <button 
              onClick={() => scrollToSection('faq')}
              className="text-sm text-muted-foreground hover:text-foreground transition-colors"
            >
              {t('nav.faq')}
            </button>
          </nav>

          {/* CTA Buttons */}
          <div className="hidden md:flex items-center gap-3">
            <LanguageSelector />
            <Button variant="ghost" size="sm" onClick={handleSignIn}>
              {t('nav.signIn')}
            </Button>
            <Button size="sm" className="bg-profit hover:bg-profit/90 text-profit-foreground" onClick={handleStartTrial}>
              {t('nav.startTrial')}
            </Button>
          </div>

          {/* Mobile Menu Button */}
          <button
            onClick={() => setIsMenuOpen(!isMenuOpen)}
            className="md:hidden p-2"
          >
            {isMenuOpen ? <X className="w-5 h-5" /> : <Menu className="w-5 h-5" />}
          </button>
        </div>

        {/* Mobile Menu */}
        {isMenuOpen && (
          <div className="md:hidden py-4 border-t border-border/50">
            <nav className="flex flex-col gap-4">
              <button 
                onClick={() => scrollToSection('features')}
                className="text-left text-sm text-muted-foreground hover:text-foreground transition-colors"
              >
                {t('nav.features')}
              </button>
              <button 
                onClick={() => scrollToSection('pricing')}
                className="text-left text-sm text-muted-foreground hover:text-foreground transition-colors"
              >
                {t('nav.pricing')}
              </button>
              <button 
                onClick={() => scrollToSection('testimonials')}
                className="text-left text-sm text-muted-foreground hover:text-foreground transition-colors"
              >
                {t('nav.reviews')}
              </button>
              <button 
                onClick={() => scrollToSection('faq')}
                className="text-left text-sm text-muted-foreground hover:text-foreground transition-colors"
              >
                {t('nav.faq')}
              </button>
              <div className="flex flex-col gap-3 pt-4 border-t border-border/50">
                <LanguageSelector />
                <Button variant="ghost" size="sm" className="justify-start" onClick={handleSignIn}>
                  {t('nav.signIn')}
                </Button>
                <Button size="sm" className="bg-profit hover:bg-profit/90 text-profit-foreground justify-start" onClick={handleStartTrial}>
                  {t('nav.startTrial')}
                </Button>
              </div>
            </nav>
          </div>
        )}
      </div>
    </header>
  );
}