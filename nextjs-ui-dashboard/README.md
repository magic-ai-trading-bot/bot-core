# Crypto Trading Bot Dashboard (Next.js UI)

Modern, responsive dashboard for the Crypto Trading Bot system built with React, TypeScript, Vite, and Shadcn/UI.

## 🚀 Features

- **Real-time Market Data**: Live WebSocket connections for price updates
- **Paper Trading**: Simulate trading strategies without risking real funds
- **AI-Powered Signals**: Integration with Python AI service for trading recommendations
- **Interactive Charts**: TradingView-style charts with technical indicators
- **Performance Metrics**: Comprehensive portfolio analytics and PnL tracking
- **Dark/Light Mode**: Fully themed with Tailwind CSS
- **Responsive Design**: Mobile-first, works on all screen sizes
- **Multi-language**: i18n support (English, Vietnamese)

## 🛠️ Tech Stack

- **Framework**: React 18 + Vite 7
- **Language**: TypeScript 5
- **UI Components**: Shadcn/UI (Radix UI)
- **Styling**: Tailwind CSS 3
- **Charts**: Recharts + 3D visualizations (Three.js)
- **API Integration**: Axios + React Query
- **State Management**: React Context + Custom Hooks
- **Testing**: Vitest + React Testing Library + Playwright
- **Build**: Docker multi-stage builds

## 📦 Quick Start

### Prerequisites

```bash
node >= 18.0.0
npm >= 9.0.0
```

### Installation

```bash
# Install dependencies
npm install

# Copy environment template
cp .env.example .env

# Edit .env and add your configuration
nano .env
```

### Development

```bash
# Start development server (http://localhost:3000)
npm run dev

# Run tests
npm run test

# Run tests with UI
npm run test:ui

# Run E2E tests
npm run test:e2e

# Lint code
npm run lint

# Build for production
npm run build

# Preview production build
npm run preview
```

## 🌍 Environment Variables

Create a `.env` file with:

```env
# Hugging Face API (for chatbot)
VITE_HF_API_KEY=your_huggingface_api_key

# Enable/disable real-time features
VITE_ENABLE_REALTIME=true

# API Endpoints (default: localhost)
VITE_API_URL=http://localhost:8080
VITE_AI_API_URL=http://localhost:8000
```

Get your Hugging Face API key from: https://huggingface.co/settings/tokens

## 🧪 Testing

### Unit & Integration Tests (Vitest)

```bash
# Run all tests
npm run test

# Run with coverage
npm run test:coverage

# Run specific test file
npm run test -- src/__tests__/hooks/useWebSocket.test.tsx
```

**Test Coverage:**
- 675+ unit/integration tests
- Components, hooks, pages, services, contexts
- 95%+ coverage on critical paths

### E2E Tests (Playwright)

```bash
# Run E2E tests
npm run test:e2e

# Run with UI mode (interactive)
npm run test:e2e:ui

# Run in headed mode (see browser)
npm run test:e2e:headed

# Debug tests
npm run test:e2e:debug
```

**E2E Coverage:**
- 60 end-to-end tests
- Authentication flow (10 tests)
- Dashboard flow (17 tests)
- Paper trading flow (16 tests)
- Settings flow (17 tests)

See [e2e/README.md](./e2e/README.md) for detailed E2E testing documentation.

## 📁 Project Structure

```
nextjs-ui-dashboard/
├── src/
│   ├── components/          # React components
│   │   ├── dashboard/       # Dashboard-specific components
│   │   ├── landing/         # Landing page components
│   │   └── ui/              # Shadcn/UI components (57 components)
│   ├── pages/               # Page components (7 pages)
│   ├── hooks/               # Custom React hooks (9 hooks)
│   ├── services/            # API services
│   ├── contexts/            # React Context providers
│   ├── lib/                 # Utility libraries
│   ├── utils/               # Helper functions
│   ├── types/               # TypeScript type definitions
│   ├── i18n/                # Internationalization
│   ├── test/                # Test setup and utilities
│   └── __tests__/           # Test files
├── e2e/                     # Playwright E2E tests
├── public/                  # Static assets
├── docs/                    # Documentation
├── .env.example             # Environment template
├── vite.config.ts           # Vite configuration
├── vitest.config.ts         # Vitest configuration
├── playwright.config.ts     # Playwright configuration
├── tailwind.config.ts       # Tailwind CSS configuration
├── tsconfig.json            # TypeScript configuration
└── package.json             # Dependencies and scripts
```

