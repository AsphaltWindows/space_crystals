# Feature Update: Unit Commands - Enter Command & EnteringTunnel Behavior

**Feature file**: `features/unit_commands_and_behaviors.md`
**Design sources**: `design/control_system.md`

## Modifications

### Enter Command Added (9th command)
- New command: Enter — unit walks to Tunnel Side A and enters the Tunnel Network
- Sets BaseCommandState: CommandType=Enter, TargetLocation=None, TargetObject=Tunnel (ObjectInstance)
- Availability: Syndicate units only, when target Tunnel's tier is sufficient for unit's base category

### EnteringTunnel Behavior Added (10th behavior)
- Moves to target Tunnel's Side A via MovingToObject sub-behavior
- On arrival, unit is removed from the map and added to the Tunnel Network unit pool
- No detailed behavior algorithm in design source — intent is clear from command description
