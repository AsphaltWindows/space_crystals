# Space Crystals RTS

## Design

### Map Features

#### Tile Properties

* Buildable  
  * Buildings can be built or placed on this tile.  
* Traversible  
  * All ground units can traverse this tile  
* Drillable  
  * Underground units can traverse this tile  
* Rugged  
  * Units incapable of traversing rugged terrain may not traverse this tile  
* Elevation  
  * Ground units shooting from higher to lower elevation gain additional range  
* Recruitable  
  * Tile counts towards a controlled recruitable area

#### Default Tile Types

* Plane  
  * Buildable  
  * Traversible  
  * Drillable  
* Rugged Terrain  
  * Not Buildable  
  * Traversible  
  * Rugged  
  * Drillable  
* Cliff  
  * Not Buildable  
  * Not Traversible  
  * Drillable  
* Mountain  
  * Not Buildable  
  * Not Traversible  
  * Not Drillable  
* Water  
  * Not Buildable  
  * Not Traversible  
  * Not Drillable  
  * Not Recruitable

#### Resources

* Space Crystals Patch (SCP)  
  * Not Buildable with an exception (Extraction Plate)  
  * Not Traversible  
  * Mineable via Space Crystal collection mechanics of the Factions  
  * Selectable  
    * When in vision displays the remaining amount of Space Crystals in the patch  
  * When the amount of Space Crystals in the SCP reaches 0 the SCP disappears.  
* Supply Delivery Station (SDS)  
  * Not buildable  
  * Not Traversible with an exception (Supply Chopper)  
  * Receives Supplies based on the following properties  
    * Delivery size  
      * The amount of supplies which will land on the SDS  
    * Delivery interval  
      * The amount of time that it takes from the Supplies to land on the SDS  
      * The countdown begins only once the SDS current Supply content is 0  
  * Selectable  
    * When out of vision displays  
      * Delivery Size  
      * Delivery Interval  
    * When in vision displays  
      * Remaining amount of Supplies on the station

### Units

* All Units have a Unit Base  
  * Unit Bases determine the unit behaviors  
* Some units can attack.  
  * Units that can attack with some Unit Bases have a Unit Turret attack from that turret  
  * Units that can attack with Unit Bases that do not have a Unit Turret attack from the Unit Base  
* All unit Attacks have an Attack Type

#### Unit Bases

* Light Infantry  
  * Ground unit  
  * Turns in place or while moving or firing  
  * Can Traverse Rugged Terrain  
  * Receives a damage reduction bonus on Rugged Terrain  
  * Does not have a turret  
  * Base Unit Properties  
    * Turn Rate  
    * Acceleration  
    * Deceleration  
    * Maximum Speed  
    * Rugged Terrain Maximum Speed  
  * Common Property Values  
    * Very Small Unit Size  
    * Very high turn rate  
    * Very high acceleration  
    * Very high deceleration  
    * Moderate Maximum Speed  
    * Low Rugged Terrain Maximum Speed  
* Heavy Infantry  
  * Ground unit  
  * Turns in place or while moving or firing  
  * Can Traverse Rugged Terrain  
  * Does not have a turret  
  * Base Unit Properties  
    * Turn Rate  
    * Acceleration  
    * Deceleration  
    * Maximum Speed  
    * Rugged Terrain Maximum Speed  
  * Common Property Values  
    * Small Unit Size  
    * Very high turn rate  
    * Very high acceleration  
    * Very high deceleration  
    * Moderate Maximum Speed  
    * Low Rugged Terrain Maximum Speed  
* Wheeled Vehicle  
  * Ground Unit  
  * Can not turn in place  
  * Can reverse  
  * Can not Traverse Rugged Terrain  
  * Has directional armor  
    * Damage Reduction bonus when attacked from the front  
    * Damage Increase penalty when attacked from the rear  
  * Has a turret  
  * Base Unit Properties  
    * Minimum Turn Radius  
    * Forward Acceleration  
    * Forward Maximum Speed  
    * Reverse Acceleration  
    * Deceleration  
    * Reverse Maximum Speed  
  * Common Property Values  
    * Small to Large Unit Size  
    * High to Very High Forward Maximum Speed  
    * Moderate Acceleration and Deceleration  
    * Lower Reverse Maximum Speed than Forward Maximum Speed  
