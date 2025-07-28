# 📁 Cấu Trúc Thư Mục Bot Core

## 🎯 Tổng Quan

Bot Core sử dụng cấu trúc thư mục được tổ chức tốt để dễ dàng quản lý và scale:

```
bot-core/
├── documents/           # Documentation
├── infrastructure/     # Infrastructure configs
├── tests/              # Test suites  
├── scripts/           # Utility scripts
├── rust-core-engine/   # Rust service
├── python-ai-service/  # Python service
└── nextjs-ui-dashboard/ # Frontend
```

## 📂 Chi Tiết Từng Thư Mục

### 📚 documents/
Tất cả documentation của project:
- `CHANGELOG.md` - Lịch sử thay đổi
- `DATABASE_ARCHITECTURE.md` - Kiến trúc database
- `DEPLOYMENT.md` - Hướng dẫn deploy
- `DISASTER_RECOVERY.md` - Kế hoạch phục hồi
- `FOLDER_STRUCTURE.md` - File này
- `NEW_FEATURES.md` - Tính năng mới
- `PRODUCTION_DEPLOYMENT.md` - Deploy production
- `SECURITY.md` - Best practices bảo mật
- `SYSTEM_OVERVIEW_10.md` - Tổng quan hệ thống

### 🔧 infrastructure/
Tất cả infrastructure configurations:

#### docker/
- `docker-compose.yml` - Main compose file
- `docker-compose.prod.yml` - Production overrides

#### kubernetes/
- `istio-services.yaml` - Service mesh configs

#### terraform/
- `main.tf` - Infrastructure as Code

#### nginx/
- `nginx.conf` - Load balancer config

#### kong/
- `kong.yml` - API Gateway config

#### rabbitmq/
- `rabbitmq.conf` - RabbitMQ config
- `definitions.json` - Queue definitions

#### mongodb/
- `init-replica.js` - Replica set init
- `replica.key` - Security key

#### monitoring/
- `prometheus.yml` - Metrics config
- `alerts/alerts.yml` - Alert rules

### 🧪 tests/
Centralized testing:

#### e2e/
- Cypress end-to-end tests
- Full user flow testing

#### integration/
- Service integration tests (future)

#### performance/
- Load & stress tests (future)

### 🛠️ scripts/
Utility scripts:
- `bot.sh` - Main control script
- `demo.sh` - Feature demonstration
- `generate-secrets.sh` - Generate secure tokens
- `verify-setup.sh` - Verify configuration
- `reorganize-structure.sh` - Folder reorganization

### 🦀 rust-core-engine/
Rust trading engine:
- `src/` - Source code
- `Cargo.toml` - Dependencies
- `config.toml` - Configuration
- `Dockerfile` - Container build

### 🐍 python-ai-service/
Python AI/ML service:
- `models/` - ML models
- `utils/` - Utilities
- `features/` - Feature engineering
- `config.yaml` - Configuration

### ⚛️ nextjs-ui-dashboard/
React frontend:
- `src/` - Source code
- `public/` - Static assets
- `package.json` - Dependencies
- `vite.config.ts` - Build config

## 🔄 Symlinks

Để maintain backward compatibility:
- `docker-compose.yml` → `infrastructure/docker/docker-compose.yml`
- `docker-compose.prod.yml` → `infrastructure/docker/docker-compose.prod.yml`

## 🚀 Benefits

1. **Organization**: Infrastructure configs tách biệt khỏi code
2. **Scalability**: Dễ thêm services mới
3. **Maintainability**: Clear separation of concerns
4. **Testing**: Centralized test location
5. **Documentation**: Tất cả docs trong một folder

## 📝 Migration Notes

Nếu upgrade từ version cũ:
1. Run `./scripts/reorganize-structure.sh`
2. Verify với `./scripts/bot.sh verify`
3. Remove old folders nếu mọi thứ OK

## 🎯 Best Practices

1. **Docs**: Luôn update documents/ khi có changes
2. **Tests**: Viết tests trong tests/ folder phù hợp
3. **Scripts**: Tạo reusable scripts trong scripts/
4. **Config**: Infrastructure configs trong infrastructure/