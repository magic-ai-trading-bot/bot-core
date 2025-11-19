# Exit Strategy Settings - Integration Guide

Quick guide to integrate the ExitStrategySettings component into your dashboard.

## Quick Start (3 Steps)

### Step 1: Import the Component

```tsx
import { ExitStrategySettings } from "@/components/dashboard/ExitStrategySettings";
```

### Step 2: Add to Your Dashboard

```tsx
export function Dashboard() {
  return (
    <div className="container mx-auto p-4 space-y-6">
      {/* Your existing components */}
      <BotStatus />
      <PerformanceChart />

      {/* Add Exit Strategy Settings */}
      <ExitStrategySettings />
    </div>
  );
}
```

### Step 3: Configure Backend Endpoint

Ensure your Rust backend implements the endpoint:

```rust
// src/api/paper_trading.rs

#[derive(Debug, Serialize, Deserialize)]
pub struct ExitStrategySettings {
    pub trailing_stop: TrailingStopConfig,
    pub partial_profit: PartialProfitConfig,
    pub time_based_exit: TimeBasedExitConfig,
}

// GET /api/paper-trading/exit-strategy-settings
#[get("/exit-strategy-settings")]
pub async fn get_exit_strategy_settings() -> Result<Json<ApiResponse<ExitStrategySettings>>> {
    // Return current settings
}

// PUT /api/paper-trading/exit-strategy-settings
#[put("/exit-strategy-settings")]
pub async fn update_exit_strategy_settings(
    settings: Json<ExitStrategySettings>
) -> Result<Json<ApiResponse<SimpleResponse>>> {
    // Update settings
}
```

## Full Integration Examples

### Example 1: Add to Settings Page

```tsx
// src/pages/Settings.tsx
import { ExitStrategySettings } from "@/components/dashboard/ExitStrategySettings";
import { BotSettings } from "@/components/dashboard/BotSettings";
import { PerSymbolSettings } from "@/components/dashboard/PerSymbolSettings";

export function SettingsPage() {
  return (
    <div className="container mx-auto p-4">
      <h1 className="text-2xl font-bold mb-6">Bot Configuration</h1>

      <div className="grid grid-cols-1 lg:grid-cols-2 gap-6">
        {/* Left Column */}
        <div className="space-y-6">
          <BotSettings />
          <ExitStrategySettings />
        </div>

        {/* Right Column */}
        <div className="space-y-6">
          <PerSymbolSettings />
        </div>
      </div>
    </div>
  );
}
```

### Example 2: Add to Trading Dashboard

```tsx
// src/pages/Dashboard.tsx
import { ExitStrategySettings } from "@/components/dashboard/ExitStrategySettings";
import { BotStatus } from "@/components/dashboard/BotStatus";
import { TradingCharts } from "@/components/dashboard/TradingCharts";
import { AISignals } from "@/components/dashboard/AISignals";

export function TradingDashboard() {
  return (
    <div className="container mx-auto p-4">
      {/* Top Section */}
      <BotStatus />

      {/* Middle Section */}
      <div className="grid grid-cols-1 xl:grid-cols-3 gap-6 mt-6">
        <div className="xl:col-span-2">
          <TradingCharts />
        </div>
        <div className="space-y-6">
          <AISignals />
        </div>
      </div>

      {/* Bottom Section - Settings */}
      <div className="mt-6">
        <ExitStrategySettings />
      </div>
    </div>
  );
}
```

### Example 3: Standalone Settings Modal

```tsx
// src/components/ExitStrategyModal.tsx
import { useState } from "react";
import { Dialog, DialogContent, DialogHeader, DialogTitle, DialogTrigger } from "@/components/ui/dialog";
import { Button } from "@/components/ui/button";
import { ExitStrategySettings } from "@/components/dashboard/ExitStrategySettings";
import { Settings } from "lucide-react";

export function ExitStrategyModal() {
  const [open, setOpen] = useState(false);

  return (
    <Dialog open={open} onOpenChange={setOpen}>
      <DialogTrigger asChild>
        <Button variant="outline">
          <Settings className="h-4 w-4 mr-2" />
          Exit Strategy
        </Button>
      </DialogTrigger>
      <DialogContent className="max-w-2xl max-h-[90vh] overflow-y-auto">
        <DialogHeader>
          <DialogTitle>Exit Strategy Configuration</DialogTitle>
        </DialogHeader>
        <ExitStrategySettings />
      </DialogContent>
    </Dialog>
  );
}
```