* Tracked Vehicle  
  * Ground Unit  
  * Can turn in places and while moving  
  * Can reverse  
  * Can not Traverse Rugged Terrain  
  * Crushes enemy Light Infantry  
  * Has directional armor  
    * Damage Reduction bonus when attacked from the front  
    * Damage Increase penalty when attacked from the rear  
  * Has a turret  
  * Base Unit Properties  
    * Speed to Turn Radius ratio  
    * Acceleration  
    * Deceleration  
    * Maximum Speed  
  * Common Property Values  
    * Large to Very Large Unit Size  
    * Very Low to Low Maximum Speed  
    * Moderate Acceleration  
    * High Deceleration  
* Drill Unit  
  * Under-ground Unit  
  * Under-ground and above-ground modes. In above-ground mode Stationary or Tracked.  
  * In under-ground mode it can not fire.  
  * Has directional armor  
    * Damage Reduction bonus when attacked from the front  
    * Damage Increase penalty when attacked from the rear  
  * Invisible in under-ground mode.  
  * Can travel across all Drillable-tiles in under-ground mode.  
  * Base Unit Properties  
    * Speed to Turn Radius ratio  
    * Acceleration  
    * Deceleration  
    * Maximum Speed  
  * Common Property Values  
    * Medium to Large Unit Size  
    * Low to Medium Maximum Speed  
    * Moderate Acceleration  
    * High Deceleration  
* Hover Vehicle  
  * Ground Unit  
  * Can turn in place and while moving  
  * Can travel while facing any direction  
    * Has a maximum non-forward-only acceleration  
    * Has a higher maximum forward-only acceleration  
    * Has a Drag ratio used to calculate maximum speed based on Acceleration in currently-traveling direction.  
    * Turns in place while moving in the desired direction until it is facing in that direction and can continue moving at a higher speed.  
    * Uses its acceleration rate in the opposite direction and Drag Ratio to decelerate and change directions.  
  * Can not Traverse Rugged Terrain  
  * Has directional armor  
    * Damage Reduction bonus when attacked from the front  
    * Damage Increase penalty when attacked from the rear  
  * Has a turret  
  * Base Unit Properties  
    * Turn Rate  
    * Forward-only Acceleration  
    * Non-forward-only Acceleration  
    * Drag ratio  
  * Common Property Values  
    * Medium to Large Unit Size  
    * Moderate to High Effective Maximum Speed  
    * Moderate Forward-only Acceleration  
    * Low Non-forward-only Acceleration  
* Mech  
  * Ground Unit  
  * Can turn in place or while moving  
  * Can Traverse Rugged Terrain  
  * Crushes enemy Light Infantry  
  * Has directional armor  
    * Damage Reduction bonus when attacked from the front  
    * Damage Increase penalty when attacked from the rear  
  * Has a turret  
  * Base Unit Properties  
    * Turn Rate  
    * Maximum Speed  
    * Acceleration  
    * Deceleration  
  * Common Property Values  
    * Large to Very Large Unit Size  
    * Very Low to Low Maximum Speed  
    * Moderate Turn Rate  
    * Moderate to High Acceleration  
    * Very High Deceleration  
    * Very Narrow to Narrow Turret Turn Angle  
* Hover Craft  
  * Air Unit  
  * Can hover in the air without moving  
  * Can turn in place or while moving  
  * Only Accelerates forwards  
  * Has a turret  
  * Base Unit Properties  
    * Turn Rate  
    * Acceleration  
    * Drag Ratio  
  * Common Property Values  
    * Very Narrow to Narrow Turret Turn Angle  
    * Small to Very Large Unit Size  
    * Slow to Fast Turn Rate  
    * Very High Acceleration  
    * Very High Drag Ratio  
* Glider  
  * Air Unit  
  * Can not hover, must always keep moving  
  * Has a Turret  
  * Base Unit Properties  
    * Acceleration  
    * Maximum Speed  
    * Minimum Turn Radius  
  * Common Property Values  
    * High to Very High Maximum Speed  
    * Very Narrow to Narrow Turret Turn Angle

#### Unit Attacks

##### Attack Phases

