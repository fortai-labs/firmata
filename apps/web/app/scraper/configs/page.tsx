import { fetchConfigs } from "@/lib/api";
import Link from "next/link";

export default async function ConfigsPage() {
  const configs = await fetchConfigs();

  return (
    <div className="space-y-6">
      <div className="flex justify-between items-center">
        <div className="space-y-1">
          <h2 className="text-2xl font-bold tracking-tight">
            Scraper Configurations
          </h2>
          <p className="text-muted-foreground">
            Manage your scraper configurations with custom crawling rules.
          </p>
        </div>
        <Link
          href="/scraper/configs/new"
          className="inline-flex items-center justify-center rounded-md text-sm font-medium transition-colors focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2 disabled:opacity-50 disabled:pointer-events-none ring-offset-background bg-primary text-primary-foreground hover:bg-primary/90 h-9 px-4 py-2"
        >
          Create Configuration
        </Link>
      </div>

      {configs.length === 0 ? (
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
              aria-labelledby="config-icon-title"
            >
              <title id="config-icon-title">Configuration tool icon</title>
              <path d="M14.7 6.3a1 1 0 0 0 0 1.4l1.6 1.6a1 1 0 0 0 1.4 0l3.77-3.77a6 6 0 0 1-7.94 7.94l-6.91 6.91a2.12 2.12 0 0 1-3-3l6.91-6.91a6 6 0 0 1 7.94-7.94l-3.76 3.76z" />
            </svg>
          </div>
          <h3 className="mt-4 text-lg font-semibold">
            No configurations found
          </h3>
          <p className="mt-2 text-center text-sm text-muted-foreground">
            You haven't created any scraper configurations yet. Start by
            creating one.
          </p>
          <Link
            href="/scraper/configs/new"
            className="mt-4 inline-flex items-center justify-center rounded-md text-sm font-medium transition-colors focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2 disabled:opacity-50 disabled:pointer-events-none ring-offset-background bg-primary text-primary-foreground hover:bg-primary/90 h-9 px-4 py-2"
          >
            Create Configuration
          </Link>
        </div>
      ) : (
        <div className="grid gap-4 md:grid-cols-2 lg:grid-cols-3">
          {configs.map((config) => (
            <div
              key={config.id}
              className="rounded-lg border bg-card text-card-foreground shadow-sm"
            >
              <div className="p-6 space-y-2">
                <h3 className="text-lg font-semibold">{config.name}</h3>
                <div className="text-sm text-muted-foreground">
                  <p>Base URL: {config.base_url}</p>
                  <p>Max Depth: {config.max_depth}</p>
                  <p>Status: {config.active ? "Active" : "Inactive"}</p>
                </div>
                <div className="flex space-x-2 pt-4">
                  <Link
                    href={`/scraper/configs/${config.id}`}
                    className="inline-flex items-center justify-center rounded-md text-sm font-medium transition-colors focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2 disabled:opacity-50 disabled:pointer-events-none ring-offset-background border border-input bg-background hover:bg-accent hover:text-accent-foreground h-9 px-4 py-2"
                  >
                    View
                  </Link>
                  <Link
                    href={`/scraper/configs/${config.id}/edit`}
                    className="inline-flex items-center justify-center rounded-md text-sm font-medium transition-colors focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2 disabled:opacity-50 disabled:pointer-events-none ring-offset-background border border-input bg-background hover:bg-accent hover:text-accent-foreground h-9 px-4 py-2"
                  >
                    Edit
                  </Link>
                  <Link
                    href={`/scraper/configs/${config.id}/start`}
                    className="inline-flex items-center justify-center rounded-md text-sm font-medium transition-colors focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2 disabled:opacity-50 disabled:pointer-events-none ring-offset-background bg-primary text-primary-foreground hover:bg-primary/90 h-9 px-4 py-2"
                  >
                    Start Job
                  </Link>
                </div>
              </div>
            </div>
          ))}
        </div>
      )}
    </div>
  );
}
