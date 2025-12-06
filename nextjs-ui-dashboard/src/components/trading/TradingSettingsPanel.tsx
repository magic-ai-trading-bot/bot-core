import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { PremiumButton, PremiumInput } from "@/styles/luxury-design-system";
import { Label } from "@/components/ui/label";
import { Alert, AlertDescription } from "@/components/ui/alert";
import { Separator } from "@/components/ui/separator";
import { TradingSettings } from "@/components/dashboard/TradingSettings";
import { DollarSign, Settings, Target, AlertCircle, RotateCcw } from "lucide-react";
import { Badge } from "@/components/ui/badge";
import { SymbolConfig } from "./types";

interface PaperTradingSettings {
  basic: {
    initial_balance: number;
    default_position_size_pct: number;
    trading_fee_rate: number;
  };
  risk: {
    max_leverage: number;
    default_stop_loss_pct: number;
    default_take_profit_pct: number;
  };
}

interface TradingSettingsPanelProps {
  settingsForm: PaperTradingSettings;
  setSettingsForm: React.Dispatch<React.SetStateAction<PaperTradingSettings>>;
  handleSettingsSubmit: () => void;
  handleReset: () => void;
  showReset: boolean;
  setShowReset: (show: boolean) => void;
  isLoading: boolean;
  symbolSettings: { [key: string]: SymbolConfig };
  setShowSymbolDialog: (show: boolean) => void;
}

