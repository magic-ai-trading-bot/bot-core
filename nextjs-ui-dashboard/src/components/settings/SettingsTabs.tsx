/**
 * SettingsTabs Component
 *
 * Vertical tabs on desktop, horizontal on mobile for settings navigation.
 * Provides smooth transitions between settings sections.
 */

import { Tabs, TabsContent, TabsList, TabsTrigger } from '@/components/ui/tabs';
import { cn } from '@/lib/utils';
import {
  Settings,
  Bell,
  Key,
  Palette,
  Shield,
  TrendingUp,
} from 'lucide-react';

interface SettingsTabsProps {
  defaultTab?: string;
  children: {
    trading?: React.ReactNode;
    notifications?: React.ReactNode;
    apiKeys?: React.ReactNode;
    appearance?: React.ReactNode;
    security?: React.ReactNode;
  };
}

const tabs = [
  {
    id: 'trading',
    label: 'Trading',
    icon: TrendingUp,
    description: 'Default preferences and risk limits',
  },
  {
    id: 'notifications',
    label: 'Notifications',
    icon: Bell,
    description: 'Email, push, and trading alerts',
  },
  {
    id: 'api-keys',
    label: 'API Keys',
    icon: Key,
    description: 'Manage your exchange API keys',
  },
  {
    id: 'appearance',
    label: 'Appearance',
    icon: Palette,
    description: 'Theme and display preferences',
  },
  {
    id: 'security',
    label: 'Security',
    icon: Shield,
    description: 'Password, 2FA, and sessions',
  },
];

export function SettingsTabs({ defaultTab = 'trading', children }: SettingsTabsProps) {
  return (
    <Tabs defaultValue={defaultTab} className="w-full">
      {/* Desktop: Vertical tabs on left */}
      <div className="hidden lg:grid lg:grid-cols-[280px_1fr] lg:gap-8">
        <TabsList className="flex flex-col h-fit bg-slate-900/50 border border-slate-700/50 p-2 rounded-xl">
          {tabs.map((tab) => {
            const Icon = tab.icon;
            return (
              <TabsTrigger
                key={tab.id}
                value={tab.id}
                className={cn(
                  'w-full justify-start gap-3 px-4 py-3 text-left',
                  'data-[state=active]:bg-slate-800/70 data-[state=active]:border-l-2 data-[state=active]:border-sky-500',
                  'hover:bg-slate-800/50 transition-all rounded-lg'
                )}
              >
                <Icon className="w-5 h-5 text-gray-400" />
                <div className="flex-1">
                  <div className="font-medium text-gray-100">{tab.label}</div>
                  <div className="text-xs text-gray-400 mt-0.5">{tab.description}</div>
                </div>
              </TabsTrigger>
            );
          })}
        </TabsList>

        <div className="space-y-6">
          {tabs.map((tab) => (
            <TabsContent key={tab.id} value={tab.id} className="mt-0">
              {children[tab.id as keyof typeof children]}
            </TabsContent>
          ))}
        </div>
      </div>

      {/* Mobile: Horizontal tabs on top */}
      <div className="lg:hidden">
        <TabsList className="w-full grid grid-cols-3 bg-slate-900/50 border border-slate-700/50 p-1 rounded-xl mb-6">
          {tabs.map((tab) => {
            const Icon = tab.icon;
            return (
              <TabsTrigger
                key={tab.id}
                value={tab.id}
                className={cn(
                  'flex flex-col items-center gap-1 py-3',
                  'data-[state=active]:bg-slate-800/70',
                  'data-[state=active]:border-b-2 data-[state=active]:border-sky-500'
                )}
              >
                <Icon className="w-5 h-5" />
                <span className="text-xs">{tab.label}</span>
              </TabsTrigger>
            );
          })}
        </TabsList>

        <div className="space-y-6">
          {tabs.map((tab) => (
            <TabsContent key={tab.id} value={tab.id}>
              {children[tab.id as keyof typeof children]}
            </TabsContent>
          ))}
        </div>
      </div>
    </Tabs>
  );
}
