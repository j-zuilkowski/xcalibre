import { test, expect } from "@playwright/test"
import { loadApp, box } from "./helpers"

test.describe("App layout", () => {
  test("sidebar is 208px wide (w-52) and full height", async ({ page }) => {
    await loadApp(page)
    const sidebar = await box(page, "aside")
    expect(sidebar.width).toBeCloseTo(208, -1)
    // After h-screen fix, sidebar height should fill the viewport
    const vh = await page.evaluate(() => window.innerHeight)
    expect(sidebar.height).toBeGreaterThanOrEqual(vh - 2)
  })

  test("header is 56px tall (h-14) and spans main content width", async ({ page }) => {
    await loadApp(page)
    const header = await box(page, "header")
    expect(header.height).toBeCloseTo(56, -1)
    // Header should start at the sidebar right edge
    const sidebar = await box(page, "aside")
    expect(header.x).toBeCloseTo(sidebar.x + sidebar.width, -1)
  })

  test("no horizontal overflow at 1280×800", async ({ page }) => {
    await page.setViewportSize({ width: 1280, height: 800 })
    await loadApp(page)
    const scrollWidth = await page.evaluate(() => document.documentElement.scrollWidth)
    expect(scrollWidth).toBeLessThanOrEqual(1280)
  })

  test("no horizontal overflow at 800×600 (Tauri default)", async ({ page }) => {
    await page.setViewportSize({ width: 800, height: 600 })
    await loadApp(page)
    const scrollWidth = await page.evaluate(() => document.documentElement.scrollWidth)
    expect(scrollWidth).toBeLessThanOrEqual(800)
  })

  test("sidebar border-r is visible and not clipped", async ({ page }) => {
    await loadApp(page)
    const sidebar = await box(page, "aside")
    // Sidebar right edge must be inside viewport
    const vw = await page.evaluate(() => window.innerWidth)
    expect(sidebar.x + sidebar.width).toBeLessThan(vw)
  })

  test("app logo text is visible in header", async ({ page }) => {
    await loadApp(page)
    await expect(page.getByText("xCalibre")).toBeVisible()
  })
})
