// Top exchanges with official logos
const exchanges = [
  {
    name: "Binance",
    logo: "https://cdn.simpleicons.org/binance/F0B90B"
  },
  {
    name: "Coinbase",
    logo: "https://cdn.simpleicons.org/coinbase/0052FF"
  },
  {
    name: "OKX",
    logo: "https://cdn.simpleicons.org/okx/FFFFFF"
  },
  {
    name: "KuCoin",
    logo: "https://cdn.simpleicons.org/kucoin/23AF91"
  },
];

// Essential integrations with official logos
const integrations = [
  {
    name: "TradingView",
    logo: "https://cdn.simpleicons.org/tradingview/2962FF"
  },
  {
    name: "Telegram",
    logo: "https://cdn.simpleicons.org/telegram/26A5E4"
  },
  {
    name: "Discord",
    logo: "https://cdn.simpleicons.org/discord/5865F2"
  },
];

export function PartnersSection() {
  return (
    <section className="py-20 bg-gradient-to-b from-background via-card/5 to-background">
      <div className="container mx-auto px-4">
        {/* Section 1: Supported Exchanges */}
        <div className="mb-16">
          <div className="text-center mb-10">
            <p className="text-xs uppercase tracking-widest text-muted-foreground mb-2">
              Supported Exchanges
            </p>
            <h2 className="text-2xl md:text-3xl font-bold">
              Trade on <span className="text-profit">Top Exchanges</span>
            </h2>
          </div>

          <div className="flex flex-wrap justify-center items-center gap-8 md:gap-12">
            {exchanges.map((exchange) => (
              <div
                key={exchange.name}
                className="flex flex-col items-center gap-3 p-4 rounded-xl hover:bg-card/50 transition-colors group"
              >
                <div className="w-16 h-16 flex items-center justify-center rounded-xl bg-card/80 border border-border/50 group-hover:border-primary/30 transition-colors p-3">
                  <img
                    src={exchange.logo}
                    alt={exchange.name}
                    className="w-full h-full object-contain"
                    loading="lazy"
                  />
                </div>
                <span className="text-sm font-medium text-muted-foreground group-hover:text-foreground transition-colors">
                  {exchange.name}
                </span>
              </div>
            ))}
          </div>
        </div>

        {/* Section 2: Integrations & Tools */}
        <div className="mb-16">
          <div className="text-center mb-10">
            <p className="text-xs uppercase tracking-widest text-muted-foreground mb-2">
              Integrations & Notifications
            </p>
            <h2 className="text-2xl md:text-3xl font-bold">
              Connect Your <span className="text-info">Favorite Tools</span>
            </h2>
          </div>

          <div className="flex flex-wrap justify-center items-center gap-8 md:gap-12">
            {integrations.map((integration) => (
              <div
                key={integration.name}
                className="flex flex-col items-center gap-3 p-4 rounded-xl hover:bg-card/50 transition-colors group"
              >
                <div className="w-16 h-16 flex items-center justify-center rounded-xl bg-card/80 border border-border/50 group-hover:border-info/30 transition-colors p-3">
                  <img
                    src={integration.logo}
                    alt={integration.name}
                    className="w-full h-full object-contain"
                    loading="lazy"
                  />
                </div>
                <span className="text-sm font-medium text-muted-foreground group-hover:text-foreground transition-colors">
                  {integration.name}
                </span>
              </div>
            ))}
          </div>
        </div>

        {/* Trust indicators */}
        <div className="flex flex-wrap justify-center items-center gap-8 text-sm text-muted-foreground">
          <div className="flex items-center gap-2">
            <div className="w-2 h-2 bg-profit rounded-full animate-pulse" />
            <span>99.9% Uptime</span>
          </div>
          <div className="h-4 w-px bg-border/50 hidden sm:block" />
          <div className="flex items-center gap-2">
            <div className="w-2 h-2 bg-info rounded-full" />
            <span>Bank-grade Security</span>
          </div>
          <div className="h-4 w-px bg-border/50 hidden sm:block" />
          <div className="flex items-center gap-2">
            <div className="w-2 h-2 bg-primary rounded-full" />
            <span>SOC2 Compliant</span>
          </div>
        </div>
      </div>
    </section>
  );
}
