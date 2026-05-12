import { useState, useRef } from "react"

const FIELD_HINTS = ["title:", "author:", "tag:", "series:", "format:", "publisher:", "language:"]

interface Props {
  onSearch: (query: string) => void
  placeholder?: string
}

export function SearchBar({ onSearch, placeholder = "Search… (title:rust AND author:Klabnik)" }: Props) {
  const [value, setValue] = useState("")
  const [showHints, setShowHints] = useState(false)
  const inputRef = useRef<HTMLInputElement>(null)

  const handleChange = (v: string) => {
    setValue(v)
    // Show hints when the last typed segment ends with ':'
    const lastToken = v.split(/\s+/).pop() ?? ""
    setShowHints(lastToken.endsWith(":") && lastToken.length > 1)
  }

  const handleKeyDown = (e: React.KeyboardEvent) => {
    if (e.key === "Enter") {
      setShowHints(false)
      onSearch(value.trim())
    }
    if (e.key === "Escape") {
      setShowHints(false)
      setValue("")
      onSearch("")
    }
  }

  const handleHintClick = (hint: string) => {
    const tokens = value.split(/\s+/)
    tokens[tokens.length - 1] = hint
    const next = tokens.join(" ")
    setValue(next)
    setShowHints(false)
    inputRef.current?.focus()
  }

  return (
    <div className="relative w-full">
      <div className="flex items-center border rounded-lg px-3 py-1.5 bg-white dark:bg-gray-800
                      border-gray-300 dark:border-gray-600 focus-within:ring-2 focus-within:ring-blue-500">
        <svg className="w-4 h-4 text-gray-400 mr-2 flex-shrink-0" fill="none" viewBox="0 0 24 24" stroke="currentColor">
          <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M21 21l-6-6m2-5a7 7 0 11-14 0 7 7 0 0114 0z"/>
        </svg>
        <input
          ref={inputRef}
          type="text"
          value={value}
          onChange={e => handleChange(e.target.value)}
          onKeyDown={handleKeyDown}
          placeholder={placeholder}
          className="flex-1 bg-transparent outline-none text-sm dark:text-white placeholder-gray-400"
        />
        {value && (
          <button
            data-testid="search-clear"
            onClick={() => { setValue(""); setShowHints(false); onSearch("") }}
            className="ml-2 text-gray-400 hover:text-gray-600 text-lg leading-none"
          >×</button>
        )}
      </div>
      {showHints && (
        <ul data-testid="search-hints"
            className="absolute z-50 w-full mt-1 bg-white dark:bg-gray-800 border
                       border-gray-200 dark:border-gray-600 rounded-lg shadow-lg text-sm">
          {FIELD_HINTS.map(hint => (
            <li key={hint}>
              <button
                onMouseDown={e => { e.preventDefault(); handleHintClick(hint) }}
                className="w-full text-left px-4 py-2 hover:bg-gray-50 dark:hover:bg-gray-700
                           text-blue-600 dark:text-blue-400 font-mono"
              >
                {hint}
              </button>
            </li>
          ))}
        </ul>
      )}
    </div>
  )
}
