#include "libapp_base.h"
#include <assert.h>
#include <malloc.h>
#include <stdbool.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

int main() {
    dotenv(false);
    Logger *log = log_init();

    log_msg(ERROR, __FUNCTION__, "This is ERROR");
    log_msg(WARN, __FUNCTION__, "This is WARNING");
    log_msg(INFO, __FUNCTION__, "This is INFO");
    log_msg(DEBUG, __FUNCTION__, "This is DEBUG");
    log_msg(TRACE, __FUNCTION__, "This is TRACE");

    malloc_stats();

    if (log != NULL) {
        log_close(log);
    }
}
