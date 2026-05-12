import { render, screen, fireEvent, waitFor } from "@testing-library/react"
import { describe, it, expect, vi, beforeEach } from "vitest"
import { PluginManagerModal } from "./PluginManagerModal"
import { mockInvoke } from "../test/setup"

const mockPlugins = [
  { id: "p1", name: "open-library", version: "1.0.0", plugin_type: "metadata_source", enabled: true },
]

beforeEach(() => {
  mockInvoke("list_plugins_cmd", mockPlugins)
  mockInvoke("set_plugin_enabled_cmd", undefined)
  mockInvoke("uninstall_plugin_cmd", undefined)
})

describe("PluginManagerModal", () => {
  it("renders installed plugins", async () => {
    render(<PluginManagerModal onClose={vi.fn()} />)
    await waitFor(() => screen.getByTestId("plugin-row-p1"))
    expect(screen.getByTestId("plugin-row-p1")).toHaveTextContent("open-library")
  })

  it("shows empty state when no plugins", async () => {
    mockInvoke("list_plugins_cmd", [])
    render(<PluginManagerModal onClose={vi.fn()} />)
    await waitFor(() => screen.getByText(/No plugins installed/))
  })

  it("calls onClose when × is clicked", async () => {
    const onClose = vi.fn()
    render(<PluginManagerModal onClose={onClose} />)
    fireEvent.click(screen.getByTestId("plugin-manager-close"))
    expect(onClose).toHaveBeenCalled()
  })
})
