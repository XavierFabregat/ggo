-- Clean up test data from production database
-- Run with: sqlite3 ~/.config/ggo/data.db < cleanup_test_data.sql

-- Remove test repo entries
DELETE FROM branches WHERE repo_path LIKE '/test/repo/%';
DELETE FROM aliases WHERE repo_path LIKE '/test/repo/%';

-- Remove temp folder entries
DELETE FROM branches WHERE repo_path LIKE '/private/var/folders/%';
DELETE FROM aliases WHERE repo_path LIKE '/private/var/folders/%';

-- Remove entries with trailing slashes (from old bug)
DELETE FROM branches WHERE repo_path LIKE '%/';
DELETE FROM aliases WHERE repo_path LIKE '%/';

-- Show what's left
SELECT 'Remaining branches:' as info;
SELECT COUNT(*) as count FROM branches;

SELECT 'Remaining aliases:' as info;
SELECT COUNT(*) as count FROM aliases;

SELECT 'Branch records by repo:' as info;
SELECT repo_path, COUNT(*) as branch_count
FROM branches
GROUP BY repo_path
ORDER BY branch_count DESC;
