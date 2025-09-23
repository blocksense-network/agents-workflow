// AgentFSKitFFI.h
// Bridging header for AgentFS Rust FFI functions

#ifndef AGENTFSKITFFI_H
#define AGENTFSKITFFI_H

#include <stdint.h>
#include <stddef.h>

// Core lifecycle functions
void* agentfs_bridge_core_create(void);
void agentfs_bridge_core_destroy(void* core);

// Error handling
size_t agentfs_bridge_get_error_message(char* buffer, size_t buffer_size);

// File system operations
int agentfs_bridge_statfs(void* core, char* buffer, size_t buffer_size);
int agentfs_bridge_stat(void* core, const char* path, char* buffer, size_t buffer_size);
int agentfs_bridge_mkdir(void* core, const char* path, uint32_t mode);
int agentfs_bridge_readdir(void* core, const char* path, char* buffer, size_t buffer_size);
int agentfs_bridge_open(void* core, const char* path, const char* options, uint64_t* handle);
int agentfs_bridge_read(void* core, uint64_t handle, uint64_t offset, void* buffer, uint32_t length, uint32_t* bytes_read);
int agentfs_bridge_write(void* core, uint64_t handle, uint64_t offset, const void* buffer, uint32_t length, uint32_t* bytes_written);
int agentfs_bridge_close(void* core, uint64_t handle);

// Control plane operations
int agentfs_bridge_snapshot_create(void* core, const char* name, char* snapshot_id, size_t snapshot_id_size);
int agentfs_bridge_branch_create(void* core, const char* snapshot_id, const char* branch_name, char* branch_id, size_t branch_id_size);
int agentfs_bridge_bind_process(void* core, const char* branch_id);

#endif // AGENTFSKITFFI_H
