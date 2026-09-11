UPDATE vms
SET status = 'stopping',
    updated_at = CURRENT_TIMESTAMP
WHERE id = ?
