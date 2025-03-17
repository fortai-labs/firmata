-- Fix the pages table schema
-- Add missing columns or modify existing ones

-- Check if http_headers column exists, if not add it
DO $$
BEGIN
    IF NOT EXISTS (
        SELECT 1 
        FROM information_schema.columns 
        WHERE table_name = 'pages' AND column_name = 'http_headers'
    ) THEN
        ALTER TABLE pages ADD COLUMN http_headers JSONB NOT NULL DEFAULT '{}';
    END IF;
END $$;

-- Check if start_time column exists in any table and is causing issues
-- This is to fix the "UnexpectedNullError" for "start_time" column
DO $$
BEGIN
    IF EXISTS (
        SELECT 1 
        FROM information_schema.columns 
        WHERE table_name = 'webhook_deliveries' AND column_name = 'start_time'
    ) THEN
        -- Make start_time nullable if it exists
        ALTER TABLE webhook_deliveries ALTER COLUMN start_time DROP NOT NULL;
    END IF;
END $$;

-- Ensure all required columns in the Page struct exist in the pages table
DO $$
BEGIN
    -- Check and add columns if they don't exist
    IF NOT EXISTS (SELECT 1 FROM information_schema.columns WHERE table_name = 'pages' AND column_name = 'normalized_url') THEN
        ALTER TABLE pages ADD COLUMN normalized_url VARCHAR(2048) NOT NULL DEFAULT '';
    END IF;
    
    IF NOT EXISTS (SELECT 1 FROM information_schema.columns WHERE table_name = 'pages' AND column_name = 'content_hash') THEN
        ALTER TABLE pages ADD COLUMN content_hash VARCHAR(64) NOT NULL DEFAULT '';
    END IF;
    
    IF NOT EXISTS (SELECT 1 FROM information_schema.columns WHERE table_name = 'pages' AND column_name = 'depth') THEN
        ALTER TABLE pages ADD COLUMN depth INTEGER NOT NULL DEFAULT 0;
    END IF;
    
    IF NOT EXISTS (SELECT 1 FROM information_schema.columns WHERE table_name = 'pages' AND column_name = 'parent_url') THEN
        ALTER TABLE pages ADD COLUMN parent_url VARCHAR(2048);
    END IF;
END $$; 