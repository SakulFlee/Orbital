# Entity

- A entity is the smallest unit in a _world_.
- A _world_ can contain multiple entities
- Each entity has a descriptor which defines update behaviour, if it should render and gives the entity a Tag
- A tag is a string which can be used to find and remove (-> despawn) an entity
- A given entity can choose if it can be updated and if it can be rendered.
- If an entity chooses to be updated, it can choose between "Fast" and "Slow".
- "Fast" means, that the update function gets called by-cycle which in most cases is equal or more (>=) than the current FPS count. Or, in other words, it gets called for each UPS cycle
- "Slow" means, that the update function gets called roughly every second. Note, that lag can happen. Say we have lag for 5s, this function doesn't get called 5x times but _once_ with a delta time of 5s.
- A entity may at most contain everything about **itself**.
- Entity interop isn't supported (atm and maybe never?)

## One-Shot Entity

- is an entity that does something _once_ (i.e. gets rendered or updated _once_) and then deletes itself.
