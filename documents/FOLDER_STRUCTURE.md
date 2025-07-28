# 📁 Cấu Trúc Thư Mục Bot Core

## 🎯 Đề Xuất Cải Thiện Cấu Trúc

### Cấu Trúc Hiện Tại vs Đề Xuất

```
bot-core/
├── 📋 Core Files (Root - OK)
│   ├── README.md
│   ├── CLAUDE.md
│   ├── Makefile
│   ├── .env.example
│   ├── docker-compose.yml
│   └── docker-compose.prod.yml
│
├── 📚 documents/ (OK)
│   ├── CHANGELOG.md
│   ├── DATABASE_ARCHITECTURE.md
│   ├── DEPLOYMENT.md
│   ├── DISASTER_RECOVERY.md
│   ├── NEW_FEATURES.md
│   ├── PRODUCTION_DEPLOYMENT.md
│   ├── SECURITY.md
│   ├── SYSTEM_OVERVIEW_10.md
│   └── FOLDER_STRUCTURE.md (new)
│
├── 🔧 infrastructure/ (ĐỀ XUẤT - gom configs)
│   ├── docker/
│   │   ├── docker-compose.yml
│   │   ├── docker-compose.prod.yml
│   │   └── docker-compose.dev.yml
│   ├── kubernetes/
│   │   ├── istio/
│   │   └── k8s/
│   ├── terraform/
│   ├── nginx/
│   ├── kong/
│   ├── rabbitmq/
│   ├── mongodb/
│   └── monitoring/
│
├── 🧪 tests/ (ĐỀ XUẤT - gom tests)
│   ├── e2e/
│   ├── integration/
│   └── performance/
│
├── 🛠️ scripts/ (OK)
│   ├── bot.sh
│   ├── demo.sh
│   ├── deploy.sh
│   ├── generate-secrets.sh
│   └── verify-setup.sh
│
├── 🦀 rust-core-engine/ (OK)
├── 🐍 python-ai-service/ (OK)
└── ⚛️ nextjs-ui-dashboard/ (OK)
```

## 🔄 Lợi Ích Của Cấu Trúc Mới

### 1. **infrastructure/** - Tập trung cấu hình
- Dễ quản lý tất cả infrastructure configs
- Clear separation of concerns
- Dễ tìm kiếm và maintain

### 2. **tests/** - Testing tập trung
- E2E tests
- Integration tests
- Performance tests
- Load tests

### 3. **Clean Root Directory**
- Chỉ giữ essential files ở root
- Infrastructure configs trong subfolder
- Dễ navigate hơn

## 📝 Files Cần Di Chuyển

### Bước 1: Tạo infrastructure/
```bash
mkdir -p infrastructure/{docker,kubernetes,terraform,nginx,kong,rabbitmq,mongodb,monitoring}
```

### Bước 2: Di chuyển files
```bash
# Docker files
mv docker-compose*.yml infrastructure/docker/

# Kubernetes
mv istio/ infrastructure/kubernetes/

# Other configs
mv nginx/ infrastructure/
mv kong/ infrastructure/
mv rabbitmq/ infrastructure/
mv mongodb/ infrastructure/
mv monitoring/ infrastructure/
mv terraform/ infrastructure/
```

### Bước 3: Tạo tests/
```bash
mkdir -p tests/{e2e,integration,performance}
mv e2e/* tests/e2e/
```

### Bước 4: Update paths
- Update Makefile
- Update scripts/bot.sh
- Update CI/CD workflows

## 🚫 Không Nên Di Chuyển

1. **Service directories** - Giữ nguyên:
   - rust-core-engine/
   - python-ai-service/
   - nextjs-ui-dashboard/

2. **Root configs** - Giữ nguyên:
   - README.md
   - CLAUDE.md
   - Makefile
   - .env.example

## 📌 Kết Luận

Cấu trúc hiện tại đã khá tốt, nhưng có thể cải thiện:
- ✅ Gom infrastructure configs
- ✅ Centralize testing
- ✅ Cleaner root directory
- ✅ Better organization

Tuy nhiên, nếu bạn thấy cấu trúc hiện tại đã quen thuộc và hoạt động tốt, có thể giữ nguyên!