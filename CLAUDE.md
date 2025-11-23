# CLAUDE.md - Smart Navigation Hub

**Purpose**: This file helps Claude Code quickly find exact code locations and documentation for any feature without reading entire codebase.

---

## ⚠️ CRITICAL: SPEC-DRIVEN DEVELOPMENT WORKFLOW

**THIS IS A FINANCE PROJECT - MISTAKES = MONEY LOSS**

### Mandatory Workflow for ALL New Features

```
USER REQUEST → CREATE SPEC → USE AGENTS → UPDATE SPECS → DONE
```

#### Step 1: CREATE SPEC FIRST (Before ANY code)

When user requests a new feature:

1. **Search existing specs** to understand current system:
   ```bash
   # Read relevant existing specs first
   specs/01-requirements/1.1-functional-requirements/FR-*.md
   specs/02-design/2.3-api/API-*.md
   specs/02-design/2.5-components/COMP-*.md
   ```

2. **Create new requirement spec** using template:
   - Location: `specs/01-requirements/1.1-functional-requirements/FR-XXX.md`
   - Use: `specs/_SPEC_TEMPLATE.md` as base
   - Include: Acceptance criteria (☐ checkboxes), examples, edge cases
   - Reference: Related specs, design docs, dependencies

3. **Add to TRACEABILITY_MATRIX.md**:
   - Add new FR-XXX entry to appropriate module section
   - Link to design docs, test cases, code locations
   - Maintain 100% traceability

4. **Create test cases** (TC-XXX.md):
   - Location: `specs/03-testing/3.2-test-cases/TC-XXX.md`
   - Format: Gherkin (Given/When/Then)
   - Coverage: Happy path + edge cases + errors

**DO NOT write ANY code until specs are complete and reviewed!**

#### Step 2: USE AGENTS FOR IMPLEMENTATION (MANDATORY)

**NEVER code manually for complex features. ALWAYS use specialized agents:**

---

## 🤖 COMPLETE AGENT CATALOG (36+ Specialized Agents)

### **Core Development Workflow** 🔧

#### `/plan [task]` - Research & Create Implementation Plan
- **Purpose**: Research best practices, analyze requirements, create detailed implementation plan
- **Output**: Plan document in `./plans/` directory
- **Use When**: Starting any new feature, need architectural guidance
- **Example**: `/plan Add trailing stop loss feature`

#### `/cook [tasks]` - Implement Feature Step-by-Step (MAIN AGENT)
- **Purpose**: Full-cycle implementation (research → plan → implement → test → review → docs)
- **Workflow**:
  1. Research (multiple `researcher` + `scout` subagents in parallel)
  2. Plan (use `planner` subagent)
  3. Implementation (use `general agent` + `ui-ux-designer`)
  4. Testing (use `tester` subagent)
  5. Code Review (use `code-reviewer` subagent)
  6. Documentation (use `docs-manager` + `project-manager`)
  7. Onboarding + Final Report
- **Use When**: Implementing any complex feature
- **Example**: `/cook Implement trailing stop loss with tests and docs`

#### `/test` - Run Tests & Analyze Coverage
- **Purpose**: Run all tests locally and analyze summary report
- **Output**: Test results, coverage report
- **Use When**: After implementation, before commit, debugging test failures
- **Example**: `/test`

#### `/debug [issues]` - Debug Technical Issues
- **Purpose**: Use `debugger` subagent to find root cause of issues
- **Output**: Detailed analysis report with root causes
- **Use When**: System errors, unexpected behavior, hard-to-diagnose issues
- **Example**: `/debug WebSocket keeps disconnecting`

#### `/watzup` - Review Recent Changes & Wrap Up
- **Purpose**: Review current branch and recent commits, provide detailed summary
- **Output**: Summary of all changes (modified/added/removed), impact analysis
- **Use When**: End of work session, before PR, project status check
- **Example**: `/watzup`

---

### **Question & Brainstorming** 💡

#### `/ask [technical-question]` - Answer Architectural Questions
- **Purpose**: Senior Systems Architect consultation with 4 expert advisors:
  - Systems Designer (boundaries, interfaces, components)
  - Technology Strategist (tech stack, frameworks, patterns)
  - Scalability Consultant (performance, reliability, growth)
  - Risk Analyst (issues, trade-offs, mitigation)
- **Output**: Architecture analysis, design recommendations, technology guidance
- **Use When**: Architectural decisions, technical challenges, design choices
- **Example**: `/ask Should we use REST or GraphQL for trading API?`

#### `/brainstorm [question]` - Brainstorm Solutions (YAGNI, KISS, DRY)
- **Purpose**: Elite Solution Brainstormer, explore multiple approaches with brutal honesty
- **Workflow**: Discovery → Research → Analysis → Debate → Consensus → Documentation
- **Output**: Markdown summary report with evaluated approaches, pros/cons, final recommendation
- **Use When**: Feature design, exploring alternatives, architectural decisions
- **Example**: `/brainstorm How to implement real-time portfolio updates efficiently?`

#### `/scout [user-prompt] [scale]` - Scout Codebase Quickly
- **Purpose**: Fast, token-efficient search for files needed to complete task
- **Scale**: 1-3 (gemini), 4-5 (opencode), 6+ (Explore subagents in parallel)
- **Output**: List of relevant files saved to `plans/scouts/`
- **Use When**: Starting new task, need to find related code quickly
- **Example**: `/scout Find all authentication-related files 3`

---

### **Fixing Issues** 🔨

