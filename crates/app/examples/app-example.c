#include "../include/libapp.h"
#include <assert.h>
#include <malloc.h>
#include <stdbool.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

size_t main_module(App *app, AppEvent event);

int main(int argc, const char *argv[]) {
    size_t (*modules[])(App *, AppEvent) = {main_module};

    App *app = app_new(modules, 1);
    app_boot(app, argc, argv);
    app_run(app);
    app_free(app);

    return 0;
}

size_t main_module(App *, AppEvent event) {
    char msg[100] = "";
    sprintf(msg, "Catched event: %d", event);
    log_msg(INFO, __FUNCTION__, msg);

    if (event == APP_RUN) {
        log_msg(INFO, __FUNCTION__, "Application running...");
        malloc_stats();
    }

    return 0;
}
