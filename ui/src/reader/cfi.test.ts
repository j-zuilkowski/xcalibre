import { describe, it, expect } from "vitest"
import { calcProgress } from "./cfi"

describe("calcProgress", () => {
  it("returns 0 for first item at scroll 0", () => {
    expect(calcProgress(0, 10, 0)).toBe(0)
  })
  it("handles empty spine", () => {
    expect(calcProgress(0, 0, 0)).toBe(0)
  })
  it("returns 100 for last item at scroll 1", () => {
    expect(calcProgress(9, 10, 1)).toBe(100)
  })
  it("clamps at 100", () => {
    expect(calcProgress(10, 10, 1)).toBe(100)
  })
})
