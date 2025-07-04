import { useState } from "react";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Label } from "@/components/ui/label";
import { useNavigate } from "react-router-dom";
import { useToast } from "@/hooks/use-toast";
import { useAuth } from "@/contexts/AuthContext";
import { useEffect } from "react";

const Login = () => {
  const [email, setEmail] = useState("");
  const [password, setPassword] = useState("");
  const [isLoading, setIsLoading] = useState(false);
  const navigate = useNavigate();
  const { toast } = useToast();
  const { login, isAuthenticated } = useAuth();

  // Redirect if already authenticated
  useEffect(() => {
    if (isAuthenticated) {
      navigate("/dashboard", { replace: true });
    }
  }, [isAuthenticated, navigate]);

  const handleLogin = async (e: React.FormEvent) => {
    e.preventDefault();
    setIsLoading(true);

    // Simulate login process
    setTimeout(async () => {
      try {
        if (email && password) {
          const success = await login(email, password);
          if (success) {
            toast({
              title: "Đăng nhập thành công",
              description: "Chào mừng trở lại với Trading Bot Dashboard!",
            });
            // Force navigation after successful login
            window.location.href = "/dashboard";
          }
        } else {
          toast({
            title: "Lỗi đăng nhập",
            description: "Vui lòng nhập email và mật khẩu",
            variant: "destructive",
          });
        }
      } catch (error) {
        toast({
          title: "Lỗi đăng nhập",
          description: "Có lỗi xảy ra khi đăng nhập",
          variant: "destructive",
        });
      }
      setIsLoading(false);
    }, 1000);
  };

  return (
    <div className="min-h-screen bg-background flex items-center justify-center p-4">
      {/* Background Pattern */}
      <div className="absolute inset-0 bg-[radial-gradient(ellipse_at_center,_var(--tw-gradient-stops))] from-primary/5 via-background to-background"></div>
      
      <div className="relative z-10 w-full max-w-sm md:max-w-md">
        {/* Logo/Brand */}
        <div className="text-center mb-6 md:mb-8">
          <div className="w-12 h-12 md:w-16 md:h-16 bg-gradient-to-br from-primary to-accent rounded-2xl flex items-center justify-center mx-auto mb-4">
            <span className="text-primary-foreground font-bold text-lg md:text-2xl">BT</span>
          </div>
          <h1 className="text-2xl md:text-3xl font-bold">Crypto Trading Bot</h1>
          <p className="text-muted-foreground mt-2 text-sm md:text-base">Đăng nhập để quản lý bot trading của bạn</p>
        </div>

        <Card className="shadow-2xl border-border/50 bg-card/80 backdrop-blur">
          <CardHeader className="space-y-1">
            <CardTitle className="text-2xl text-center">Đăng nhập</CardTitle>
          </CardHeader>
          <CardContent>
            <form onSubmit={handleLogin} className="space-y-4">
              <div className="space-y-2">
                <Label htmlFor="email">Email</Label>
                <Input
                  id="email"
                  type="email"
                  placeholder="admin@tradingbot.com"
                  value={email}
                  onChange={(e) => setEmail(e.target.value)}
                  className="bg-background/50"
                />
              </div>
              
              <div className="space-y-2">
                <Label htmlFor="password">Mật khẩu</Label>
                <Input
                  id="password"
                  type="password"
                  placeholder="Nhập mật khẩu của bạn"
                  value={password}
                  onChange={(e) => setPassword(e.target.value)}
                  className="bg-background/50"
                />
              </div>

              <Button
                type="submit"
                className="w-full bg-profit hover:bg-profit/90 text-profit-foreground"
                disabled={isLoading}
              >
                {isLoading ? "Đang đăng nhập..." : "Đăng nhập"}
              </Button>
            </form>

            {/* Demo Credentials */}
            <div className="mt-6 p-4 bg-muted/50 rounded-lg border border-border/50">
              <p className="text-sm text-muted-foreground mb-2">Demo credentials:</p>
              <div className="text-xs space-y-1">
                <p><strong>Email:</strong> admin@tradingbot.com</p>
                <p><strong>Password:</strong> demo123</p>
              </div>
            </div>

            {/* Features Preview */}
            <div className="mt-6 space-y-3">
              <div className="flex items-center gap-3 text-sm">
                <div className="w-2 h-2 bg-profit rounded-full"></div>
                <span>AI-Powered Trading Signals</span>
              </div>
              <div className="flex items-center gap-3 text-sm">
                <div className="w-2 h-2 bg-info rounded-full"></div>
                <span>Real-time Performance Analytics</span>
              </div>
              <div className="flex items-center gap-3 text-sm">
                <div className="w-2 h-2 bg-warning rounded-full"></div>
                <span>Advanced Risk Management</span>
              </div>
            </div>
          </CardContent>
        </Card>

        <div className="text-center mt-6 text-sm text-muted-foreground">
          <p>Bảo mật với mã hóa end-to-end và xác thực 2FA</p>
        </div>
      </div>
    </div>
  );
};

export default Login;