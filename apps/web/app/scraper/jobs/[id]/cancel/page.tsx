import { cancelJob, fetchJob } from "@/lib/api";
import Link from "next/link";
import { notFound, redirect } from "next/navigation";

export default async function CancelJobPage({
  params,
}: {
  params: Promise<{ id: string }>;
}) {
  const { id } = await params;
  const job = await fetchJob(id);

  console.log(`job: ${JSON.stringify(job)}`);

  if (!job) {
    notFound();
  }

  // If job is already completed, failed, or cancelled, redirect to job details
  if (job.status !== "running" && job.status !== "pending") {
    redirect(`/scraper/jobs/${id}`);
  }

  async function cancelJobAction() {
    "use server";
    const job = await cancelJob(id);
    console.log(`job: ${JSON.stringify(job)}`);
    redirect(`/scraper/jobs/${id}`);
  }

  return (
    <div className="space-y-6">
      <div className="flex justify-between items-center">
        <div className="space-y-1">
          <h2 className="text-2xl font-bold tracking-tight">Cancel Job</h2>
          <p className="text-muted-foreground">
            Cancel the running job {id.substring(0, 8)}...
          </p>
        </div>
        <Link
          href={`/scraper/jobs/${id}`}
          className="inline-flex items-center justify-center rounded-md text-sm font-medium transition-colors focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2 disabled:opacity-50 disabled:pointer-events-none ring-offset-background border border-input bg-background hover:bg-accent hover:text-accent-foreground h-9 px-4 py-2"
        >
          Back to Job
        </Link>
      </div>

      <div className="rounded-lg border p-6">
        <div className="space-y-4">
          <h3 className="text-lg font-medium">Job Information</h3>

          <div className="grid gap-4 md:grid-cols-2">
            <div>
              <div className="text-sm font-medium text-muted-foreground">
                Status
              </div>
              <div className="mt-1">
                <span className="inline-flex items-center rounded-full px-2.5 py-0.5 text-xs font-semibold bg-blue-50 text-blue-700 dark:bg-blue-900/20 dark:text-blue-400">
                  {job.status}
                </span>
              </div>
            </div>

            <div>
              <div className="text-sm font-medium text-muted-foreground">
                Created At
              </div>
              <div className="mt-1">
                {new Date(job.created_at).toLocaleString()}
              </div>
            </div>

            <div>
              <div className="text-sm font-medium text-muted-foreground">
                Pages Crawled
              </div>
              <div className="mt-1">{job.pages_crawled}</div>
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
                  {job.config?.name || job.config_id}
                </Link>
              </div>
            </div>
          </div>
        </div>

        <div className="mt-6">
          <div className="rounded-lg border border-red-200 bg-red-50 p-4 dark:border-red-900/20 dark:bg-red-900/10">
            <h3 className="text-lg font-medium text-red-800 dark:text-red-400">
              Warning
            </h3>
            <p className="mt-2 text-sm text-red-700 dark:text-red-300">
              Cancelling a job will stop the crawler immediately. This action
              cannot be undone. Any pages that have already been crawled will
              remain in the database.
            </p>
          </div>

          <div className="mt-4 flex justify-end">
            <div className="flex space-x-2">
              <Link
                href={`/scraper/jobs/${id}`}
                className="inline-flex items-center justify-center rounded-md text-sm font-medium transition-colors focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2 disabled:opacity-50 disabled:pointer-events-none ring-offset-background border border-input bg-background hover:bg-accent hover:text-accent-foreground h-9 px-4 py-2"
              >
                Go Back
              </Link>
              <form action={cancelJobAction}>
                <button
                  type="submit"
                  className="inline-flex items-center justify-center rounded-md text-sm font-medium transition-colors focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2 disabled:opacity-50 disabled:pointer-events-none ring-offset-background bg-destructive text-destructive-foreground hover:bg-destructive/90 h-9 px-4 py-2"
                >
                  Cancel Job
                </button>
              </form>
            </div>
          </div>
        </div>
      </div>
    </div>
  );
}