#### `/fix:fast [issues]` - Quick Fixes for Small Issues
- **Purpose**: Analyze and fix small, straightforward issues quickly
- **Workflow**: Analyze → Fix → Test with `tester` agent → Repeat until pass
- **Use When**: Simple bugs, typos, minor logic errors, quick patches
- **Example**: `/fix:fast Fix typo in error message`

#### `/fix:hard [issues]` - Complex Fixes Using Subagents
- **Purpose**: Plan and fix hard issues with full subagent orchestration
- **Workflow**: Plan (`planner` + `researcher`) → Implement → Test → Debug → Review → Report
- **Use When**: Complex bugs, architectural issues, multi-component problems
- **Example**: `/fix:hard Fix race condition in WebSocket connection`

#### `/fix:test [issues]` - Run & Fix Test Failures
- **Purpose**: Run test suite and fix any failures
- **Workflow**: Run tests → Analyze failures → Fix → Repeat until all pass
- **Use When**: CI/CD failures, broken tests, test suite maintenance
- **Example**: `/fix:test`

#### `/fix:types` - Fix TypeScript Type Errors
- **Purpose**: Fix all TypeScript type errors in the project
- **Use When**: Type checking failures, after refactoring, adding strict mode
- **Example**: `/fix:types`

#### `/fix:ui [issue]` - Fix UI/UX Issues
- **Purpose**: Use `ui-ux-designer` subagent to analyze and fix UI issues
- **Workflow**: Analyze (with screenshot if provided) → Fix → Screenshot → Verify → Test
- **Reads**: `./docs/design-guidelines.md` for consistency
- **Use When**: Visual bugs, layout issues, design inconsistencies, UX problems
- **Example**: `/fix:ui Button alignment is broken on mobile`

#### `/fix:ci [github-actions-url]` - Analyze & Fix CI/CD Failures
- **Purpose**: Read GitHub Actions logs, analyze root causes, implement fixes
- **Workflow**: Read logs → Analyze → Plan → Implement → Test → Repeat
- **Requires**: GitHub CLI (`gh`) installed and authorized
- **Use When**: CI/CD pipeline failures, build errors, test failures in CI
- **Example**: `/fix:ci https://github.com/user/repo/actions/runs/12345`

#### `/fix:logs [issue]` - Analyze Logs & Fix Issues
- **Purpose**: Analyze application logs to find and fix issues
- **Use When**: Production errors, debugging via logs, error tracking
- **Example**: `/fix:logs Analyze error logs from last 24 hours`

---

### **Design & UI/UX** 🎨

#### `/design:good [tasks]` - Create Immersive Design
- **Purpose**: Create high-quality, award-winning design (Dribbble/Behance/Awwwards level)
- **Workflow**: Research (styles, trends, fonts, colors) → Plan → Implement → Review → Update guidelines
- **Capabilities**: Generate images, edit images, remove backgrounds, create 3D experiences
- **Output**: HTML/CSS/JS design (unless specified otherwise)
- **Use When**: New features, redesigns, landing pages, marketing materials
- **Example**: `/design:good Create a modern trading dashboard with dark mode`

#### `/design:fast [tasks]` - Create Quick Design
- **Purpose**: Create functional design quickly without extensive research
- **Use When**: Prototypes, MVPs, internal tools, quick iterations
- **Example**: `/design:fast Create a simple settings page`

#### `/design:3d [tasks]` - Create 3D Designs with Three.js
- **Purpose**: Create immersive 3D interactive experiences
- **Use When**: 3D visualizations, interactive demos, portfolio pieces
- **Example**: `/design:3d Create 3D portfolio performance visualization`

#### `/design:screenshot [screenshot]` - Design Based on Screenshot
- **Purpose**: Analyze screenshot and create matching or improved design
- **Use When**: Replicating designs, improving existing UIs, design inspiration
- **Example**: `/design:screenshot /path/to/screenshot.png`

#### `/design:video [video]` - Design Based on Video
- **Purpose**: Extract design from video and implement
- **Use When**: Video mockups, animated prototypes, design demos
- **Example**: `/design:video /path/to/demo-video.mp4`

#### `/design:describe [screenshot]` - Describe Design from Screenshot
- **Purpose**: Detailed analysis of design elements (fonts, colors, spacing, layout)
- **Output**: Comprehensive design specification document
- **Use When**: Understanding existing designs, creating design documentation
- **Example**: `/design:describe /path/to/ui-screenshot.png`

---

### **Documentation** 📚

#### `/docs:init` - Analyze Codebase & Create Initial Docs
- **Purpose**: Full codebase analysis and initial documentation generation
- **Creates**: README, project overview, codebase summary, architecture, standards, roadmap
- **Use When**: New project setup, onboarding documentation, initial documentation
- **Example**: `/docs:init`

#### `/docs:update` - Update Documentation After Changes
- **Purpose**: Update all documentation to match current codebase
- **Updates**:
  - README.md
  - docs/project-overview-pdr.md
  - docs/codebase-summary.md
  - docs/code-standards.md
  - docs/system-architecture.md
  - docs/project-roadmap.md
  - docs/deployment-guide.md (optional)
  - docs/design-guidelines.md (optional)
- **Use When**: After major changes, before releases, regular maintenance
- **Example**: `/docs:update`

#### `/docs:summarize` - Summarize Documentation
- **Purpose**: Create concise summary of all documentation
- **Output**: Documentation overview and quick reference guide
- **Use When**: Creating executive summaries, onboarding new team members
- **Example**: `/docs:summarize`

