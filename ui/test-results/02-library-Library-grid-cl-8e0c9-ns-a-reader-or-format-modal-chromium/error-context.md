# Instructions

- Following Playwright test failed.
- Explain why, be concise, respect Playwright best practices.
- Provide a snippet of code with the fix, if possible.

# Test info

- Name: 02-library.spec.ts >> Library grid >> clicking a book card opens a reader or format modal
- Location: e2e/02-library.spec.ts:37:3

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
  13 |     // Each book card contains the title
  14 |     for (const book of MOCK_BOOKS) {
  15 |       await expect(page.getByText(book.title)).toBeVisible()
  16 |     }
  17 |   })
  18 | 
  19 |   test("book card shows format badge when no cover", async ({ page }) => {
  20 |     await loadApp(page)
  21 |     // The first EPUB book (no cover) shows the format label inside the cover area
  22 |     await expect(page.getByText("epub").first()).toBeVisible()
  23 |   })
  24 | 
  25 |   test("book card shows author name", async ({ page }) => {
  26 |     await loadApp(page)
  27 |     await expect(page.getByText("F. Scott Fitzgerald")).toBeVisible()
  28 |   })
  29 | 
  30 |   test("book card shows progress bar for books with progress > 0", async ({ page }) => {
  31 |     await loadApp(page)
  32 |     // Dune has progress_percent: 67, which renders a blue progress bar
  33 |     const progressBars = page.locator(".bg-blue-500.rounded-full")
  34 |     await expect(progressBars.first()).toBeVisible()
  35 |   })
  36 | 
  37 |   test("clicking a book card opens a reader or format modal", async ({ page }) => {
  38 |     await loadApp(page)
  39 |     // Click the first EPUB book (The Great Gatsby)
  40 |     await page.getByText("The Great Gatsby").click()
  41 |     // Should open ReaderView (EPUB) — wait for any overlay/modal
  42 |     await page.waitForTimeout(500)
  43 |     // Reader is a fixed inset-0 overlay
  44 |     const reader = page.locator(".fixed.inset-0").first()
> 45 |     await expect(reader).toBeVisible()
     |                          ^ Error: expect(locator).toBeVisible() failed
  46 |   })
  47 | 
  48 |   test("checkbox selects a book", async ({ page }) => {
  49 |     await loadApp(page)
  50 |     const checkbox = page.locator('input[type="checkbox"]').first()
  51 |     await checkbox.check()
  52 |     await expect(checkbox).toBeChecked()
  53 |   })
  54 | 
  55 |   test("selecting a book shows the BulkActionBar", async ({ page }) => {
  56 |     await loadApp(page)
  57 |     const checkbox = page.locator('input[type="checkbox"]').first()
  58 |     await checkbox.check()
  59 |     await expect(page.getByText("1 selected")).toBeVisible()
  60 |   })
  61 | 
  62 |   test("search filters visible books", async ({ page }) => {
  63 |     await loadApp(page)
  64 |     const searchInput = page.locator('input[type="search"], input[placeholder*="Search"], input[placeholder*="search"]').first()
  65 |     await searchInput.fill("gatsby")
  66 |     await page.waitForTimeout(400)
  67 |     await expect(page.getByText("The Great Gatsby")).toBeVisible()
  68 |   })
  69 | })
  70 | 
```