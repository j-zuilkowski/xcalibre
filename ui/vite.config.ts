import { defineConfig } from "vite"
import react from "@vitejs/plugin-react"
import { readFileSync } from "node:fs"
import { fileURLToPath } from "node:url"

const packageJsonPath = fileURLToPath(new URL("./package.json", import.meta.url))
const { version } = JSON.parse(readFileSync(packageJsonPath, "utf8")) as { version: string }

export default defineConfig({
  plugins: [react()],
  define: {
    "import.meta.env.VITE_APP_VERSION": JSON.stringify(version),
  },
  build: { outDir: "dist" },
})
