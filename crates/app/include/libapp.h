

#pragma once

#include <stdarg.h>
#include <stdbool.h>
#include <stdint.h>
#include <stdlib.h>
#include "../../app-base/include/libapp_base.h"


typedef enum AppEvent {
  APP_INIT,
  APP_LOAD_CONFIG,
  APP_BOOT,
  APP_RUN,
  APP_END,
} AppEvent;

typedef struct App App;

typedef size_t (*AppModule)(struct App*, enum AppEvent);

#define MODULE_APP module_app

#define MODULE_APP_CONFIG module_app_config

struct App *app_new(AppModule *modules, unsigned int count);

void app_boot(struct App *app, int argc, const char *const *argv);

void app_run(struct App *app);

void app_free(struct App *app);