---

### **Git Operations** 🔀

#### `/git:cm` - Stage All & Create Commit
- **Purpose**: Stage all files and create meaningful commit message
- **Output**: Commit (NOT pushed to remote)
- **Use When**: Local commits, work-in-progress saves
- **Example**: `/git:cm`

#### `/git:cp` - Stage, Commit & Push (Current Branch)
- **Purpose**: Stage all files, create commit, push to remote repository
- **Output**: Commit + Push to current branch
- **Use When**: Sharing work, backing up changes, preparing for PR
- **Example**: `/git:cp`

#### `/git:pr [branch] [from-branch]` - Create Pull Request
- **Purpose**: Create pull request with detailed description
- **Output**: PR with summary, test plan, changes overview
- **Use When**: Feature complete, ready for code review
- **Example**: `/git:pr feature/trailing-stop main`

---

### **Content Writing** ✍️

#### `/content:good [user-request]` - Write High-Quality Copy
- **Purpose**: Write creative, smart, high-quality content
- **Use When**: Marketing copy, documentation, user-facing content, blog posts
- **Example**: `/content:good Write landing page copy for trading bot`

#### `/content:fast [user-request]` - Write Copy Quickly
- **Purpose**: Write functional content quickly
- **Use When**: Internal docs, draft content, quick updates
- **Example**: `/content:fast Write quick README for new feature`

#### `/content:enhance [issues]` - Enhance Existing Copy
- **Purpose**: Analyze and improve existing content
- **Use When**: Improving clarity, fixing tone, adding details
- **Example**: `/content:enhance Improve error messages in auth module`

#### `/content:cro [issues]` - Optimize for Conversion (CRO)
- **Purpose**: Analyze and optimize content for conversion rates
- **Use When**: Landing pages, CTAs, marketing funnels, sign-up flows
- **Example**: `/content:cro Optimize pricing page for conversions`

---

### **Integration** 🔌

#### `/integrate:sepay [tasks]` - Implement SePay.vn Payment
- **Purpose**: Integrate SePay.vn payment gateway
- **Use When**: Adding Vietnamese payment method
- **Example**: `/integrate:sepay Add SePay payment for subscription`

#### `/integrate:polar [tasks]` - Implement Polar.sh Payment
- **Purpose**: Integrate Polar.sh payment for developer tools
- **Use When**: Monetizing developer products, subscriptions
- **Example**: `/integrate:polar Add Polar checkout for premium features`

---

### **Bootstrap & Skills** 🚀

#### `/bootstrap:auto [user-requirements]` - Bootstrap New Project Automatically
- **Purpose**: Automatically set up new project with best practices
- **Output**: Complete project structure, config files, CI/CD, documentation
- **Use When**: Starting new projects, creating microservices
- **Example**: `/bootstrap:auto Create new FastAPI service for notifications`

#### `/skill:create [prompt]` - Create New Agent Skill
- **Purpose**: Create custom agent skill/command
- **Output**: New .md file in `.claude/commands/`
- **Use When**: Adding custom workflows, project-specific agents
- **Example**: `/skill:create Create agent for database migrations`

#### `/journal` - Write Journal Entries
- **Purpose**: Create development journal entries
- **Use When**: Daily logs, decision documentation, progress tracking
- **Example**: `/journal`

---

## 🎯 PROACTIVE AGENT SUGGESTION RULES

### **Claude MUST suggest agents when detecting these patterns:**

#### **Pattern: User mentions bugs/errors/issues**
- **Detect**: "bug", "error", "broken", "not working", "fails", "crash"
- **Suggest**:
  - Simple issue → `/fix:fast [issue]`
  - Complex issue → `/fix:hard [issue]`
  - UI issue → `/fix:ui [issue]`
  - CI/CD failure → `/fix:ci [url]`

#### **Pattern: User wants to implement features**
- **Detect**: "add", "implement", "create", "build", "new feature"
- **Suggest**:
  - Complex feature → `/plan [feature]` → `/cook [tasks]`
  - UI/Design → `/design:good [tasks]` → `/cook [tasks]`
  - Simple feature → `/cook [tasks]` directly

#### **Pattern: User asks questions**
- **Detect**: "how to", "should I", "what's the best way", "architecture"
- **Suggest**:
  - Architectural → `/ask [question]`
  - Multiple solutions → `/brainstorm [question]`
  - Find code → `/scout [prompt] [scale]`

#### **Pattern: User mentions design/UI/UX**
- **Detect**: "design", "UI", "UX", "layout", "looks", "style", "frontend"
- **Suggest**:
  - New design → `/design:good [tasks]`
  - Quick prototype → `/design:fast [tasks]`
  - Fix UI bug → `/fix:ui [issue]`
  - Analyze design → `/design:describe [screenshot]`

#### **Pattern: User wants to commit/push**
- **Detect**: "commit", "push", "save changes", "git"
- **Suggest**:
  - Commit only → `/git:cm`
  - Commit + push → `/git:cp`
  - Create PR → `/git:pr [branch] [from-branch]`

#### **Pattern: User asks about recent work**
- **Detect**: "what did I change", "recent work", "summary", "review changes"
- **Suggest**: `/watzup`

#### **Pattern: User mentions docs/documentation**
- **Detect**: "documentation", "docs", "README", "update docs"
- **Suggest**:
  - Update docs → `/docs:update`
  - New project → `/docs:init`
  - Summary → `/docs:summarize`

