#include <stdint.h>
#include <stdbool.h>

typedef struct trussfs_ctx trussfs_ctx;
typedef uint64_t listhandle_t;
typedef uint64_t archivehandle_t;
typedef uint64_t watcherhandle_t;

uint64_t trussfs_version();
trussfs_ctx* trussfs_init();
void trussfs_shutdown(trussfs_ctx* ctx);

const char* trussfs_get_error(trussfs_ctx* ctx);
void trussfs_clear_error(trussfs_ctx* ctx);

uint64_t trussfs_recursive_makedir(trussfs_ctx* ctx, const char* path);

const char* trussfs_working_dir(trussfs_ctx* ctx);
const char* trussfs_binary_dir(trussfs_ctx* ctx);

const char* trussfs_readline(trussfs_ctx* ctx, const char* prompt);

bool trussfs_is_handle_valid(uint64_t handle);

watcherhandle_t trussfs_watcher_create(trussfs_ctx* ctx, const char* path, bool recursive);
bool trussfs_watcher_augment(trussfs_ctx* ctx, watcherhandle_t watcher, const char* path, bool recursive);
void trussfs_watcher_free(trussfs_ctx* ctx, watcherhandle_t watcher);
listhandle_t trussfs_watcher_poll(trussfs_ctx* ctx, watcherhandle_t watcher);

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