

#pragma once

#include <stdarg.h>
#include <stdbool.h>
#include <stdint.h>
#include <stdlib.h>


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

/**
 * Logger
 */
typedef struct Logger Logger;

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
