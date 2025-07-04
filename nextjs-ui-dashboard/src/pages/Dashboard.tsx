import { DashboardHeader } from "@/components/dashboard/DashboardHeader";
import { BotStatus } from "@/components/dashboard/BotStatus";
import { AISignals } from "@/components/dashboard/AISignals";
import { PerformanceChart } from "@/components/dashboard/PerformanceChart";
import { TransactionHistory } from "@/components/dashboard/TransactionHistory";
import { TradingCharts } from "@/components/dashboard/TradingCharts";

const Dashboard = () => {
  return (
    <div className="min-h-screen bg-background">
      <DashboardHeader />

      <div className="p-4 lg:p-6 space-y-4 lg:space-y-6">
        {/* Top Section - Bot Status */}
        <BotStatus />

        {/* Trading Charts Section */}
        <TradingCharts />

        {/* Middle Section - AI Signals and Performance */}
        <div className="grid grid-cols-1 lg:grid-cols-3 gap-4 lg:gap-6">
          <div className="lg:col-span-1">
            <AISignals />
          </div>
          <div className="lg:col-span-2">
            <PerformanceChart />
          </div>
        </div>

        {/* Bottom Section - Transaction History */}
        <TransactionHistory />
      </div>
    </div>
  );
};

export default Dashboard;
