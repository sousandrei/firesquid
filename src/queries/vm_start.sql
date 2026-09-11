UPDATE vms
SET status = 'starting',
    last_error = NULL,
    pid = NULL,
    updated_at = CURRENT_TIMESTAMP
WHERE id = ?
