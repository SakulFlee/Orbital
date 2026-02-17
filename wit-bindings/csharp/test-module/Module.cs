using OrbitalWorld.wit.Exports.orbital.core.v0_1_0;

public partial class ModuleExportsImpl {
  public static ICommandsExports.CommandBuffer? Startup() {
    Console.WriteLine("'Hello World!' from WIT TestModule: C#");

    // // Create a sample command buffer with a register-system command
    // var commands = new
    // System.Collections.Generic.List<ICommandsExports.Command>();
    // commands.Add(ICommandsExports.Command.RegisterSystem("test-system"));
    //
    // var commandBuffer = new ICommandsExports.CommandBuffer(commands);
    // return commandBuffer;

    return null;
  }
}
