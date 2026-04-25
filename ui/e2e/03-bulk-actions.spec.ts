import { test, expect } from "@playwright/test"
import { loadApp, box } from "./helpers"

test.describe("BulkActionBar", () => {
  test("is hidden when no books are selected", async ({ page }) => {
    await loadApp(page)
    await expect(page.getByText("selected")).not.toBeVisible()
  })

  test("appears after selecting one book", async ({ page }) => {
    await loadApp(page)
    await page.locator('input[type="checkbox"]').first().check()
    await expect(page.getByText("1 selected")).toBeVisible()
  })

  test("shows correct count for multiple selections", async ({ page }) => {
    await loadApp(page)
    const checkboxes = page.locator('input[type="checkbox"]')
    await checkboxes.nth(0).check()
    await checkboxes.nth(1).check()
    await checkboxes.nth(2).check()
    await expect(page.getByText("3 selected")).toBeVisible()
  })

  test("all action buttons are visible when bar is shown", async ({ page }) => {
    await loadApp(page)
    await page.locator('input[type="checkbox"]').first().check()
    await expect(page.getByRole("button", { name: "Edit metadata" })).toBeVisible()
    await expect(page.getByRole("button", { name: /convert|tweak/i })).toBeVisible()
    await expect(page.getByRole("button", { name: "Repair" })).toBeVisible()
    await expect(page.getByRole("button", { name: "Re-ingest" })).toBeVisible()
    await expect(page.getByRole("button", { name: "Export CSV" })).toBeVisible()
    await expect(page.getByRole("button", { name: "Delete" })).toBeVisible()
    await expect(page.getByRole("button", { name: "Cancel" })).toBeVisible()
  })

  test("Cancel button clears selection and hides bar", async ({ page }) => {
    await loadApp(page)
    await page.locator('input[type="checkbox"]').first().check()
    await expect(page.getByText("1 selected")).toBeVisible()
    await page.getByRole("button", { name: "Cancel" }).click()
    await expect(page.getByText("selected")).not.toBeVisible()
  })

  test("bar is horizontally centered on main content area (not full viewport)", async ({ page }) => {
    await page.setViewportSize({ width: 1280, height: 800 })
    await loadApp(page)
    await page.locator('input[type="checkbox"]').first().check()

    const sidebar = await box(page, "aside")
    const sidebarRight = sidebar.x + sidebar.width

    // Find the BulkActionBar container (the fixed pill at the bottom)
    const barBox = await box(page, '.fixed.bottom-4')
    const barCenter = barBox.x + barBox.width / 2

    // Main content spans from sidebarRight to viewport width
    const vw = 1280
    const mainCenter = sidebarRight + (vw - sidebarRight) / 2

    // Bar center must be within 20px of main content center
    expect(Math.abs(barCenter - mainCenter)).toBeLessThan(20)
  })

  test("bar is fully visible — not obscured by sidebar", async ({ page }) => {
    await page.setViewportSize({ width: 1280, height: 800 })
    await loadApp(page)
    await page.locator('input[type="checkbox"]').first().check()

    const sidebar = await box(page, "aside")
    const barBox = await box(page, ".fixed.bottom-4")

    // Left edge of bar must be to the right of the sidebar
    expect(barBox.x).toBeGreaterThan(sidebar.x + sidebar.width - 10)
  })

  test("EPUB single selection shows Tweak EPUB label", async ({ page }) => {
    await loadApp(page)
    // The Great Gatsby is EPUB — click its card then check its checkbox
    const epubCard = page.getByText("The Great Gatsby").locator("..").locator("..")
    await epubCard.locator('input[type="checkbox"]').check()
    // Click the epub book to set selectedBook
    await page.getByText("The Great Gatsby").click()
    await page.waitForTimeout(200)
    // Cancel the reader if it opened
    const closeBtn = page.getByRole("button", { name: /close/i }).first()
    if (await closeBtn.isVisible()) await closeBtn.click()
    // Re-check
    await epubCard.locator('input[type="checkbox"]').check()
    await expect(page.getByRole("button", { name: /convert|tweak/i })).toBeVisible()
  })
})
