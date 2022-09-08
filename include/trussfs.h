#include <stdint.h>
#include <stdbool.h>

typedef struct trussfs_ctx trussfs_ctx;
typedef uint64_t listhandle_t;
typedef uint64_t archivehandle_t;

uint64_t trussfs_version();
trussfs_ctx* trussfs_init();
void trussfs_shutdown(trussfs_ctx* ctx);

uint64_t trussfs_recursive_makedir(trussfs_ctx* ctx, const char* path);

const char* trussfs_working_dir(trussfs_ctx* ctx);
const char* trussfs_binary_dir(trussfs_ctx* ctx);

archivehandle_t trussfs_archive_mount(trussfs_ctx* ctx, const char* path);
void trussfs_archive_free(trussfs_ctx* ctx, archivehandle_t archive);
listhandle_t trussfs_archive_list(trussfs_ctx* ctx, archivehandle_t archive);
uint64_t trussfs_archive_filesize_name(trussfs_ctx* ctx, archivehandle_t archive, const char* name);
uint64_t trussfs_archive_filesize_index(trussfs_ctx* ctx, archivehandle_t archive, uint64_t index);
int64_t trussfs_archive_read_name(trussfs_ctx* ctx, archivehandle_t archive, const char* name, uint8_t* dest, uint64_t dest_size);
int64_t trussfs_archive_read_index(trussfs_ctx* ctx, archivehandle_t archive, uint64_t index, uint8_t* dest, uint64_t dest_size);

listhandle_t trussfs_list_dir(trussfs_ctx* ctx, const char* path, bool files_only, bool include_metadata);

listhandle_t trussfs_split_path(trussfs_ctx* ctx, const char* path);

listhandle_t trussfs_list_new(trussfs_ctx* ctx);
void trussfs_list_free(trussfs_ctx* ctx, listhandle_t list);
uint64_t trussfs_list_length(trussfs_ctx* ctx, listhandle_t list);
const char* trussfs_list_get(trussfs_ctx* ctx, listhandle_t list, uint64_t index);
uint64_t trussfs_list_push(trussfs_ctx* ctx, listhandle_t list, const char* item);