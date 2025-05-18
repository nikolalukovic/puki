#include <stdio.h>
#include <sys/syslog.h>
#include <stdarg.h>
#include <errno.h>
#include <string.h>

#define MAX_LOG_MESSAGE_LEN 2048
#define MAX_ERROR_MSG_LEN 256

static const char* priority_to_str(int priority) {
  switch (priority) {
    case LOG_EMERG: return "EMERG";
    case LOG_ALERT: return "ALERT";
    case LOG_CRIT: return "CRIT";
    case LOG_ERR: return "ERR";
    case LOG_WARNING: return "WARN";
    case LOG_NOTICE: return "NOTICE";
    case LOG_INFO: return "INFO";
    case LOG_DEBUG: return "DEBUG";
    default: return "UNKNOWN";
  }
}

void pk_log(int priority, const char* message, ...) {
  va_list args;
  va_start(args, message);

  const char* prio_str = priority_to_str(priority);

  #ifdef DEBUG
  printf("%s:", prio_str); 
  vprintf(message, args);
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


