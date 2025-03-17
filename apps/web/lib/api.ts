// API base URL - can be configured based on environment
const API_BASE_URL = process.env.NEXT_PUBLIC_SCRAPER_API_URL || 'http://localhost:8080';

// Types
export interface ScraperConfig {
  id: string;
  name: string;
  base_url: string;
  description?: string;
  include_patterns: string[];
  exclude_patterns: string[];
  max_depth: number;
  max_pages_per_job?: number;
  respect_robots_txt?: boolean;
  user_agent?: string;
  request_delay_ms?: number;
  max_concurrent_requests?: number;
  schedule?: string;
  headers?: Record<string, unknown>;
  created_at: string;
  updated_at: string;
  active: boolean;
}

export interface Job {
  id: string;
  config_id: string;
  status: string;
  created_at: string;
  updated_at: string;
  started_at?: string;
  completed_at?: string;
  error_message?: string;
  pages_crawled: number;
  pages_failed: number;
  pages_skipped: number;
  next_run_at?: string;
  worker_id?: string;
  metadata?: Record<string, unknown>;
  error?: string;
  config?: ScraperConfig;
}

export interface Page {
  id: string;
  job_id: string;
  url: string;
  normalized_url: string;
  content_hash: string;
  http_status: number;
  http_headers: Record<string, unknown>;
  crawled_at: string;
  html_storage_path?: string;
  markdown_storage_path?: string;
  title?: string;
  metadata?: Record<string, unknown>;
  error_message?: string;
  depth: number;
  parent_url?: string;
  headers: Record<string, unknown>;
}

// API functions
export async function fetchConfigs(): Promise<ScraperConfig[]> {
  try {
    const response = await fetch(`${API_BASE_URL}/api/configs`);
    if (!response.ok) {
      console.error(`Failed to fetch configs: ${response.statusText}`);
      return [];
    }
    const data = await response.json();
    console.log(`data: ${JSON.stringify(data, null, 2)}`);
    return data.configs || [];
  } catch (error) {
    console.error("Error fetching configs:", error);
    return [];
  }
}

export async function fetchConfig(id: string): Promise<ScraperConfig | null> {
  try {
    const response = await fetch(`${API_BASE_URL}/api/configs/${id}`);
    if (!response.ok) {
      console.error(`Failed to fetch config: ${response.statusText}`);
      return null;
    }
    const data = await response.json();
    return data.config;
  } catch (error) {
    console.error("Error fetching config:", error);
    return null;
  }
}

export async function updateConfig(id: string, config: Partial<ScraperConfig>): Promise<ScraperConfig | null> {
  try {
    const response = await fetch(`${API_BASE_URL}/api/configs/${id}`, {
      method: 'PUT',
      headers: {
        'Content-Type': 'application/json',
      },
      body: JSON.stringify(config),
    });
    if (!response.ok) {
      console.error(`Failed to update config: ${response.statusText}`);
      return null;
    }
    const data = await response.json();
    return data.config;
  } catch (error) {
    console.error("Error updating config:", error);
    return null;
  }
}

export async function deleteConfig(id: string): Promise<boolean> {
  try {
    const response = await fetch(`http://localhost:8080/api/configs/${id}`, {
      method: 'DELETE',
    });
    if (!response.ok) {
      console.error(`Failed to delete config: ${response.statusText}`);
      return false;
    }
    return true;
  } catch (error) {
    console.error("Error deleting config:", error);
    return false;
  }
}

export async function createConfig(config: Omit<ScraperConfig, 'id'>): Promise<ScraperConfig | null> {
  try {
    const response = await fetch(`${API_BASE_URL}/api/configs`, {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
      },
      body: JSON.stringify(config),
    });
    if (!response.ok) {
      console.error(`Failed to create config: ${response.statusText}`);
      return null;
    }
    const data = await response.json();
    return data.config;
  } catch (error) {
    console.error("Error creating config:", error);
    return null;
  }
}

export async function startJob(configId: string): Promise<Job | null> {
  try {
    const response = await fetch(`${API_BASE_URL}/api/configs/${configId}/start`, {
      method: 'POST',
    });
    if (!response.ok) {
      console.error(`Failed to start job: ${response.statusText}`, response);
      return null;
    }
    const data = await response.json();
    return data.job;
  } catch (error) {
    console.error("Error starting job:", error);
    return null;
  }
}

export async function fetchJobs(): Promise<Job[]> {
  try {
    const response = await fetch(`${API_BASE_URL}/api/jobs`);
    if (!response.ok) {
      console.error(`Failed to fetch jobs: ${response.statusText}`);
      return [];
    }
    const data = await response.json();
    return data.jobs || [];
  } catch (error) {
    console.error("Error fetching jobs:", error);
    return [];
  }
}

export async function fetchJob(id: string): Promise<Job | null> {
  try {
    const response = await fetch(`${API_BASE_URL}/api/jobs/${id}`);
    if (!response.ok) {
      console.error(`Failed to fetch job: ${response.statusText}`);
      return null;
    }
    const data = await response.json();
    return data.job;
  } catch (error) {
    console.error("Error fetching job:", error);
    return null;
  }
}

export async function cancelJob(id: string): Promise<boolean> {
  try {
    const response = await fetch(`${API_BASE_URL}/api/jobs/${id}/cancel`, {
      method: 'POST',
    });
    if (!response.ok) {
      console.error(`Failed to cancel job: ${response.statusText}`);
      return false;
    }
    return true;
  } catch (error) {
    console.error("Error cancelling job:", error);
    return false;
  }
}

export async function fetchPages(jobId?: string): Promise<Page[]> {
  try {
    const url = jobId 
      ? `${API_BASE_URL}/api/pages?job_id=${jobId}` 
      : `${API_BASE_URL}/api/pages`;
    
    const response = await fetch(url);
    if (!response.ok) {
      console.error(`Failed to fetch pages: ${response.statusText}`);
      return [];
    }
    const data = await response.json();
    return data.pages || [];
  } catch (error) {
    console.error("Error fetching pages:", error);
    return [];
  }
}

export async function fetchPage(id: string): Promise<Page | null> {
  try {
    const response = await fetch(`${API_BASE_URL}/api/pages/${id}`);
    if (!response.ok) {
      console.error(`Failed to fetch page: ${response.statusText}`);
      return null;
    }
    const data = await response.json();
    return data.page;
  } catch (error) {
    console.error("Error fetching page:", error);
    return null;
  }
}

export async function fetchPageHtml(id: string): Promise<string | null> {
  try {
    const response = await fetch(`${API_BASE_URL}/api/pages/${id}/html`);
    if (!response.ok) {
      console.error(`Failed to fetch page HTML: ${response.statusText}`);
      return null;
    }
    return await response.text();
  } catch (error) {
    console.error("Error fetching page HTML:", error);
    return null;
  }
}

export async function fetchPageMarkdown(id: string): Promise<string | null> {
  try {
    const response = await fetch(`${API_BASE_URL}/api/pages/${id}/markdown`);
    if (!response.ok) {
      console.error(`Failed to fetch page Markdown: ${response.statusText}`);
      return null;
    }
    return await response.text();
  } catch (error) {
    console.error("Error fetching page Markdown:", error);
    return null;
  }
} 