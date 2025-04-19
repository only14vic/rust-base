

#pragma once

#include <stdarg.h>
#include <stdbool.h>
#include <stdint.h>
#include <stdlib.h>


typedef enum LogLevel {
  ERROR = 1,
  WARN = 2,
  INFO = 3,
  DEBUG = 4,
  TRACE = 5,
} LogLevel;

int dotenv(bool overwrite);

int log_init(void);

void log_msg(enum LogLevel level, const char *msg);