#### **Pattern: User mentions tests**
- **Detect**: "test", "testing", "coverage", "failing tests"
- **Suggest**:
  - Run tests → `/test`
  - Fix tests → `/fix:test`

---

## 💡 EXAMPLE PROACTIVE CONVERSATIONS

### Example 1: Feature Implementation
```
User: "I want to add trailing stop loss"

Claude (PROACTIVE):
"Great idea! Để implement trailing stop loss đúng cách, tôi suggest workflow này:

1. `/plan Add trailing stop loss feature`
   → Research best practices, create detailed plan

2. `/cook Implement trailing stop loss with specs, tests, docs`
   → Full implementation with subagent orchestration

3. `/test`
   → Verify all tests pass

4. `/git:cp`
   → Commit and push when done

Bạn muốn tôi bắt đầu với /plan không?"
```

### Example 2: Bug Fix
```
User: "WebSocket keeps disconnecting randomly"

Claude (PROACTIVE):
"Đây là issue phức tạp. Tôi suggest:

1. `/debug WebSocket keeps disconnecting randomly`
   → Analyze root cause với debugger subagent

2. Sau khi có root cause:
   - Simple fix → `/fix:fast [issue]`
   - Complex fix → `/fix:hard [issue]`

3. `/test`
   → Verify fix works

Để tôi dùng /debug trước để tìm root cause nhé?"
```

### Example 3: Design Work
```
User: "Dashboard cần thiết kế lại, trông cũ quá"

Claude (PROACTIVE):
"Perfect timing! Tôi có thể giúp với workflow này:

1. `/design:describe [current-dashboard-screenshot]`
   → Analyze current design, identify issues

2. `/design:good Create modern trading dashboard with dark mode`
   → Create award-winning design với full UX research

3. `/cook Implement new dashboard design`
   → Implement với proper components và tests

4. `/docs:update`
   → Update design guidelines

Bạn có screenshot của dashboard hiện tại không?
Hoặc tôi start luôn với /design:good?"
```

---

## ⚠️ CRITICAL RULES FOR AGENT USAGE

1. **ALWAYS suggest appropriate agent** when pattern detected
2. **EXPLAIN why that agent** is the right choice
3. **OFFER workflow** (multiple agents in sequence)
4. **ASK permission** before executing (unless obvious)
5. **NEVER code manually** for complex features
6. **PREFER agents over direct implementation** (90% of cases)

---

**Why agents are MANDATORY:**
- ✅ Follow specs precisely
- ✅ Add proper @spec tags automatically
- ✅ Run tests and validation
- ✅ Update documentation
- ✅ Maintain code quality standards
- ✅ Prevent finance-critical mistakes
- ✅ Orchestrate subagents efficiently
- ✅ Generate comprehensive reports

#### Step 3: ADD @spec TAGS TO CODE (REQUIRED)

After implementation, verify ALL code has @spec tags:

**Python:**
```python
# @spec:FR-ASYNC-001 - Async ML Model Training
# @ref:specs/01-requirements/1.1-functional-requirements/FR-ASYNC-TASKS.md
# @test:TC-ASYNC-001, TC-ASYNC-002, TC-ASYNC-003
async def train_model(...):
    ...
```

**Rust:**
```rust
// @spec:FR-RISK-007 - Trailing Stop Loss (Long Positions)
// @ref:specs/01-requirements/1.1-functional-requirements/FR-RISK.md
// @test:TC-TRADING-054, TC-TRADING-055, TC-TRADING-056
pub fn update_trailing_stop(...) -> Result<...> {
    ...
}
```

**TypeScript:**
```typescript
// @spec:FR-DASHBOARD-006 - WebSocket Integration
// @ref:specs/02-design/2.5-components/COMP-FRONTEND-DASHBOARD.md
// @test:TC-INTEGRATION-040
export function useWebSocket() {
  ...
}
```

#### Step 4: UPDATE SPECS & DOCS (Mark Complete)

After implementation is tested and working:

1. **Update requirement spec**:
   - Change checkboxes: `- [ ]` → `- [x]`
   - Add implementation notes
   - Update code locations

2. **Update TRACEABILITY_MATRIX.md**:
   - Verify FR-XXX entry has correct code location
   - Update status: "Pending" → "✅ Implemented"
   - Add actual test case IDs

3. **Update feature documentation**:
   - Update `docs/features/*.md` if needed
   - Add to `CHANGELOG.md`
   - Update `README.md` if user-facing

4. **Run validation**:
   ```bash
   python3 scripts/validate-specs.py
   # Must show 0 errors for new requirement
   ```

#### Step 5: VERIFY TRACEABILITY (100% Required)

Before marking feature complete, verify:

```bash
# Check: Requirement exists
grep -r "FR-XXX-YYY" specs/01-requirements/

# Check: In traceability matrix
grep "FR-XXX-YYY" specs/TRACEABILITY_MATRIX.md

# Check: Has @spec tag in code
grep -r "@spec:FR-XXX-YYY" {rust-core-engine,python-ai-service,nextjs-ui-dashboard}/

# Check: Test cases exist
grep "TC-XXX-YYY" specs/03-testing/3.2-test-cases/

# Run validation
python3 scripts/validate-specs.py
```

**All checks must pass ✅**

---

## 📖 HOW CLAUDE SHOULD UNDERSTAND THE PROJECT

### ALWAYS Read Specs First (Before ANY task)

**When user asks about existing feature:**

