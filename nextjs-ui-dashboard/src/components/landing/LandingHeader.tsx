import { PremiumButton } from "@/styles/luxury-design-system";
import { useState } from "react";
import { Menu, X, LayoutDashboard } from "lucide-react";
import { useTranslation } from "react-i18next";
import { LanguageSelector } from "@/components/LanguageSelector";
import { useNavigate, Link } from "react-router-dom";
import { Logo } from "@/components/ui/Logo";
import { useAuth } from "@/contexts/AuthContext";

export function LandingHeader() {
  const [isMenuOpen, setIsMenuOpen] = useState(false);
  const { t } = useTranslation();
  const navigate = useNavigate();
  const { isAuthenticated } = useAuth();

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
        <div className="flex items-center justify-between h-16 relative">
          {/* Logo - Click to go home */}
          <Link to="/" className="flex-shrink-0">
            <Logo size="md" />
          </Link>

          {/* Desktop Navigation - Absolutely centered */}
          <nav className="hidden md:flex items-center gap-6 absolute left-1/2 -translate-x-1/2">
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
            {isAuthenticated ? (
              <PremiumButton size="sm" onClick={() => navigate('/dashboard')}>
                <LayoutDashboard className="w-4 h-4 mr-2" />
                Dashboard
              </PremiumButton>
            ) : (
              <>
                <PremiumButton variant="ghost" size="sm" onClick={handleSignIn}>
                  {t('nav.signIn')}
                </PremiumButton>
                <PremiumButton size="sm" onClick={handleStartTrial}>
                  {t('nav.startTrial')}
                </PremiumButton>
              </>
            )}
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
                {isAuthenticated ? (
                  <PremiumButton size="sm" className="justify-start" onClick={() => navigate('/dashboard')}>
                    <LayoutDashboard className="w-4 h-4 mr-2" />
                    Dashboard
                  </PremiumButton>
                ) : (
                  <>
                    <PremiumButton variant="ghost" size="sm" className="justify-start" onClick={handleSignIn}>
                      {t('nav.signIn')}
                    </PremiumButton>
                    <PremiumButton size="sm" className="justify-start" onClick={handleStartTrial}>
                      {t('nav.startTrial')}
                    </PremiumButton>
                  </>
                )}
              </div>
            </nav>
          </div>
        )}
      </div>
    </header>
  );
}