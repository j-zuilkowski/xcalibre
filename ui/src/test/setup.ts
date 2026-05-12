import "@testing-library/jest-dom"
import { vi, afterEach } from "vitest"

const _invokeHandlers: Record<string, unknown> = {}
const localStorageStore = new Map<string, string>()

const localStorageMock = {
  get length() {
    return localStorageStore.size
  },
  clear() {
    localStorageStore.clear()
  },
  getItem(key: string) {
    return localStorageStore.has(key) ? localStorageStore.get(key)! : null
  },
  key(index: number) {
    return Array.from(localStorageStore.keys())[index] ?? null
  },
  removeItem(key: string) {
    localStorageStore.delete(key)
  },
  setItem(key: string, value: string) {
    localStorageStore.set(key, String(value))
  },
}

export function mockInvoke(cmd: string, value: unknown) {
  _invokeHandlers[cmd] = value
}

const invokeMock = vi.fn((cmd: string) => {
  const val = _invokeHandlers[cmd]
  if (val instanceof Error) return Promise.reject(val)
  return Promise.resolve(val ?? null)
})

Object.defineProperty(window, "__TAURI_INTERNALS__", {
  value: {
    invoke: invokeMock,
    transformCallback: vi.fn((cb: unknown) => {
      ;(window as any)._cb = cb
      return 1
    }),
    unregisterCallback: vi.fn(),
    metadata: { currentWindow: { label: "main" } },
    convertFileSrc: (src: string) => `asset://${src}`,
  },
  writable: true,
})

vi.stubGlobal("localStorage", localStorageMock)

// jsdom doesn't implement scrollIntoView
window.HTMLElement.prototype.scrollIntoView = vi.fn()

afterEach(() => {
  localStorageMock.clear()
  invokeMock.mockClear()
  Object.keys(_invokeHandlers).forEach((k) => delete _invokeHandlers[k])
})

export { invokeMock }
