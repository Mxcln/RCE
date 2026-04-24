#ifndef RCE_KOCIEMBA_WRAPPER_H
#define RCE_KOCIEMBA_WRAPPER_H

typedef enum {
    RCE_KOCIEMBA_OK = 0,
    RCE_KOCIEMBA_INVALID_INPUT = 1,
    RCE_KOCIEMBA_MAX_DEPTH_EXCEEDED = 2,
    RCE_KOCIEMBA_TIMEOUT = 3,
    RCE_KOCIEMBA_INTERNAL_ERROR = 4
} rce_kociemba_status_t;

typedef struct {
    int status;
    char* solution_text;
} rce_kociemba_result_t;

rce_kociemba_result_t rce_kociemba_solve(
    const char* facelets,
    int max_depth,
    long timeout_seconds,
    const char* cache_dir
);

void rce_kociemba_free_string(char* ptr);

#endif