export function TradingSettingsPanel({
  settingsForm,
  setSettingsForm,
  handleSettingsSubmit,
  handleReset,
  showReset,
  setShowReset,
  isLoading,
  symbolSettings,
  setShowSymbolDialog,
}: TradingSettingsPanelProps) {
  return (
    <div className="space-y-4 lg:space-y-6">
      {/* Paper Trading Basic Settings */}
      <Card>
        <CardHeader>
          <CardTitle className="flex items-center gap-2">
            <DollarSign className="h-5 w-5" />
            Cài đặt Paper Trading
          </CardTitle>
        </CardHeader>
        <CardContent className="space-y-4">
          <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
            <div className="space-y-2">
              <Label htmlFor="initial-balance">Vốn ban đầu (USDT)</Label>
              <PremiumInput
                id="initial-balance"
                type="number"
                value={settingsForm.basic.initial_balance}
                onChange={(e) =>
                  setSettingsForm((prev) => ({
                    ...prev,
                    basic: {
                      ...prev.basic,
                      initial_balance: parseFloat(e.target.value) || 0,
                    },
                  }))
                }
              />
            </div>
            <div className="space-y-2">
              <Label htmlFor="max-leverage">Đòn bẩy tối đa</Label>
              <PremiumInput
                id="max-leverage"
                type="number"
                value={settingsForm.risk.max_leverage}
                onChange={(e) =>
                  setSettingsForm((prev) => ({
                    ...prev,
                    risk: {
                      ...prev.risk,
                      max_leverage: parseFloat(e.target.value) || 1,
                    },
                  }))
                }
              />
            </div>
            <div className="space-y-2">
              <Label htmlFor="position-size">Kích thước vị thế (%)</Label>
              <PremiumInput
                id="position-size"
                type="number"
                value={settingsForm.basic.default_position_size_pct}
                onChange={(e) =>
                  setSettingsForm((prev) => ({
                    ...prev,
                    basic: {
                      ...prev.basic,
                      default_position_size_pct:
                        parseFloat(e.target.value) || 0,
                    },
                  }))
                }
              />
            </div>
            <div className="space-y-2">
              <Label htmlFor="stop-loss">Stop Loss mặc định (%)</Label>
              <PremiumInput
                id="stop-loss"
                type="number"
                value={settingsForm.risk.default_stop_loss_pct}
                onChange={(e) =>
                  setSettingsForm((prev) => ({
                    ...prev,
                    risk: {
                      ...prev.risk,
                      default_stop_loss_pct:
                        parseFloat(e.target.value) || 0,
                    },
                  }))
                }
              />
            </div>
            <div className="space-y-2">
              <Label htmlFor="take-profit">
                Take Profit mặc định (%)
              </Label>
              <PremiumInput
                id="take-profit"
                type="number"
                value={settingsForm.risk.default_take_profit_pct}
                onChange={(e) =>
                  setSettingsForm((prev) => ({
                    ...prev,
                    risk: {
                      ...prev.risk,
                      default_take_profit_pct:
                        parseFloat(e.target.value) || 0,
                    },
                  }))
                }
              />
            </div>
            <div className="space-y-2">
              <Label htmlFor="trading-fee">Phí giao dịch (%)</Label>
              <PremiumInput
                id="trading-fee"
                type="number"
                value={settingsForm.basic.trading_fee_rate}
                onChange={(e) =>
                  setSettingsForm((prev) => ({
                    ...prev,
                    basic: {
                      ...prev.basic,
                      trading_fee_rate: parseFloat(e.target.value) || 0,
                    },
                  }))
                }
              />
            </div>
          </div>
          <div className="flex gap-4 pt-4">
            <PremiumButton
              onClick={handleSettingsSubmit}
              className="flex-1"
              disabled={isLoading}
            >
              {isLoading ? "Đang lưu..." : "Lưu cài đặt"}
            </PremiumButton>
            <PremiumButton
              variant="secondary"
              onClick={() => setShowReset(true)}
              className="flex-1"
            >
              <RotateCcw className="h-4 w-4 mr-2" />
              Reset dữ liệu
            </PremiumButton>
          </div>
          {showReset && (
            <Alert>
              <AlertCircle className="h-4 w-4" />
              <AlertDescription>
                <div className="flex items-center justify-between">
                  <span>
                    Xác nhận reset toàn bộ dữ liệu paper trading?
                  </span>
                  <div className="flex gap-2">
                    <PremiumButton
                      variant="danger"
                      size="sm"
                      onClick={handleReset}
                    >
                      Xác nhận
                    </PremiumButton>
                    <PremiumButton
                      variant="secondary"
                      size="sm"
                      onClick={() => setShowReset(false)}
                    >
                      Hủy
                    </PremiumButton>
                  </div>
                </div>
              </AlertDescription>
            </Alert>
          )}
        </CardContent>
      </Card>

      {/* Separator */}
      <div className="flex items-center gap-4">
        <Separator className="flex-1" />
        <div className="flex items-center gap-2 text-sm text-muted-foreground">
          <Settings className="h-4 w-4" />
          Cài đặt nâng cao
        </div>
        <Separator className="flex-1" />
      </div>

      {/* Symbol Configuration Settings */}
      <Card>
        <CardHeader>
          <CardTitle className="flex items-center gap-2">
            <Target className="h-5 w-5" />
            Cài đặt Symbols
            <Badge variant="secondary" className="text-xs">
              {Object.keys(symbolSettings).length} symbols
            </Badge>
          </CardTitle>
          <p className="text-sm text-muted-foreground">
            Cấu hình riêng cho từng symbol: leverage, kích thước vị thế,
            stop loss/take profit.
          </p>
        </CardHeader>
        <CardContent>
          <PremiumButton
            onClick={() => setShowSymbolDialog(true)}
            variant="secondary"
            className="w-full"
          >
            <Settings className="h-4 w-4 mr-2" />
            Mở cài đặt Symbols
          </PremiumButton>
        </CardContent>
      </Card>

      {/* Advanced Trading Strategy Settings */}
      <Card>
        <CardHeader>
          <CardTitle className="flex items-center gap-2">
            <Target className="h-5 w-5" />
            Cài đặt Chiến lược Trading
            <Badge variant="secondary" className="text-xs">
              Thích hợp cho thị trường ít biến động
            </Badge>
          </CardTitle>
          <p className="text-sm text-muted-foreground">
            Điều chỉnh các tham số chiến lược để tối ưu cho điều kiện thị
            trường hiện tại. Sử dụng preset "Low Volatility" cho thị
            trường ít biến động.
          </p>
        </CardHeader>
        <CardContent>
          <TradingSettings />
        </CardContent>
      </Card>
    </div>
  );
}
