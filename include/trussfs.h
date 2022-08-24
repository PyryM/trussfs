#include <stdint.h>
#include <stdbool.h>

typedef struct trussfs_ctx trussfs_ctx;

uint64_t trussfs_version();
trussfs_ctx* trussfs_init();
void trussfs_shutdown(trussfs_ctx* ctx);

uint64_t trussfs_recursive_makedir(trussfs_ctx* ctx, const char* path);

const char* trussfs_working_dir(trussfs_ctx* ctx);
const char* trussfs_binary_dir(trussfs_ctx* ctx);

uint64_t trussfs_archive_mount(trussfs_ctx* ctx, const char* path);
void trussfs_archive_free(trussfs_ctx* ctx, uint64_t archive_handle);
uint64_t trussfs_archive_list(trussfs_ctx* ctx, uint64_t archive_handle);
uint64_t trussfs_archive_filesize_name(trussfs_ctx* ctx, uint64_t archive_handle, const char* name);
uint64_t trussfs_archive_filesize_index(trussfs_ctx* ctx, uint64_t archive_handle, uint64_t index);
int64_t trussfs_archive_read_name(trussfs_ctx* ctx, uint64_t archive_handle, const char* name, uint8_t* dest, uint64_t dest_size);
int64_t trussfs_archive_read_index(trussfs_ctx* ctx, uint64_t archive_handle, uint64_t index, uint8_t* dest, uint64_t dest_size);

uint64_t trussfs_list_dir(trussfs_ctx* ctx, const char* path, bool files_only, bool include_metadata);

uint64_t trussfs_split_path(trussfs_ctx* ctx, const char* path);

void trussfs_list_free(trussfs_ctx* ctx, uint64_t list_handle);
uint64_t trussfs_list_length(trussfs_ctx* ctx, uint64_t list_handle);
const char* trussfs_list_get(trussfs_ctx* ctx, uint64_t list_handle, uint64_t index);