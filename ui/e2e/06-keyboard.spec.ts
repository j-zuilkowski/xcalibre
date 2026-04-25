import { test, expect } from "@playwright/test"
import { loadApp } from "./helpers"

test.describe("Keyboard shortcuts", () => {
  test("Cmd+, opens Settings modal", async ({ page }) => {
    await loadApp(page)
    await page.keyboard.press("Meta+,")
    await expect(page.getByRole("heading", { name: "Settings" })).toBeVisible()
  })

  test("Ctrl+, opens Settings modal (Linux/Windows style)", async ({ page }) => {
    await loadApp(page)
    await page.keyboard.press("Control+,")
    await expect(page.getByRole("heading", { name: "Settings" })).toBeVisible()
  })

  test("Escape closes Settings modal", async ({ page }) => {
    await loadApp(page)
    await page.keyboard.press("Meta+,")
    await expect(page.getByRole("heading", { name: "Settings" })).toBeVisible()
    await page.keyboard.press("Escape")
    await expect(page.getByRole("heading", { name: "Settings" })).not.toBeVisible()
  })

  test("Cmd+F focuses the search input", async ({ page }) => {
    await loadApp(page)
    await page.keyboard.press("Meta+f")
    const focused = await page.evaluate(() => {
      const el = document.activeElement
      return el ? (el as HTMLInputElement).type || el.tagName.toLowerCase() : ""
    })
    // Focused element should be a text-type input
    expect(["text", "search", "input"]).toContain(focused)
  })

  test("Ctrl+F focuses the search input", async ({ page }) => {
    await loadApp(page)
    await page.keyboard.press("Control+f")
    const focused = await page.evaluate(() => {
      const el = document.activeElement
      return el ? (el as HTMLInputElement).type || el.tagName.toLowerCase() : ""
    })
    expect(["text", "search", "input"]).toContain(focused)
  })
})