1. **Check specs first** (NOT code):
   ```bash
   # Search requirements
   grep -r "feature-name" specs/01-requirements/

   # Read full spec
   cat specs/01-requirements/1.1-functional-requirements/FR-XXX.md

   # Check traceability
   grep "FR-XXX" specs/TRACEABILITY_MATRIX.md
   ```

2. **Then read code** (to verify implementation):
   - Use code location from TRACEABILITY_MATRIX.md
   - Verify @spec tags match
   - Check implementation matches spec

3. **Answer based on SPEC** (not just code):
   - Spec = source of truth
   - Code should match spec
   - If mismatch → code is wrong, not spec

**When user asks to modify existing feature:**

1. **Read current spec** to understand requirements
2. **Plan changes** (what needs updating in spec)
3. **Update spec FIRST** (before code)
4. **Then use agents** to update code
5. **Verify** code matches updated spec

**When explaining how system works:**

1. **Reference specs** for authoritative info
2. **Use code locations** from TRACEABILITY_MATRIX.md
3. **Cite specific FR-XXX** requirement IDs
4. **Show examples** from spec, not just code

### Spec File Reading Order

**For new features:**
```
1. FR-XXX.md (requirement) → understand WHAT to build
2. COMP-XXX.md (component) → understand HOW it's architected
3. API-XXX.md (API design) → understand interface
4. DB-SCHEMA.md (database) → understand data model
5. TC-XXX.md (test cases) → understand validation
6. Code (implementation) → see actual code
```

**For bug fixes:**
```
1. TRACEABILITY_MATRIX.md → find which FR-XXX covers this
2. FR-XXX.md → understand intended behavior
3. TC-XXX.md → understand test cases
4. Code → identify bug
5. Fix code to match spec (not vice versa)
```

**For questions:**
```
1. Search specs: grep -r "keyword" specs/
2. Read matched FR-XXX.md files
3. Check TRACEABILITY_MATRIX.md for code location
4. Read code to verify
5. Answer based on spec + code verification
```

---

## 🎯 VALIDATION RULES (Must Follow)

### Before Starting ANY Task

```bash
# 1. Validate current state
python3 scripts/validate-specs.py

# 2. Read relevant specs
cat specs/01-requirements/1.1-functional-requirements/FR-*.md

# 3. Check traceability
cat specs/TRACEABILITY_MATRIX.md | grep "FR-XXX"
```

### After Completing ANY Task

```bash
# 1. Verify @spec tags added
grep -r "@spec:FR-XXX" .

# 2. Update traceability matrix
# Edit: specs/TRACEABILITY_MATRIX.md

# 3. Run validation
python3 scripts/validate-specs.py
# Must show 0 new errors

# 4. Verify 100% traceability maintained
# Check: All new FRs in matrix
# Check: All code has @spec tags
```

### Continuous Validation

- ✅ Specs must be written BEFORE code
- ✅ All code must have @spec tags
- ✅ TRACEABILITY_MATRIX.md must be 100% complete
- ✅ Validation script must pass (0 critical errors)
- ✅ Finance-critical features must have A+ safety grade

**If any validation fails → DO NOT PROCEED until fixed**

---

---

## 🎯 QUICK FEATURE LOCATION MAP

### Paper Trading (Execution + Risk Management)
📄 **Doc**: `docs/features/paper-trading.md` (comprehensive guide)
📂 **Code**: `rust-core-engine/src/paper_trading/`
- **engine.rs**
  - `738-845`: Execution simulation (slippage, market impact, partial fills)
  - `847-1039`: Risk management (daily loss limit, cool-down, correlation)
  - `509-560`: process_trading_signal() - Risk checks before execution
  - `1041-1197`: execute_trade() - Full execution with all simulations
  - `1425-1452`: close_trade() - Consecutive loss tracking
- **portfolio.rs**
  - `77-81`: Cool-down state fields (consecutive_losses, cool_down_until)
  - `223-224`: Field initialization
- **trade.rs**
  - `145-152`: Latency tracking fields (signal_timestamp, execution_latency_ms)
  - `223-225`: Field initialization
- **settings.rs**: All configuration options

🧪 **Tests**: `rust-core-engine/tests/test_paper_trading.rs`
📊 **Quality**: 98% realism, 94.5/100 overall (A+)

**Common Tasks**:
- Enable slippage: Set `execution.simulate_slippage = true`
- Check daily loss: See `engine.rs:847 check_daily_loss_limit()`
- Monitor execution: `docker logs -f | grep "💸|⏳|📊|⚡"`

---

### Authentication & Authorization
📄 **Doc**: `docs/features/authentication.md`
📂 **Code**: `rust-core-engine/src/auth/`
- **jwt.rs**: JWT generation, validation, refresh
- **handlers.rs**: Login, logout, register endpoints
- **middleware.rs**: Auth middleware, token extraction
- **database.rs**: User database operations

🔑 **API**:
- `POST /api/auth/login` - Login with email/password
- `POST /api/auth/register` - Create new user
- `POST /api/auth/refresh` - Refresh expired token
- `GET /api/auth/me` - Get current user (protected)

🧪 **Tests**: `rust-core-engine/tests/test_auth.rs`
🔒 **Security**: 98/100 (A+), RS256 JWT, bcrypt hashing

**Common Tasks**:
- Generate keys: `openssl genrsa -out private_key.pem 2048`
- Test login: `curl -X POST /api/auth/login -d '{"email":...}'`

---

