import { test, expect } from "@playwright/test"
import { loadApp } from "./helpers"

test.describe("Dark mode", () => {
  test("html element has no 'dark' class by default (light theme)", async ({ page }) => {
    await loadApp(page)
    const hasDark = await page.evaluate(() =>
      document.documentElement.classList.contains("dark"),
    )
    expect(hasDark).toBe(false)
  })

  test("switching theme to dark adds 'dark' class to <html>", async ({ page }) => {
    await loadApp(page)
    // Open settings and switch theme to Dark
    await page.getByRole("button", { name: "Settings" }).click()
    const themeSelect = page.locator("select").first()
    await themeSelect.selectOption("dark")
    await page.getByRole("button", { name: "Save" }).click()

    // After Fix 5, the useEffect in App.tsx syncs theme → document class
    await page.waitForFunction(() =>
      document.documentElement.classList.contains("dark"),
    )
    const hasDark = await page.evaluate(() =>
      document.documentElement.classList.contains("dark"),
    )
    expect(hasDark).toBe(true)
  })

  test("switching back to light removes 'dark' class", async ({ page }) => {
    await loadApp(page)

    // Enable dark
    await page.getByRole("button", { name: "Settings" }).click()
    await page.locator("select").first().selectOption("dark")
    await page.getByRole("button", { name: "Save" }).click()
    await page.waitForFunction(() => document.documentElement.classList.contains("dark"))

    // Back to light
    await page.getByRole("button", { name: "Settings" }).click()
    await page.locator("select").first().selectOption("light")
    await page.getByRole("button", { name: "Save" }).click()
    await page.waitForFunction(() => !document.documentElement.classList.contains("dark"))

    const hasDark = await page.evaluate(() =>
      document.documentElement.classList.contains("dark"),
    )
    expect(hasDark).toBe(false)
  })

  test("app root has correct dark background in dark mode", async ({ page }) => {
    await loadApp(page)
    // Force dark class on html to test Tailwind dark: styles
    await page.evaluate(() => document.documentElement.classList.add("dark"))

    const bgColor = await page.evaluate(() => {
      const root = document.querySelector(".h-screen, .min-h-screen") as HTMLElement
      return root ? window.getComputedStyle(root).backgroundColor : ""
    })
    // dark:bg-gray-900 = rgb(17, 24, 39)
    expect(bgColor).toBe("rgb(17, 24, 39)")
  })

  test("sidebar text is visible in dark mode", async ({ page }) => {
    await loadApp(page)
    await page.evaluate(() => document.documentElement.classList.add("dark"))
    await expect(page.getByText("All Books")).toBeVisible()
    // Check the element is readable (not same color as dark background)
    const color = await page.getByText("All Books").evaluate(
      (el) => window.getComputedStyle(el).color,
    )
    // Should not be dark (not rgb(17, 24, 39) or similar dark)
    expect(color).not.toBe("rgb(17, 24, 39)")
  })

  test("header buttons have visible text in dark mode", async ({ page }) => {
    await loadApp(page)
    await page.evaluate(() => document.documentElement.classList.add("dark"))
    // After Fix 4, border buttons get dark:text-gray-300
    const repairBtn = page.getByRole("button", { name: "Repair library" })
    await expect(repairBtn).toBeVisible()
    const color = await repairBtn.evaluate((el) => window.getComputedStyle(el).color)
    // dark:text-gray-300 = rgb(209, 213, 219), not black or invisible
    expect(color).not.toBe("rgb(0, 0, 0)")
  })
})
