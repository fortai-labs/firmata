import { fetchPages } from "@/lib/api";
import Link from "next/link";

export default async function PagesPage({
  searchParams,
}: {
  searchParams: { job_id?: string };
}) {
  const jobId = searchParams.job_id;
  const pages = await fetchPages(jobId);

  return (
    <div className="space-y-6">
      <div className="flex justify-between items-center">
        <div className="space-y-1">
          <h2 className="text-2xl font-bold tracking-tight">Crawled Pages</h2>
          <p className="text-muted-foreground">
            {jobId
              ? `Pages crawled for job ${jobId.substring(0, 8)}...`
              : "All crawled pages from scraper jobs"}
          </p>
        </div>
        {jobId && (
          <Link
            href={`/scraper/jobs/${jobId}`}
            className="inline-flex items-center justify-center rounded-md text-sm font-medium transition-colors focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2 disabled:opacity-50 disabled:pointer-events-none ring-offset-background border border-input bg-background hover:bg-accent hover:text-accent-foreground h-9 px-4 py-2"
          >
            Back to Job
          </Link>
        )}
      </div>

      {pages.length === 0 ? (
        <div className="flex flex-col items-center justify-center rounded-lg border border-dashed p-8">
          <div className="flex h-20 w-20 items-center justify-center rounded-full bg-muted">
            <svg
              xmlns="http://www.w3.org/2000/svg"
              width="24"
              height="24"
              viewBox="0 0 24 24"
              fill="none"
              stroke="currentColor"
              strokeWidth="2"
              strokeLinecap="round"
              strokeLinejoin="round"
              className="h-10 w-10 text-muted-foreground"
              role="img"
              aria-labelledby="page-icon-title"
            >
              <title id="page-icon-title">Page icon</title>
              <path d="M14.5 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V7.5L14.5 2z" />
              <polyline points="14 2 14 8 20 8" />
            </svg>
          </div>
          <h3 className="mt-4 text-lg font-semibold">No pages found</h3>
          <p className="mt-2 text-center text-sm text-muted-foreground">
            {jobId
              ? "This job hasn't crawled any pages yet or is still in progress."
              : "No pages have been crawled yet. Start by creating a configuration and running a job."}
          </p>
          <Link
            href={jobId ? `/scraper/jobs/${jobId}` : "/scraper/configs"}
            className="mt-4 inline-flex items-center justify-center rounded-md text-sm font-medium transition-colors focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2 disabled:opacity-50 disabled:pointer-events-none ring-offset-background bg-primary text-primary-foreground hover:bg-primary/90 h-9 px-4 py-2"
          >
            {jobId ? "View Job" : "View Configurations"}
          </Link>
        </div>
      ) : (
        <div className="rounded-md border">
          <div className="relative w-full overflow-auto">
            <table className="w-full caption-bottom text-sm">
              <thead className="[&_tr]:border-b">
                <tr className="border-b transition-colors hover:bg-muted/50 data-[state=selected]:bg-muted">
                  <th className="h-12 px-4 text-left align-middle font-medium text-muted-foreground">
                    URL
                  </th>
                  <th className="h-12 px-4 text-left align-middle font-medium text-muted-foreground">
                    Status
                  </th>
                  <th className="h-12 px-4 text-left align-middle font-medium text-muted-foreground">
                    Crawled At
                  </th>
                  <th className="h-12 px-4 text-left align-middle font-medium text-muted-foreground">
                    Depth
                  </th>
                  <th className="h-12 px-4 text-left align-middle font-medium text-muted-foreground">
                    Actions
                  </th>
                </tr>
              </thead>
              <tbody className="[&_tr:last-child]:border-0">
                {pages.map((page) => (
                  <tr
                    key={page.id}
                    className="border-b transition-colors hover:bg-muted/50 data-[state=selected]:bg-muted"
                  >
                    <td className="p-4 align-middle max-w-xs truncate">
                      <a
                        href={page.url}
                        target="_blank"
                        rel="noopener noreferrer"
                        className="text-blue-600 hover:underline dark:text-blue-400"
                      >
                        {page.url}
                      </a>
                    </td>
                    <td className="p-4 align-middle">
                      {page.error_message ? (
                        <span className="inline-flex items-center rounded-full px-2.5 py-0.5 text-xs font-semibold bg-red-50 text-red-700 dark:bg-red-900/20 dark:text-red-400">
                          Failed
                        </span>
                      ) : (
                        <span className="inline-flex items-center rounded-full px-2.5 py-0.5 text-xs font-semibold bg-green-50 text-green-700 dark:bg-green-900/20 dark:text-green-400">
                          {page.http_status}
                        </span>
                      )}
                    </td>
                    <td className="p-4 align-middle">
                      {new Date(page.crawled_at).toLocaleString()}
                    </td>
                    <td className="p-4 align-middle">{page.depth}</td>
                    <td className="p-4 align-middle">
                      <div className="flex space-x-2">
                        <Link
                          href={`/scraper/pages/${page.id}`}
                          className="inline-flex items-center justify-center rounded-md text-sm font-medium transition-colors focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2 disabled:opacity-50 disabled:pointer-events-none ring-offset-background border border-input bg-background hover:bg-accent hover:text-accent-foreground h-9 px-4 py-2"
                        >
                          View
                        </Link>
                        {page.html_storage_path && (
                          <Link
                            href={`/scraper/pages/${page.id}/html`}
                            className="inline-flex items-center justify-center rounded-md text-sm font-medium transition-colors focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2 disabled:opacity-50 disabled:pointer-events-none ring-offset-background border border-input bg-background hover:bg-accent hover:text-accent-foreground h-9 px-4 py-2"
                          >
                            HTML
                          </Link>
                        )}
                        {page.markdown_storage_path && (
                          <Link
                            href={`/scraper/pages/${page.id}/markdown`}
                            className="inline-flex items-center justify-center rounded-md text-sm font-medium transition-colors focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2 disabled:opacity-50 disabled:pointer-events-none ring-offset-background border border-input bg-background hover:bg-accent hover:text-accent-foreground h-9 px-4 py-2"
                          >
                            Markdown
                          </Link>
                        )}
                      </div>
                    </td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
        </div>
      )}
    </div>
  );
}
