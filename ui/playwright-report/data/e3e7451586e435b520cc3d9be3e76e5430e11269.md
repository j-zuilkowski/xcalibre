# Instructions

- Following Playwright test failed.
- Explain why, be concise, respect Playwright best practices.
- Provide a snippet of code with the fix, if possible.

# Test info

- Name: 02-library.spec.ts >> Library grid >> clicking a book card opens a full-screen overlay
- Location: e2e/02-library.spec.ts:41:3

# Error details

```
Error: expect(locator).toBeVisible() failed

Locator: locator('.fixed.inset-0').first()
Expected: visible
Timeout: 8000ms
Error: element(s) not found

Call log:
  - Expect "toBeVisible" with timeout 8000ms
  - waiting for locator('.fixed.inset-0').first()

```

# Page snapshot

```yaml
- generic [ref=e4]:
  - heading "Something went wrong" [level=1] [ref=e5]
  - paragraph [ref=e6]: Cannot read properties of null (reading '0')
  - button "Try again" [ref=e7] [cursor=pointer]
```

# Test source

```ts
  1  | import { test, expect } from "@playwright/test"
  2  | import { loadApp } from "./helpers"
  3  | import { MOCK_BOOKS } from "./tauri-mock"
  4  | 
  5  | test.describe("Library grid", () => {
  6  |   test("shows empty-state message when no books", async ({ page }) => {
  7  |     await loadApp(page, { books: [] })
  8  |     await expect(page.getByText("No books yet")).toBeVisible()
  9  |   })
  10 | 
  11 |   test("renders one card per book", async ({ page }) => {
  12 |     await loadApp(page)
  13 |     for (const book of MOCK_BOOKS) {
  14 |       await expect(page.getByText(book.title)).toBeVisible()
  15 |     }
  16 |   })
  17 | 
  18 |   test("book card shows format badge when no cover", async ({ page }) => {
  19 |     await loadApp(page)
  20 |     // Wait for cards to be rendered before checking format badge
  21 |     await expect(page.getByText("The Great Gatsby")).toBeVisible()
  22 |     // Cover area renders book.format as lowercase text when cover_path is null
  23 |     // Scope to the cover div (h-48) to avoid matching the FilterBar format dropdown
  24 |     const coverBadge = page.locator(".h-48 span").first()
  25 |     await expect(coverBadge).toBeVisible()
  26 |     const text = await coverBadge.textContent()
  27 |     expect(["epub", "pdf", "mobi", "EPUB", "PDF", "MOBI"]).toContain(text?.trim())
  28 |   })
  29 | 
  30 |   test("book card shows author name", async ({ page }) => {
  31 |     await loadApp(page)
  32 |     await expect(page.getByText("F. Scott Fitzgerald")).toBeVisible()
  33 |   })
  34 | 
  35 |   test("book card shows progress bar for books with progress > 0", async ({ page }) => {
  36 |     await loadApp(page)
  37 |     // Dune has progress_percent: 67, which renders a blue progress bar
  38 |     await expect(page.locator(".bg-blue-500.rounded-full").first()).toBeVisible()
  39 |   })
  40 | 
  41 |   test("clicking a book card opens a full-screen overlay", async ({ page }) => {
  42 |     await loadApp(page)
  43 |     await expect(page.getByText("The Great Gatsby")).toBeVisible()
  44 |     // Click the card title — setSelectedBook is called with the book object
  45 |     await page.getByText("The Great Gatsby").click()
  46 |     await page.waitForTimeout(400)
  47 |     // ReaderView and format modal both render a .fixed.inset-0 overlay
  48 |     const overlays = page.locator(".fixed.inset-0")
> 49 |     await expect(overlays.first()).toBeVisible()
     |                                    ^ Error: expect(locator).toBeVisible() failed
  50 |   })
  51 | 
  52 |   test("checkbox selects a book", async ({ page }) => {
  53 |     await loadApp(page)
  54 |     await expect(page.getByText("The Great Gatsby")).toBeVisible()
  55 |     const checkbox = page.locator('input[type="checkbox"]').first()
  56 |     await checkbox.check()
  57 |     await expect(checkbox).toBeChecked()
  58 |   })
  59 | 
  60 |   test("selecting a book shows the BulkActionBar", async ({ page }) => {
  61 |     await loadApp(page)
  62 |     await expect(page.getByText("The Great Gatsby")).toBeVisible()
  63 |     await page.locator('input[type="checkbox"]').first().check()
  64 |     await expect(page.getByText("1 selected")).toBeVisible()
  65 |   })
  66 | 
  67 |   test("search filters visible books", async ({ page }) => {
  68 |     await loadApp(page)
  69 |     await expect(page.getByText("The Great Gatsby")).toBeVisible()
  70 |     const searchInput = page.locator('input[type="search"]')
  71 |     await searchInput.fill("gatsby")
  72 |     await page.waitForTimeout(400)
  73 |     await expect(page.getByText("The Great Gatsby")).toBeVisible()
  74 |   })
  75 | })
  76 | 
```