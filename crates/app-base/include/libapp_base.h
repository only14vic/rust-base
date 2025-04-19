

#pragma once

#include <stdarg.h>
#include <stdbool.h>
#include <stdint.h>
#include <stdlib.h>


int dotenv(bool overwrite);

int log_init(void);

int log_msg(unsigned char level, const char *msg);
