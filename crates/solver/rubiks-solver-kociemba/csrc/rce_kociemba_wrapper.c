#include <stdlib.h>

#include "coordcube.h"
#include "facecube.h"
#include "search.h"
#include "rce_kociemba_wrapper.h"

static int is_valid_facelets(const char* facelets)
{
    int count[6] = {0};
    int i;

    if (facelets == NULL) {
        return 0;
    }

    for (i = 0; i < 54; i++) {
        switch (facelets[i]) {
        case 'U':
            count[U]++;
            break;
        case 'R':
            count[R]++;
            break;
        case 'F':
            count[F]++;
            break;
        case 'D':
            count[D]++;
            break;
        case 'L':
            count[L]++;
            break;
        case 'B':
            count[B]++;
            break;
        default:
            return 0;
        }
    }

    if (facelets[54] != '\0') {
        return 0;
    }

    for (i = 0; i < 6; i++) {
        if (count[i] != 9) {
            return 0;
        }
    }

    return 1;
}

static int verify_facelets(const char* facelets)
{
    facecube_t* fc;
    cubiecube_t* cc;
    int result;

    if (!is_valid_facelets(facelets)) {
        return RCE_KOCIEMBA_INVALID_INPUT;
    }

    fc = get_facecube_fromstring((char*) facelets);
    cc = toCubieCube(fc);
    result = verify(cc);
    free(fc);
    free(cc);

    if (result != 0) {
        return RCE_KOCIEMBA_INVALID_INPUT;
    }

    return RCE_KOCIEMBA_OK;
}

rce_kociemba_result_t rce_kociemba_solve(
    const char* facelets,
    int max_depth,
    long timeout_seconds,
    const char* cache_dir
)
{
    char* solution_text;
    int verify_result;
    rce_kociemba_result_t result;

    result.status = RCE_KOCIEMBA_INTERNAL_ERROR;
    result.solution_text = NULL;

    verify_result = verify_facelets(facelets);
    if (verify_result != RCE_KOCIEMBA_OK) {
        result.status = verify_result;
        return result;
    }

    solution_text = solution((char*) facelets, max_depth, timeout_seconds, 0, cache_dir);
    if (solution_text == NULL) {
        if (timeout_seconds <= 0) {
            result.status = RCE_KOCIEMBA_TIMEOUT;
        } else {
            result.status = RCE_KOCIEMBA_MAX_DEPTH_EXCEEDED;
        }
        return result;
    }

    result.status = RCE_KOCIEMBA_OK;
    result.solution_text = solution_text;
    return result;
}

void rce_kociemba_free_string(char* ptr)
{
    free(ptr);
}
