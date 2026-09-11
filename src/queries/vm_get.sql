SELECT id, name, status, last_error, kernel_path, rootfs_path, vcpus, memory_mib, pid
FROM vms
WHERE id = ?
