#!/usr/bin/env node
/**
 * Applies known GUI fixes to xcalibre UI source files.
 * Safe to run multiple times — each fix checks if already applied.
 */
import { readFileSync, writeFileSync } from "node:fs"
import { fileURLToPath } from "node:url"
import { join, dirname } from "node:path"

const __dirname = dirname(fileURLToPath(import.meta.url))
const SRC = join(__dirname, "../src")

let fixed = 0
let alreadyFixed = 0
let missed = 0

function patch(filePath, oldStr, newStr, description, { all = false } = {}) {
  const content = readFileSync(filePath, "utf8")
  if (content.includes(newStr.slice(0, 40))) {
    console.log(`  ✓ already applied: ${description}`)
    alreadyFixed++
    return
  }
  if (!content.includes(oldStr.slice(0, 40))) {
    console.warn(`  ? source not found: ${description}`)
    missed++
    return
  }
  const updated = all ? content.replaceAll(oldStr, newStr) : content.replace(oldStr, newStr)
  writeFileSync(filePath, updated)
  console.log(`  ✔ applied: ${description}`)
  fixed++
}

console.log("Applying GUI fixes...\n")

// ── Fix 1 ─────────────────────────────────────────────────────────────────
// BulkActionBar uses `fixed left-1/2 -translate-x-1/2` which centers on the
// full viewport. The sidebar is w-52 (208px), so main content center is
// at viewport/2 + 104px (= 6.5rem). Shift by half the sidebar width.
patch(
  join(SRC, "components/BulkActionBar.tsx"),
  `"fixed bottom-4 left-1/2 -translate-x-1/2 flex items-center gap-3`,
  `"fixed bottom-4 left-[calc(50%+6.5rem)] -translate-x-1/2 flex items-center gap-3`,
  "BulkActionBar: center bar on main content area, not full viewport",
)

// ── Fix 2 ─────────────────────────────────────────────────────────────────
// SettingsModal only shows Server URL when tokenStored === true, leaving the
// General section empty for new users who haven't connected yet.
// Always show it so they can enter the URL to connect.
patch(
  join(SRC, "components/SettingsModal.tsx"),
  `          {tokenStored && (
            <label className="flex flex-col gap-1 mb-3">
              <span className="text-sm">Server URL</span>
              <input
                value={autolibUrl}
                onChange={(e) => setAutolibUrl(e.target.value)}
                className="border border-gray-300 rounded px-2 py-1 text-sm"
                placeholder="https://api.xcalibre.app"
              />
            </label>
          )}`,
  `          <label className="flex flex-col gap-1 mb-3">
            <span className="text-sm">Server URL</span>
            <input
              value={autolibUrl}
              onChange={(e) => setAutolibUrl(e.target.value)}
              className="border border-gray-300 rounded px-2 py-1 text-sm"
              placeholder="https://api.xcalibre.app"
            />
          </label>`,
  "SettingsModal: always show Server URL field in General section",
)

// ── Fix 3 ─────────────────────────────────────────────────────────────────
// Root div uses min-h-screen which allows page-level scroll. App should be
// a fixed-height viewport-filling shell with internal scrolling regions.
patch(
  join(SRC, "App.tsx"),
  `className="min-h-screen bg-gray-50 dark:bg-gray-900 flex"`,
  `className="h-screen overflow-hidden bg-gray-50 dark:bg-gray-900 flex"`,
  "App root: h-screen overflow-hidden for fixed-height app layout",
)

// ── Fix 4 ─────────────────────────────────────────────────────────────────
// Header border buttons (Repair library, Settings) have no dark-mode text or
// border color, so they disappear on dark backgrounds.
patch(
  join(SRC, "App.tsx"),
  `"px-3 py-1.5 text-sm rounded-lg border border-gray-300 hover:bg-gray-100 dark:hover:bg-gray-800"`,
  `"px-3 py-1.5 text-sm rounded-lg border border-gray-300 dark:border-gray-600 dark:text-gray-300 hover:bg-gray-100 dark:hover:bg-gray-800"`,
  "App header: border buttons get dark:border and dark:text classes",
  { all: true },
)

// ── Fix 5 ─────────────────────────────────────────────────────────────────
// darkMode: "class" is set in tailwind.config but nothing ever adds/removes
// the "dark" class on <html>. The theme stored in settingsStore is never
// applied to the document, so dark: variants never activate.
//
// Part A: import useSettingsStore in App.tsx
const appPath = join(SRC, "App.tsx")
let appContent = readFileSync(appPath, "utf8")

if (!appContent.includes("useSettingsStore")) {
  appContent = appContent.replace(
    `import { useLibraryStore } from "./store/libraryStore"`,
    `import { useLibraryStore } from "./store/libraryStore"\nimport { useSettingsStore } from "./store/settingsStore"`,
  )
  writeFileSync(appPath, appContent)
  console.log("  ✔ applied: App.tsx: import useSettingsStore")
  fixed++
} else {
  console.log("  ✓ already applied: App.tsx: import useSettingsStore")
  alreadyFixed++
}

// Part B: read theme and sync to document.documentElement
appContent = readFileSync(appPath, "utf8")

if (!appContent.includes("document.documentElement.classList.toggle")) {
  const anchor = `  const fetchBooks = useLibraryStore((s) => s.fetchBooks)`
  const replacement = `  const fetchBooks = useLibraryStore((s) => s.fetchBooks)
  const theme = useSettingsStore((s) => s.theme)

  useEffect(() => {
    document.documentElement.classList.toggle("dark", theme === "dark")
  }, [theme])`

  if (appContent.includes(anchor)) {
    writeFileSync(appPath, appContent.replace(anchor, replacement))
    console.log("  ✔ applied: App.tsx: sync theme to document.documentElement.classList")
    fixed++
  } else {
    console.warn("  ? could not find anchor for dark mode sync — skipping")
    missed++
  }
} else {
  console.log("  ✓ already applied: App.tsx: dark mode sync")
  alreadyFixed++
}

// ── Summary ───────────────────────────────────────────────────────────────
console.log()
if (fixed > 0) console.log(`  ${fixed} fix${fixed === 1 ? "" : "es"} applied`)
if (alreadyFixed > 0) console.log(`  ${alreadyFixed} already in place`)
if (missed > 0) console.log(`  ${missed} source pattern${missed === 1 ? "" : "s"} not matched — check manually`)
if (fixed === 0 && missed === 0) console.log("  Nothing to do — all fixes already applied.")
