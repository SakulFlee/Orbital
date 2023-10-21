# Problems

This project is working as-is at time of writing this.  
However, there are a couple of issues I want to address before archiving this repository.  
You can find said issues below here.

## Window related issues

Currently, there is no actual window configuration or similar that saves, or sets, window parameters such as:

- Size (Width & Height)
- Display
- Maximized
- Fullscreen

Additionally, window focus isn't correctly handled at the moment.

> In an older version there was `AppConfig` for this!

## Device and API selection

Currently, the best GPU + Rendering API combination is automatically choosen.  
In some sceanarios, users may choose to set a differnet combination.

This is currently not possible.

> In an older version there was `AppConfig` for this!

## Input

Controller inputs aren't handled at all and Keyboard + Mouse handlers _may_ have jitter.  
This would require more work to figure out ...

## Post-Processing

There is currently no Antialiasing.  
MSAA is technically supported out-of-the-box by WGPU, but it's still quite challenging to implement in the end.
Especially since either we'd need a work around OR all textures must be MSAA-supportive.

## Lights

While, temporarily, there was support for multiple lights ... it broke pretty quickly and messed up a lot of things.
We ended up removing the lights again and making it a single point light + ambient light.
Which works! For simple scenes. If multiple lights are required one can render one half, then the other.
But it's far from efficient and effective.

On the same node, light currently is universal in the scene and can penetrate e.g. walls.
To prevent light from e.g. a torch on a wall to go through the wall and light up the other side, one would have to render first the part that is lid up, then the dark part.

## Shaders

Shaders are as-is in WGSL format. Which is fine mostly, but complex shaders may benefit from e.g. being able to be split up.
Bevy, as an example, has a pre-processor that lets you include other shader files.
Something like this would be ideal to implement at some point.

## Other platforms

Technically, this should work on any platform and target.  
However, mobile (Android + iOS), as well as Web/WASM, support is untested and may or may not work.
Furthermore, touch inputs are not handled at all at this moment and files are expected to be locally available.
In case of Web this can't work and assets would have to be streamed in.

## World generation

World generationg "works" but is clunky and inefficient. 
More complex things may require much more work.

Settings for this are currently also not exposed to the App.

## Model formats

Currently, only glTF is supported.
Originally, we had OBJ support but that broke at some point.
glTF also isn't optimal since the parsin library occasionally fails to read materials correctly.

Indendent of the issues, OBJ may be a good addition as a legacy format.
Something like FBX may also be a good addition.

## "ECS"

A ECS (entity-component-system) is quite hard to implement in Rust as it turns out.
At least, without an external tool. The current system works, but is kinda ... inefficient and complicated to use.
Like, let alone the requirement that there must be at least an initial system to spawn everything else is weird.

Also, without properly keeping track of systems _by_ a system, information may simply get lost and/or overwritten.
There is no proper global system to keep track of everything and no "garbage collector".

