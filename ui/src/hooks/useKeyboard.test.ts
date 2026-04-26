import { beforeEach, describe, expect, test, vi } from "vitest"
import { renderHook } from "@testing-library/react"
import { useKeyboard } from "./useKeyboard"

describe("useKeyboard", () => {
  beforeEach(() => {
    vi.restoreAllMocks()
  })

  test("meta+, fires handler", () => {
    const handler = vi.fn()
    renderHook(() =>
      useKeyboard({
        "meta+,": handler,
      }),
    )

    window.dispatchEvent(new KeyboardEvent("keydown", { key: ",", metaKey: true }))

    expect(handler).toHaveBeenCalledTimes(1)
  })

  test("ctrl+, fires handler", () => {
    const handler = vi.fn()
    renderHook(() =>
      useKeyboard({
        "ctrl+,": handler,
      }),
    )

    window.dispatchEvent(new KeyboardEvent("keydown", { key: ",", ctrlKey: true }))

    expect(handler).toHaveBeenCalledTimes(1)
  })

  test("meta+f fires handler", () => {
    const handler = vi.fn()
    renderHook(() =>
      useKeyboard({
        "meta+f": handler,
      }),
    )

    window.dispatchEvent(new KeyboardEvent("keydown", { key: "f", metaKey: true }))

    expect(handler).toHaveBeenCalledTimes(1)
  })

  test("escape fires handler", () => {
    const handler = vi.fn()
    renderHook(() =>
      useKeyboard({
        escape: handler,
      }),
    )

    window.dispatchEvent(new KeyboardEvent("keydown", { key: "Escape" }))

    expect(handler).toHaveBeenCalledTimes(1)
  })

  test("handler not called for unregistered key", () => {
    const handler = vi.fn()
    renderHook(() =>
      useKeyboard({
        "meta+,": handler,
      }),
    )

    window.dispatchEvent(new KeyboardEvent("keydown", { key: "z" }))

    expect(handler).not.toHaveBeenCalled()
  })

  test("preventDefault called on matching key", () => {
    const handler = vi.fn()
    renderHook(() =>
      useKeyboard({
        "meta+,": handler,
      }),
    )

    const event = new KeyboardEvent("keydown", { key: ",", metaKey: true, cancelable: true })
    const preventDefault = vi.spyOn(event, "preventDefault")

    window.dispatchEvent(event)

    expect(preventDefault).toHaveBeenCalledTimes(1)
  })

  test("cleanup on unmount", () => {
    const handler = vi.fn()
    const { unmount } = renderHook(() =>
      useKeyboard({
        "meta+,": handler,
      }),
    )

    unmount()
    window.dispatchEvent(new KeyboardEvent("keydown", { key: ",", metaKey: true }))

    expect(handler).not.toHaveBeenCalled()
  })
})
