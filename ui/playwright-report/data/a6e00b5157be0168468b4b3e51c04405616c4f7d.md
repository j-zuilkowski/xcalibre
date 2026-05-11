# Instructions

- Following Playwright test failed.
- Explain why, be concise, respect Playwright best practices.
- Provide a snippet of code with the fix, if possible.

# Test info

- Name: 03-bulk-actions.spec.ts >> BulkActionBar >> Cancel button clears selection and hides bar
- Location: e2e/03-bulk-actions.spec.ts:37:3

# Error details

```
Error: expect(locator).not.toBeVisible() failed

Locator:  getByText('selected')
Expected: not visible
Received: visible
Timeout:  8000ms

Call log:
  - Expect "not toBeVisible" with timeout 8000ms
  - waiting for getByText('selected')
    12 × locator resolved to <button disabled class="mt-2 text-xs text-blue-600 hover:underline text-left disabled:opacity-40 disabled:hover:no-underline">+ Add selected to collection</button>
       - unexpected value "visible"

```

# Page snapshot

```yaml
- generic [ref=e3]:
  - complementary [ref=e4]:
    - button "All Books" [ref=e5] [cursor=pointer]
    - generic [ref=e6]:
      - button "Sci-Fi" [ref=e7] [cursor=pointer]
      - button "Classics" [ref=e8] [cursor=pointer]
    - button "+ Add selected to collection" [disabled] [ref=e9]
    - generic [ref=e10]:
      - textbox "New collection…" [ref=e11]
      - button "+" [ref=e12] [cursor=pointer]
  - generic [ref=e13]:
    - banner [ref=e14]:
      - generic [ref=e15]:
        - heading "xCalibre" [level=1] [ref=e16]
        - paragraph [ref=e17]: Local-first ebook reader
      - generic [ref=e18]:
        - button "Import Calibre" [ref=e19] [cursor=pointer]
        - button "Repair library" [ref=e20] [cursor=pointer]
        - searchbox "Search title, author, text…" [ref=e21]
        - button "Settings" [ref=e22] [cursor=pointer]
    - generic [ref=e23]:
      - combobox [ref=e24]:
        - option "All formats" [selected]
        - option "EPUB"
        - option "PDF"
        - option "MOBI"
        - option "AZW3"
        - option "CBZ"
        - option "CBR"
        - option "TXT"
      - combobox [ref=e25]:
        - option "All statuses" [selected]
        - option "PENDING"
        - option "READY_TO_PUSH"
        - option "PUSHING"
        - option "RETRYING"
        - option "COMPLETED"
        - option "FAILED"
      - textbox "Author…" [ref=e26]
      - combobox [ref=e27]:
        - option "All series" [selected]
      - combobox [ref=e28]:
        - option "All tags" [selected]
        - option "fiction"
        - option "sci-fi"
        - option "classic"
      - button "Clear" [ref=e29] [cursor=pointer]
    - main [ref=e30]:
      - generic [ref=e31]:
        - generic [ref=e32] [cursor=pointer]:
          - checkbox [ref=e34]
          - generic [ref=e36]: epub
          - generic [ref=e37]:
            - paragraph [ref=e38]: The Great Gatsby
            - paragraph [ref=e39]: F. Scott Fitzgerald
        - generic [ref=e42] [cursor=pointer]:
          - checkbox [ref=e44]
          - generic [ref=e46]: pdf
          - generic [ref=e47]:
            - paragraph [ref=e48]: "1984"
            - paragraph [ref=e49]: George Orwell
        - generic [ref=e50] [cursor=pointer]:
          - checkbox [ref=e52]
          - generic [ref=e54]: mobi
          - generic [ref=e55]:
            - paragraph [ref=e56]: Dune
            - paragraph [ref=e57]: Frank Herbert
        - generic [ref=e60] [cursor=pointer]:
          - checkbox [ref=e62]
          - generic [ref=e64]: epub
          - generic [ref=e65]:
            - paragraph [ref=e66]: Foundation
            - paragraph [ref=e67]: Isaac Asimov
        - generic [ref=e68] [cursor=pointer]:
          - checkbox [ref=e70]
          - generic [ref=e72]: epub
          - generic [ref=e73]:
            - paragraph [ref=e74]: Neuromancer
            - paragraph [ref=e75]: William Gibson
```

# Test source

