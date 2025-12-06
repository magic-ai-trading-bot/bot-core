import { lazy, Suspense } from "react";
import { Toaster } from "@/components/ui/toaster";
import { Toaster as Sonner } from "@/components/ui/sonner";
import { TooltipProvider } from "@/components/ui/tooltip";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { BrowserRouter, Routes, Route } from "react-router-dom";
import { AuthProvider } from "@/contexts/AuthContext";
import { AIAnalysisProvider } from "@/contexts/AIAnalysisContext";
import { PaperTradingProvider } from "@/contexts/PaperTradingContext";
import { WebSocketProvider } from "@/contexts/WebSocketContext";
import { TradingModeProvider } from "@/contexts/TradingModeContext";
import { NotificationProvider } from "@/contexts/NotificationContext";
import ProtectedRoute from "@/components/ProtectedRoute";
import { MainLayout } from "@/components/layout/MainLayout";
import { ErrorBoundary } from "@/components/ui/ErrorBoundary";

// Lazy load all pages for code splitting
const Index = lazy(() => import("./pages/Index"));
const Login = lazy(() => import("./pages/Login"));
const Register = lazy(() => import("./pages/Register"));
const Dashboard = lazy(() => import("./pages/Dashboard"));
const Settings = lazy(() => import("./pages/Settings"));
const Profile = lazy(() => import("./pages/Profile"));
const PaperTrading = lazy(() => import("./pages/PaperTrading"));
const RealTrading = lazy(() => import("./pages/RealTrading"));
const TradingPaper = lazy(() => import("./pages/TradingPaper"));
const TradeAnalyses = lazy(() => import("./pages/TradeAnalyses"));
const HowItWorks = lazy(() => import("./pages/HowItWorks"));
const Portfolio = lazy(() => import("./pages/Portfolio"));
const AISignals = lazy(() => import("./pages/AISignals"));
const Error = lazy(() => import("./pages/Error"));
const NotFound = lazy(() => import("./pages/NotFound"));

// Public pages (no auth required)
const Features = lazy(() => import("./pages/Features"));
const Pricing = lazy(() => import("./pages/Pricing"));
const API = lazy(() => import("./pages/API"));
const Documentation = lazy(() => import("./pages/Documentation"));
const About = lazy(() => import("./pages/About"));
const Blog = lazy(() => import("./pages/Blog"));
const Careers = lazy(() => import("./pages/Careers"));
const Contact = lazy(() => import("./pages/Contact"));
const Privacy = lazy(() => import("./pages/Privacy"));
const Terms = lazy(() => import("./pages/Terms"));
const SecurityPage = lazy(() => import("./pages/SecurityPage"));
const Compliance = lazy(() => import("./pages/Compliance"));

const queryClient = new QueryClient();

// Loading fallback component
const LoadingFallback = () => (
  <div className="flex items-center justify-center min-h-screen">
    <div className="animate-spin rounded-full h-12 w-12 border-b-2 border-primary"></div>
  </div>
);

// Testing hot reload functionality
const App = () => (
  <QueryClientProvider client={queryClient}>
    <ErrorBoundary>
      <AuthProvider>
        <WebSocketProvider>
          <AIAnalysisProvider>
            <PaperTradingProvider>
              <TradingModeProvider>
                <NotificationProvider>
                <TooltipProvider>
                  <Toaster />
                  <Sonner />
                  <BrowserRouter>
                    <Suspense fallback={<LoadingFallback />}>
                      <Routes>
                {/* Public routes - no layout */}
                <Route path="/" element={<Index />} />
                <Route path="/login" element={<Login />} />
                <Route path="/register" element={<Register />} />

                {/* Protected routes - with MainLayout */}
                <Route
                  path="/dashboard"
                  element={
                    <ProtectedRoute>
                      <MainLayout>
                        <Dashboard />
                      </MainLayout>
                    </ProtectedRoute>
                  }
                />
                <Route
                  path="/trading/paper"
                  element={
                    <ProtectedRoute>
                      <MainLayout>
                        <PaperTrading />
                      </MainLayout>
                    </ProtectedRoute>
                  }
                />
                <Route
                  path="/trading/real"
                  element={
                    <ProtectedRoute>
                      <MainLayout>
                        <RealTrading />
                      </MainLayout>
                    </ProtectedRoute>
                  }
                />
                <Route
                  path="/profile"
                  element={
                    <ProtectedRoute>
                      <MainLayout>
                        <Profile />
                      </MainLayout>
                    </ProtectedRoute>
                  }
                />
                <Route
                  path="/portfolio"
                  element={
                    <ProtectedRoute>
                      <MainLayout>
                        <Portfolio />
                      </MainLayout>
                    </ProtectedRoute>
                  }
                />
                <Route
                  path="/signals"
                  element={
                    <ProtectedRoute>
                      <MainLayout>
                        <AISignals />
                      </MainLayout>
                    </ProtectedRoute>
                  }
                />
                <Route
                  path="/settings"
                  element={
                    <ProtectedRoute>
                      <MainLayout>
                        <Settings />
                      </MainLayout>
                    </ProtectedRoute>
                  }
                />
                <Route
                  path="/trade-analyses"
                  element={
                    <ProtectedRoute>
                      <MainLayout>
                        <TradeAnalyses />
                      </MainLayout>
                    </ProtectedRoute>
                  }
                />
                <Route
                  path="/how-it-works"
                  element={
                    <ProtectedRoute>
                      <MainLayout>
                        <HowItWorks />
                      </MainLayout>
                    </ProtectedRoute>
                  }
                />

                {/* Keep old route for backward compatibility */}
                <Route
                  path="/trading-paper"
                  element={
                    <ProtectedRoute>
                      <MainLayout>
                        <TradingPaper />
                      </MainLayout>
                    </ProtectedRoute>
                  }
                />

                {/* Public pages - no layout */}
                <Route path="/features" element={<Features />} />
                <Route path="/pricing" element={<Pricing />} />
                <Route path="/api" element={<API />} />
                <Route path="/docs" element={<Documentation />} />
                <Route path="/about" element={<About />} />
                <Route path="/blog" element={<Blog />} />
                <Route path="/careers" element={<Careers />} />
                <Route path="/contact" element={<Contact />} />
                <Route path="/privacy" element={<Privacy />} />
                <Route path="/terms" element={<Terms />} />
                <Route path="/security" element={<SecurityPage />} />
                <Route path="/compliance" element={<Compliance />} />

                {/* Error Page */}
                <Route path="/error" element={<Error />} />

                {/* ADD ALL CUSTOM ROUTES ABOVE THE CATCH-ALL "*" ROUTE */}
                <Route path="*" element={<NotFound />} />
                      </Routes>
                    </Suspense>
                  </BrowserRouter>
                </TooltipProvider>
                </NotificationProvider>
              </TradingModeProvider>
            </PaperTradingProvider>
          </AIAnalysisProvider>
        </WebSocketProvider>
      </AuthProvider>
    </ErrorBoundary>
  </QueryClientProvider>
);

export default App;
