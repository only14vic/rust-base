#include "../include/libapp_base.h"
#include <assert.h>
#include <malloc.h>
#include <stdbool.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

int main() {
    dotenv(false);
    Logger *log = log_init();

    log_msg(ERROR, "This is ERROR");
    log_msg(WARN, "This is WARNING");
    log_msg(INFO, "This is INFO");
    log_msg(DEBUG, "This is DEBUG");
    log_msg(TRACE, "This is TRACE");

    malloc_stats();

    if (log != NULL) {
        log_close(log);
    }
}
