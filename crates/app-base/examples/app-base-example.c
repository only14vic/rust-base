#include "../include/libapp_base.h"
#include <assert.h>
#include <malloc.h>
#include <stdbool.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

int main() {
  dotenv(false);
  log_init();

  log_msg(0, "This is nothing");
  log_msg(1, "This is ERROR");
  log_msg(2, "This is WARNING");
  log_msg(3, "This is INFO");
  log_msg(4, "This is DEBUG");
  log_msg(5, "This is TRACE");
  log_msg(6, "This is nothing again");
}
