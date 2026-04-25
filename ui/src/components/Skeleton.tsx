interface Props {
  className?: string
  count?: number
}

/** Animated loading placeholder bar. */
export function Skeleton({ className = "", count = 1 }: Props) {
  return (
    <>
      {Array.from({ length: count }).map((_, i) => (
        <div
          key={i}
          className={`animate-pulse bg-gray-200 dark:bg-gray-700 rounded ${className}`}
        />
      ))}
    </>
  )
}

export function BookCardSkeleton() {
  return (
    <div className="flex flex-col rounded-lg border border-gray-200 dark:border-gray-700 overflow-hidden">
      <Skeleton className="w-full aspect-[2/3]" />
      <div className="p-2 flex flex-col gap-1">
        <Skeleton className="h-4 w-3/4" />
        <Skeleton className="h-3 w-1/2" />
      </div>
    </div>
  )
}

/** Full library grid skeleton shown while books load. */
export function LibrarySkeleton() {
  return (
    <div className="grid grid-cols-2 sm:grid-cols-3 lg:grid-cols-4 xl:grid-cols-5 gap-4 p-6">
      {Array.from({ length: 10 }).map((_, i) => (
        <div key={i} className="flex flex-col rounded-lg overflow-hidden shadow bg-white dark:bg-gray-800">
          <div className="h-48 animate-pulse bg-gray-200 dark:bg-gray-700" />
          <div className="p-2 flex flex-col gap-1.5">
            <Skeleton className="h-3 w-3/4" />
            <Skeleton className="h-2.5 w-1/2" />
          </div>
        </div>
      ))}
    </div>
  )
}
