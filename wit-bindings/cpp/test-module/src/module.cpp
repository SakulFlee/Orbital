#include "../../bindings/orbital_cpp.h"
#include <iostream>

namespace exports {
namespace orbital {
namespace core {
namespace module {
std::optional<commands::CommandBuffer> Startup() {
  std::cout << "'Hello World!' from WIT TestModule: C++" << std::endl;

  // Return Rust's "None"
  return std::nullopt;

  /* To return Rust's "Some":
   * commands::CommandBuffer buffer;
   * commands::Command cmd;
   * cmd.variants = commands::Command::RegisterSystem { wit::string("MySystem")
   * }; buffer.commands.push_back(std::move(cmd)); return buffer;
   */
}

} // namespace module
} // namespace core
} // namespace orbital
} // namespace exports
