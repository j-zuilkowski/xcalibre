export function getCurrentCfi(spineIndex: number): string {
  const elements = document.body.querySelectorAll("*")
  let elementIndex = 0
  for (let i = 0; i < elements.length; i++) {
    const rect = (elements[i] as HTMLElement).getBoundingClientRect()
    if (rect.top >= 0) {
      elementIndex = i
      break
    }
  }
  const scrollY = Math.round(window.scrollY)
  return `${spineIndex}:${elementIndex}:${scrollY}`
}

export function restoreCfi(cfi: string, _doc: Document): void {
  const parts = cfi.split(":")
  if (parts.length < 3) return
  const elementIndex = parseInt(parts[1], 10)
  const scrollY = parseInt(parts[2], 10)
  const elements = document.body.querySelectorAll("*")
  if (elementIndex < elements.length) {
    ;(elements[elementIndex] as HTMLElement).scrollIntoView()
  } else {
    window.scrollTo(0, scrollY)
  }
}

export function calcProgress(
  spineIndex: number,
  totalSpineItems: number,
  scrollRatio: number,
): number {
  if (totalSpineItems <= 0) return 0
  const perItem = 100 / totalSpineItems
  return Math.min(100, spineIndex * perItem + scrollRatio * perItem)
}
