import { fetchPage, fetchPageHtml, fetchPageMarkdown } from "@/lib/api";
import Link from "next/link";
import { notFound } from "next/navigation";
import { useState } from "react";

// Define a type for page status
interface PageWithStatus
  extends Omit<
    ReturnType<typeof fetchPage> extends Promise<infer T> ? T : never,
    "status"
  > {
  status?: "success" | "error" | "pending";
}

export default async function PageDetailPage({
  params,
}: {
  params: { id: string };
}) {
  const pageId = params.id;
  const page = await fetchPage(pageId);

  if (!page) {
    notFound();
  }

  // Cast page to include status
  const pageWithStatus = page as PageWithStatus;

  // Determine status based on http_status or error_message
  if (!pageWithStatus.status) {
    if (page.error_message) {
      pageWithStatus.status = "error";
    } else if (page.http_status >= 200 && page.http_status < 300) {
      pageWithStatus.status = "success";
    } else {
      pageWithStatus.status = "pending";
    }
  }

  const htmlContent =
    (await fetchPageHtml(pageId)) || "No HTML content available";
  const markdownContent =
    (await fetchPageMarkdown(pageId)) || "No Markdown content available";

  return (
    <div className="space-y-6">
      <div className="flex justify-between items-center">
        <div className="space-y-1">
          <h2 className="text-2xl font-bold tracking-tight">Page Content</h2>
          <p className="text-muted-foreground">
            Viewing content for page {pageId}
          </p>
        </div>
        <div className="flex space-x-2">
          <Link
            href={`/scraper/jobs/${page.job_id}`}
            className="inline-flex items-center justify-center rounded-md text-sm font-medium transition-colors focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2 disabled:opacity-50 disabled:pointer-events-none ring-offset-background border border-input bg-background hover:bg-accent hover:text-accent-foreground h-9 px-4 py-2"
          >
            Back to Job
          </Link>
          <a
            href={page.url}
            target="_blank"
            rel="noopener noreferrer"
            className="inline-flex items-center justify-center rounded-md text-sm font-medium transition-colors focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2 disabled:opacity-50 disabled:pointer-events-none ring-offset-background bg-primary text-primary-foreground hover:bg-primary/90 h-9 px-4 py-2"
          >
            Visit Original Page
          </a>
        </div>
      </div>

      <div className="rounded-lg border p-6 space-y-4">
        <h3 className="text-lg font-medium">Page Information</h3>

        <div className="grid gap-4 md:grid-cols-2">
          <div>
            <div className="text-sm font-medium text-muted-foreground">URL</div>
            <div className="mt-1 break-all">
              <a
                href={page.url}
                target="_blank"
                rel="noopener noreferrer"
                className="text-blue-600 hover:underline dark:text-blue-400"
              >
                {page.url}
              </a>
            </div>
          </div>

          <div>
            <div className="text-sm font-medium text-muted-foreground">
              Title
            </div>
            <div className="mt-1">{page.title || "No title"}</div>
          </div>

          <div>
            <div className="text-sm font-medium text-muted-foreground">
              Job ID
            </div>
            <div className="mt-1">
              <Link
                href={`/scraper/jobs/${page.job_id}`}
                className="text-blue-600 hover:underline dark:text-blue-400"
              >
                {page.job_id}
              </Link>
            </div>
          </div>

          <div>
            <div className="text-sm font-medium text-muted-foreground">
              Status
            </div>
            <div className="mt-1">
              <span
                className={`inline-flex items-center rounded-full px-2.5 py-0.5 text-xs font-medium ${
                  pageWithStatus.status === "success"
                    ? "bg-green-100 text-green-800 dark:bg-green-900/20 dark:text-green-400"
                    : pageWithStatus.status === "error"
                      ? "bg-red-100 text-red-800 dark:bg-red-900/20 dark:text-red-400"
                      : "bg-gray-100 text-gray-800 dark:bg-gray-800 dark:text-gray-400"
                }`}
              >
                {pageWithStatus.status.charAt(0).toUpperCase() +
                  pageWithStatus.status.slice(1)}
              </span>
            </div>
          </div>
        </div>
      </div>

      <div className="space-y-4">
        <div className="flex justify-between items-center">
          <h3 className="text-lg font-medium">Content</h3>
          <div className="flex space-x-2">
            <ContentTabs
              htmlContent={htmlContent}
              markdownContent={markdownContent}
            />
          </div>
        </div>
      </div>
    </div>
  );
}

// Client component for tabs
("use client");

function ContentTabs({
  htmlContent,
  markdownContent,
}: {
  htmlContent: string;
  markdownContent: string;
}) {
  const [activeTab, setActiveTab] = useState<"html" | "markdown" | "preview">(
    "preview"
  );

  return (
    <div className="w-full space-y-4">
      <div className="flex border-b">
        <button
          type="button"
          onClick={() => setActiveTab("preview")}
          className={`px-4 py-2 text-sm font-medium ${
            activeTab === "preview"
              ? "border-b-2 border-primary text-primary"
              : "text-muted-foreground hover:text-foreground"
          }`}
        >
          Preview
        </button>
        <button
          type="button"
          onClick={() => setActiveTab("html")}
          className={`px-4 py-2 text-sm font-medium ${
            activeTab === "html"
              ? "border-b-2 border-primary text-primary"
              : "text-muted-foreground hover:text-foreground"
          }`}
        >
          HTML
        </button>
        <button
          type="button"
          onClick={() => setActiveTab("markdown")}
          className={`px-4 py-2 text-sm font-medium ${
            activeTab === "markdown"
              ? "border-b-2 border-primary text-primary"
              : "text-muted-foreground hover:text-foreground"
          }`}
        >
          Markdown
        </button>
      </div>

      <div className="rounded-lg border p-4">
        {activeTab === "preview" && (
          <div className="prose prose-sm max-w-none dark:prose-invert overflow-auto max-h-[500px]">
            <iframe
              srcDoc={htmlContent}
              title="Page content preview"
              className="w-full h-[500px] border-0"
              sandbox="allow-same-origin"
            />
          </div>
        )}

        {activeTab === "html" && (
          <pre className="p-4 bg-muted rounded-md overflow-auto max-h-[500px] text-xs">
            <code>{htmlContent}</code>
          </pre>
        )}

        {activeTab === "markdown" && (
          <pre className="p-4 bg-muted rounded-md overflow-auto max-h-[500px] text-xs">
            <code>{markdownContent}</code>
          </pre>
        )}
      </div>
    </div>
  );
}
