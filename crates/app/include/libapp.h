

#pragma once

#include <stdarg.h>
#include <stdbool.h>
#include <stdint.h>
#include <stdlib.h>
#include "../../app-base/include/libapp_base.h"


typedef enum AppEvent {
  APP_INIT,
  APP_LOADED,
  APP_BOOT,
  APP_RUN,
  APP_END,
} AppEvent;

typedef struct App App;

typedef unsigned int (*AppModule)(struct App*, enum AppEvent);

#define MODULE_APP module_app_c

#define MODULE_APP_CONFIG module_app_config_c

struct App *app_new(AppModule *modules, unsigned int count);

void app_boot(struct App *app, int argc, const char *const *argv);

void app_run(struct App *app);

void app_free(struct App *app);

void app_register_command(struct App *app, const char *command, AppModule module);

unsigned int module_app_config_c(struct App *app, enum AppEvent event);
