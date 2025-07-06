import { DashboardHeader } from "@/components/dashboard/DashboardHeader";
import { BotSettings } from "@/components/dashboard/BotSettings";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { Switch } from "@/components/ui/switch";
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui/tabs";
import { Badge } from "@/components/ui/badge";
import { useState } from "react";
import ChatBot from "@/components/ChatBot";

const Settings = () => {
  const [apiKey, setApiKey] = useState(
    "************************************1234"
  );
  const [secretKey, setSecretKey] = useState(
    "************************************5678"
  );
  const [notifications, setNotifications] = useState({
    email: true,
    push: false,
    telegram: true,
    discord: false,
  });

  return (
    <div className="min-h-screen bg-background">
      <DashboardHeader />

      <div className="p-4 lg:p-6">
        <div className="mb-4 lg:mb-6">
          <h1 className="text-2xl lg:text-3xl font-bold">Cài đặt Bot</h1>
          <p className="text-muted-foreground text-sm lg:text-base">
            Quản lý cấu hình và tùy chọn cho trading bot của bạn
          </p>
        </div>

        <Tabs defaultValue="bot" className="space-y-4 lg:space-y-6">
          <TabsList className="grid w-full grid-cols-2 lg:grid-cols-4 gap-1">
            <TabsTrigger value="bot" className="text-xs lg:text-sm">
              Bot Settings
            </TabsTrigger>
            <TabsTrigger value="api" className="text-xs lg:text-sm">
              API Keys
            </TabsTrigger>
            <TabsTrigger value="notifications" className="text-xs lg:text-sm">
              Thông báo
            </TabsTrigger>
            <TabsTrigger value="security" className="text-xs lg:text-sm">
              Bảo mật
            </TabsTrigger>
          </TabsList>

          {/* Bot Configuration Tab */}
          <TabsContent value="bot">
            <BotSettings />
          </TabsContent>

          {/* API Keys Tab */}
          <TabsContent value="api" className="space-y-6">
            <Card>
              <CardHeader>
                <CardTitle className="flex items-center gap-2">
                  Binance API Configuration
                  <Badge
                    variant="outline"
                    className="bg-warning/10 text-warning border-warning/20"
                  >
                    Testnet
                  </Badge>
                </CardTitle>
              </CardHeader>
              <CardContent className="space-y-4">
                <div className="space-y-2">
                  <Label htmlFor="api-key">API Key</Label>
                  <Input
                    id="api-key"
                    type="password"
                    value={apiKey}
                    onChange={(e) => setApiKey(e.target.value)}
                    placeholder="Nhập Binance API Key của bạn"
                  />
                </div>

                <div className="space-y-2">
                  <Label htmlFor="secret-key">Secret Key</Label>
                  <Input
                    id="secret-key"
                    type="password"
                    value={secretKey}
                    onChange={(e) => setSecretKey(e.target.value)}
                    placeholder="Nhập Binance Secret Key của bạn"
                  />
                </div>

                <div className="flex items-center space-x-2 p-4 bg-info/10 rounded-lg border border-info/20">
                  <div className="w-2 h-2 bg-info rounded-full"></div>
                  <div className="text-sm">
                    <p className="text-info font-semibold">Lưu ý bảo mật</p>
                    <p className="text-muted-foreground">
                      API keys được mã hóa và lưu trữ an toàn. Chỉ cấp quyền
                      Futures Trading cho bot.
                    </p>
                  </div>
                </div>

                <div className="grid grid-cols-2 gap-3">
                  <Button variant="outline">Test Connection</Button>
                  <Button className="bg-profit hover:bg-profit/90">
                    Lưu API Keys
                  </Button>
                </div>
              </CardContent>
            </Card>

            {/* Trading Permissions */}
            <Card>
              <CardHeader>
                <CardTitle>Quyền hạn Trading</CardTitle>
              </CardHeader>
              <CardContent className="space-y-4">
                <div className="space-y-3">
                  {[
                    {
                      name: "Spot Trading",
                      enabled: false,
                      description: "Giao dịch spot cơ bản",
                    },
                    {
                      name: "Futures Trading",
                      enabled: true,
                      description: "Giao dịch futures với đòn bẩy",
                    },
                    {
                      name: "Margin Trading",
                      enabled: false,
                      description: "Giao dịch ký quỹ",
                    },
                    {
                      name: "Options Trading",
                      enabled: false,
                      description: "Giao dịch quyền chọn",
                    },
                  ].map((permission) => (
                    <div
                      key={permission.name}
                      className="flex items-center justify-between p-3 rounded-lg bg-secondary/50"
                    >
                      <div>
                        <div className="font-semibold">{permission.name}</div>
                        <div className="text-sm text-muted-foreground">
                          {permission.description}
                        </div>
                      </div>
                      <Switch
                        checked={permission.enabled}
                        disabled={permission.name === "Futures Trading"}
                      />
                    </div>
                  ))}
                </div>
              </CardContent>
            </Card>
          </TabsContent>

          {/* Notifications Tab */}
          <TabsContent value="notifications" className="space-y-6">
            <Card>
              <CardHeader>
                <CardTitle>Tùy chọn thông báo</CardTitle>
              </CardHeader>
              <CardContent className="space-y-4">
                <div className="space-y-4">
                  <div className="flex items-center justify-between p-3 rounded-lg bg-secondary/50">
                    <div>
                      <div className="font-semibold">Email Notifications</div>
                      <div className="text-sm text-muted-foreground">
                        Nhận thông báo qua email
                      </div>
                    </div>
                    <Switch
                      checked={notifications.email}
                      onCheckedChange={(checked) =>
                        setNotifications((prev) => ({
                          ...prev,
                          email: checked,
                        }))
                      }
                    />
                  </div>

                  <div className="flex items-center justify-between p-3 rounded-lg bg-secondary/50">
                    <div>
                      <div className="font-semibold">Push Notifications</div>
                      <div className="text-sm text-muted-foreground">
                        Thông báo đẩy trên trình duyệt
                      </div>
                    </div>
                    <Switch
                      checked={notifications.push}
                      onCheckedChange={(checked) =>
                        setNotifications((prev) => ({ ...prev, push: checked }))
                      }
                    />
                  </div>

                  <div className="flex items-center justify-between p-3 rounded-lg bg-secondary/50">
                    <div>
                      <div className="font-semibold">Telegram Bot</div>
                      <div className="text-sm text-muted-foreground">
                        Thông báo qua Telegram
                      </div>
                    </div>
                    <Switch
                      checked={notifications.telegram}
                      onCheckedChange={(checked) =>
                        setNotifications((prev) => ({
                          ...prev,
                          telegram: checked,
                        }))
                      }
                    />
                  </div>

                  <div className="flex items-center justify-between p-3 rounded-lg bg-secondary/50">
                    <div>
                      <div className="font-semibold">Discord Webhook</div>
                      <div className="text-sm text-muted-foreground">
                        Thông báo qua Discord
                      </div>
                    </div>
                    <Switch
                      checked={notifications.discord}
                      onCheckedChange={(checked) =>
                        setNotifications((prev) => ({
                          ...prev,
                          discord: checked,
                        }))
                      }
                    />
                  </div>
                </div>

                {notifications.telegram && (
                  <div className="space-y-2 p-4 bg-info/10 rounded-lg border border-info/20">
                    <Label htmlFor="telegram-token">Telegram Bot Token</Label>
                    <Input
                      id="telegram-token"
                      placeholder="Nhập bot token từ @BotFather"
                    />
                  </div>
                )}

                <Button className="w-full bg-profit hover:bg-profit/90">
                  Lưu cài đặt thông báo
                </Button>
              </CardContent>
            </Card>
          </TabsContent>

          {/* Security Tab */}
          <TabsContent value="security" className="space-y-6">
            <Card>
              <CardHeader>
                <CardTitle>Bảo mật tài khoản</CardTitle>
              </CardHeader>
              <CardContent className="space-y-6">
                <div className="space-y-4">
                  <div className="flex items-center justify-between p-4 rounded-lg bg-profit/10 border border-profit/20">
                    <div>
                      <div className="font-semibold text-profit">
                        Two-Factor Authentication
                      </div>
                      <div className="text-sm text-muted-foreground">
                        Xác thực 2 yếu tố đã được bật
                      </div>
                    </div>
                    <Badge className="bg-profit text-profit-foreground">
                      Đã kích hoạt
                    </Badge>
                  </div>

                  <div className="p-4 rounded-lg bg-secondary/50 border">
                    <div className="font-semibold mb-2">Đổi mật khẩu</div>
                    <div className="space-y-3">
                      <Input type="password" placeholder="Mật khẩu hiện tại" />
                      <Input type="password" placeholder="Mật khẩu mới" />
                      <Input
                        type="password"
                        placeholder="Xác nhận mật khẩu mới"
                      />
                      <Button variant="outline" className="w-full">
                        Cập nhật mật khẩu
                      </Button>
                    </div>
                  </div>

                  <div className="p-4 rounded-lg bg-secondary/50 border">
                    <div className="font-semibold mb-2">Phiên đăng nhập</div>
                    <div className="space-y-2 text-sm">
                      <div className="flex justify-between">
                        <span>Chrome on Windows</span>
                        <span className="text-profit">Active now</span>
                      </div>
                      <div className="flex justify-between">
                        <span>Mobile App</span>
                        <span className="text-muted-foreground">
                          2 hours ago
                        </span>
                      </div>
                    </div>
                    <Button variant="outline" size="sm" className="w-full mt-3">
                      Đăng xuất tất cả thiết bị
                    </Button>
                  </div>
                </div>
              </CardContent>
            </Card>
          </TabsContent>
        </Tabs>
      </div>

      {/* Chatbot Widget */}
      <ChatBot />
    </div>
  );
};

export default Settings;
