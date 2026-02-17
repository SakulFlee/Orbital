package export_orbital_core_module

import (
	"github.com/bytecodealliance/wit-bindgen/wit_types"

	"wit_component/orbital_core_commands"
)

func Startup() wit_types.Option[orbital_core_commands.CommandBuffer] {
	println("'Hello World!' from TestModule: Go")

	return wit_types.None[orbital_core_commands.CommandBuffer]()
}
