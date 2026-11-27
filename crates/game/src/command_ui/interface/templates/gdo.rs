//! The GDO's own panels: the Deployment Center's, with its build menus, the Extraction Facility's, and the Field
//! Engineer's, with its Construct menu (slice M5-N9); (M6-GD) the Drone Operator's, with its R and F
//! ([`drone_operator`]), and its drone's ([`operator_drone`]). The Peacekeeper's is the BasicCombatUnit template, and the
//! Barracks' the Production template (`super`), its Field Engineer in E and its Drone Operator in R ([`barracks`]).

pub mod barracks;
pub mod deployment_center;
pub mod drone_operator;
pub mod extraction_facility;
pub mod field_engineer;
pub mod operator_drone;
