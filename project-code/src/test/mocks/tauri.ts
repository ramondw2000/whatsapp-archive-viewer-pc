import { vi } from "vitest";

/**
 * Default mock for @tauri-apps/api/core invoke.
 * Individual tests can override per-command with:
 *   mockInvoke.mockImplementation((cmd) => { ... });
 */
export const mockInvoke = vi.fn().mockResolvedValue("");

vi.mock("@tauri-apps/api/core", () => ({
  invoke: mockInvoke,
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
  mediaCache: {
    get: vi.fn(),
    set: vi.fn(),
    delete: vi.fn(),
    clear: vi.fn(),
  },
  showModuleToast: vi.fn(),
}));
