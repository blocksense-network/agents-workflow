// AgentFSKitFFI.h
// Bridging header for AgentFS Rust FFI functions

#ifndef AGENTFSKITFFI_H
#define AGENTFSKITFFI_H

#import <Foundation/Foundation.h>

// Core lifecycle functions
void* agentfs_bridge_core_create(void);
void agentfs_bridge_core_destroy(void* core);

// Error handling
size_t agentfs_bridge_get_error_message(char* buffer, size_t buffer_size);

#endif // AGENTFSKITFFI_H