## 🔌 Integration with Backend

This dashboard integrates with two backend services:

### Rust Core Engine (Port 8080)
- **Trading execution** and order management
- **WebSocket** for real-time market data
- **Portfolio management** and position tracking
- **Authentication** (JWT)

### Python AI Service (Port 8000)
- **ML-powered trading signals** (LSTM, GRU, Transformer)
- **Technical analysis** and indicators
- **Market predictions** and confidence scores
- **GPT-4 powered** market analysis

## 🐳 Docker Deployment

```bash
# Development build
docker build -f Dockerfile.dev -t trading-dashboard:dev .
docker run -p 3000:3000 trading-dashboard:dev

# Production build
docker build -f Dockerfile.production -t trading-dashboard:prod .
docker run -p 80:80 trading-dashboard:prod

# Using Docker Compose (from root)
cd ..
docker-compose up nextjs-ui-dashboard
```

## 📊 Key Components

### Dashboard
- **DashboardHeader**: Portfolio summary, balance, PnL
- **TradingCharts**: Real-time price charts with indicators
- **AISignals**: ML-powered trading recommendations
- **PerformanceChart**: Historical performance visualization

### Paper Trading
- **TradingInterface**: Order entry, position management
- **Portfolio**: Real-time portfolio tracking
- **Trade History**: Complete trading history with PnL

### Settings
- **BotSettings**: Configure trading bot parameters
- **TradingSettings**: Risk management, strategies, timeframes
- **API Configuration**: Connect to Binance or other exchanges

## 🎨 Customization

### Adding New Components

```bash
# Add new Shadcn/UI component
npx shadcn-ui@latest add [component-name]

# Example: Add button component
npx shadcn-ui@latest add button
```

### Theming

Colors and themes are configured in:
- `tailwind.config.ts` - Tailwind theme configuration
- `src/index.css` - CSS variables for light/dark mode

## 🤝 Contributing

1. Create a new branch for your feature
2. Write tests for new functionality
3. Ensure all tests pass (`npm run test` && `npm run test:e2e`)
4. Run linter (`npm run lint`)
5. Format code (automatically handled by Prettier)
6. Submit a pull request

## 📝 Code Quality

- **ESLint**: Configured for React + TypeScript
- **TypeScript**: Strict type checking enabled
- **Prettier**: Auto-formatting on save (via ESLint)
- **Vitest**: Fast unit test runner
- **Playwright**: Reliable E2E testing
- **Git Hooks**: Pre-commit hooks for quality checks

## 🔒 Security

- ✅ No hardcoded API keys (use environment variables)
- ✅ JWT tokens for authentication
- ✅ CORS configured properly
- ✅ No console.log in production code
- ✅ Comprehensive security tests

## 🚀 Performance

- **Vite**: Lightning-fast HMR (Hot Module Replacement)
- **Code Splitting**: Automatic route-based splitting
- **Lazy Loading**: Components loaded on-demand
- **Memoization**: React.memo for expensive components
- **WebSocket Optimization**: Stable callbacks, no infinite loops

## 📚 Documentation

- [E2E Testing Guide](./e2e/README.md) - Playwright E2E tests
- [Fixes Report](./docs/FIXES_REPORT.md) - Recent security and performance fixes

## 🐛 Troubleshooting

### Port 3000 Already in Use
```bash
# Kill process on port 3000
lsof -ti:3000 | xargs kill -9
```

### WebSocket Connection Issues
- Ensure Rust Core Engine is running on port 8080
- Check CORS settings in backend
- Verify `VITE_ENABLE_REALTIME=true` in .env

### Build Errors
```bash
# Clear node_modules and reinstall
rm -rf node_modules package-lock.json
npm install

# Clear Vite cache
rm -rf .vite
```

## 📄 License

Part of the Crypto Trading Bot project. See root LICENSE for details.

## 🙏 Credits

Built with:
- [React](https://react.dev/)
- [Vite](https://vitejs.dev/)
- [Shadcn/UI](https://ui.shadcn.com/)
- [Tailwind CSS](https://tailwindcss.com/)
- [Recharts](https://recharts.org/)
- [Playwright](https://playwright.dev/)
- [Vitest](https://vitest.dev/)

---

**Note**: This is part of a microservices architecture. Make sure all backend services are running for full functionality.

For the complete system documentation, see the [main README](../README.md) at the project root.