* There are 4 phases to an attack sequence each with its own duration and execution phase which may be exclusionary to other unit behavior.  
  * Aiming  
    * During this portion of the attack if the unit is given a different command the attack sequence is interrupted.  
    * The target must remain valid throughout this phase or the attack sequence will be cancelled.  
    * During this phase the unit will attempt to engage in behavior that maintains its target’s validity.   
  * Firing  
    * Once the Firing phase begins it cannot be interrupted. The Attack’s effect either takes place or will become imminent with the completion of the attack phase.  
  * Cooldown  
    * This phase is usually a very short phase during which the unit is still unresponsive just as it is during the Firing phase, however it occurs after the Attack’s effect occurs or becomes imminent.  
  * Reloading  
    * This Phase serves as the main cooldown and source of delay between attacks. During this phase the unit may engage in other behavior unrelated to the attack sequence.  
    * During the reloading phase the unit may be performing other actions unrelated to the attack sequence.

##### Attack Types

* There are 4 types of attacks, resulting in different animations during the attack sequence. Each unit’s attack will be of one of these 4 types.  
* Fully Connected Attack  
  * The fully-connected attack is an attack in which the animation and the effect on the target of the attack’s Firing Phase is a single animation.  
  * Neither the animation nor the effect of the attack can “miss” the target  
  * Units with fully-connected cannot be used to force-target the ground and can only target specific units.  
* Tail Disjointed Attack  
  * The Tail Disjointed attack is an attack in which the Firing Phase is short and spawns a projectile animation which is executed independently of the Attacking unit’s further actions. The Unit will likely enter the Reloading Phase, while the effect of the attack is delayed until the completion of the projectile animation.  
  * Tail Disjointed attacks can not miss.  
  * Units with Tail-Disjointed attacks cannot be used to force-target the ground and can only target specific units.  
* Head Disjointed Attack  
  * The Head Disjointed attack is an attack in which both the Animation and the Effect of the attack are applied to a target location which is the location of the attack’s target at the end of the Aiming phase. The effect of the attack will be applied to whatever units are in the original target’s location at the end of the Firing phase.  
  * Units with Head Disjointed attacks may intentionally target specific locations.  
* Doubly Disjointed Attack  
  * Combination of Tail and Head disjointed.  
  * After a short firing phase which spawns a new animation and defers the Effect of the attack which will be applied onto the location the target was at at the end of Aiming phase. The Attacking unit enters the Cooldown Phase before the Attack Effect takes place

##### Attack Sources

* There are two different attack sources for units that can attack.  
  * Unit Base  
    * Unit Bases that do not have a turret attack from the Unit Base  
    * The Unit may perform the following actions while being in the various Attack Phases  
      * Aiming Phase  
        * Turning  
      * Firing Phase  
        * Nothing  
      * Cooldown Phase  
        * Nothing  
      * Reloading Phase  
        * Turning  
        * Moving  
  * Unit Turret  
    * Unit Bases that have a turret attack from the turret  
    * A Unit Turret has two stats specific to the Turret.  
      * Turn Angle  
        * Magnitude of the angle which Unit Turret can turn independently of the direction the Unit Base is facing. Maximum being 360 degrees or 2pi radians, allowing for a full turn.  
        * If the Angle is less than 360 degrees or 2pi radians, then the Unit Turret is considered centered when facing in the same direction as the vehicle’s primary movement direction with the available turn angle being split equally in the clockwise and counter-clockwise direction away from the centered position.  
      * Turn Rate  
        * The Turn Rate of the Unit Turret has a Turn Rate independent of the Turn Rate of the unit itself.  
    * A Unit Turret can in some ways behave independently from the Unit Base it belongs to.  
    * A Unit Turret may perform the following actions while being in the various Attack Phases  
      * Aiming Phase  
        * Turning  
      * Firing Phase  
        * Nothing  
      * Cooldown Phase  
        * Nothing  
      * Reloading Phase  
        * Turning

#### Generic Unit Commands

* Unit Commands are available in the Unit Hud that is usable with mouse or hotkeys. In addition there is a Default Command which is given with the Right Click mouse button.  
* Default Command  
  * Right Click mouse button  
* Move Command  
* Attack Command  
* Attack Ground Command  
* Patrol Command  
* Hold Position Command  
* Stop Command  
* Use Ability Commands  
* More 

#### Unit Actions

* Attacking target  
  * Unit must be in range and position from which it can Attack the target  
  * Unit performs the Attack sequence as long as it is possible from the current position  
  * When it is no longer possible for the Unit to attack the target from its current position, the unit is no longer Attacking target  
