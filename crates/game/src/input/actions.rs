//! What the player can ask the client to do ([`InputAction`], with the map editor's own [`EditorInput`]), the input
//! contexts in which they work ([`InputContext`]), and the control groups and command panel slots that actions
//! name. Which keys and buttons trigger them is the bindings table's business ([`super::Bindings`]).

use bevy::input::keyboard::KeyCode;

/// Which bindings apply: the game's or the map editor's (`ARCHITECTURE.md > Client architecture > Input`). A game
/// and an editor play-test take the game's, the editor its own (`crate::ClientMode::input_context`). An action works
/// in its own context only ([`InputAction::context`]), so a key may mean one thing in the game and another in the
/// editor (V is a command slot in the game and the Select tool in the editor).
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum InputContext {
    #[default]
    Game,
    Editor,
}

impl InputContext {
    pub const ALL: [InputContext; 2] = [InputContext::Game, InputContext::Editor];

    /// Whether a held Ctrl keeps a key binding of `action` that doesn't need Ctrl from firing. In the editor, as in
    /// desktop editors, Ctrl + key is a command of its own, so Ctrl + B is nothing rather than the Paint tool; this
    /// holds for the editor's own actions, not the shared ones (Ctrl + G still shows the grid). In the game a
    /// modifier is a role (Shift queues), so a held modifier never blocks there.
    pub fn ctrl_blocks(self, action: InputAction) -> bool {
        self == InputContext::Editor && matches!(action, InputAction::Editor(_))
    }
}

/// Everything the player can ask the client to do with a key, a mouse button, the wheel or a modifier
/// (`ARCHITECTURE.md > Client architecture > Input`). Gameplay and editor code ask for these, never for raw keys. The
/// default bindings are [`super::Bindings::default`].
///
/// Most actions fire on a press. The modifier actions ([`InputAction::Queue`], [`InputAction::ToggleSelection`],
/// [`InputAction::SelectSameType`], [`InputAction::CenterOnObject`]) are only ever held: they say how a click or a
/// command is meant. A wheel action ([`EditorInput::Zoom`]) carries how far the wheel turned
/// ([`super::ActionState::scrolled`]).
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
    /// The primary pointer action: select by clicking (`systems/controls/selection.md > MapClickSelection`)
    /// or by dragging a box (`systems/controls/selection.md > BoxSelection`), or pick the target of a pending
    /// command (`systems/controls/interface.md > AwaitingTarget`). Its clicks and boxes arrive as
    /// [`super::MapClick`]s and [`super::SelectionBox`]es.
    Select,
    /// The secondary pointer action: the context-sensitive command (`systems/controls/interface.md >
    /// RightClickResolution`), or cancelling a pending one (`systems/controls/interface.md > AwaitingTarget`).
    /// Its clicks arrive as [`super::MapClick`]s.
    ContextCommand,
    /// `systems/controls/interface.md > AwaitingTarget`: Escape cancels a pending command.
    Cancel,
    /// `systems/controls/selection.md > GroupCycling > Forward`.
    NextGroup,
    /// `systems/controls/selection.md > GroupCycling > Backward`.
    PreviousGroup,
    /// `systems/controls/selection.md > ControlGroups > Operations > Assign`.
    AssignGroup(ControlGroup),
    /// `systems/controls/selection.md > ControlGroups > Operations > Add`.
    AddToGroup(ControlGroup),
    /// `systems/controls/selection.md > ControlGroups > Operations > Recall`.
    RecallGroup(ControlGroup),
    /// `systems/controls/selection.md > ControlGroups > Operations > Recall and Center`.
    RecallAndCenterGroup(ControlGroup),
    /// `systems/controls/interface.md > CommandPanel > Grid Layout`: the hotkey of a command panel slot.
    CommandSlot(CommandSlot),
    /// Modifier: `systems/controls/commands.md > Unit Command`: a command issued while it is held is appended to
    /// the queue instead of replacing it.
    Queue,
    /// Modifier of the selection's add inputs (design changes 0049, 0052 and 0053): a click toggles an own unit in
    /// the selection (`systems/controls/selection.md > MapClickSelection`), a box adds what it selects
    /// (`systems/controls/selection.md > BoxSelection > Shift + box (add)`), and with
    /// [`InputAction::SelectSameType`] a click adds all own units of that type on screen. On a portrait, it removes
    /// (`systems/controls/hud.md > SelectionPanel > Interactions`).
    ToggleSelection,
    /// Modifier: `systems/controls/selection.md > MapClickSelection`: a click selects all own units of that
    /// type on screen.
    SelectSameType,
    /// Modifier: `systems/controls/hud.md > SelectionPanel > Interactions`: a click on a portrait centers
    /// the camera on that unit instead of selecting it.
    CenterOnObject,
    /// Debug only (`ARCHITECTURE.md > Commands, events and replays`, `SimConfig::debug`): the local client acts as the
    /// next player, so one client can control both sides. Placeholder binding: the design has no debug keys.
    DebugSwitchPlayer,
    /// Debug only, like [`InputAction::DebugSwitchPlayer`]: turns the fog of war off and on (`crate::fog`), a view
    /// option for QA. Placeholder binding.
    DebugToggleFog,
    /// (Slice M5-X2) Debug only, in a game with a data directory (`--tuning`, `crate::data`): restarts the game as the
    /// playtest menu's Restart does, reads the directory again and gives the new game its rules in its first tick.
    /// Placeholder binding: F8, which nothing else takes.
    DebugRestartWithData,
    /// Replay playback (`--replay`, `crate::replay::playback`): pauses and resumes. Placeholder binding, like the
    /// other replay keys: the design has no replays yet. K since slice M5-MM2, whose alerts took Space
    /// ([`InputAction::JumpToAlert`]).
    ReplayPause,
    /// Replay playback: plays one tick and pauses.
    ReplayStep,
    /// Replay playback: the next slower speed (`crate::replay::playback::SPEEDS`).
    ReplaySlower,
    /// Replay playback: the next faster speed.
    ReplayFaster,
    /// (Slice M5-S7, design change 0413) `core/camera.md > Window Mode`: switches the window between borderless full
    /// screen and a window (`crate::platform`). The design's placeholder keys: F11, and Alt + Enter.
    ToggleFullscreen,
    /// (Slice M5-S7, design change 0414) `core/game_rules.md > Playtest menu (temporary)`: opens the playtest menu
    /// during a game, which pauses it, and closes it again (`crate::playtest_menu`). The design's key: F10.
    PlaytestMenu,
    /// (Slice M5-MM2, design change 0443) `systems/controls/hud.md > Minimap > Alerts`: "Jump to the latest alert:
    /// **Space** (user, 2026-11-26; default key): centres the camera on the **latest** alert's spot"
    /// (`crate::hud::minimap::alerts`); "Later notifications of other kinds (none yet) will use the same key". The
    /// game's, in a game, a play-test and a replay's playback alike, where the pings are the shown player's.
    JumpToAlert,
    /// The map editor's input (`ARCHITECTURE.md > Map editor`).
    Editor(EditorInput),
}

