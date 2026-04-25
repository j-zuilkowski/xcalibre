# Instructions

- Following Playwright test failed.
- Explain why, be concise, respect Playwright best practices.
- Provide a snippet of code with the fix, if possible.

# Test info

- Name: 02-library.spec.ts >> Library grid >> book card shows format badge when no cover
- Location: e2e/02-library.spec.ts:19:3

# Error details

```
Error: expect(locator).toBeVisible() failed

Locator:  getByText('epub').first()
Expected: visible
Received: hidden
Timeout:  8000ms

Call log:
  - Expect "toBeVisible" with timeout 8000ms
  - waiting for getByText('epub').first()
    12 × locator resolved to <option value="EPUB">EPUB</option>
       - unexpected value "hidden"

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
> 22 |     await expect(page.getByText("epub").first()).toBeVisible()
     |                                                  ^ Error: expect(locator).toBeVisible() failed
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
  45 |     await expect(reader).toBeVisible()
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