## Backend Implementation Guide

### Rust Backend (Actix-Web)

```rust
// src/api/paper_trading/exit_strategy.rs

use actix_web::{get, put, web, HttpResponse, Result};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrailingStopConfig {
    pub enabled: bool,
    pub distance_pct: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProfitTarget {
    pub profit_pct: f64,
    pub quantity_pct: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PartialProfitConfig {
    pub enabled: bool,
    pub first_target: ProfitTarget,
    pub second_target: ProfitTarget,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeBasedExitConfig {
    pub enabled: bool,
    pub max_hold_time_hours: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExitStrategySettings {
    pub trailing_stop: TrailingStopConfig,
    pub partial_profit: PartialProfitConfig,
    pub time_based_exit: TimeBasedExitConfig,
}

#[derive(Debug, Serialize)]
pub struct ApiResponse<T> {
    pub success: bool,
    pub data: Option<T>,
    pub error: Option<String>,
    pub timestamp: String,
}

// GET endpoint
#[get("/exit-strategy-settings")]
pub async fn get_exit_strategy_settings(
    app_state: web::Data<AppState>,
) -> Result<HttpResponse> {
    // Load settings from database or config
    let settings = app_state.exit_strategy_settings.lock().unwrap().clone();

    Ok(HttpResponse::Ok().json(ApiResponse {
        success: true,
        data: Some(settings),
        error: None,
        timestamp: chrono::Utc::now().to_rfc3339(),
    }))
}

// PUT endpoint
#[put("/exit-strategy-settings")]
pub async fn update_exit_strategy_settings(
    settings: web::Json<ExitStrategySettings>,
    app_state: web::Data<AppState>,
) -> Result<HttpResponse> {
    // Validate settings
    if let Err(e) = validate_exit_strategy(&settings) {
        return Ok(HttpResponse::BadRequest().json(ApiResponse::<()> {
            success: false,
            data: None,
            error: Some(e),
            timestamp: chrono::Utc::now().to_rfc3339(),
        }));
    }

    // Update settings in app state
    *app_state.exit_strategy_settings.lock().unwrap() = settings.into_inner();

    // Save to database/config
    // ...

    Ok(HttpResponse::Ok().json(ApiResponse {
        success: true,
        data: Some(serde_json::json!({ "message": "Settings updated successfully" })),
        error: None,
        timestamp: chrono::Utc::now().to_rfc3339(),
    }))
}

fn validate_exit_strategy(settings: &ExitStrategySettings) -> Result<(), String> {
    // Validate trailing stop
    if settings.trailing_stop.enabled {
        if settings.trailing_stop.distance_pct < 0.5 || settings.trailing_stop.distance_pct > 5.0 {
            return Err("Trailing stop distance must be between 0.5% and 5%".to_string());
        }
    }

    // Validate partial profit
    if settings.partial_profit.enabled {
        if settings.partial_profit.second_target.profit_pct <= settings.partial_profit.first_target.profit_pct {
            return Err("Second target profit must be higher than first target".to_string());
        }

        let total_qty = settings.partial_profit.first_target.quantity_pct
                      + settings.partial_profit.second_target.quantity_pct;
        if total_qty > 100.0 {
            return Err("Total quantity cannot exceed 100%".to_string());
        }
    }

    // Validate time-based exit
    if settings.time_based_exit.enabled {
        if settings.time_based_exit.max_hold_time_hours < 1
           || settings.time_based_exit.max_hold_time_hours > 168 {
            return Err("Max hold time must be between 1 and 168 hours".to_string());
        }
    }

    Ok(())
}

// Register routes
pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/api/paper-trading")
            .service(get_exit_strategy_settings)
            .service(update_exit_strategy_settings)
    );
}
```

