#include <stdio.h>
#include <sys/syslog.h>
#include <syslog.h>
#include <stdarg.h>
#include <errno.h>
#include <string.h>

void pk_log(int priority, const char* message, ...) {
  va_list args;
  va_start(args, message);

  #ifdef DEBUG
  (void) priority;
  vprintf(message, args);
  printf("\n");
  #else
  vsyslog(priority, message, args);
  #endif
  
  va_end(args);
}

void pk_log_error(const char* message, ...) {
  va_list args;
  va_start(args, message);

  char buffer[256];
  vsnprintf(buffer, sizeof(buffer), message, args);

  char* error_message = strerror(errno);
  
  #ifdef DEBUG
  vprintf(message, args);
  printf(": %s\n", error_message);
  #else
  syslog(LOG_ERR, "%s: %s", buffer, error_message);
  #endif

  va_end(args);
}


