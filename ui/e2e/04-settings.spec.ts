import { test, expect } from "@playwright/test"
import { loadApp } from "./helpers"

test.describe("Settings modal", () => {
  test("opens via Settings button", async ({ page }) => {
    await loadApp(page)
    await page.getByRole("button", { name: "Settings" }).click()
    await expect(page.getByRole("heading", { name: "Settings" })).toBeVisible()
  })

  test("opens via Cmd+, keyboard shortcut", async ({ page }) => {
    await loadApp(page)
    await page.keyboard.press("Meta+,")
    await expect(page.getByRole("heading", { name: "Settings" })).toBeVisible()
  })

  test("General section always shows Server URL field (not gated on token)", async ({ page }) => {
    // Load with no token stored
    await loadApp(page, { hasToken: false })
    await page.getByRole("button", { name: "Settings" }).click()
    // After Fix 2, Server URL must always be present
    await expect(page.getByText("Server URL")).toBeVisible()
    await expect(page.locator('input[placeholder*="xcalibre"]')).toBeVisible()
  })

  test("Server URL field shows existing value when token is stored", async ({ page }) => {
    await loadApp(page, { hasToken: true, xsUrl: "https://my.server.local" })
    await page.getByRole("button", { name: "Settings" }).click()
    const urlInput = page.locator('input[placeholder*="xcalibre"]')
    await expect(urlInput).toHaveValue("https://my.server.local")
  })

  test("Reader section has font size slider", async ({ page }) => {
    await loadApp(page)
    await page.getByRole("button", { name: "Settings" }).click()
    await expect(page.getByText("Font size")).toBeVisible()
    await expect(page.locator('input[type="range"]')).toBeVisible()
  })

  test("Reader section has theme selector", async ({ page }) => {
    await loadApp(page)
    await page.getByRole("button", { name: "Settings" }).click()
    const themeSelect = page.locator("select").first()
    await expect(themeSelect).toBeVisible()
    await expect(themeSelect.locator('option[value="light"]')).toHaveCount(1)
    await expect(themeSelect.locator('option[value="dark"]')).toHaveCount(1)
    await expect(themeSelect.locator('option[value="sepia"]')).toHaveCount(1)
  })

  test("Reader section has font family selector", async ({ page }) => {
    await loadApp(page)
    await page.getByRole("button", { name: "Settings" }).click()
    const selects = page.locator("select")
    await expect(selects).toHaveCount(2)
    const fontSelect = selects.nth(1)
    await expect(fontSelect.locator('option[value="serif"]')).toHaveCount(1)
  })

  test("About section shows version", async ({ page }) => {
    await loadApp(page)
    await page.getByRole("button", { name: "Settings" }).click()
    await expect(page.getByText(/xCalibre v/)).toBeVisible()
  })

  test("Cancel button closes modal", async ({ page }) => {
    await loadApp(page)
    await page.getByRole("button", { name: "Settings" }).click()
    await page.getByRole("button", { name: "Cancel" }).click()
    await expect(page.getByRole("heading", { name: "Settings" })).not.toBeVisible()
  })

  test("× button closes modal", async ({ page }) => {
    await loadApp(page)
    await page.getByRole("button", { name: "Settings" }).click()
    await page.getByRole("button", { name: "×" }).click()
    await expect(page.getByRole("heading", { name: "Settings" })).not.toBeVisible()
  })

  test("Escape key closes modal", async ({ page }) => {
    await loadApp(page)
    await page.getByRole("button", { name: "Settings" }).click()
    await page.keyboard.press("Escape")
    await expect(page.getByRole("heading", { name: "Settings" })).not.toBeVisible()
  })

  test("Save button closes modal after click", async ({ page }) => {
    await loadApp(page)
    await page.getByRole("button", { name: "Settings" }).click()
    await page.getByRole("button", { name: "Save" }).click()
    await expect(page.getByRole("heading", { name: "Settings" })).not.toBeVisible()
  })
})