### AI & ML Integration
📄 **Doc**: `docs/features/ai-integration.md`
📂 **Code**: `python-ai-service/`
- **models/**: LSTM, GRU, Transformer implementations
- **main.py**:
  - `150-250`: GPT-4 analysis endpoint
  - `predict_price()`: ML model predictions
  - `analyze_market_sentiment()`: Sentiment analysis
- **features/**: Technical indicators, feature engineering

🤖 **Models**: LSTM (68%), GRU (65%), Transformer (70%), Ensemble (72%)

🔌 **API**:
- `POST /predict` - Price prediction
- `POST /analyze` - GPT-4 market analysis
- `POST /sentiment` - Sentiment analysis
- `POST /train` - Retrain models

🧪 **Tests**: `python-ai-service/tests/`
🎯 **Accuracy**: 70% average directional accuracy

**Common Tasks**:
- Get prediction: `curl -X POST /predict -d '{"symbol":"BTCUSDT"}'`
- Check OpenAI key: `echo $OPENAI_API_KEY`

---

### Trading Strategies
📄 **Doc**: `docs/features/trading-strategies.md`
📂 **Code**: `rust-core-engine/src/strategies/`
- **rsi_strategy.rs**: RSI strategy (62% win rate)
- **macd_strategy.rs**: MACD strategy (58% win rate)
- **bollinger_strategy.rs**: Bollinger Bands (60% win rate)
- **volume_strategy.rs**: Volume-based trading (52% win rate)
- **strategy_engine.rs**: Strategy orchestration
- **indicators.rs**: Technical calculations (RSI, MACD, EMA, etc.)

📊 **Performance**: 65% combined win rate, 1.5% avg profit, Sharpe 1.6

🔌 **API**:
- `GET /api/strategies/active` - List active strategies
- `GET /api/strategies/signals/:symbol` - Get signals
- `POST /api/strategies/backtest` - Run backtest

🧪 **Tests**: `rust-core-engine/tests/test_strategies.rs`

**Common Tasks**:
- Enable strategy: Set `strategies.rsi_enabled = true` in config.toml
- Backtest: `curl -X POST /api/strategies/backtest -d '{"strategy":"rsi"}'`

---

### WebSocket & Real-Time Communication
📄 **Doc**: `docs/features/websocket-realtime.md`
📂 **Code**:
- **Backend**: `rust-core-engine/src/binance/websocket.rs`, `src/websocket/`
- **Frontend**: `nextjs-ui-dashboard/src/hooks/useWebSocket.ts`

📡 **Endpoints**:
- Input: `wss://stream.binance.com:9443/ws` (Binance)
- Output: `ws://localhost:8080/ws` (Frontend)

📨 **Events**: price_update, signal_generated, trade_executed, portfolio_update, risk_event

⚡ **Latency**: <100ms end-to-end, 99.9% uptime

🧪 **Tests**: `nextjs-ui-dashboard/src/hooks/useWebSocket.test.tsx`

**Common Tasks**:
- Connect: `const { socket, connected } = useWebSocket();`
- Broadcast: `broadcaster.send(WebSocketEvent {...})`
- Monitor: `docker logs -f | grep "WebSocket"`

---

### Risk Management
📄 **Doc**: `docs/features/paper-trading.md#risk-management` (part of paper trading)
📂 **Code**: `rust-core-engine/src/paper_trading/engine.rs:847-1039`

🛡️ **Features**:
1. Daily Loss Limit (5% max)
2. Cool-Down Mechanism (60 min after 5 losses)
3. Position Correlation Limits (70% max directional)
4. Consecutive Loss Tracking (auto-reset on profit)

**Common Tasks**:
- Check daily loss: `check_daily_loss_limit()` at line 847
- Check cool-down: `is_in_cooldown()` at line 892
- Update settings: Modify `risk` section in config.toml

---

### Frontend Dashboard
📄 **Doc**: `nextjs-ui-dashboard/README.md`
📂 **Code**: `nextjs-ui-dashboard/src/`
- **pages/**: Route components
- **components/**: Reusable UI components (71 total)
- **hooks/**: Custom React hooks (useWebSocket, useAIAnalysis, usePaperTrading)
- **contexts/**: Global state (AuthContext, WebSocketContext)
- **lib/**: Utilities, API client

🎨 **UI Library**: Shadcn/UI + TailwindCSS
📦 **Bundle**: 400KB (optimized with code splitting)

🧪 **Tests**: `nextjs-ui-dashboard/src/**/*.test.tsx` (601 tests)

**Common Tasks**:
- Start dev: `cd nextjs-ui-dashboard && npm run dev`
- Build: `npm run build`
- Test: `npm run test`

---

### Database Schema
📄 **Doc**: `specs/02-design/2.2-database/DB-SCHEMA.md`
📂 **Code**: Database operations in each service

📊 **Collections** (17 total):
- `users` - User accounts
- `paper_portfolios` - Paper trading portfolios
- `paper_trades` - Executed paper trades
- `strategies` - Strategy configurations
- `market_data` - Historical candles
- `signals` - AI trading signals
- See full schema in DB-SCHEMA.md

**Common Tasks**:
- Connect: MongoDB running on localhost:27017
- View data: Use MongoDB Compass or mongosh
- Indexes: See `DB-INDEXES.md` for 37 indexes

---

## 📚 DOCUMENTATION STRUCTURE

**Two main directories** - Clean and organized:

### 1️⃣ `/docs/` - Operational Documentation (for users & developers)
- **`features/`** - Feature-specific guides (5 docs, <500 lines each)
  - `paper-trading.md` - Paper trading system
  - `authentication.md` - Auth & JWT
  - `ai-integration.md` - ML models & GPT-4
  - `trading-strategies.md` - RSI, MACD, Bollinger, Volume
  - `websocket-realtime.md` - Real-time communication
- **`guides/`** - User guides & how-to documents
- **`reports/`** - Implementation reports, phase summaries
- **`plans/`** - Planning documents, validation guides
- **`testing/`** - Testing documentation
- **`certificates/`** - Quality certificates & achievements
- **`archive/`** - Legacy documentation (old `/documents` content)
- **Root docs**: `CONTRIBUTING.md`, `TESTING_GUIDE.md`, `TROUBLESHOOTING.md`, `PRODUCTION_DEPLOYMENT_GUIDE.md`

### 2️⃣ `/specs/` - Technical Specifications (for spec-driven development)
- **`01-requirements/`** - 24 docs (194 requirements, 63 user stories)
- **`02-design/`** - 20 docs (Architecture, API, DB schema)
- **`03-testing/`** - 12 docs (186 test cases, 45 scenarios)
- **`04-deployment/`** - 7 docs (Infrastructure, CI/CD)
- **`05-operations/`** - 3 docs (Operations, DR plan)
- **Root specs**: `TRACEABILITY_MATRIX.md`, `TASK_TRACKER.md`, `README.md`

**Why 2 directories?**
- `/docs` = Operational docs for **daily use** (guides, troubleshooting, reports)
- `/specs` = Formal specifications for **development** (requirements, design, traceability)

---

## 🚀 DEVELOPMENT WORKFLOW

### Quick Start Commands
```bash
# Setup
cp .env.example .env && ./scripts/generate-secrets.sh

