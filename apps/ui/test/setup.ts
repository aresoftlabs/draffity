import { vi } from 'vitest';

// Default mock for the Tauri IPC module so tests don't try to call into a
// real backend. Individual tests override `invoke` per case via vi.mocked().
vi.mock('@tauri-apps/api/core', () => ({
  invoke: vi.fn(),
  convertFileSrc: vi.fn((path: string) => `asset://${path}`),
}));
