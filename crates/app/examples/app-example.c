#include "../include/libapp.h"
#include <assert.h>
#include <malloc.h>
#include <stdbool.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

const void *main_module(App *app, AppEvent event);

const AppModuleC MAIN_MODULE = main_module;

int main(int argc, const char *argv[]) {
    const void *(*modules[])(App *, AppEvent) = {
        MAIN_MODULE, MODULE_APP_CONFIG
    };

    App *app = app_new(modules, 2);
    app_boot(app, argc, argv);
    app_run(app);
    app_free(app);

    return 0;
}

const void *main_module(App *app, AppEvent event) {
    char msg[100] = "";
    sprintf(msg, "Catched event: %d", event);
    log_msg(TRACE, __FUNCTION__, msg);

    if (event == APP_INIT) {
        app_register_command(app, "run", MAIN_MODULE);
    }

    if (event == APP_RUN) {
        log_msg(INFO, __FUNCTION__, "Hello from C!");
    }

    return NULL;
    // OR
    // return app_error("Error from C module");
}
