/**
 * Settings UI Component
 *
 * Component này render giao diện cài đặt bot dựa trên settings-config.json
 * Hỗ trợ các loại input: slider, toggle, select, number, multiselect
 */

import React, { useState, useEffect } from 'react';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';
import { Label } from '@/components/ui/label';
import { Slider } from '@/components/ui/slider';
import { Switch } from '@/components/ui/switch';
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/components/ui/select';
import { Input } from '@/components/ui/input';
import { Button } from '@/components/ui/button';
import { Alert, AlertDescription } from '@/components/ui/alert';
import { Tabs, TabsContent, TabsList, TabsTrigger } from '@/components/ui/tabs';
import { Badge } from '@/components/ui/badge';
import { Info, AlertTriangle, CheckCircle, TrendingUp, Shield, Brain, ChartLine, Bell } from 'lucide-react';
import settingsConfig from '@/config/settings-config.json';

interface SettingsValues {
  [key: string]: any;
}

export function SettingsUI() {
  const [values, setValues] = useState<SettingsValues>({});
  const [hasChanges, setHasChanges] = useState(false);

  // Load initial values from API or use defaults
  useEffect(() => {
    loadSettings();
  }, []);

  const loadSettings = async () => {
    try {
      const response = await fetch('/api/paper-trading/settings');
      const data = await response.json();
      setValues(data.data);
    } catch (error) {
      // Use defaults from config
      const defaults: SettingsValues = {};
      settingsConfig.categories.forEach(category => {
        category.settings.forEach(setting => {
          defaults[setting.id] = setting.default;
        });
      });
      setValues(defaults);
    }
  };

  const handleChange = (id: string, value: any) => {
    setValues(prev => ({ ...prev, [id]: value }));
    setHasChanges(true);
  };

  const handleSave = async () => {
    try {
      const response = await fetch('/api/paper-trading/settings', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify(values)
      });

      if (response.ok) {
        alert('✅ Cài đặt đã được lưu thành công!');
        setHasChanges(false);
      }
    } catch (error) {
      alert('❌ Lỗi khi lưu cài đặt');
    }
  };

  const handleReset = () => {
    if (confirm('Bạn có chắc muốn đặt lại tất cả cài đặt về mặc định?')) {
      loadSettings();
      setHasChanges(false);
    }
  };

  const applyPreset = (presetKey: string) => {
    const preset = settingsConfig.presets[presetKey as keyof typeof settingsConfig.presets];
    if (preset && preset.values) {
      setValues(prev => ({ ...prev, ...preset.values }));
      setHasChanges(true);
      alert(`✅ Đã áp dụng cài đặt "${preset.name}"`);
    }
  };

  const getCategoryIcon = (icon: string) => {
    const icons: { [key: string]: React.ReactNode } = {
      'settings': <Info className="h-5 w-5" />,
      'shield': <Shield className="h-5 w-5" />,
      'trending-up': <TrendingUp className="h-5 w-5" />,
      'brain': <Brain className="h-5 w-5" />,
      'chart-line': <ChartLine className="h-5 w-5" />,
      'bell': <Bell className="h-5 w-5" />
    };
    return icons[icon] || <Info className="h-5 w-5" />;
  };

  const renderSetting = (setting: any) => {
    const value = values[setting.id] ?? setting.default;

    switch (setting.type) {
      case 'slider':
        return (
          <div className="space-y-3">
            <div className="flex justify-between items-center">
              <Label htmlFor={setting.id}>{setting.name}</Label>
              <Badge variant="secondary">
                {value}{setting.unit}
              </Badge>
            </div>
            <Slider
              id={setting.id}
              min={setting.min}
              max={setting.max}
              step={setting.step}
              value={[value]}
              onValueChange={([newValue]) => handleChange(setting.id, newValue)}
            />
            <p className="text-sm text-muted-foreground">{setting.description}</p>
            {setting.help && (
              <div className="flex gap-2 text-sm text-blue-600 dark:text-blue-400">
                <Info className="h-4 w-4 mt-0.5" />
                <span>{setting.help}</span>
              </div>
            )}
            {setting.recommendation && (
              <div className="grid grid-cols-3 gap-2 text-xs">
                <div className="p-2 bg-green-50 dark:bg-green-900/20 rounded">
                  <div className="font-medium">Bảo thủ</div>
                  <div className="text-muted-foreground">
                    {setting.recommendation.conservative}{setting.unit}
                  </div>
                </div>
                <div className="p-2 bg-yellow-50 dark:bg-yellow-900/20 rounded">
                  <div className="font-medium">Trung bình</div>
                  <div className="text-muted-foreground">
                    {setting.recommendation.moderate}{setting.unit}
                  </div>
                </div>
                <div className="p-2 bg-red-50 dark:bg-red-900/20 rounded">
                  <div className="font-medium">Tích cực</div>
                  <div className="text-muted-foreground">
                    {setting.recommendation.aggressive}{setting.unit}
                  </div>
                </div>
              </div>
            )}
            {setting.warning && value > (setting.recommendation?.moderate || 0) && (
              <Alert variant="destructive">
                <AlertTriangle className="h-4 w-4" />
                <AlertDescription>{setting.warning}</AlertDescription>
              </Alert>
            )}
          </div>
        );

      case 'toggle':
        return (
          <div className="space-y-3">
            <div className="flex items-center justify-between">
              <div className="space-y-1">
                <Label htmlFor={setting.id}>{setting.name}</Label>
                <p className="text-sm text-muted-foreground">{setting.description}</p>
              </div>
              <Switch
                id={setting.id}
                checked={value}
                onCheckedChange={(checked) => handleChange(setting.id, checked)}
              />
            </div>
            {setting.help && (
              <div className="flex gap-2 text-sm text-blue-600 dark:text-blue-400">
                <Info className="h-4 w-4 mt-0.5" />
                <span>{setting.help}</span>
              </div>
            )}
            {setting.states && (
              <div className="text-sm">
                <span className="font-medium">
                  {value ? '✅ ' + setting.states.on : '❌ ' + setting.states.off}
                </span>
              </div>
            )}
            {setting.warning && !value && (
              <Alert>
                <AlertTriangle className="h-4 w-4" />
                <AlertDescription>{setting.warning}</AlertDescription>
              </Alert>
            )}
          </div>
        );

      case 'select':
        return (
          <div className="space-y-3">
            <Label htmlFor={setting.id}>{setting.name}</Label>
            <Select
              value={value.toString()}
              onValueChange={(newValue) => handleChange(setting.id, parseInt(newValue))}
            >
              <SelectTrigger>
                <SelectValue />
              </SelectTrigger>
              <SelectContent>
                {setting.options.map((option: any) => (
                  <SelectItem key={option.value} value={option.value.toString()}>
                    <div className="flex flex-col">
                      <span>{option.label}</span>
                      <span className="text-xs text-muted-foreground">
                        {option.signals_per_day} tín hiệu/ngày
                      </span>
                    </div>
                  </SelectItem>
                ))}
              </SelectContent>
            </Select>
            <p className="text-sm text-muted-foreground">{setting.description}</p>
            {setting.help && (
              <div className="flex gap-2 text-sm text-blue-600 dark:text-blue-400">
                <Info className="h-4 w-4 mt-0.5" />
                <span>{setting.help}</span>
              </div>
            )}
          </div>
        );

      case 'number':
        return (
          <div className="space-y-3">
            <Label htmlFor={setting.id}>{setting.name}</Label>
            <div className="flex gap-2">
              <Input
                id={setting.id}
                type="number"
                min={setting.min}
                max={setting.max}
                step={setting.step}
                value={value}
                onChange={(e) => handleChange(setting.id, parseFloat(e.target.value))}
              />
              {setting.unit && (
                <span className="flex items-center text-muted-foreground px-3">
                  {setting.unit}
                </span>
              )}
            </div>
            <p className="text-sm text-muted-foreground">{setting.description}</p>
            {setting.help && (
              <div className="flex gap-2 text-sm text-blue-600 dark:text-blue-400">
                <Info className="h-4 w-4 mt-0.5" />
                <span>{setting.help}</span>
              </div>
            )}
          </div>
        );

      case 'multiselect':
        return (
          <div className="space-y-3">
            <Label>{setting.name}</Label>
            <div className="flex flex-wrap gap-2">
              {setting.options.map((option: string) => (
                <Button
                  key={option}
                  variant={value.includes(option) ? 'default' : 'outline'}
                  size="sm"
                  onClick={() => {
                    const newValue = value.includes(option)
                      ? value.filter((v: string) => v !== option)
                      : [...value, option];
                    handleChange(setting.id, newValue);
                  }}
                >
                  {option}
                </Button>
              ))}
            </div>
            <p className="text-sm text-muted-foreground">{setting.description}</p>
            {setting.help && (
              <div className="flex gap-2 text-sm text-blue-600 dark:text-blue-400">
                <Info className="h-4 w-4 mt-0.5" />
                <span>{setting.help}</span>
              </div>
            )}
          </div>
        );

      default:
        return null;
    }
  };

  return (
    <div className="space-y-6">
      {/* Header with Presets */}
      <Card>
        <CardHeader>
          <CardTitle>⚙️ Cài Đặt Bot Trading</CardTitle>
          <CardDescription>
            Tùy chỉnh cách bot hoạt động theo phong cách giao dịch của bạn
          </CardDescription>
        </CardHeader>
        <CardContent>
          <div className="space-y-4">
            <div>
              <Label>Cài Đặt Nhanh (Presets)</Label>
              <p className="text-sm text-muted-foreground mb-3">
                Áp dụng bộ cài đặt được tối ưu sẵn
              </p>
              <div className="grid grid-cols-1 md:grid-cols-3 gap-3">
                {Object.entries(settingsConfig.presets).map(([key, preset]) => (
                  <Button
                    key={key}
                    variant="outline"
                    className="h-auto flex flex-col items-start p-4"
                    onClick={() => applyPreset(key)}
                  >
                    <div className="flex items-center gap-2 mb-2">
                      {key === 'conservative' && <Shield className="h-5 w-5 text-green-500" />}
                      {key === 'moderate' && <CheckCircle className="h-5 w-5 text-yellow-500" />}
                      {key === 'aggressive' && <TrendingUp className="h-5 w-5 text-red-500" />}
                      <span className="font-semibold">{preset.name}</span>
                    </div>
                    <p className="text-xs text-left text-muted-foreground">
                      {preset.description}
                    </p>
                    {preset.warning && (
                      <Alert className="mt-2">
                        <AlertTriangle className="h-4 w-4" />
                        <AlertDescription className="text-xs">
                          {preset.warning}
                        </AlertDescription>
                      </Alert>
                    )}
                  </Button>
                ))}
              </div>
            </div>
          </div>
        </CardContent>
      </Card>

      {/* Settings Tabs */}
      <Tabs defaultValue="basic" className="w-full">
        <TabsList className="grid w-full grid-cols-6">
          {settingsConfig.categories.map(category => (
            <TabsTrigger key={category.id} value={category.id} className="flex items-center gap-2">
              {getCategoryIcon(category.icon)}
              <span className="hidden md:inline">{category.name}</span>
            </TabsTrigger>
          ))}
        </TabsList>

        {settingsConfig.categories.map(category => (
          <TabsContent key={category.id} value={category.id}>
            <Card>
              <CardHeader>
                <CardTitle className="flex items-center gap-2">
                  {getCategoryIcon(category.icon)}
                  {category.name}
                </CardTitle>
                <CardDescription>{category.description}</CardDescription>
              </CardHeader>
              <CardContent className="space-y-6">
                {category.settings.map(setting => (
                  <div key={setting.id} className="pb-6 border-b last:border-0">
                    {renderSetting(setting)}
                  </div>
                ))}
              </CardContent>
            </Card>
          </TabsContent>
        ))}
      </Tabs>

      {/* Action Buttons */}
      <Card>
        <CardContent className="pt-6">
          <div className="flex gap-3 justify-end">
            <Button variant="outline" onClick={handleReset}>
              🔄 Đặt Lại Mặc Định
            </Button>
            <Button
              onClick={handleSave}
              disabled={!hasChanges}
              className="bg-green-600 hover:bg-green-700"
            >
              ✅ Lưu Cài Đặt
            </Button>
          </div>
          {hasChanges && (
            <Alert className="mt-4">
              <Info className="h-4 w-4" />
              <AlertDescription>
                Bạn có thay đổi chưa lưu. Nhớ nhấn "Lưu Cài Đặt" để áp dụng.
              </AlertDescription>
            </Alert>
          )}
        </CardContent>
      </Card>

      {/* Glossary */}
      <Card>
        <CardHeader>
          <CardTitle>📖 Thuật Ngữ</CardTitle>
          <CardDescription>
            Giải thích các thuật ngữ giao dịch quan trọng
          </CardDescription>
        </CardHeader>
        <CardContent>
          <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
            {Object.entries(settingsConfig.glossary).map(([key, item]) => (
              <div key={key} className="p-3 border rounded-lg">
                <h4 className="font-semibold mb-1">{item.term}</h4>
                <p className="text-sm text-muted-foreground mb-2">{item.definition}</p>
                <div className="text-xs bg-blue-50 dark:bg-blue-900/20 p-2 rounded">
                  <strong>Ví dụ:</strong> {item.example}
                </div>
              </div>
            ))}
          </div>
        </CardContent>
      </Card>
    </div>
  );
}
