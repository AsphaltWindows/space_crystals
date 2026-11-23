//! What the player can ask the client to do ([`InputAction`]), and the control groups and command panel slots that
//! actions name. Which keys and buttons trigger them is the bindings table's business ([`super::Bindings`]).

use bevy::input::keyboard::KeyCode;

/// Everything the player can ask the client to do with a key, a mouse button or a modifier
/// (`ARCHITECTURE.md > Client architecture > Input`). Gameplay code asks for these, never for raw keys. The default
/// bindings are [`super::Bindings::default`].
///
/// Most actions fire on a press. The modifier actions ([`InputAction::Queue`], [`InputAction::ToggleSelection`],
/// [`InputAction::SelectSameType`]) are only ever held: they say how a click or a command is meant.
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum InputAction {
    /// `core/camera.md > Panning > Arrow keys and screen edge`: pans west while held.
    PanLeft,
    /// Pans east while held.
    PanRight,
    /// Pans towards the top of the screen (north) while held.
    PanUp,
    /// Pans towards the bottom of the screen (south) while held.
    PanDown,
    /// `core/camera.md > Panning > Middle-mouse grab`: while held, the map point grabbed stays under the cursor.
    GrabPan,
    /// Shows or hides the tile grid overlay (a development view).
    ToggleGrid,
    /// Shows or hides each unit's remaining path (a development view, [`crate::path_overlay`]).
    TogglePaths,
    /// The primary pointer action: select by clicking (`systems/controls/selection_and_hud.md > MapClickSelection`)
    /// or by dragging a box (`> BoxSelection`), or pick the target of a pending command (`> AwaitingTarget`). Its
    /// clicks and boxes arrive as [`super::MapClick`]s and [`super::SelectionBox`]es.
    Select,
    /// The secondary pointer action: the context-sensitive command (`> RightClickResolution`), or cancelling a
    /// pending one (`> AwaitingTarget`). Its clicks arrive as [`super::MapClick`]s.
    ContextCommand,
    /// `> AwaitingTarget`: Escape cancels a pending command.
    Cancel,
    /// `> GroupCycling > Forward`.
    NextGroup,
    /// `> GroupCycling > Backward`.
    PreviousGroup,
    /// `> ControlGroups > Operations > Assign`.
    AssignGroup(ControlGroup),
    /// `> ControlGroups > Operations > Add`.
    AddToGroup(ControlGroup),
    /// `> ControlGroups > Operations > Recall`.
    RecallGroup(ControlGroup),
    /// `> ControlGroups > Operations > Recall and Center`.
    RecallAndCenterGroup(ControlGroup),
    /// `> CommandPanel > Grid Layout`: the hotkey of a command panel slot.
    CommandSlot(CommandSlot),
    /// Modifier: `systems/controls/commands.md > Unit Command`: a command issued while it is held is appended to
    /// the queue instead of replacing it.
    Queue,
    /// Modifier: `> MapClickSelection`: a click toggles an own unit in the selection.
    ToggleSelection,
    /// Modifier: `> MapClickSelection`: a click selects all own units of that type on screen.
    SelectSameType,
}

/// One of the 10 control groups (`systems/controls/selection_and_hud.md > ControlGroups`), numbered 0–9 like their
/// default keys.
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ControlGroup(u8);

/// The digit keys 0–9, the control groups' default keys.
const DIGIT_KEYS: [KeyCode; ControlGroup::COUNT as usize] = [
    KeyCode::Digit0,
    KeyCode::Digit1,
    KeyCode::Digit2,
    KeyCode::Digit3,
    KeyCode::Digit4,
    KeyCode::Digit5,
    KeyCode::Digit6,
    KeyCode::Digit7,
    KeyCode::Digit8,
    KeyCode::Digit9,
];

impl ControlGroup {
    /// `> ControlGroups`: a set of 10 saved selections.
    pub const COUNT: u8 = 10;

    /// Group `index`, 0–9.
    pub fn new(index: u8) -> Option<Self> {
        (index < Self::COUNT).then_some(Self(index))
    }

    pub fn index(self) -> u8 {
        self.0
    }

    /// All groups, 0 to 9.
    pub fn all() -> impl Iterator<Item = ControlGroup> {
        (0..Self::COUNT).map(Self)
    }

    /// The digit key of the group's number, which all its default bindings use (`> ControlGroups > Default keys`).
    pub fn digit_key(self) -> KeyCode {
        DIGIT_KEYS[usize::from(self.0)]
    }
}

/// A slot of the command panel's 4 × 3 grid (`systems/controls/selection_and_hud.md > CommandPanel > Grid Layout`),
/// by row (0 is the top row) and column (0 is the left column).
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct CommandSlot {
    row: u8,
    column: u8,
}

/// `> CommandPanel > Grid Layout`: each slot's default hotkey.
const SLOT_HOTKEYS: [[KeyCode; CommandSlot::COLUMNS as usize]; CommandSlot::ROWS as usize] = [
    [KeyCode::KeyQ, KeyCode::KeyW, KeyCode::KeyE, KeyCode::KeyR],
    [KeyCode::KeyA, KeyCode::KeyS, KeyCode::KeyD, KeyCode::KeyF],
    [KeyCode::KeyZ, KeyCode::KeyX, KeyCode::KeyC, KeyCode::KeyV],
];

impl CommandSlot {
    pub const COLUMNS: u8 = 4;
    pub const ROWS: u8 = 3;
    /// `> Standard Slot Assignments`: Z, the bottom row's first slot: Back / Cancel menu.
    pub const BACK: CommandSlot = CommandSlot { row: 2, column: 0 };
    /// `> Standard Slot Assignments`: X, the bottom row's second slot: Cancel Production / Cancel Upgrade.
    pub const CANCEL_PRODUCTION: CommandSlot = CommandSlot { row: 2, column: 1 };
    /// `> Standard Slot Assignments`: C, the bottom row's third slot: Set Rally Point. V, the fourth, has no
    /// standard function.
    pub const RALLY_POINT: CommandSlot = CommandSlot { row: 2, column: 2 };

    pub fn new(row: u8, column: u8) -> Option<Self> {
        (row < Self::ROWS && column < Self::COLUMNS).then_some(Self { row, column })
    }

    pub fn row(self) -> u8 {
        self.row
    }

    pub fn column(self) -> u8 {
        self.column
    }

    /// All 12 slots, row by row from the top left.
    pub fn all() -> impl Iterator<Item = CommandSlot> {
        (0..Self::ROWS).flat_map(|row| (0..Self::COLUMNS).map(move |column| Self { row, column }))
    }

    /// The slot's default hotkey (`> Grid Layout`).
    pub fn default_key(self) -> KeyCode {
        SLOT_HOTKEYS[usize::from(self.row)][usize::from(self.column)]
    }
}
