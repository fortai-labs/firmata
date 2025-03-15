CREATE TABLE IF NOT EXISTS scraper_configs (
    id UUID PRIMARY KEY,
    name VARCHAR(255) NOT NULL,
    description TEXT,
    base_url VARCHAR(2048) NOT NULL,
    include_patterns TEXT[] NOT NULL,
    exclude_patterns TEXT[] NOT NULL,
    max_depth INTEGER NOT NULL,
    max_pages_per_job INTEGER,
    respect_robots_txt BOOLEAN NOT NULL DEFAULT TRUE,
    user_agent VARCHAR(255) NOT NULL,
    request_delay_ms INTEGER NOT NULL,
    max_concurrent_requests INTEGER NOT NULL,
    schedule VARCHAR(100),
    headers JSONB NOT NULL DEFAULT '{}',
    created_at TIMESTAMP WITH TIME ZONE NOT NULL,
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL,
    active BOOLEAN NOT NULL DEFAULT TRUE
);

CREATE INDEX idx_scraper_configs_name ON scraper_configs(name);
CREATE INDEX idx_scraper_configs_active ON scraper_configs(active); 