```ts
  1  | import { test, expect } from "@playwright/test"
  2  | import { loadApp, box } from "./helpers"
  3  | 
  4  | test.describe("BulkActionBar", () => {
  5  |   test("is hidden when no books are selected", async ({ page }) => {
  6  |     await loadApp(page)
  7  |     await expect(page.getByText("selected")).not.toBeVisible()
  8  |   })
  9  | 
  10 |   test("appears after selecting one book", async ({ page }) => {
  11 |     await loadApp(page)
  12 |     await page.locator('input[type="checkbox"]').first().check()
  13 |     await expect(page.getByText("1 selected")).toBeVisible()
  14 |   })
  15 | 
  16 |   test("shows correct count for multiple selections", async ({ page }) => {
  17 |     await loadApp(page)
  18 |     const checkboxes = page.locator('input[type="checkbox"]')
  19 |     await checkboxes.nth(0).check()
  20 |     await checkboxes.nth(1).check()
  21 |     await checkboxes.nth(2).check()
  22 |     await expect(page.getByText("3 selected")).toBeVisible()
  23 |   })
  24 | 
  25 |   test("all action buttons are visible when bar is shown", async ({ page }) => {
  26 |     await loadApp(page)
  27 |     await page.locator('input[type="checkbox"]').first().check()
  28 |     await expect(page.getByRole("button", { name: "Edit metadata" })).toBeVisible()
  29 |     await expect(page.getByRole("button", { name: /convert|tweak/i })).toBeVisible()
  30 |     await expect(page.getByRole("button", { name: "Repair" })).toBeVisible()
  31 |     await expect(page.getByRole("button", { name: "Re-ingest" })).toBeVisible()
  32 |     await expect(page.getByRole("button", { name: "Export CSV" })).toBeVisible()
  33 |     await expect(page.getByRole("button", { name: "Delete" })).toBeVisible()
  34 |     await expect(page.getByRole("button", { name: "Cancel" })).toBeVisible()
  35 |   })
  36 | 
  37 |   test("Cancel button clears selection and hides bar", async ({ page }) => {
  38 |     await loadApp(page)
  39 |     await page.locator('input[type="checkbox"]').first().check()
  40 |     await expect(page.getByText("1 selected")).toBeVisible()
  41 |     await page.getByRole("button", { name: "Cancel" }).click()
> 42 |     await expect(page.getByText("selected")).not.toBeVisible()
     |                                                  ^ Error: expect(locator).not.toBeVisible() failed
  43 |   })
  44 | 
  45 |   test("bar is horizontally centered on main content area (not full viewport)", async ({ page }) => {
  46 |     await page.setViewportSize({ width: 1280, height: 800 })
  47 |     await loadApp(page)
  48 |     await page.locator('input[type="checkbox"]').first().check()
  49 | 
  50 |     const sidebar = await box(page, "aside")
  51 |     const sidebarRight = sidebar.x + sidebar.width
  52 | 
  53 |     // Find the BulkActionBar container (the fixed pill at the bottom)
  54 |     const barBox = await box(page, '.fixed.bottom-4')
  55 |     const barCenter = barBox.x + barBox.width / 2
  56 | 
  57 |     // Main content spans from sidebarRight to viewport width
  58 |     const vw = 1280
  59 |     const mainCenter = sidebarRight + (vw - sidebarRight) / 2
  60 | 
  61 |     // Bar center must be within 20px of main content center
  62 |     expect(Math.abs(barCenter - mainCenter)).toBeLessThan(20)
  63 |   })
  64 | 
  65 |   test("bar is fully visible — not obscured by sidebar", async ({ page }) => {
  66 |     await page.setViewportSize({ width: 1280, height: 800 })
  67 |     await loadApp(page)
  68 |     await page.locator('input[type="checkbox"]').first().check()
  69 | 
  70 |     const sidebar = await box(page, "aside")
  71 |     const barBox = await box(page, ".fixed.bottom-4")
  72 | 
  73 |     // Left edge of bar must be to the right of the sidebar
  74 |     expect(barBox.x).toBeGreaterThan(sidebar.x + sidebar.width - 10)
  75 |   })
  76 | 
  77 |   test("EPUB single selection shows Tweak EPUB label", async ({ page }) => {
  78 |     await loadApp(page)
  79 |     // The Great Gatsby is EPUB — click its card then check its checkbox
  80 |     const epubCard = page.getByText("The Great Gatsby").locator("..").locator("..")
  81 |     await epubCard.locator('input[type="checkbox"]').check()
  82 |     // Click the epub book to set selectedBook
  83 |     await page.getByText("The Great Gatsby").click()
  84 |     await page.waitForTimeout(200)
  85 |     // Cancel the reader if it opened
  86 |     const closeBtn = page.getByRole("button", { name: /close/i }).first()
  87 |     if (await closeBtn.isVisible()) await closeBtn.click()
  88 |     // Re-check
  89 |     await epubCard.locator('input[type="checkbox"]').check()
  90 |     await expect(page.getByRole("button", { name: /convert|tweak/i })).toBeVisible()
  91 |   })
  92 | })
  93 | 
```