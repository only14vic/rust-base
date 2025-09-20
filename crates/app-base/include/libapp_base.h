

#pragma once

#include <stdarg.h>
#include <stdbool.h>
#include <stdint.h>
#include <stdlib.h>

#define MODULE_APP_CONFIG module_app_config



typedef enum AppEvent {
  APP_INIT,
  APP_BOOT,
  APP_SETUP,
  APP_RUN,
  APP_END,
} AppEvent;

/**
 * Logging levels for C
 */
typedef enum LogLevel {
  OFF = 0,
  ERROR = 1,
  WARN = 2,
  INFO = 3,
  DEBUG = 4,
  TRACE = 5,
} LogLevel;

typedef struct App_app_simple_config App_app_simple_config;

/**
 * Logger
 */
typedef struct Logger Logger;

typedef struct App_app_simple_config App;

typedef const void *(*AppModuleC)(App*, enum AppEvent);

App *app_new(AppModuleC *modules, unsigned int count);

void app_boot(App *app, int argc, const char *const *argv);

void app_run(App *app);

void app_free(App *app);

void app_register_command(App *app, const char *command, AppModuleC module);

void app_unregister_command(App *app, const char *command);

void app_register_module(App *app, AppModuleC module);

void app_unregister_module(App *app, AppModuleC module);

const void *app_error(const char *err);

const void *module_app_config(App *app, enum AppEvent event);

/**
 * Loads .env file variables
 *
 * Returns zero if initialization is successfull.
 * Otherwise returns int less zero.
 */
int dotenv(bool overwrite);

/**
 * Initializes logging
 *
 * Returns non-zero pointer if initialization is successfull.
 * Otherwise returns zero.
 */
struct Logger *log_init(void);

/**
 * Logs messages in C
 */
void log_msg(enum LogLevel level, const char *target, const char *msg);

/**
 * Set max log level in C
 */
void log_max_level(enum LogLevel level);

/**
 * Close log file descriptor
 */
void log_close(struct Logger *self);
