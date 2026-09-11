UPDATE vms
SET status = 'failed',
    pid = NULL,
    last_error = ?,
    updated_at = CURRENT_TIMESTAMP
WHERE id = ?
