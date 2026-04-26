import { beforeEach, describe, expect, test, vi } from "vitest"
import { fireEvent, render, screen } from "@testing-library/react"
import userEvent from "@testing-library/user-event"
import { SettingsModal } from "./SettingsModal"
import { useSettingsStore } from "../store/settingsStore"
import { invokeMock, mockInvoke } from "../test/setup"

describe("SettingsModal", () => {
  beforeEach(() => {
    useSettingsStore.setState({
      fontSize: 18,
      theme: "light",
      fontFamily: "serif",
    })
  })

  test("renders nothing when open_=false", () => {
    render(<SettingsModal open_={false} onClose={vi.fn()} />)

    expect(screen.queryByRole("heading", { name: "Settings" })).not.toBeInTheDocument()
  })

  test("renders when open_=true", () => {
    render(<SettingsModal open_={true} onClose={vi.fn()} />)

    expect(screen.getByRole("heading", { name: "Settings" })).toBeInTheDocument()
  })

  test("loads xs_url on open", async () => {
    mockInvoke("get_xs_url", "https://x.io")
    mockInvoke("has_token", false)

    render(<SettingsModal open_={true} onClose={vi.fn()} />)

    expect(await screen.findByDisplayValue("https://x.io")).toBeInTheDocument()
    expect(invokeMock.mock.calls.some(([cmd]) => cmd === "has_token")).toBe(true)
  })

  test("Server URL input always visible (no token)", async () => {
    mockInvoke("get_xs_url", "")
    mockInvoke("has_token", false)

    render(<SettingsModal open_={true} onClose={vi.fn()} />)

    expect(await screen.findByLabelText("Server URL")).toBeInTheDocument()
  })

  test("Server URL input visible with token", async () => {
    mockInvoke("get_xs_url", "")
    mockInvoke("has_token", true)

    render(<SettingsModal open_={true} onClose={vi.fn()} />)

    expect(await screen.findByLabelText("Server URL")).toBeInTheDocument()
  })

  test("Save calls save_config with url", async () => {
    const user = userEvent.setup()
    mockInvoke("get_xs_url", "")
    mockInvoke("has_token", false)
    mockInvoke("save_config", null)

    render(<SettingsModal open_={true} onClose={vi.fn()} />)

    const input = await screen.findByLabelText("Server URL")
    await user.type(input, "https://api.xcalibre.app")
    await user.click(screen.getByRole("button", { name: "Save" }))

    expect(
      invokeMock.mock.calls.some(
        ([cmd, payload]) =>
          cmd === "save_config" &&
          Boolean(payload) &&
          (payload as { autolibUrl?: string | null }).autolibUrl ===
            "https://api.xcalibre.app",
      ),
    ).toBe(true)
  })

  test("Save with empty url passes null", async () => {
    const user = userEvent.setup()
    mockInvoke("get_xs_url", "https://api.xcalibre.app")
    mockInvoke("has_token", false)
    mockInvoke("save_config", null)

    render(<SettingsModal open_={true} onClose={vi.fn()} />)

    const input = await screen.findByLabelText("Server URL")
    await user.clear(input)
    await user.click(screen.getByRole("button", { name: "Save" }))

    expect(
      invokeMock.mock.calls.some(
        ([cmd, payload]) =>
          cmd === "save_config" &&
          Boolean(payload) &&
          (payload as { autolibUrl?: string | null }).autolibUrl === null,
      ),
    ).toBe(true)
  })

  test("Save calls onClose", async () => {
    const user = userEvent.setup()
    const onClose = vi.fn()
    mockInvoke("get_xs_url", "")
    mockInvoke("has_token", false)
    mockInvoke("save_config", null)

    render(<SettingsModal open_={true} onClose={onClose} />)

    await user.click(await screen.findByRole("button", { name: "Save" }))

    expect(onClose).toHaveBeenCalledTimes(1)
  })

  test("Cancel calls onClose without saving", async () => {
    const user = userEvent.setup()
    const onClose = vi.fn()
    mockInvoke("get_xs_url", "")
    mockInvoke("has_token", false)

    render(<SettingsModal open_={true} onClose={onClose} />)

    await user.click(await screen.findByRole("button", { name: "Cancel" }))

    expect(onClose).toHaveBeenCalledTimes(1)
    expect(invokeMock.mock.calls.some(([cmd]) => cmd === "save_config")).toBe(false)
  })

  test("× button calls onClose", async () => {
    const user = userEvent.setup()
    const onClose = vi.fn()
    mockInvoke("get_xs_url", "")
    mockInvoke("has_token", false)

    render(<SettingsModal open_={true} onClose={onClose} />)

    await user.click(screen.getByRole("button", { name: "×" }))

    expect(onClose).toHaveBeenCalledTimes(1)
  })

  test("font size slider updates store", async () => {
    mockInvoke("get_xs_url", "")
    mockInvoke("has_token", false)

    render(<SettingsModal open_={true} onClose={vi.fn()} />)

    const slider = await screen.findByRole("slider")
    fireEvent.change(slider, { target: { value: "20" } })

    expect(useSettingsStore.getState().fontSize).toBe(20)
  })

  test("theme select updates store", async () => {
    const user = userEvent.setup()
    mockInvoke("get_xs_url", "")
    mockInvoke("has_token", false)

    render(<SettingsModal open_={true} onClose={vi.fn()} />)

    await user.selectOptions(await screen.findByRole("combobox", { name: "Theme" }), "dark")

    expect(useSettingsStore.getState().theme).toBe("dark")
  })

  test("font family select updates store", async () => {
    const user = userEvent.setup()
    mockInvoke("get_xs_url", "")
    mockInvoke("has_token", false)

    render(<SettingsModal open_={true} onClose={vi.fn()} />)

    await user.selectOptions(await screen.findByRole("combobox", { name: "Font family" }), "mono")

    expect(useSettingsStore.getState().fontFamily).toBe("mono")
  })
})