* Stopping  
  * If the Unit is attacking target it is no longer Attacking target.  
  * Unit begins to decelerate until it is stationary.  
  * Once the unit is stationary and not Attacking a target it is no longer stopping.  
* Moving To Target  
  * Unit is moving to target  
  * Once the unit reaches target it begins the Stopping action.  
  * If the unit repeatedly fails to make progress in approaching the target it begins the Stopping action.

#### Unit Behaviors

* Moving to Target Location  
  * Action Sequence  
    * Moving To Target  
* Attacking Target  
  * Action Sequence  
    * Moving to Target until target is in Attack Range  
    * Stopping  
    * Attacking target  
* Attacking Target Location  
  * Action Sequence  
    * Moving To Target until target location is in Attack Range  
    * Stopping  
    * Attacking Target Location   
* Attack Move to Target Location  
  * Action Sequence  
    * Until Unit is at Target Location Loop:  
      * Moving To Target Location until an enemy is in Attack Range  
      * Attacking Target until Target is destroyed  
    * Stopping

#### Unit State

* Busy   
  * A Unit is Busy if it has an active Behavior  
  * When the Unit is busy it will continue executing its current action  
* Idle  
  * A Unit is Idle if it does not have an active Behavior  
  * If an enemy unit enters the Idle Unit attack range it will Attack Target this enemy unit  
  * If an enemy unit attacks the Idle Unit it will Attack target this enemy unit  
* Holding Position  
  * A Unit which is Holding Position will not perform any Moving To Target Actions

#### Unit Commands

* Move  
* Attack  
* Attack Ground  
* Patrol  
* Hold Position

### Factions

* Global Defense Ordinance  
* The Syndicate  
* The Cults  
* Colonists

#### Global Defense Ordinance

* Ordinance makes use of three resources:  
  * Space Crystals  
    * Core resource  
  * Supplies  
    * Tactical/Tech resource  
  * Power  
    * Building Maintenance resource  
* Most structures built by the Ordinance are built by the Deployment Center.  
  * The Deployment Center has a build area around it where it can place buildings.  
  * Placed buildings extend the build area further.  
  * To build a building  
    * Select the Deployment Center  
    * Select a building for which the tech requirements are met and resources are sufficient  
    * The resources will be deducted and the building construction will be in progress  
    * A Constructing building may be cancelled receiving a full refund.  
    * Once the construction is complete select the Deployment Center  
    * Select the ready-to-place building  
    * Place the building within the build radius.  
    * The ready-to-place building may instead be cancelled, receiving a majority refund.  
    * A Deployment Center may construct only one building at a time.  
    * A Deployment Center with a ready-to-place building may not begin construction of another building until the building is placed or cancelled.  
    * A Deployment Center may have only one ready-to-place building at a time.  
* Ordinance maintains a global power total.  
  * Buildings may either grant or consume Power.  
  * If the total power is negative, then all buildings requiring power operate slower in proportion of total available power to the total required power power.  
* Ordinance gathers Space Crystals using the Extraction Facility and Extraction Plates.  
  * Extraction Facility is a building which can be constructed by the Deployment Center.  
  * Extraction Facility has a build area around it, within which it is able to place Extraction Plates onto Space Crystal patches.  
  * Extraction Facility constructs Extraction Plates using the same mechanics as the Deployment Center constructs buildings.  
  * Unlike how buildings do for the Deployment Center, Extraction Plates do not extend the Extraction Facility’s build area.  
  * Each Extraction Plate provides income by mining the Space Crystal patch.  
  * Selecting the Extraction Plate displays the amount of Space Crystals left in the Space Crystal patch.  
  * If a Space Crystal patch is exhausted entirely, the Extraction Plate continues to provide a small percentage of its normal Space Crystal income indefinitely.  
  * If an Extraction Plate is destroyed, the Space Crystal patch it covers becomes uncovered and accessible.  
  * If an Extraction Plate is destroyed which is built on a Space Crystal patch which is now exhausted, no Space Crystal patch becomes uncovered, and a new Extraction Plate can not be placed on the location.  
