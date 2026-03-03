# Kubernetes Configuration Specification

**Document Version:** 1.0.0
**Last Updated:** 2025-10-11
**Status:** Active
**Owner:** DevOps Team

---

## Table of Contents

- [1. Overview](#1-overview)
- [2. Cluster Architecture](#2-cluster-architecture)
- [3. Namespace Configuration](#3-namespace-configuration)
- [4. Deployment Manifests](#4-deployment-manifests)
- [5. Service Configuration](#5-service-configuration)
- [6. Ingress Configuration](#6-ingress-configuration)
- [7. ConfigMaps and Secrets](#7-configmaps-and-secrets)
- [8. Persistent Volumes](#8-persistent-volumes)
- [9. Auto-Scaling](#9-auto-scaling)
- [10. Service Mesh](#10-service-mesh)
- [11. Monitoring Integration](#11-monitoring-integration)

---

## 1. Overview

### 1.1 Purpose

This document provides Kubernetes deployment specifications for the Bot Core platform in production environments.

### 1.2 Cluster Requirements

**Minimum Cluster Size:**
- Nodes: 3 (for HA)
- Node Type: 4 vCPU, 16GB RAM
- Kubernetes Version: 1.27+
- Storage: 1TB SSD per node

---

## 2. Cluster Architecture

```
┌─────────────────────────────────────────────────────────┐
│              Kubernetes Cluster (Production)             │
├─────────────────────────────────────────────────────────┤
│                                                           │
│  ┌───────────┐  ┌───────────┐  ┌───────────┐           │
│  │  Node-1   │  │  Node-2   │  │  Node-3   │           │
│  │  (Master) │  │  (Worker) │  │  (Worker) │           │
│  └─────┬─────┘  └─────┬─────┘  └─────┬─────┘           │
│        │              │              │                   │
│  ┌─────▼──────────────▼──────────────▼─────┐           │
│  │         Namespace: bot-core-production    │           │
│  │                                            │           │
│  │  ┌──────────┐  ┌──────────┐  ┌────────┐ │           │
│  │  │ Frontend │  │   Rust   │  │ Python │ │           │
│  │  │ (3 pods) │  │(3 pods)  │  │(3 pods)│ │           │
│  │  └─────┬────┘  └─────┬────┘  └────┬───┘ │           │
│  │        └──────────────┼────────────┘     │           │
│  │                       │                   │           │
│  │               ┌───────▼────────┐         │           │
│  │               │   MongoDB      │         │           │
│  │               │   StatefulSet  │         │           │
│  │               │   (3 replicas) │         │           │
│  │               └────────────────┘         │           │
│  └────────────────────────────────────────┘           │
└─────────────────────────────────────────────────────────┘
```

---

## 3. Namespace Configuration

### 3.1 Namespace Manifest

**File:** `infrastructure/kubernetes/base/namespace.yaml`

```yaml
apiVersion: v1
kind: Namespace
metadata:
  name: bot-core-production
  labels:
    environment: production
    project: bot-core
    managed-by: kubectl
```

### 3.2 Resource Quotas

```yaml
apiVersion: v1
kind: ResourceQuota
metadata:
  name: bot-core-quota
  namespace: bot-core-production
spec:
  hard:
    requests.cpu: "40"
    requests.memory: 80Gi
    limits.cpu: "80"
    limits.memory: 160Gi
    persistentvolumeclaims: "10"
    services.loadbalancers: "2"
```

### 3.3 Limit Ranges

```yaml
apiVersion: v1
kind: LimitRange
metadata:
  name: bot-core-limits
  namespace: bot-core-production
spec:
  limits:
  - max:
      cpu: "8"
      memory: 16Gi
    min:
      cpu: "100m"
      memory: 128Mi
    default:
      cpu: "1"
      memory: 1Gi
    defaultRequest:
      cpu: "500m"
      memory: 512Mi
    type: Container
```

---

## 4. Deployment Manifests

### 4.1 Frontend Deployment

**File:** `infrastructure/kubernetes/services/frontend/deployment.yaml`

```yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: nextjs-ui-dashboard
  namespace: bot-core-production
  labels:
    app: nextjs-ui-dashboard
    tier: frontend
    version: v1.0.0
spec:
  replicas: 3
  strategy:
    type: RollingUpdate
    rollingUpdate:
      maxSurge: 1
      maxUnavailable: 1
  selector:
    matchLabels:
      app: nextjs-ui-dashboard
  template:
    metadata:
      labels:
        app: nextjs-ui-dashboard
        tier: frontend
        version: v1.0.0
      annotations:
        prometheus.io/scrape: "true"
        prometheus.io/port: "3000"
        prometheus.io/path: "/metrics"
    spec:
      affinity:
        podAntiAffinity:
          preferredDuringSchedulingIgnoredDuringExecution:
          - weight: 100
            podAffinityTerm:
              labelSelector:
                matchExpressions:
                - key: app
                  operator: In
                  values:
                  - nextjs-ui-dashboard
              topologyKey: kubernetes.io/hostname
      containers:
      - name: nextjs-ui-dashboard
        image: your-registry.com/nextjs-ui-dashboard:latest
        imagePullPolicy: Always
        ports:
        - containerPort: 3000
          name: http
          protocol: TCP
        env:
        - name: NODE_ENV
          value: "production"
        - name: VITE_RUST_API_URL
          value: "http://rust-core-engine:8080"
        - name: VITE_PYTHON_AI_URL
          value: "http://python-ai-service:8000"
        - name: VITE_WS_URL
          value: "ws://rust-core-engine:8080/ws"
        - name: DASHBOARD_SESSION_SECRET
          valueFrom:
            secretKeyRef:
              name: bot-core-secrets
              key: dashboard-session-secret
        resources:
          requests:
            memory: "256Mi"
            cpu: "500m"
          limits:
            memory: "1Gi"
            cpu: "1000m"
        livenessProbe:
          httpGet:
            path: /health
            port: 3000
          initialDelaySeconds: 30
          periodSeconds: 10
          timeoutSeconds: 5
          successThreshold: 1
          failureThreshold: 3
        readinessProbe:
          httpGet:
            path: /health
            port: 3000
          initialDelaySeconds: 10
          periodSeconds: 5
          timeoutSeconds: 3
          successThreshold: 1
          failureThreshold: 3
        lifecycle:
          preStop:
            exec:
              command: ["/bin/sh", "-c", "sleep 15"]
      terminationGracePeriodSeconds: 30
```

### 4.2 Rust Core Engine Deployment

**File:** `infrastructure/kubernetes/services/rust-engine/deployment.yaml`

```yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: rust-core-engine
  namespace: bot-core-production
  labels:
    app: rust-core-engine
    tier: backend
    version: v1.0.0
spec:
  replicas: 3
  strategy:
    type: RollingUpdate
    rollingUpdate:
      maxSurge: 1
      maxUnavailable: 0  # Zero downtime
  selector:
    matchLabels:
      app: rust-core-engine
  template:
    metadata:
      labels:
        app: rust-core-engine
        tier: backend
        version: v1.0.0
      annotations:
        prometheus.io/scrape: "true"
        prometheus.io/port: "8080"
        prometheus.io/path: "/metrics"
    spec:
      affinity:
        podAntiAffinity:
          requiredDuringSchedulingIgnoredDuringExecution:
          - labelSelector:
              matchExpressions:
              - key: app
                operator: In
                values:
                - rust-core-engine
            topologyKey: kubernetes.io/hostname
      containers:
      - name: rust-core-engine
        image: your-registry.com/rust-core-engine:latest
        imagePullPolicy: Always
        ports:
        - containerPort: 8080
          name: http
          protocol: TCP
        env:
        - name: RUST_LOG
          value: "info"
        - name: DATABASE_URL
          valueFrom:
            secretKeyRef:
              name: bot-core-secrets
              key: database-url
        - name: PYTHON_AI_SERVICE_URL
          value: "http://python-ai-service:8000"
        - name: BINANCE_API_KEY
          valueFrom:
            secretKeyRef:
              name: binance-secrets
              key: api-key
        - name: BINANCE_SECRET_KEY
          valueFrom:
            secretKeyRef:
              name: binance-secrets
              key: secret-key
        - name: BINANCE_TESTNET
          value: "true"
        - name: TRADING_ENABLED
          value: "false"
        - name: INTER_SERVICE_TOKEN
          valueFrom:
            secretKeyRef:
              name: bot-core-secrets
              key: inter-service-token
        - name: RUST_API_KEY
          valueFrom:
            secretKeyRef:
              name: bot-core-secrets
              key: rust-api-key
        volumeMounts:
        - name: config
          mountPath: /app/config.toml
          subPath: config.toml
        - name: data
          mountPath: /app/data
        - name: logs
          mountPath: /app/logs
        resources:
          requests:
            memory: "1Gi"
            cpu: "1000m"
          limits:
            memory: "2Gi"
            cpu: "2000m"
        livenessProbe:
          httpGet:
            path: /api/health
            port: 8080
          initialDelaySeconds: 60
          periodSeconds: 10
          timeoutSeconds: 5
          failureThreshold: 3
        readinessProbe:
          httpGet:
            path: /api/health
            port: 8080
          initialDelaySeconds: 10
          periodSeconds: 5
          timeoutSeconds: 3
          failureThreshold: 3
        lifecycle:
          preStop:
            exec:
              command: ["/bin/sh", "-c", "sleep 20"]
      volumes:
      - name: config
        configMap:
          name: rust-config
      - name: data
        emptyDir: {}
      - name: logs
        emptyDir: {}
      terminationGracePeriodSeconds: 30
```

### 4.3 Python AI Service Deployment

**File:** `infrastructure/kubernetes/services/python-ai/deployment.yaml`

```yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: python-ai-service
  namespace: bot-core-production
  labels:
    app: python-ai-service
    tier: backend
    version: v1.0.0
spec:
  replicas: 3
  strategy:
    type: RollingUpdate
    rollingUpdate:
      maxSurge: 1
      maxUnavailable: 0
  selector:
    matchLabels:
      app: python-ai-service
  template:
    metadata:
      labels:
        app: python-ai-service
        tier: backend
        version: v1.0.0
      annotations:
        prometheus.io/scrape: "true"
        prometheus.io/port: "8000"
        prometheus.io/path: "/metrics"
    spec:
      affinity:
        podAntiAffinity:
          preferredDuringSchedulingIgnoredDuringExecution:
          - weight: 100
            podAffinityTerm:
              labelSelector:
                matchExpressions:
                - key: app
                  operator: In
                  values:
                  - python-ai-service
              topologyKey: kubernetes.io/hostname
      containers:
      - name: python-ai-service
        image: your-registry.com/python-ai-service:latest
        imagePullPolicy: Always
        ports:
        - containerPort: 8000
          name: http
          protocol: TCP
        env:
        - name: PYTHONPATH
          value: "/app"
        - name: PYTHONUNBUFFERED
          value: "1"
        - name: PYTHONDONTWRITEBYTECODE
          value: "1"
        - name: LOG_LEVEL
          value: "INFO"
        - name: INTER_SERVICE_TOKEN
          valueFrom:
            secretKeyRef:
              name: bot-core-secrets
              key: inter-service-token
        - name: PYTHON_API_KEY
          valueFrom:
            secretKeyRef:
              name: bot-core-secrets
              key: python-api-key
        - name: OPENAI_API_KEY
          valueFrom:
            secretKeyRef:
              name: openai-secrets
              key: api-key
        - name: DATABASE_URL
          valueFrom:
            secretKeyRef:
              name: bot-core-secrets
              key: database-url
        volumeMounts:
        - name: config
          mountPath: /app/config.yaml
          subPath: config.yaml
        - name: models
          mountPath: /app/models
        - name: logs
          mountPath: /app/logs
        - name: data
          mountPath: /app/data
        resources:
          requests:
            memory: "1Gi"
            cpu: "1000m"
          limits:
            memory: "2Gi"
            cpu: "2000m"
        livenessProbe:
          httpGet:
            path: /health
            port: 8000
          initialDelaySeconds: 60
          periodSeconds: 10
          timeoutSeconds: 5
          failureThreshold: 3
        readinessProbe:
          httpGet:
            path: /health
            port: 8000
          initialDelaySeconds: 10
          periodSeconds: 5
          timeoutSeconds: 3
          failureThreshold: 3
        lifecycle:
          preStop:
            exec:
              command: ["/bin/sh", "-c", "sleep 20"]
      volumes:
      - name: config
        configMap:
          name: python-config
      - name: models
        persistentVolumeClaim:
          claimName: ai-models-pvc
      - name: logs
        emptyDir: {}
      - name: data
        emptyDir: {}
      terminationGracePeriodSeconds: 30
```

---

## 5. Service Configuration

### 5.1 Frontend Service

```yaml
apiVersion: v1
kind: Service
metadata:
  name: nextjs-ui-dashboard
  namespace: bot-core-production
  labels:
    app: nextjs-ui-dashboard
    tier: frontend
spec:
  type: ClusterIP
  selector:
    app: nextjs-ui-dashboard
  ports:
  - port: 3000
    targetPort: 3000
    protocol: TCP
    name: http
  sessionAffinity: ClientIP
```

### 5.2 Rust Engine Service

```yaml
apiVersion: v1
kind: Service
metadata:
  name: rust-core-engine
  namespace: bot-core-production
  labels:
    app: rust-core-engine
    tier: backend
spec:
  type: ClusterIP
  selector:
    app: rust-core-engine
  ports:
  - port: 8080
    targetPort: 8080
    protocol: TCP
    name: http
  sessionAffinity: ClientIP
```

### 5.3 Python AI Service

```yaml
apiVersion: v1
kind: Service
metadata:
  name: python-ai-service
  namespace: bot-core-production
  labels:
    app: python-ai-service
    tier: backend
spec:
  type: ClusterIP
  selector:
    app: python-ai-service
  ports:
  - port: 8000
    targetPort: 8000
    protocol: TCP
    name: http
  sessionAffinity: None
```

---

## 6. Ingress Configuration

### 6.1 Ingress Controller Setup

**Install NGINX Ingress Controller:**
```bash
kubectl apply -f https://raw.githubusercontent.com/kubernetes/ingress-nginx/controller-v1.8.1/deploy/static/provider/cloud/deploy.yaml
```

### 6.2 Main Ingress

**File:** `infrastructure/kubernetes/services/ingress.yaml`

```yaml
apiVersion: networking.k8s.io/v1
kind: Ingress
metadata:
  name: bot-core-ingress
  namespace: bot-core-production
  annotations:
    kubernetes.io/ingress.class: "nginx"
    cert-manager.io/cluster-issuer: "letsencrypt-prod"
    nginx.ingress.kubernetes.io/ssl-redirect: "true"
    nginx.ingress.kubernetes.io/force-ssl-redirect: "true"
    nginx.ingress.kubernetes.io/rate-limit: "100"
    nginx.ingress.kubernetes.io/proxy-body-size: "10m"
    nginx.ingress.kubernetes.io/proxy-connect-timeout: "600"
    nginx.ingress.kubernetes.io/proxy-send-timeout: "600"
    nginx.ingress.kubernetes.io/proxy-read-timeout: "600"
spec:
  tls:
  - hosts:
    - botcore.app
    - www.botcore.app
    - api.botcore.app
    secretName: botcore-tls
  rules:
  - host: botcore.app
    http:
      paths:
      - path: /
        pathType: Prefix
        backend:
          service:
            name: nextjs-ui-dashboard
            port:
              number: 3000
  - host: www.botcore.app
    http:
      paths:
      - path: /
        pathType: Prefix
        backend:
          service:
            name: nextjs-ui-dashboard
            port:
              number: 3000
  - host: api.botcore.app
    http:
      paths:
      - path: /api
        pathType: Prefix
        backend:
          service:
            name: rust-core-engine
            port:
              number: 8080
      - path: /ws
        pathType: Prefix
        backend:
          service:
            name: rust-core-engine
            port:
              number: 8080
```

### 6.3 WebSocket Ingress

```yaml
apiVersion: networking.k8s.io/v1
kind: Ingress
metadata:
  name: bot-core-websocket
  namespace: bot-core-production
  annotations:
    kubernetes.io/ingress.class: "nginx"
    nginx.ingress.kubernetes.io/proxy-read-timeout: "3600"
    nginx.ingress.kubernetes.io/proxy-send-timeout: "3600"
    nginx.ingress.kubernetes.io/websocket-services: "rust-core-engine"
spec:
  tls:
  - hosts:
    - ws.botcore.app
    secretName: botcore-ws-tls
  rules:
  - host: ws.botcore.app
    http:
      paths:
      - path: /
        pathType: Prefix
        backend:
          service:
            name: rust-core-engine
            port:
              number: 8080
```

---

## 7. ConfigMaps and Secrets

### 7.1 ConfigMap for Rust

```yaml
apiVersion: v1
kind: ConfigMap
metadata:
  name: rust-config
  namespace: bot-core-production
data:
  config.toml: |
    [binance]
    testnet = true
    base_url = "https://testnet.binance.vision"
    ws_url = "wss://stream.testnet.binance.vision"

    [market_data]
    symbols = ["BTCUSDT", "ETHUSDT", "BNBUSDT", "SOLUSDT"]
    timeframes = ["1m", "5m", "15m", "1h", "4h"]
    kline_limit = 100
    update_interval_ms = 100

    [trading]
    enabled = false
    max_positions = 3
    default_quantity = 0.01
    risk_percentage = 2.0

    [api]
    host = "0.0.0.0"
    port = 8080
    cors_origins = ["*"]
    enable_metrics = true
```

### 7.2 ConfigMap for Python

```yaml
apiVersion: v1
kind: ConfigMap
metadata:
  name: python-config
  namespace: bot-core-production
data:
  config.yaml: |
    server:
      host: "0.0.0.0"
      port: 8000
      reload: false

    model:
      type: "lstm"
      sequence_length: 60
      features_count: 15
      hidden_size: 64

    trading:
      long_threshold: 0.55
      short_threshold: 0.45
      confidence_threshold: 0.45

    logging:
      level: "INFO"
      format: "{time} | {level} | {message}"
```

### 7.3 Secrets

```yaml
apiVersion: v1
kind: Secret
metadata:
  name: bot-core-secrets
  namespace: bot-core-production
type: Opaque
stringData:
  database-url: "mongodb://botuser:PASSWORD@mongodb:27017/trading_bot"
  inter-service-token: "GENERATED_TOKEN_HERE"
  rust-api-key: "GENERATED_API_KEY_HERE"
  python-api-key: "GENERATED_API_KEY_HERE"
  dashboard-session-secret: "GENERATED_SESSION_SECRET_HERE"
---
apiVersion: v1
kind: Secret
metadata:
  name: binance-secrets
  namespace: bot-core-production
type: Opaque
stringData:
  api-key: "YOUR_BINANCE_API_KEY"
  secret-key: "YOUR_BINANCE_SECRET_KEY"
---
apiVersion: v1
kind: Secret
metadata:
  name: openai-secrets
  namespace: bot-core-production
type: Opaque
stringData:
  api-key: "YOUR_OPENAI_API_KEY"
```

**Create secrets from command line:**
```bash
kubectl create secret generic bot-core-secrets \
  --from-literal=database-url='mongodb://...' \
  --from-literal=inter-service-token='...' \
  --from-literal=rust-api-key='...' \
  --from-literal=python-api-key='...' \
  --from-literal=dashboard-session-secret='...' \
  -n bot-core-production
```

---

## 8. Persistent Volumes

### 8.1 AI Models PVC

```yaml
apiVersion: v1
kind: PersistentVolumeClaim
metadata:
  name: ai-models-pvc
  namespace: bot-core-production
spec:
  accessModes:
    - ReadWriteMany
  storageClassName: fast-ssd
  resources:
    requests:
      storage: 50Gi
```

### 8.2 MongoDB StatefulSet

```yaml
apiVersion: apps/v1
kind: StatefulSet
metadata:
  name: mongodb
  namespace: bot-core-production
spec:
  serviceName: mongodb
  replicas: 3
  selector:
    matchLabels:
      app: mongodb
  template:
    metadata:
      labels:
        app: mongodb
    spec:
      containers:
      - name: mongodb
        image: mongo:7.0
        ports:
        - containerPort: 27017
        volumeMounts:
        - name: data
          mountPath: /data/db
        resources:
          requests:
            memory: "4Gi"
            cpu: "2"
          limits:
            memory: "8Gi"
            cpu: "4"
  volumeClaimTemplates:
  - metadata:
      name: data
    spec:
      accessModes: ["ReadWriteOnce"]
      storageClassName: fast-ssd
      resources:
        requests:
          storage: 100Gi
```

---

## 9. Auto-Scaling

### 9.1 Horizontal Pod Autoscaler - Frontend

```yaml
apiVersion: autoscaling/v2
kind: HorizontalPodAutoscaler
metadata:
  name: nextjs-ui-dashboard-hpa
  namespace: bot-core-production
spec:
  scaleTargetRef:
    apiVersion: apps/v1
    kind: Deployment
    name: nextjs-ui-dashboard
  minReplicas: 3
  maxReplicas: 10
  metrics:
  - type: Resource
    resource:
      name: cpu
      target:
        type: Utilization
        averageUtilization: 70
  - type: Resource
    resource:
      name: memory
      target:
        type: Utilization
        averageUtilization: 80
  behavior:
    scaleUp:
      stabilizationWindowSeconds: 60
      policies:
      - type: Percent
        value: 50
        periodSeconds: 60
      - type: Pods
        value: 2
        periodSeconds: 60
      selectPolicy: Max
    scaleDown:
      stabilizationWindowSeconds: 300
      policies:
      - type: Pods
        value: 1
        periodSeconds: 60
      selectPolicy: Min
```

### 9.2 HPA - Rust Engine

```yaml
apiVersion: autoscaling/v2
kind: HorizontalPodAutoscaler
metadata:
  name: rust-core-engine-hpa
  namespace: bot-core-production
spec:
  scaleTargetRef:
    apiVersion: apps/v1
    kind: Deployment
    name: rust-core-engine
  minReplicas: 3
  maxReplicas: 10
  metrics:
  - type: Resource
    resource:
      name: cpu
      target:
        type: Utilization
        averageUtilization: 70
  - type: Resource
    resource:
      name: memory
      target:
        type: Utilization
        averageUtilization: 75
  behavior:
    scaleUp:
      stabilizationWindowSeconds: 60
      policies:
      - type: Pods
        value: 3
        periodSeconds: 60
    scaleDown:
      stabilizationWindowSeconds: 300
      policies:
      - type: Pods
        value: 1
        periodSeconds: 120
```

### 9.3 HPA - Python AI

```yaml
apiVersion: autoscaling/v2
kind: HorizontalPodAutoscaler
metadata:
  name: python-ai-service-hpa
  namespace: bot-core-production
spec:
  scaleTargetRef:
    apiVersion: apps/v1
    kind: Deployment
    name: python-ai-service
  minReplicas: 3
  maxReplicas: 10
  metrics:
  - type: Resource
    resource:
      name: cpu
      target:
        type: Utilization
        averageUtilization: 75
  - type: Resource
    resource:
      name: memory
      target:
        type: Utilization
        averageUtilization: 80
  behavior:
    scaleUp:
      stabilizationWindowSeconds: 120
      policies:
      - type: Pods
        value: 2
        periodSeconds: 60
    scaleDown:
      stabilizationWindowSeconds: 300
      policies:
      - type: Pods
        value: 1
        periodSeconds: 180
```

---

## 10. Service Mesh

### 10.1 Istio Installation

**Install Istio:**
```bash
curl -L https://istio.io/downloadIstio | sh -
cd istio-*
export PATH=$PWD/bin:$PATH
istioctl install --set profile=production -y
```

**Enable sidecar injection:**
```bash
kubectl label namespace bot-core-production istio-injection=enabled
```

### 10.2 Virtual Service

```yaml
apiVersion: networking.istio.io/v1beta1
kind: VirtualService
metadata:
  name: rust-core-engine
  namespace: bot-core-production
spec:
  hosts:
  - rust-core-engine
  http:
  - match:
    - uri:
        prefix: /api
    route:
    - destination:
        host: rust-core-engine
        port:
          number: 8080
    timeout: 30s
    retries:
      attempts: 3
      perTryTimeout: 10s
      retryOn: 5xx,reset,connect-failure,refused-stream
```

### 10.3 Destination Rule

```yaml
apiVersion: networking.istio.io/v1beta1
kind: DestinationRule
metadata:
  name: rust-core-engine
  namespace: bot-core-production
spec:
  host: rust-core-engine
  trafficPolicy:
    connectionPool:
      tcp:
        maxConnections: 1000
      http:
        http1MaxPendingRequests: 1024
        http2MaxRequests: 1024
        maxRequestsPerConnection: 10
    loadBalancer:
      simple: LEAST_REQUEST
    outlierDetection:
      consecutiveErrors: 5
      interval: 30s
      baseEjectionTime: 30s
      maxEjectionPercent: 50
      minHealthPercent: 50
```

---

## 11. Monitoring Integration

### 11.1 ServiceMonitor for Prometheus

```yaml
apiVersion: monitoring.coreos.com/v1
kind: ServiceMonitor
metadata:
  name: bot-core-services
  namespace: bot-core-production
  labels:
    prometheus: kube-prometheus
spec:
  selector:
    matchLabels:
      tier: backend
  endpoints:
  - port: http
    interval: 30s
    path: /metrics
```

### 11.2 PrometheusRule

```yaml
apiVersion: monitoring.coreos.com/v1
kind: PrometheusRule
metadata:
  name: bot-core-alerts
  namespace: bot-core-production
spec:
  groups:
  - name: bot_core_alerts
    interval: 30s
    rules:
    - alert: HighErrorRate
      expr: rate(http_requests_total{status=~"5.."}[5m]) > 0.05
      for: 5m
      labels:
        severity: warning
      annotations:
        summary: "High error rate on {{ $labels.service }}"
        description: "Error rate is {{ $value }} errors/sec"

    - alert: PodCrashLooping
      expr: kube_pod_container_status_restarts_total > 5
      for: 5m
      labels:
        severity: critical
      annotations:
        summary: "Pod {{ $labels.pod }} is crash looping"

    - alert: HighMemoryUsage
      expr: container_memory_usage_bytes / container_spec_memory_limit_bytes > 0.9
      for: 5m
      labels:
        severity: warning
      annotations:
        summary: "High memory usage on {{ $labels.pod }}"
```

---

## Appendix: Deployment Commands

### Deploy to Kubernetes

```bash
# Apply all manifests
kubectl apply -k infrastructure/kubernetes/overlays/production/

# Or apply individually
kubectl apply -f infrastructure/kubernetes/base/namespace.yaml
kubectl apply -f infrastructure/kubernetes/services/

# Check status
kubectl get pods -n bot-core-production
kubectl get svc -n bot-core-production
kubectl get ingress -n bot-core-production

# View logs
kubectl logs -f deployment/rust-core-engine -n bot-core-production

# Scale deployment
kubectl scale deployment rust-core-engine --replicas=5 -n bot-core-production

# Rollout status
kubectl rollout status deployment/rust-core-engine -n bot-core-production

# Rollback
kubectl rollout undo deployment/rust-core-engine -n bot-core-production
```

---

## Revision History

| Version | Date | Author | Changes |
|---------|------|--------|---------|
| 1.0.0 | 2025-10-11 | DevOps Team | Initial version |

---

**Document End**