impl InputAction {
    /// The input context whose bindings fire the action, or `None` for both: the camera's pans and grab, the grid,
    /// Cancel (Esc cancels in the game and in the editor alike), the window's full-screen toggle, the play-test
    /// switch, which must work in the editor and in its play-tests, and the placement keys
    /// ([`InputAction::is_placement`]), which turn the structure being placed in the editor and in the game's
    /// AwaitingPlacement alike. Every other action belongs to the game or to the editor.
    pub fn context(self) -> Option<InputContext> {
        use InputAction as A;
        match self {
            A::PanLeft | A::PanRight | A::PanUp | A::PanDown | A::GrabPan | A::ToggleGrid | A::Cancel => None,
            A::ToggleFullscreen => None,
            A::Editor(EditorInput::PlayTest) => None,
            placement if placement.is_placement() => None,
            A::Editor(_) => Some(InputContext::Editor),
            // Selection, commands, control groups, the modifiers: the game's.
            _ => Some(InputContext::Game),
        }
    }

    /// Whether the action's bindings fire in `context`.
    pub fn works_in(self, context: InputContext) -> bool {
        self.context().is_none_or(|c| c == context)
    }

    /// (Slice M5-S7) Whether the action reaches the client's systems while a modal UI holds the input
    /// ([`super::Modal`]: the playtest menu, while game input is blocked): the window's full-screen toggle and the
    /// menu's own key, which work whatever holds the input, and the debug keys, which change only what the client
    /// shows (whose view, the fog), so that at the end of a hot-seat game each player still sees their own result under
    /// the menu (`crate::hud::outcome`). Every other action is the modal UI's alone
    /// ([`super::ActionState::modal_pressed`]).
    pub fn passes_modal(self) -> bool {
        use InputAction as A;
        matches!(
            self,
            A::ToggleFullscreen | A::PlaytestMenu | A::DebugSwitchPlayer | A::DebugToggleFog
        )
    }

    /// Whether the action turns the structure being placed. Its bindings take a press only while one is
    /// ([`super::Placing`]), and then before every other binding of that press, which they override: in the editor F
    /// flips while a structure is being placed and chooses the Fill tool otherwise; in the game R and F turn the
    /// ghost of the building being placed (`factions/gdo/structures/deployment_center.md >
    /// ObjectInterfaceState[DeploymentCenter] > AwaitingPlacement`) instead of pressing the command slots R and F,
    /// and Shift + R and Shift + F turn it the other way, the Shift that is otherwise the Queue modifier (plan
    /// decision T10, risk R11). One definition of the placement keys for both: these actions work in either context
    /// ([`InputAction::context`]).
    pub fn is_placement(self) -> bool {
        use EditorInput as E;
        matches!(
            self,
            InputAction::Editor(E::RotateClockwise | E::RotateCounterClockwise | E::FlipHorizontal | E::FlipVertical)
        )
    }
}