## Environment Variables

Add to `.env`:

```bash
# API Configuration
VITE_RUST_API_URL=http://localhost:8080
VITE_WS_URL=ws://localhost:8080/ws
```

## Testing

### Frontend Unit Tests

```typescript
// src/components/dashboard/__tests__/ExitStrategySettings.test.tsx
import { render, screen, fireEvent, waitFor } from "@testing-library/react";
import { ExitStrategySettings } from "../ExitStrategySettings";

describe("ExitStrategySettings", () => {
  test("renders component", () => {
    render(<ExitStrategySettings />);
    expect(screen.getByText("Exit Strategy Settings")).toBeInTheDocument();
  });

  test("toggles trailing stop", async () => {
    render(<ExitStrategySettings />);
    const toggle = screen.getByLabelText("Enable trailing stop loss");

    fireEvent.click(toggle);
    expect(toggle).toBeChecked();
  });

  test("validates profit targets", async () => {
    render(<ExitStrategySettings />);

    // Enable partial profit
    const toggle = screen.getByLabelText("Enable partial profit taking");
    fireEvent.click(toggle);

    // Set invalid values (second target lower than first)
    const firstTarget = screen.getByLabelText("First target profit percentage");
    const secondTarget = screen.getByLabelText("Second target profit percentage");

    fireEvent.change(firstTarget, { target: { value: "10" } });
    fireEvent.change(secondTarget, { target: { value: "5" } });

    // Try to save
    const saveButton = screen.getByLabelText("Save exit strategy settings");
    fireEvent.click(saveButton);

    // Should show validation error
    await waitFor(() => {
      expect(screen.getByText(/Second target profit must be higher/i)).toBeInTheDocument();
    });
  });
});
```

### Backend Integration Tests

```rust
// tests/api/exit_strategy_test.rs
#[actix_web::test]
async fn test_get_exit_strategy_settings() {
    let app = test::init_service(App::new().configure(configure)).await;

    let req = test::TestRequest::get()
        .uri("/api/paper-trading/exit-strategy-settings")
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());

    let body: ApiResponse<ExitStrategySettings> = test::read_body_json(resp).await;
    assert!(body.success);
}

#[actix_web::test]
async fn test_update_exit_strategy_settings() {
    let app = test::init_service(App::new().configure(configure)).await;

    let settings = ExitStrategySettings {
        trailing_stop: TrailingStopConfig {
            enabled: true,
            distance_pct: 2.5,
        },
        partial_profit: PartialProfitConfig {
            enabled: true,
            first_target: ProfitTarget {
                profit_pct: 3.0,
                quantity_pct: 50.0,
            },
            second_target: ProfitTarget {
                profit_pct: 6.0,
                quantity_pct: 50.0,
            },
        },
        time_based_exit: TimeBasedExitConfig {
            enabled: false,
            max_hold_time_hours: 24,
        },
    };

    let req = test::TestRequest::put()
        .uri("/api/paper-trading/exit-strategy-settings")
        .set_json(&settings)
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());
}
```

## Troubleshooting

### Issue: Component Not Rendering

**Solution**: Check imports and ensure all dependencies are installed:

```bash
npm install lucide-react @radix-ui/react-slider @radix-ui/react-switch
```

### Issue: API Errors

**Solution**: Verify backend is running and endpoint is correct:

```bash
curl http://localhost:8080/api/paper-trading/exit-strategy-settings
```

### Issue: TypeScript Errors

**Solution**: Ensure types match between frontend and backend. Run type check:

```bash
npm run type-check
```

## Next Steps

1. ✅ Component integrated
2. ⬜ Backend endpoint implemented
3. ⬜ Database persistence configured
4. ⬜ Trading logic updated to use exit strategies
5. ⬜ Tests written and passing
6. ⬜ Documentation reviewed

## Support

For questions or issues:
- Check component documentation: `src/components/dashboard/ExitStrategySettings.md`
- Review examples: `src/components/dashboard/ExitStrategySettings.example.tsx`
- See project docs: `/docs/CONTRIBUTING.md`
