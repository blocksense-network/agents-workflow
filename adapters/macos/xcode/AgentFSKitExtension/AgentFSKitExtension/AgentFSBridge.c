// AgentFSBridge.c
// C wrapper for AgentFS Rust FFI functions to ensure proper C linkage

#include "AgentFSKitFFI.h"
#include <stdint.h>

// Forward declarations of Rust FFI functions
extern int32_t af_fs_create(const char* config_json, uint64_t* out_fs);
extern int32_t af_fs_destroy(uint64_t fs);
extern int32_t af_mkdir(uint64_t fs, const char* path, uint32_t mode);
extern int32_t af_snapshot_create(uint64_t fs, const char* name, uint8_t* out_id);
extern int32_t af_branch_create_from_snapshot(uint64_t fs, const uint8_t* snap, const char* name, uint8_t* out_id);
extern int32_t af_bind_process_to_branch(uint64_t fs, const uint8_t* branch);

// C wrapper implementations
void* agentfs_bridge_core_create(void) {
    const char* config = "{\"max_memory_bytes\": 67108864, \"max_open_handles\": 1024, \"max_branches\": 10, \"max_snapshots\": 10}";
    uint64_t handle = 0;

    if (af_fs_create(config, &handle) == 0) {
        return (void*)handle;
    }
    return NULL;
}

void agentfs_bridge_core_destroy(void* core) {
    if (core) {
        uint64_t handle = (uint64_t)core;
        af_fs_destroy(handle);
    }
}

size_t agentfs_bridge_get_error_message(char* buffer, size_t buffer_size) {
    // Simple implementation - return empty for now
    if (buffer && buffer_size > 0) {
        buffer[0] = '\0';
    }
    return 0;
}
