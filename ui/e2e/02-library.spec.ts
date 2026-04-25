import { test, expect } from "@playwright/test"
import { loadApp } from "./helpers"
import { MOCK_BOOKS } from "./tauri-mock"

test.describe("Library grid", () => {
  test("shows empty-state message when no books", async ({ page }) => {
    await loadApp(page, { books: [] })
    await expect(page.getByText("No books yet")).toBeVisible()
  })

  test("renders one card per book", async ({ page }) => {
    await loadApp(page)
    for (const book of MOCK_BOOKS) {
      await expect(page.getByText(book.title)).toBeVisible()
    }
  })

  test("book card shows format badge when no cover", async ({ page }) => {
    await loadApp(page)
    // Wait for cards to be rendered before checking format badge
    await expect(page.getByText("The Great Gatsby")).toBeVisible()
    // Cover area renders book.format as lowercase text when cover_path is null
    // Scope to the cover div (h-48) to avoid matching the FilterBar format dropdown
    const coverBadge = page.locator(".h-48 span").first()
    await expect(coverBadge).toBeVisible()
    const text = await coverBadge.textContent()
    expect(["epub", "pdf", "mobi", "EPUB", "PDF", "MOBI"]).toContain(text?.trim())
  })

  test("book card shows author name", async ({ page }) => {
    await loadApp(page)
    await expect(page.getByText("F. Scott Fitzgerald")).toBeVisible()
  })

  test("book card shows progress bar for books with progress > 0", async ({ page }) => {
    await loadApp(page)
    // Dune has progress_percent: 67, which renders a blue progress bar
    await expect(page.locator(".bg-blue-500.rounded-full").first()).toBeVisible()
  })

  test("clicking a book card opens a full-screen overlay", async ({ page }) => {
    await loadApp(page)
    await expect(page.getByText("The Great Gatsby")).toBeVisible()
    // Click the card title — setSelectedBook is called with the book object
    await page.getByText("The Great Gatsby").click()
    await page.waitForTimeout(400)
    // ReaderView and format modal both render a .fixed.inset-0 overlay
    const overlays = page.locator(".fixed.inset-0")
    await expect(overlays.first()).toBeVisible()
  })

  test("checkbox selects a book", async ({ page }) => {
    await loadApp(page)
    await expect(page.getByText("The Great Gatsby")).toBeVisible()
    const checkbox = page.locator('input[type="checkbox"]').first()
    await checkbox.check()
    await expect(checkbox).toBeChecked()
  })

  test("selecting a book shows the BulkActionBar", async ({ page }) => {
    await loadApp(page)
    await expect(page.getByText("The Great Gatsby")).toBeVisible()
    await page.locator('input[type="checkbox"]').first().check()
    await expect(page.getByText("1 selected")).toBeVisible()
  })

  test("search filters visible books", async ({ page }) => {
    await loadApp(page)
    await expect(page.getByText("The Great Gatsby")).toBeVisible()
    const searchInput = page.locator('input[type="search"]')
    await searchInput.fill("gatsby")
    await page.waitForTimeout(400)
    await expect(page.getByText("The Great Gatsby")).toBeVisible()
  })
})