* Ordinance Gathers Supplies with the help of a Supply Tower and Supply Chopper.  
  * Supply Chopper  
    * A Supply Chopper is a hovercraft flying unit.  
    * Supply Chopper may contain one of the following:  
      * Up to 4 infantry units.  
      * A single vehicle  
      * Supplies  
    * To load units a Supply Chopper must land onto the ground.  
    * To drop off Supplies a Supply Chopper must land onto the Supply Tower.  
    * A Supply Chopper may be directed to pick up supplies from a Supply Delivery Station.  
      * A Supply Chopper will land on the Supply Delivery Station and pick up all available supplies.  
    * A Supply Chopper may be directed to attach to a Supply Tower.  
      * The Supply Chopper must not be carrying any units to be able to be directed to do so.  
      * A Supply Chopper may only attach to a Supply Tower to which no other Supply Chopper is currently attached.  
      * Any command given to an Attached Supply Chopper will break its attachment to the Supply Tower.  
  * Supply Tower  
    * A Supply Tower is a building that can be constructed by the Deployment Center.  
    * When a Supply Tower is placed it comes with a landed Supply Chopper.  
    * A Supply Chopper that is landed onto a Supply Tower will drop off all of the Supplies it carries and will be gradually repaired at no cost.  
    * Supply Tower can build Supply Choppers.  
    * When a Supply Tower is attached to by a Supply Chopper, it will immediately evict any existing landed Supply Chopper if it is not its attached Supply Chopper.  
    * When a Supply Tower has a Supply Chopper attached to it, no other Supply Chopper may land on it.  
    * A Supply Tower with an attached Supply Chopper may Schedule Deliveries from a Supply Delivery Station.  
      * The Supply Chopper attached to a Supply Tower that has Scheduled Deliveries from a Supply Delivery Station will depart from the Supply Tower at a time such that it will arrive simultaneously with the next expected Supply Delivery at that Station.  
        * If the travel distance from the Supply Tower to the Supply Delivery Station is too long or the Supply Deliveries at that Station are too frequent then the Supply Chopper will depart immediately after arriving at the Supply Tower and dropping off the Supplies.  
        * If the player has multiple Supply Towers with attached Supply Choppers with Scheduled Deliveries from a single Supply Delivery Station, then only one Chopper will ever be on the way to the Supply Delivery Station at a time.

#### The Syndicate

* The Syndicate makes use of three resources:  
  * Space Crystals  
    * Core resource  
  * Supplies  
    * Resource necessary for expanding and tech’ing  
  * Tunnel Space  
    * Sets the limit for The Syndicate army size  
* The primary Syndicate building the Tunnel and defensive structures are built by their worker unit: Agent  
* Other Syndicate Buildings are built in the Tunnels  
* To construct a tunnel or a defensive structure  
  * Select an Agent  
  * Select the Construct option  
  * Select the desired building  
  * Left Click on the ground to place it  
  * The Agent will walk to the target area and begin Constructing the building  
  * Tunnels have a Tunnel area surrounding them.  
  * A Tunnel cannot be built in a location such that its Tunnel area overlaps with another Tunnel’s Tunnel area.  
  * Tunnels may be upgraded.  
  * Construction and Upgrade of Tunnels costs Supplies.  
  * The cost of each additional Tunnel is proportional to the number of Tunnels the player already has  
  * The cost of upgrading a Tunnel to the next level is proportional to the number of Tunnels already at that level.  
  * Tunnels allow for transport of units between them by having units enter the Tunnel Network.  
  * The Units in a Tunnel Network may enter any Tunnel that the player owns as long as the level of the Tunnel is sufficiently high to support the passage of those units.  
* To construct a Tunnel Building  
  * Select a Tunnel  
  * You will see the modest build radius of the Tunnel area  
  * Select the Expand option  
  * Select the desired building  
  * Left Click on the ground within the Tunnel area.  
  * The building must be fully within the Tunnel area.  
  * The Building will begin construction  
  * A Tunnel can only build a single building at a time.  
  * Tunnel Buildings are underground and can be walked over by units. Tunnel Buildings are invisible to the enemy unless they have a detector.  
  * All Syndicate Units are built by Tunnel buildings. When a Unit is built it either emerges from the Tunnel or remains in the Tunnel network depending on the Building’s rally point.  
