import { create } from "zustand"
import { persist } from "zustand/middleware"

interface Settings {
  fontSize:      number
  theme:         "light" | "dark" | "sepia"
  fontFamily:    "serif" | "sans" | "mono"
  setFontSize:   (n: number) => void
  setTheme:      (t: Settings["theme"]) => void
  setFontFamily: (f: Settings["fontFamily"]) => void
}

export const useSettingsStore = create<Settings>()(
  persist(
    (set) => ({
      fontSize:      18,
      theme:         "light",
      fontFamily:    "serif",
      setFontSize:   (fontSize)   => set({ fontSize }),
      setTheme:      (theme)      => set({ theme }),
      setFontFamily: (fontFamily) => set({ fontFamily }),
    }),
    { name: "xcalibre-settings" },
  ),
)