/// What the map editor can be asked to do with a key, a mouse button or the wheel (`ARCHITECTURE.md > Map editor`;
/// its F1 help lists the default bindings). The editor turns them into its `EditorAction`s
/// (`crate::editor::input`). Esc ([`InputAction::Cancel`]), the grid ([`InputAction::ToggleGrid`]) and the camera's
/// pans and middle-mouse grab are the game's actions, shared with the editor.
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum EditorInput {
    /// The editor's primary pointer action: uses the tool from the press on the map, through the drag, to the
    /// release. The pointer tracks the press ([`super::Pointer::press`]).
    UseTool,
    /// The eyedropper: picks the preset and the height of the tile clicked. Its clicks arrive as
    /// [`super::MapClick`]s, sent when pressed.
    Eyedropper,
    /// Zooms at the cursor, by how far the wheel turned.
    Zoom,
    SelectTool,
    PaintTool,
    HeightTool,
    FillTool,
    PlaceTool,
    StartTool,
    CameraTool,
    /// The Starting Layout tool (`core/map_editor.md > Starting Layouts`).
    LayoutTool,
    SmallerBrush,
    LargerBrush,
    /// Switches the height tool to its next mode (Set, Raise, Lower).
    NextHeightMode,
    /// Raises the level that the height tool sets.
    RaiseLevel,
    /// Lowers the level that the height tool sets.
    LowerLevel,
    /// Shows or hides the slope tint.
    ToggleSlopes,
    /// Zooms out to the whole map.
    FitMap,
    DeleteSelection,
    Undo,
    Redo,
    Save,
    NewMap,
    /// Starts a play-test of the map, or ends the running one. The one editor action that works in play-tests too,
    /// where Esc is the game's Cancel.
    PlayTest,
    ToggleHelp,
    /// While a structure is being placed: turns it a quarter turn clockwise. The editor places with the keys of the
    /// game's placement ghost (`factions/gdo/structures/deployment_center.md > ObjectInterfaceState[DeploymentCenter]
    /// > AwaitingPlacement`: "R rotates the ghost 90 degrees clockwise, Shift+R rotates 90 degrees counter-clockwise
    /// ... F flips the ghost horizontally, Shift+F flips vertically"), as `core/map_editor.md` gives none. These four
    /// are placement actions ([`InputAction::is_placement`]), shared with the game, whose AwaitingPlacement turns
    /// its ghost with them (`crate::command_ui`).
    RotateClockwise,
    /// While a structure is being placed: turns it a quarter turn counter-clockwise.
    RotateCounterClockwise,
    /// While a structure is being placed: mirrors it left and right, as seen.
    FlipHorizontal,
    /// While a structure is being placed: mirrors it top and bottom, as seen.
    FlipVertical,
}

/// One of the 10 control groups (`systems/controls/selection.md > ControlGroups`), numbered 0–9 like their
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
    /// `systems/controls/selection.md > ControlGroups`: a set of 10 saved selections.
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

    /// The digit key of the group's number, which all its default bindings use
    /// (`systems/controls/selection.md > ControlGroups > Default keys`).
    pub fn digit_key(self) -> KeyCode {
        DIGIT_KEYS[usize::from(self.0)]
    }
}

/// A slot of the command panel's 4 × 3 grid (`systems/controls/interface.md > CommandPanel > Grid Layout`),
/// by row (0 is the top row) and column (0 is the left column).
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct CommandSlot {
    row: u8,
    column: u8,
}

/// `systems/controls/interface.md > CommandPanel > Grid Layout`: each slot's default hotkey.
const SLOT_HOTKEYS: [[KeyCode; CommandSlot::COLUMNS as usize]; CommandSlot::ROWS as usize] = [
    [KeyCode::KeyQ, KeyCode::KeyW, KeyCode::KeyE, KeyCode::KeyR],
    [KeyCode::KeyA, KeyCode::KeyS, KeyCode::KeyD, KeyCode::KeyF],
    [KeyCode::KeyZ, KeyCode::KeyX, KeyCode::KeyC, KeyCode::KeyV],
];

impl CommandSlot {
    pub const COLUMNS: u8 = 4;
    pub const ROWS: u8 = 3;
    /// `systems/controls/interface.md > Standard Slot Assignments`: Z, the bottom row's first slot: Back /
    /// Cancel menu.
    pub const BACK: CommandSlot = CommandSlot { row: 2, column: 0 };
    /// `systems/controls/interface.md > Standard Slot Assignments`: X, the bottom row's second slot: Cancel
    /// Production / Cancel Upgrade.
    pub const CANCEL_PRODUCTION: CommandSlot = CommandSlot { row: 2, column: 1 };
    /// `systems/controls/interface.md > Standard Slot Assignments`: C, the bottom row's third slot: Set Rally
    /// Point. V, the fourth, has no standard function.
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

    /// The slot's default hotkey (`systems/controls/interface.md > Grid Layout`).
    pub fn default_key(self) -> KeyCode {
        SLOT_HOTKEYS[usize::from(self.row)][usize::from(self.column)]
    }
}
