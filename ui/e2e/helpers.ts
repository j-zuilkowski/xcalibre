import { type Page } from "@playwright/test"
import { buildMockScript, MOCK_BOOKS } from "./tauri-mock"

export type MockOptions = {
  books?: typeof MOCK_BOOKS
  hasToken?: boolean
  xsUrl?: string | null
}

/** Navigates to the app root with Tauri internals mocked. */
export async function loadApp(page: Page, opts: MockOptions = {}) {
  const { books = MOCK_BOOKS, hasToken = false, xsUrl = null } = opts
  await page.addInitScript({ content: buildMockScript(books, { hasToken, xsUrl }) })
  await page.goto("/")
  // Wait for the library grid or empty state to appear — indicates invoke("list_books") resolved
  await Promise.race([
    page.waitForSelector('input[type="checkbox"]', { timeout: 8_000 }),
    page.waitForSelector('text=No books yet', { timeout: 8_000 }),
  ]).catch(() => {})
  await page.waitForTimeout(100)
}

/** Locates the Settings modal overlay (scoped so selects inside won't match FilterBar). */
export function settingsModal(page: Page) {
  return page.locator(".fixed.inset-0").filter({ hasText: "Settings" })
}

/** Returns the bounding box of an element, asserting it is visible. */
export async function box(page: Page, selector: string) {
  const el = page.locator(selector).first()
  await el.waitFor({ state: "visible" })
  const b = await el.boundingBox()
  if (!b) throw new Error(`No bounding box for: ${selector}`)
  return b
}
