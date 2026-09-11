UPDATE vms
SET status = 'running',
    pid = ?,
    last_error = NULL,
    updated_at = CURRENT_TIMESTAMP
WHERE id = ?
