//! The GDO's own panels: the Deployment Center's, with its build menus, the Extraction Facility's, and the Field
//! Engineer's, with its Construct menu (slice M5-N9); (M6-GD) the Drone Operator's, with its R and F
//! ([`drone_operator`]), and its drone's ([`operator_drone`]); (M6-GR) the Radar Station's, which scans, and the
//! Defense System's, which builds Missile Turrets as the Extraction Facility its plate; (M6-GH1) the Airfield's, which
//! trains the Hornet ([`airfield`]). The Peacekeeper's is the BasicCombatUnit template, and the Barracks' the
//! Production template (`super`), its Field Engineer in E and its Drone Operator in R ([`barracks`]); (M6-GF) the
//! Factory's too, its Prospector and Artillery Rig held in E and R ([`factory`]).

pub mod airfield;
pub mod barracks;
pub mod defense_system;
pub mod deployment_center;
pub mod drone_operator;
pub mod extraction_facility;
pub mod factory;
pub mod field_engineer;
pub mod operator_drone;
pub mod radar_station;
