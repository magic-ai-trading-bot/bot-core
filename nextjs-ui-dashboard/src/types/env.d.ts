/// <reference types="vite/client" />

interface ImportMetaEnv {
  readonly VITE_RUST_API_URL: string;
  readonly VITE_PYTHON_AI_URL: string;
  readonly VITE_WS_URL: string;
  readonly VITE_API_TIMEOUT: string;
  readonly VITE_REFRESH_INTERVAL: string;
  readonly VITE_ENABLE_REALTIME: string;
}

interface ImportMeta {
  readonly env: ImportMetaEnv;
}
