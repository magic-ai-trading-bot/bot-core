import { lazy, Suspense } from "react";
import { DashboardHeader } from "@/components/dashboard/DashboardHeader";
import { BotStatus } from "@/components/dashboard/BotStatus";
import { AISignals } from "@/components/dashboard/AISignals";
import { AIStrategySelector } from "@/components/dashboard/AIStrategySelector";
import { TransactionHistory } from "@/components/dashboard/TransactionHistory";

// Lazy load heavy components
const TradingCharts = lazy(() => import("@/components/dashboard/TradingCharts").then(module => ({ default: module.TradingCharts })));
const PerformanceChart = lazy(() => import("@/components/dashboard/PerformanceChart").then(module => ({ default: module.PerformanceChart })));
const ChatBot = lazy(() => import("@/components/ChatBot"));

const ChartFallback = () => (
  <div className="w-full h-64 bg-muted animate-pulse rounded-lg"></div>
);

const Dashboard = () => {
  return (
    <div className="min-h-screen bg-background">
      <DashboardHeader />

      <div className="p-4 lg:p-6 space-y-4 lg:space-y-6">
        {/* Top Section - Bot Status */}
        <BotStatus />

        {/* Trading Charts Section */}
        <Suspense fallback={<ChartFallback />}>
          <TradingCharts />
        </Suspense>

        {/* AI Section - Strategy Configuration & Trading Signals */}
        <div className="grid grid-cols-1 lg:grid-cols-3 gap-4 lg:gap-6 lg:items-start">
          <div className="lg:col-span-1 h-full">
            <AIStrategySelector />
          </div>
          <div className="lg:col-span-2 h-full">
            <AISignals />
          </div>
        </div>

        {/* Performance Section - Overview & Metrics */}
        <div>
          <Suspense fallback={<ChartFallback />}>
            <PerformanceChart />
          </Suspense>
        </div>

        {/* Bottom Section - Transaction History */}
        <TransactionHistory />
      </div>

      {/* Chatbot Widget */}
      <Suspense fallback={null}>
        <ChatBot />
      </Suspense>
    </div>
  );
};

export default Dashboard;