* Resource Acquisition  
  * Space Crystals are collected by Agents. Agents collect Space Crystals quickly and in large batches.  
    * Only a single Agent may drop off Space Crystals at a Tunnel at a time.  
    * Maximum Space Crystal throughput for a tunnel that is located close to the Space Crystal patches may be achieved with 3 Agents, with the third Agent being less efficient than the first two. (At any given time, 1 is dropping off, 1 is mining, 1 is either mining, walking or waiting in line to begin dropping off)  
  * Supplies  
    * Supplies are collected by Agents from Supply Delivery Stations.  
    * A single Agent may only carry 1 unit of Supplies at a time.  
    * Only a single Agent may drop off Space Crystals at a Tunnel at a time.  
    * The Syndicate player is not shown information about the time of the next Supply delivery at the Supply Delivery Station unless the Supply Delivery Station was in their line of sight when it was last emptied.  
  * Tunnel Space  
    * Each Tunnel provides Tunnel Space depending on the level of the Tunnel.  
    * Each Syndicate Unit has a certain Tunnel Space requirement. The Player can not build units as for their total Tunnel Space requirements to exceed the total Tunnel Space provided by the Tunnels.

#### The Cults

* The Cults make use of two resources:  
  * Space Crystals  
    * Core resource  
  * Recruit  
    * Core worker unit which is trained into all Cult units  
* The Cults buildings are built by their primary worker unit: Recruit  
* A recruit that is used to build a building builds it from within and does not return even when the building is completed.  
* If the building is cancelled, the recruit is returned  
* Multiple recruits can be assigned to build the same building. The building speed is proportional to the number of recruits. None of the recruits assigned to build the building are returned.  
* If multiple Recruits are building a building they can not be selectively ordered to exit, only the entire building can be cancelled.  
* To construct a building  
  * Select one or multiple Recruits  
  * Select the Construct option  
  * Select the desired building  
  * Left Click on the ground to place it  
  * All selected Recruits will be committed to building this building.  
* To assign Recruit(s) to an in-progress building  
  * Select one or multiple Recruits  
  * Select the Assist Construction option  
  * Left Click on the in-progress building  
  * Once the selected Recruits reach the building they will enter it and increase the building speed.  
* Resource Acquisition  
  * Space Crystals  
    * Recruits collect Space Crystals.  
    * Recruits Drop the Space Crystals off at the Storage.  
    * Multiple Recruits can simultaneously drop off Space Crystals at a Storage.  
    * Multiple Recruits can simultaneously collect from the same Space Crystal patch.  
  * Recruits  
    * Recruits are automatically trained by Recruitment Center.  
    * Recruits do not cost any resources.  
    * A Recruitment Center has a Recruitment Radius.  
    * Recruitment Center will train Recruits at the rate proportional to the area of Recruitable Tiles from which it recruits within its radius.  
    * If the Recruitment Radius falls entirely onto Recruitable Tiles that are not covered by any already-existing Recruitment Center, the Recruitment Center will be operating at full capacity.  
    * Tiles that are not Recruitable do not count, Tiles which are Recruitable but which are within the Recruitment Radius of an already-existing Recruitment Center also do not count.  
    * A Recruitment Center has a Recruitment Limit of how many Recruits it can support at a time. The Recruitment Limit is local to each Recruitment Center

#### Colonists

* Colonists make use of five resources:  
  * Space Crystals  
    * Core resource  
  * Alloys  
    * Refined from Space Crystals  
    * More advanced material necessary for construction of buildings and vehicles  
  * Extracts  
    * Refined from Space Crystals  
    * Necessary for research and more advanced psychic abilities  
  * Ascension Credits  
    * Refined from Alloys and Extracts  
    * Necessary for advanced research and psionic weaponry  
  * Beacon Capacity  
    * Sets the limit for Colonists army size  
* Colonist primary worker unit: Prospector is able to construct Beacons  
* Colonist buildings are warped in from abroad onto established Beacons  
* Each Beacon provides a certain amount of Beacon Capacity  
* When a Beacon Warps In a building, it will continue to provide the previously provided Beacon Capacity in addition to its new functions  
* To build a beacon  
  * Select a Prospector  
  * Order it to construct a Beacon  
  * Click to place the Beacon  
  * The Beacon must be constructed outside a radius of any other nearby Beacons  
* To build a non-beacon building  
  * Select a beacon  
  * Select the Warp In option  
  * Select the desired building