# Start all services (memory-optimized)
./scripts/bot.sh start --memory-optimized

# Development mode with hot reload
./scripts/bot.sh dev

# Status & logs
./scripts/bot.sh status
./scripts/bot.sh logs --service rust-core-engine
```

### Build & Test
```bash
# Build all
make build              # Or: make build-fast (sequential, memory-safe)

# Test all (2,202+ tests)
make test               # Rust (1,336) + Python (409) + Frontend (601)

# Quality checks
make lint               # Zero errors required
make quality-metrics    # Current: 94/100 (Grade A)
```

### Service-Specific
```bash
# Rust
cd rust-core-engine
cargo fmt --check && cargo clippy -- -D warnings && cargo test

# Python
cd python-ai-service
black . && flake8 . && pytest --cov

# Frontend
cd nextjs-ui-dashboard
npm run lint && npm run type-check && npm test
```

---

## 🎯 PROJECT STATUS

### Quality Metrics
- **Overall**: 94/100 (Grade A)
- **Security**: 98/100 (A+)
- **Test Coverage**: 90.4% average (Rust 90%, Python 95%, Frontend 90%+)
- **Mutation Score**: 84% average (Rust 78%, Python 76%, Frontend 75%)
- **Code Quality**: PERFECT 10/10
- **Documentation**: 96/100 (A+)
- **Performance**: 95/100 (A+)

### Production Readiness
- ✅ Zero HIGH/CRITICAL vulnerabilities
- ✅ All tests passing (2,202+ tests)
- ✅ Zero ESLint errors
- ✅ Zero compiler warnings
- ✅ Comprehensive documentation (15,000+ lines)
- ✅ Production deployment guide ready

### Tech Stack
- **Backend**: Rust 1.86+ (Actix-web, MongoDB)
- **AI/ML**: Python 3.11+ (FastAPI, TensorFlow, PyTorch, OpenAI GPT-4)
- **Frontend**: TypeScript, React 18, Vite, Shadcn/UI, TailwindCSS
- **Database**: MongoDB with replica sets
- **Real-Time**: WebSocket (Binance + Frontend)

---

## ⚡ COMMON QUESTIONS (Quick Answers)

### "Where is paper trading execution simulation?"
→ `rust-core-engine/src/paper_trading/engine.rs:1041-1197`
→ Read `docs/features/paper-trading.md` for details

### "How do I enable slippage?"
→ Set `execution.simulate_slippage = true` in settings
→ Or: `curl -X POST /api/paper-trading/settings -d '{"execution":{"simulate_slippage":true}}'`

### "Where are JWT tokens generated?"
→ `rust-core-engine/src/auth/jwt.rs`
→ Read `docs/features/authentication.md` for API usage

### "How do I get AI price predictions?"
→ `POST http://localhost:8000/predict` with `{"symbol":"BTCUSDT"}`
→ Read `docs/features/ai-integration.md` for all endpoints

### "Where are trading strategies defined?"
→ `rust-core-engine/src/strategies/` (4 strategies)
→ Read `docs/features/trading-strategies.md` for performance metrics

### "How do I monitor WebSocket connections?"
→ `docker logs -f rust-core-engine-dev | grep "WebSocket"`
→ Read `docs/features/websocket-realtime.md` for event types

### "Where is the database schema?"
→ `specs/02-design/2.2-database/DB-SCHEMA.md` (17 collections)

### "How do I deploy to production?"
→ Read `docs/PRODUCTION_DEPLOYMENT_GUIDE.md` (1,300+ lines, comprehensive)

### "Where are test files?"
→ Rust: `rust-core-engine/tests/`
→ Python: `python-ai-service/tests/`
→ Frontend: `nextjs-ui-dashboard/src/**/*.test.tsx`

