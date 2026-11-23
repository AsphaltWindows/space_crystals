# Unit Behavior Systems

Split from the monolithic `behaviors.rs`. Each submodule handles a specific behavior domain.

## Files

- **mod.rs** -- Re-exports all public systems for external use
- **movement.rs** -- MovingToLocation, MovingToObject, Reversing, Stopping, behavior completion
- **tunnel_interaction.rs** -- Enter tunnel/armory dispatch and behavior systems
- **construction.rs** -- Building structure/tunnel behaviors, Cults recruit construction entry
- **resource_gathering.rs** -- Gathering resources, dropping off resources, tunnel side position helpers
- **supply_chopper.rs** -- Supply chopper pickup, attach, dropoff, detach, repair systems
