

#pragma once

#include <stdarg.h>
#include <stdbool.h>
#include <stdint.h>
#include <stdlib.h>
#include "../../app-base/include/libapp_base.h"


void *app_boot(int argc, const char *const *argv);

void app_finish(void *app);

void app_run(void *app);
