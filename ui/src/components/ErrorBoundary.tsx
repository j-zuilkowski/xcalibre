import { Component, ErrorInfo, ReactNode } from "react"

interface State { hasError: boolean; message: string; stack?: string }

export class ErrorBoundary extends Component<{ children: ReactNode }, State> {
  state: State = { hasError: false, message: "" }

  static getDerivedStateFromError(error: Error) {
    return { hasError: true, message: error.message }
  }

  componentDidCatch(error: Error, info: ErrorInfo) {
    console.error("xCalibre unhandled error:", error, info)
  }

  render() {
    if (this.state.hasError) {
      return (
        <div className="min-h-screen flex flex-col items-center justify-center p-8 bg-gray-50 dark:bg-gray-900">
          <div className="max-w-lg w-full bg-white dark:bg-gray-800 rounded-xl shadow-lg p-6">
            <h1 className="text-xl font-semibold text-red-600 mb-2">Something went wrong</h1>
            <p className="text-sm text-gray-600 dark:text-gray-400 mb-4">
              {this.state.message}
            </p>
            <button
              onClick={() => this.setState({ hasError: false, message: "" })}
              className="mt-4 px-4 py-2 bg-blue-500 text-white rounded-lg hover:bg-blue-600 text-sm"
            >
              Try again
            </button>
          </div>
        </div>
      )
    }
    return this.props.children
  }
}
