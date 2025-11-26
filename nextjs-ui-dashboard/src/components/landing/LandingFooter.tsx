import { Badge } from "@/components/ui/badge";
import { Mail, Twitter, Linkedin, Github } from "lucide-react";
import { Logo } from "@/components/ui/Logo";
import { Link } from "react-router-dom";

export function LandingFooter() {
  return (
    <footer className="bg-card/30 border-t border-border/50">
      <div className="container mx-auto px-4 py-12">
        <div className="grid grid-cols-1 md:grid-cols-4 gap-8">
          {/* Company Info */}
          <div className="space-y-4">
            <Link to="/">
              <Logo size="sm" />
            </Link>
            <p className="text-sm text-muted-foreground">
              The most advanced AI trading platform for cryptocurrency futures, 
              trusted by thousands of traders worldwide.
            </p>
            <div className="flex gap-3">
              <a href="#" className="text-muted-foreground hover:text-primary transition-colors">
                <Twitter className="w-4 h-4" />
              </a>
              <a href="#" className="text-muted-foreground hover:text-primary transition-colors">
                <Linkedin className="w-4 h-4" />
              </a>
              <a href="#" className="text-muted-foreground hover:text-primary transition-colors">
                <Github className="w-4 h-4" />
              </a>
              <a href="#" className="text-muted-foreground hover:text-primary transition-colors">
                <Mail className="w-4 h-4" />
              </a>
            </div>
          </div>

          {/* Product */}
          <div className="space-y-4">
            <h4 className="font-semibold">Product</h4>
            <ul className="space-y-2 text-sm text-muted-foreground">
              <li><a href="#" className="hover:text-foreground transition-colors">Features</a></li>
              <li><a href="#" className="hover:text-foreground transition-colors">Pricing</a></li>
              <li><a href="#" className="hover:text-foreground transition-colors">API Documentation</a></li>
              <li><a href="#" className="hover:text-foreground transition-colors">Trading Strategies</a></li>
              <li><a href="#" className="hover:text-foreground transition-colors">Performance Analytics</a></li>
            </ul>
          </div>

          {/* Support */}
          <div className="space-y-4">
            <h4 className="font-semibold">Support</h4>
            <ul className="space-y-2 text-sm text-muted-foreground">
              <li><a href="#" className="hover:text-foreground transition-colors">Help Center</a></li>
              <li><a href="#" className="hover:text-foreground transition-colors">Getting Started</a></li>
              <li><a href="#" className="hover:text-foreground transition-colors">Contact Support</a></li>
              <li><a href="#" className="hover:text-foreground transition-colors">Community Forum</a></li>
              <li><a href="#" className="hover:text-foreground transition-colors">Status Page</a></li>
            </ul>
          </div>

          {/* Legal */}
          <div className="space-y-4">
            <h4 className="font-semibold">Legal</h4>
            <ul className="space-y-2 text-sm text-muted-foreground">
              <li><a href="#" className="hover:text-foreground transition-colors">Privacy Policy</a></li>
              <li><a href="#" className="hover:text-foreground transition-colors">Terms of Service</a></li>
              <li><a href="#" className="hover:text-foreground transition-colors">Risk Disclosure</a></li>
              <li><a href="#" className="hover:text-foreground transition-colors">Cookie Policy</a></li>
              <li><a href="#" className="hover:text-foreground transition-colors">Compliance</a></li>
            </ul>
          </div>
        </div>

        <div className="border-t border-border/50 mt-8 pt-8">
          <div className="flex flex-col md:flex-row justify-between items-center gap-4">
            <div className="text-sm text-muted-foreground">
              © 2025 BotCore. All rights reserved.
            </div>
            
            <div className="flex items-center gap-4">
              <Badge variant="outline" className="bg-profit/10 text-profit border-profit/20 text-xs">
                🔒 Secure & Regulated
              </Badge>
              <Badge variant="outline" className="bg-info/10 text-info border-info/20 text-xs">
                📊 Real-time Data
              </Badge>
            </div>
          </div>
          
          <div className="text-center mt-4 text-xs text-muted-foreground">
            <p>
              <strong>Risk Warning:</strong> Trading cryptocurrencies carries a high level of risk. 
              Past performance does not guarantee future results. Only invest what you can afford to lose.
            </p>
          </div>
        </div>
      </div>
    </footer>
  );
}