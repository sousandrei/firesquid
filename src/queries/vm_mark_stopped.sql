UPDATE vms
SET status = 'stopped',
    pid = NULL,
    updated_at = CURRENT_TIMESTAMP
WHERE id = ?
