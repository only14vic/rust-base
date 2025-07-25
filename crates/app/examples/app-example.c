#include "../include/libapp.h"
#include <assert.h>
#include <malloc.h>
#include <stdbool.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

int main(int argc, const char *argv[]) {
    void *app = app_boot(argc, argv);

    log_msg(INFO, __FUNCTION__, "Starting...");

    app_run(app);
    app_free(app);
}
