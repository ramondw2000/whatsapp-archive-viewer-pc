import { vi } from "vitest";

/**
 * Default mock for @tauri-apps/api/core invoke.
 * Individual tests can override per-command with:
 *   mockInvoke.mockImplementation((cmd) => { ... });
 */
export const mockInvoke = vi.fn().mockResolvedValue("");

vi.mock("@tauri-apps/api/core", () => ({
  invoke: mockInvoke,
  // Mirrors the real helper's shape closely enough for assertions on asset URLs
  convertFileSrc: (path: string) => `asset://localhost/${encodeURIComponent(path)}`,
}));

/**
 * Mock for useLazyVisibility — always reports the component as visible
 * so inner content renders immediately in tests.
 */
vi.mock("../../LazyMediaImage", () => ({
  useLazyVisibility: () => ({
    ref: { current: null },
    isVisible: true,
    style: {},
    className: "",
  }),
  showModuleToast: vi.fn(),
}));