### "How do I troubleshoot issues?"
→ Read `docs/TROUBLESHOOTING.md` for common issues
→ Or: Check feature-specific docs in `docs/features/`

---

## 🔒 SECURITY & BEST PRACTICES

### Secrets Management
- **NEVER** commit: API keys, passwords, JWT secrets, tokens
- **ALWAYS** use `.env`: `cp .env.example .env`
- **Generate** secure secrets: `./scripts/generate-secrets.sh`
- **Validate**: `make validate-secrets`

### Code Quality Standards
- **Rust**: Zero unwrap()/expect() in production, use `?` operator
- **Python**: Black formatted, 98%+ type hints, Flake8 compliant
- **TypeScript**: ESLint clean, strict mode, 0 errors/warnings

### Before Committing
```bash
make lint               # Must pass (zero errors)
make test               # All tests must pass
make quality-metrics    # Must maintain ≥94/100
make security-check     # Zero HIGH/CRITICAL vulns
```

### Trading Safety (CRITICAL!)
- ✅ Testnet by default: `BINANCE_TESTNET=true`
- ✅ Trading disabled: `TRADING_ENABLED=false`
- ⚠️ **NEVER** enable production trading without explicit user request
- ⚠️ **ALWAYS** test with testnet first

---

## 📋 SPEC-DRIVEN DEVELOPMENT

This project follows **spec-driven development**. All features must conform to specifications BEFORE implementation.

### Specification System (100% Complete)
- **Location**: `specs/` directory (75 documents, 2.6MB)
- **Traceability**: `specs/TRACEABILITY_MATRIX.md` (100% bidirectional)
- **Code Tagging**: 47 @spec tags across 30 files

### Code Tagging Convention
```rust
// @doc:docs/features/paper-trading.md#execution-simulation
// @spec:FR-PAPER-001
// @ref:specs/02-design/2.5-components/COMP-RUST-TRADING.md
async fn execute_trade(...) { ... }
```

### Development Workflow
1. **Read spec first**: Check `specs/01-requirements/`, `specs/02-design/`
2. **Verify code tagging**: Look for @spec tags
3. **Update spec if needed**: BEFORE coding
4. **Add @spec tags**: To new code
5. **Test against spec**: Validate requirements

**Key Principles**:
- Spec is source of truth (code matches spec)
- No undocumented features
- Code tagging required
- 100% traceability

---

## 🎖️ ACHIEVEMENTS

**World-Class Status**:
- 🏆 PERFECT 10/10 quality score
- ⭐ 94/100 overall (Grade A)
- 🔒 98/100 security (A+)
- 📊 90.4% coverage, 2,202+ tests
- 🧬 84% mutation score
- 📚 96/100 documentation (A+)
- ⚡ 95/100 performance (A+)
- 🎯 **Top 10% worldwide**

---

## 🆘 GETTING HELP

### Quick Reference
- **Service URLs**: Frontend (3000), Rust API (8080), Python AI (8000)
- **Logs**: `./scripts/bot.sh logs --service <name>`
- **Health**: `curl http://localhost:8080/api/health`

### Documentation
- **Features**: `docs/features/` (5 focused guides)
- **Specs**: `specs/` (75 comprehensive docs)
- **Guides**: `docs/CONTRIBUTING.md`, `docs/TESTING_GUIDE.md`, `docs/TROUBLESHOOTING.md`

### Common Issues
1. **Out of Memory**: Use `./scripts/bot.sh start --memory-optimized`
2. **Port Conflicts**: Check with `lsof -i :3000/8000/8080`
3. **Service Unhealthy**: View logs `./scripts/bot.sh logs --service <name>`
4. **Build Failures**: Use `make build-fast` (sequential)

---

**Last Updated**: 2025-11-23
**Status**: PRODUCTION-READY | WORLD-CLASS QUALITY | SPEC-DRIVEN | AGENT-FIRST
**Version**: 4.0 (Complete Agent Catalog + Proactive Agent Suggestions)

**Major Changes in v4.0**:
- ✅ **COMPLETE AGENT CATALOG** - 36+ specialized agents fully documented
- ✅ **PROACTIVE SUGGESTION RULES** - Pattern detection for automatic agent suggestions
- ✅ **AGENT-FIRST APPROACH** - Claude must suggest agents proactively (90% of cases)
- ✅ **WORKFLOW EXAMPLES** - Real conversation examples showing agent usage
- ✅ **CRITICAL RULES** - 6 mandatory rules for agent usage enforcement
- ✅ Added detailed documentation for ALL agent categories:
  - Core Development (5 agents): plan, cook, test, debug, watzup
  - Q&A & Brainstorming (3 agents): ask, brainstorm, scout
  - Fixing Issues (7 agents): fast, hard, test, types, ui, ci, logs
  - Design & UI/UX (6 agents): good, fast, 3d, screenshot, video, describe
  - Documentation (3 agents): init, update, summarize
  - Git Operations (3 agents): cm, cp, pr
  - Content Writing (4 agents): good, fast, enhance, cro
  - Integration (2 agents): sepay, polar
  - Bootstrap & Skills (3 agents): auto, create, journal

**Changes from v3.0**:
- ✅ Added MANDATORY spec-driven development workflow
- ✅ Specs MUST be written BEFORE code
- ✅ All code MUST use agents for implementation
- ✅ All code MUST have @spec tags
- ✅ 100% traceability REQUIRED (256 requirements tracked)
- ✅ Validation script enforces quality standards
- ✅ Claude MUST read specs first (not just code)
