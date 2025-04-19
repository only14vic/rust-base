

#pragma once

#include <stdarg.h>
#include <stdbool.h>
#include <stdint.h>
#include <stdlib.h>


typedef enum LogLevel {
  ERROR,
  WARN,
  INFO,
  DEBUG,
  TRACE,
} LogLevel;

int dotenv(bool overwrite);

int log_init(void);

void log_msg(enum LogLevel level, const char *msg);
