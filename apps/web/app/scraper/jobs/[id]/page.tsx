import { cancelJob, fetchJob, fetchPages } from "@/lib/api";
import Link from "next/link";
import { notFound } from "next/navigation";

// Extend the Page type to include status
interface PageWithStatus {
  id: string;
  url: string;
  title?: string;
  status: "success" | "error" | "pending";
  job_id: string;
}

export default async function JobDetailPage({
  params,
}: {
  params: Promise<{ id: string }>;
}) {
  const { id } = await params;
  const job = await fetchJob(id);

  if (!job) {
    notFound();
  }

  // Fetch pages and add default status if missing
  const pagesData = await fetchPages(id);
  const pages: PageWithStatus[] = pagesData.map((page) => ({
    ...page,
    status:
      ((page as { status?: string }).status as
        | "success"
        | "error"
        | "pending") || "pending",
  }));

  // Format date for display
  const formatDate = (dateString: string) => {
    return new Date(dateString).toLocaleString();
  };

  // Calculate job duration
  const calculateDuration = () => {
    if (!job.completed_at) {
      return "In progress";
    }

    const start = new Date(job.created_at);
    const end = new Date(job.completed_at);
    const durationMs = end.getTime() - start.getTime();

    // Format as minutes and seconds
    const minutes = Math.floor(durationMs / 60000);
    const seconds = Math.floor((durationMs % 60000) / 1000);

    return `${minutes}m ${seconds}s`;
  };

  return (
    <div className="space-y-6">
      <div className="flex justify-between items-center">
        <div className="space-y-1">
          <h2 className="text-2xl font-bold tracking-tight">Job Details</h2>
          <p className="text-muted-foreground">
            View details and results for job {id}
          </p>
        </div>
        <Link
          href="/scraper/jobs"
          className="inline-flex items-center justify-center rounded-md text-sm font-medium transition-colors focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2 disabled:opacity-50 disabled:pointer-events-none ring-offset-background border border-input bg-background hover:bg-accent hover:text-accent-foreground h-9 px-4 py-2"
        >
          Back to Jobs
        </Link>
      </div>

      <div className="rounded-lg border p-6 space-y-4">
        <h3 className="text-lg font-medium">Job Information</h3>

        <div className="grid gap-4 md:grid-cols-2">
          <div>
            <div className="text-sm font-medium text-muted-foreground">
              Status
            </div>
            <div className="mt-1">
              <span
                className={`inline-flex items-center rounded-full px-2.5 py-0.5 text-xs font-medium ${
                  job.status === "completed"
                    ? "bg-green-100 text-green-800 dark:bg-green-900/20 dark:text-green-400"
                    : job.status === "in_progress"
                      ? "bg-blue-100 text-blue-800 dark:bg-blue-900/20 dark:text-blue-400"
                      : job.status === "failed"
                        ? "bg-red-100 text-red-800 dark:bg-red-900/20 dark:text-red-400"
                        : job.status === "cancelled"
                          ? "bg-yellow-100 text-yellow-800 dark:bg-yellow-900/20 dark:text-yellow-400"
                          : "bg-gray-100 text-gray-800 dark:bg-gray-800 dark:text-gray-400"
                }`}
              >
                {job.status.charAt(0).toUpperCase() +
                  job.status.slice(1).replace("_", " ")}
              </span>
            </div>
          </div>

          <div>
            <div className="text-sm font-medium text-muted-foreground">
              Configuration
            </div>
            <div className="mt-1">
              <Link
                href={`/scraper/configs/${job.config_id}`}
                className="text-blue-600 hover:underline dark:text-blue-400"
              >
                View Configuration
              </Link>
            </div>
          </div>

          <div>
            <div className="text-sm font-medium text-muted-foreground">
              Created At
            </div>
            <div className="mt-1">{formatDate(job.created_at)}</div>
          </div>

          {job.completed_at && (
            <div>
              <div className="text-sm font-medium text-muted-foreground">
                Completed At
              </div>
              <div className="mt-1">{formatDate(job.completed_at)}</div>
            </div>
          )}

          <div>
            <div className="text-sm font-medium text-muted-foreground">
              Duration
            </div>
            <div className="mt-1">{calculateDuration()}</div>
          </div>

          <div>
            <div className="text-sm font-medium text-muted-foreground">
              Pages Crawled
            </div>
            <div className="mt-1">{pages.length}</div>
          </div>
        </div>
      </div>

      <div className="rounded-lg border p-6 space-y-4">
        <div className="flex justify-between items-center">
          <h3 className="text-lg font-medium">Crawled Pages</h3>

          {job.status === "in_progress" && (
            <button
              type="button"
              className="inline-flex items-center justify-center rounded-md text-sm font-medium transition-colors focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2 disabled:opacity-50 disabled:pointer-events-none ring-offset-background border border-input bg-background hover:bg-accent hover:text-accent-foreground h-9 px-4 py-2"
              onClick={() => window.location.reload()}
            >
              Refresh
            </button>
          )}
        </div>

        {pages.length === 0 ? (
          <div className="text-center py-8 text-muted-foreground">
            No pages have been crawled yet.
            {job.status === "in_progress" && " The job is still in progress."}
          </div>
        ) : (
          <div className="overflow-x-auto">
            <table className="w-full border-collapse">
              <thead>
                <tr className="border-b">
                  <th className="text-left py-3 px-4 font-medium">URL</th>
                  <th className="text-left py-3 px-4 font-medium">Title</th>
                  <th className="text-left py-3 px-4 font-medium">Status</th>
                  <th className="text-left py-3 px-4 font-medium">Actions</th>
                </tr>
              </thead>
              <tbody>
                {pages.map((page) => (
                  <tr key={page.id} className="border-b hover:bg-muted/50">
                    <td className="py-3 px-4 max-w-xs truncate">
                      <a
                        href={page.url}
                        target="_blank"
                        rel="noopener noreferrer"
                        className="text-blue-600 hover:underline dark:text-blue-400"
                      >
                        {page.url}
                      </a>
                    </td>
                    <td className="py-3 px-4 max-w-xs truncate">
                      {page.title || "No title"}
                    </td>
                    <td className="py-3 px-4">
                      <span
                        className={`inline-flex items-center rounded-full px-2.5 py-0.5 text-xs font-medium ${
                          page.status === "success"
                            ? "bg-green-100 text-green-800 dark:bg-green-900/20 dark:text-green-400"
                            : page.status === "error"
                              ? "bg-red-100 text-red-800 dark:bg-red-900/20 dark:text-red-400"
                              : "bg-gray-100 text-gray-800 dark:bg-gray-800 dark:text-gray-400"
                        }`}
                      >
                        {page.status.charAt(0).toUpperCase() +
                          page.status.slice(1)}
                      </span>
                    </td>
                    <td className="py-3 px-4">
                      <Link
                        href={`/scraper/pages/${page.id}`}
                        className="text-blue-600 hover:underline dark:text-blue-400"
                      >
                        View Content
                      </Link>
                    </td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
        )}
      </div>

      {job.status === "in_progress" && (
        <div className="rounded-lg border border-blue-200 bg-blue-50 p-4 dark:border-blue-900/20 dark:bg-blue-900/10">
          <div className="flex items-center">
            <svg
              className="animate-spin -ml-1 mr-3 h-5 w-5 text-blue-600"
              xmlns="http://www.w3.org/2000/svg"
              fill="none"
              viewBox="0 0 24 24"
              aria-labelledby="loading-spinner-title"
            >
              <title id="loading-spinner-title">Loading spinner</title>
              <circle
                className="opacity-25"
                cx="12"
                cy="12"
                r="10"
                stroke="currentColor"
                strokeWidth="4"
              />
              <path
                className="opacity-75"
                fill="currentColor"
                d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"
              />
            </svg>
            <span className="text-blue-800 dark:text-blue-400">
              This job is currently running. Refresh the page to see new
              results.
            </span>
          </div>
        </div>
      )}

      {job.status === "failed" && (
        <div className="rounded-lg border border-red-200 bg-red-50 p-4 dark:border-red-900/20 dark:bg-red-900/10">
          <h3 className="text-lg font-medium text-red-800 dark:text-red-400">
            Job Failed
          </h3>
          <p className="mt-2 text-sm text-red-700 dark:text-red-300">
            This job encountered an error and could not complete. Check the
            server logs for more details.
          </p>
        </div>
      )}
    </div>
  );
}
