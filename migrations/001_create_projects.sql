-- Create projects table
CREATE TABLE IF NOT EXISTS projects (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    content TEXT NOT NULL, -- JSON content for Excalidraw scene data
    uid INTEGER NOT NULL,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME DEFAULT CURRENT_TIMESTAMP
);

-- Create index on uid for faster queries
CREATE INDEX IF NOT EXISTS idx_projects_uid ON projects(uid);

-- Create index on created_at for ordering
CREATE INDEX IF NOT EXISTS idx_projects_created_at ON projects(created_at);