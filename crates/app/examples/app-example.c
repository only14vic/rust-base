#include "../include/libapp.h"
#include <assert.h>
#include <malloc.h>
#include <stdbool.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

static unsigned int main_module(App *app, AppEvent event);

const AppModule MAIN_MODULE = &main_module;

int main(int argc, const char *argv[]) {
    unsigned int (*modules[])(App *, AppEvent) = {
        MAIN_MODULE, MODULE_APP_CONFIG
    };

    App *app = app_new(modules, 2);
    app_register_command(app, "run", MAIN_MODULE);
    app_boot(app, argc, argv);
    app_run(app);
    app_free(app);

    return 0;
}

static unsigned int main_module(App *, AppEvent event) {
    char msg[100] = "";
    sprintf(msg, "Catched event: %d", event);
    log_msg(INFO, __FUNCTION__, msg);

    if (event == APP_RUN) {
        log_msg(INFO, __FUNCTION__, "Application running...");
        malloc_stats();
    }

    return 0;
}
