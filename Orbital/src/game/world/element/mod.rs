use hashbrown::HashMap;
use log::warn;
use ulid::Ulid;

use crate::{app::InputEvent, game::WorldChange, variant::Variant};

pub mod registration;
pub use registration::*;

/// An [Element] is a **thing** inside a [World].  
/// Whenever you need something in your world, be it static or updated,
/// you are looking for one or multiple [Elements]!
///
/// A single [Element] can associate itself with multiple [realized resources],
/// such as [Models].  
/// If an [Element] despawns, any associated [realized resources]
/// will be despawned as well.
///
/// After registration, an [Element] can, optionally, return a
/// [Vec<WorldChange>] in most functions.
/// This is simply a list of proposed [WorldChanges].  
/// **This is how [Elements] interact with the [World]**.
///
/// # Messaging
///
/// [Elements] follow a strict _communicate by sharing information_ approach.
/// You cannot, and should not, _communicate by sharing memory_!
///
/// This means you are supposed to send messages between [Elements]
/// to communicate.
/// You do not gain access to another [Element] or it's properties.
/// Also, generally, it's best to not store information about other [Elements].
/// There are exceptions of course, but generally it is advised to purely react
/// to messages with a given context and only update local variables
/// if necessary.
///
/// Here is an example to make this more clear:  
/// Say you have a game with a player and an enemy.  
/// The goal is to kill the enemy.  
/// You loose if the enemy kills you, the player.  
/// Neither the player, nor the enemy, have memory of each other.  
/// They cannot read each others e.g. position or health.  
/// So, how do we implement this in a working way?
///
/// First, both the player and enemy need to be _tagged_.  
/// For simplicity we will simply _tag_ the player as `player` and
/// the enemy as `enemy`.
/// This is necessary for messaging to work.
/// Otherwise, we would need the [Ulid] of both and somehow interchange
/// this information.
///
/// Next, whenever the `player` or `enemy` does an attack (e.g. on button press
/// or timer), we send a message to the other [Element].
/// This message must contain vital information, such as:
/// - Current position the attack originates from
/// - (Optional) Strength of the attack
/// - (Optional) Direction we are facing
/// - ... and possibly more :)
/// It may also be useful to include something like an `action` to distinguish
/// between messages.
///
/// The other [Element] will receive said message and react to it.
/// Said receiver [Element] will do the **hit detection**.
/// Meaning, we know now where the attack came from and optionally
/// other parameters.
/// With this information we can now compare against our own position and
/// check if we would have been in range of an attack.  
/// If not, nothing happens.  
/// If yes, we can decrease our own health, and possibly die.  
/// We may also send a message back to the attacking [Element] that we got hit.
/// This could be used to e.g. play an animation of effect.  
/// We may also inform a separate UI [Element] that the health value changed.
///
/// If the local [Element] actually died, we can send another message out to an
/// e.g. score board or controlling [Element], which then can switch to
/// displaying the results.
///
/// Like you just saw, we simply _share information about an event_ and _handle
/// it locally_.
/// We don't need to store anything about another [Element], nor do we need
/// direct memory access to it!
///
/// A flow diagram would look like this:
///
/// ![Messaging Flow Diagram](https://raw.githubusercontent.com/SakulFlee/Akimo-Project/62619c14a49b87f6cea32d2c3caee7cfab59052b/.github/images/messaging_flow_example.drawio.svg)
///
/// Furthermore, this approach is easily scaleable.  
/// _Tags_ can be used multiple times!  
/// So for example we could have three enemies.  
/// All can be _tagged_ `enemy`.  
/// Upon the player doing an attack, the message will be automatically send to
/// **all** [Elements] _tagged_ `enemy`.
///
/// A full diagram would look like this:
///
/// ![Messaging Flow Diagram Multiple](https://raw.githubusercontent.com/SakulFlee/Akimo-Project/62619c14a49b87f6cea32d2c3caee7cfab59052b/.github/images/messaging_flow_example_multiple.drawio.svg)
///
/// [World]: super::World
/// [Elements]: Element
/// [realized resource]: crate::resources::realizations
/// [Models]: crate::resources::realizations::Model
/// [WorldChanges]: super::WorldChange
pub trait Element {
    fn on_registration(&mut self, _ulid: &Ulid) -> ElementRegistration {
        ElementRegistration::default()
    }

    fn on_focus_change(&mut self, _focused: bool) {}

    fn on_input_event(&mut self, _delta_time: f64, _input_event: &InputEvent) {}

    fn on_update(&mut self, _delta_time: f64) -> Option<Vec<WorldChange>> {
        None
    }

    fn on_message(&mut self, message: HashMap<String, Variant>) -> Option<Vec<WorldChange>> {
        warn!("Unhandled message received: {:#?}", message);

        None
    }
}
