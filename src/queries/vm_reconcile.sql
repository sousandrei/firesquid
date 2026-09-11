UPDATE vms
SET status = 'failed',
    last_error = 'VM runtime is not available after daemon restart',
    updated_at = CURRENT_TIMESTAMP
WHERE status IN ('starting', 'running', 'stopping')
