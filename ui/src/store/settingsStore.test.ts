import { beforeEach, describe, expect, test, vi } from "vitest"
import { useSettingsStore } from "./settingsStore"

describe("useSettingsStore", () => {
  beforeEach(() => {
    localStorage.clear()
    useSettingsStore.setState({
      fontSize: 18,
      theme: "light",
      fontFamily: "serif",
    })
  })

  test("default values", () => {
    expect(useSettingsStore.getState().fontSize).toBe(18)
    expect(useSettingsStore.getState().theme).toBe("light")
    expect(useSettingsStore.getState().fontFamily).toBe("serif")
  })

  test("setFontSize persists", () => {
    useSettingsStore.getState().setFontSize(20)

    expect(useSettingsStore.getState().fontSize).toBe(20)
    expect(localStorage.getItem("xcalibre-settings")).toContain('"fontSize":20')
  })

  test("setTheme persists", () => {
    useSettingsStore.getState().setTheme("dark")

    expect(useSettingsStore.getState().theme).toBe("dark")
    expect(localStorage.getItem("xcalibre-settings")).toContain('"theme":"dark"')
  })

  test("hydrates from localStorage", async () => {
    localStorage.clear()
    localStorage.setItem(
      "xcalibre-settings",
      JSON.stringify({
        state: {
          fontSize: 16,
          theme: "sepia",
          fontFamily: "mono",
        },
        version: 0,
      }),
    )

    vi.resetModules()
    const { useSettingsStore: hydratedStore } = await import("./settingsStore")

    expect(hydratedStore.getState().fontSize).toBe(16)
    expect(hydratedStore.getState().theme).toBe("sepia")
    expect(hydratedStore.getState().fontFamily).toBe("mono")
  })
})
