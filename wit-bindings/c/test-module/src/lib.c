#include "../../bindings/orbital.h"
#include <stdio.h>

bool exports_orbital_core_module_startup(
    exports_orbital_core_module_command_buffer_t *ret) {
  printf("'Hello World!' from WIT TestModule: C\n");

  // "return false" means Option::None in C!
  // To actually return something, we have to fill the *ret pointer passed to
  // the function:
  //   exports_orbital_core_commands_command_t cmd;
  //   cmd.tag = EXPORTS_ORBITAL_CORE_COMMANDS_COMMAND_REGISTER_SYSTEM;
  //   orbital_string_dup(&cmd.val.register_system, "MySystem");
  //
  //   exports_orbital_core_commands_command_t *cmd_ptr =
  //   malloc(sizeof(exports_orbital_core_commands_command_t)); *cmd_ptr = cmd;
  //
  //   ret->commands.ptr = cmd_ptr;
  //   ret->commands.len = 1;
  //
  //   // Now "true" means we *do* return something!
  //   return true;
  return false;
}
