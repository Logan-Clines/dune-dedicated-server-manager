use super::{BuildKind, Category, CommandSpec, FieldKind, FieldSpec, SelectOption};

const FIELD_PLAYER: FieldSpec = FieldSpec {
    key: "PlayerId",
    label: "Player",
    kind: FieldKind::String,
    required: Some(true),
    default: None,
    helper: Some("FLS player id, or \"*\" for all online"),
    options: None,
};

const FIELD_STORY_NODE: FieldSpec = FieldSpec {
    key: "StoryNodeFullName",
    label: "StoryNodeFullName",
    kind: FieldKind::String,
    required: Some(true),
    default: None,
    helper: Some("e.g. Journey1.Node3"),
    options: None,
};

const XP_CATEGORIES: &[SelectOption] = &[
    SelectOption { value: "Main", label: "Main" },
    SelectOption { value: "Combat", label: "Combat" },
    SelectOption { value: "Survival", label: "Survival" },
    SelectOption { value: "Crafting", label: "Crafting" },
    SelectOption { value: "Vehicle", label: "Vehicle" },
    SelectOption { value: "Trooper", label: "Trooper" },
    SelectOption { value: "Bene_Gesserit", label: "Bene_Gesserit" },
    SelectOption { value: "Mentat", label: "Mentat" },
    SelectOption { value: "Swordmaster", label: "Swordmaster" },
    SelectOption { value: "Planetologist", label: "Planetologist" },
];

const BROADCAST_TYPES: &[SelectOption] = &[
    SelectOption { value: "Generic", label: "Generic" },
    SelectOption { value: "ServerShutdown", label: "ServerShutdown" },
];

const ADD_ITEM_FIELDS: &[FieldSpec] = &[
    FIELD_PLAYER,
    FieldSpec { key: "ItemName", label: "ItemName", kind: FieldKind::String, required: Some(true), default: None, helper: Some("Internal FName, case-insensitive"), options: None },
    FieldSpec { key: "Quantity", label: "Quantity", kind: FieldKind::Int, required: None, default: Some(json_const_i(1)), helper: None, options: None },
    FieldSpec { key: "Durability", label: "Durability", kind: FieldKind::Float, required: None, default: Some(json_const_f(1.0)), helper: None, options: None },
];

const SERVICE_BROADCAST_FIELDS: &[FieldSpec] = &[
    FieldSpec { key: "BroadcastType", label: "BroadcastType", kind: FieldKind::Select, required: Some(true), default: Some(json_const_s("Generic")), helper: None, options: Some(BROADCAST_TYPES) },
    FieldSpec { key: "Title", label: "Title", kind: FieldKind::String, required: None, default: None, helper: Some("required for Generic"), options: None },
    FieldSpec { key: "Body", label: "Body", kind: FieldKind::Text, required: None, default: None, helper: Some("required for Generic"), options: None },
    FieldSpec { key: "BroadcastDuration", label: "Duration (s)", kind: FieldKind::Int, required: None, default: Some(json_const_i(30)), helper: None, options: None },
];

const ONLY_PLAYER: &[FieldSpec] = &[FIELD_PLAYER];

const WATER_FIELDS: &[FieldSpec] = &[
    FIELD_PLAYER,
    FieldSpec { key: "WaterAmount", label: "WaterAmount", kind: FieldKind::Int, required: None, default: Some(json_const_i(1_000_000)), helper: None, options: None },
];

const AWARD_XP_FIELDS: &[FieldSpec] = &[
    FIELD_PLAYER,
    FieldSpec { key: "Category", label: "Category", kind: FieldKind::Select, required: Some(true), default: Some(json_const_s("Main")), helper: Some("FName tag for the XP track"), options: Some(XP_CATEGORIES) },
    FieldSpec { key: "Experience", label: "Experience", kind: FieldKind::Int, required: Some(true), default: Some(json_const_i(1000)), helper: None, options: None },
];

const SKILL_MODULE_FIELDS: &[FieldSpec] = &[
    FIELD_PLAYER,
    FieldSpec { key: "Module", label: "Module", kind: FieldKind::String, required: Some(true), default: None, helper: Some("e.g. Swordmaster_T1"), options: None },
    FieldSpec { key: "Level", label: "Level", kind: FieldKind::Int, required: Some(true), default: Some(json_const_i(1)), helper: None, options: None },
];

const SKILL_POINTS_FIELDS: &[FieldSpec] = &[
    FIELD_PLAYER,
    FieldSpec { key: "SkillPoints", label: "SkillPoints", kind: FieldKind::Int, required: Some(true), default: Some(json_const_i(0)), helper: None, options: None },
];

const TELEPORT_FIELDS: &[FieldSpec] = &[
    FIELD_PLAYER,
    FieldSpec { key: "X", label: "X", kind: FieldKind::Float, required: Some(true), default: None, helper: None, options: None },
    FieldSpec { key: "Y", label: "Y", kind: FieldKind::Float, required: Some(true), default: None, helper: None, options: None },
    FieldSpec { key: "Z", label: "Z", kind: FieldKind::Float, required: Some(true), default: None, helper: None, options: None },
    FieldSpec { key: "Yaw", label: "Yaw", kind: FieldKind::Float, required: None, default: None, helper: None, options: None },
    FieldSpec { key: "CamPitch", label: "CamPitch", kind: FieldKind::Float, required: None, default: None, helper: None, options: None },
    FieldSpec { key: "CamYaw", label: "CamYaw", kind: FieldKind::Float, required: None, default: None, helper: None, options: None },
    FieldSpec { key: "CamRoll", label: "CamRoll", kind: FieldKind::Float, required: None, default: None, helper: None, options: None },
];

const SPAWN_VEHICLE_FIELDS: &[FieldSpec] = &[
    FIELD_PLAYER,
    FieldSpec { key: "ClassName", label: "ClassName", kind: FieldKind::String, required: Some(true), default: None, helper: Some("Vehicle actor class path"), options: None },
    FieldSpec { key: "X", label: "X", kind: FieldKind::Float, required: Some(true), default: None, helper: None, options: None },
    FieldSpec { key: "Y", label: "Y", kind: FieldKind::Float, required: Some(true), default: None, helper: None, options: None },
    FieldSpec { key: "Z", label: "Z", kind: FieldKind::Float, required: Some(true), default: None, helper: None, options: None },
    FieldSpec { key: "Rotation", label: "Rotation", kind: FieldKind::Float, required: None, default: None, helper: None, options: None },
    FieldSpec { key: "TemplateName", label: "TemplateName", kind: FieldKind::String, required: None, default: Some(json_const_s("Default")), helper: None, options: None },
    FieldSpec { key: "Persistent", label: "Persistent", kind: FieldKind::Float, required: None, default: Some(json_const_f(1.0)), helper: Some("0.0 = transient, 1.0 = persistent"), options: None },
    FieldSpec { key: "Faction", label: "Faction", kind: FieldKind::String, required: None, default: None, helper: Some("(blank = default)"), options: None },
];

const PLAYER_AND_STORY: &[FieldSpec] = &[FIELD_PLAYER, FIELD_STORY_NODE];

const SERVER_EXEC_FIELDS: &[FieldSpec] = &[
    FIELD_PLAYER,
    FieldSpec { key: "Exec", label: "Exec", kind: FieldKind::String, required: Some(true), default: None, helper: None, options: None },
];

const SCRIPT_NAME_FIELDS: &[FieldSpec] = &[
    FIELD_PLAYER,
    FieldSpec { key: "ScriptName", label: "ScriptName", kind: FieldKind::String, required: Some(true), default: None, helper: None, options: None },
];

pub static SPECS: &[CommandSpec] = &[
    CommandSpec { id: "AddItemToInventory", label: "Grant item", category: Category::Items, destructive: None, needs_player: true, allow_all_players: true, describe: "Adds an item to the targeted player(s) inventory.", fields: ADD_ITEM_FIELDS, build: BuildKind::Passthrough },
    CommandSpec { id: "ServiceBroadcast", label: "Broadcast", category: Category::Broadcast, destructive: None, needs_player: false, allow_all_players: false, describe: "Server-wide broadcast (Generic) or ServerShutdown notice.", fields: SERVICE_BROADCAST_FIELDS, build: BuildKind::ServiceBroadcast },
    CommandSpec { id: "KickPlayer", label: "Kick player", category: Category::Player, destructive: None, needs_player: true, allow_all_players: true, describe: "Disconnects the targeted player(s).", fields: ONLY_PLAYER, build: BuildKind::Passthrough },
    CommandSpec { id: "CleanPlayerInventory", label: "Clean inventory", category: Category::Player, destructive: Some(true), needs_player: true, allow_all_players: true, describe: "Wipes the targeted player(s) inventory. Destructive.", fields: ONLY_PLAYER, build: BuildKind::Passthrough },
    CommandSpec { id: "ResetProgression", label: "Reset progression", category: Category::Player, destructive: Some(true), needs_player: true, allow_all_players: true, describe: "Wipes XP/skills/journey progress. Destructive.", fields: ONLY_PLAYER, build: BuildKind::Passthrough },
    CommandSpec { id: "UpdateAllWaterFillables", label: "Refill water", category: Category::Player, destructive: None, needs_player: true, allow_all_players: true, describe: "Refills water in fillable containers carried by the player.", fields: WATER_FIELDS, build: BuildKind::Passthrough },
    CommandSpec { id: "AwardXP", label: "Award XP", category: Category::Progression, destructive: None, needs_player: true, allow_all_players: true, describe: "Grants XP in a category.", fields: AWARD_XP_FIELDS, build: BuildKind::Passthrough },
    CommandSpec { id: "SkillsSetModuleLevel", label: "Set skill module level", category: Category::Progression, destructive: None, needs_player: true, allow_all_players: true, describe: "Sets the level of a skill module for the player.", fields: SKILL_MODULE_FIELDS, build: BuildKind::Passthrough },
    CommandSpec { id: "SkillsSetUnspentSkillPoints", label: "Set unspent skill points", category: Category::Progression, destructive: None, needs_player: true, allow_all_players: true, describe: "Sets the unspent skill points pool.", fields: SKILL_POINTS_FIELDS, build: BuildKind::Passthrough },
    CommandSpec { id: "TeleportTo", label: "Teleport (safe)", category: Category::Movement, destructive: None, needs_player: true, allow_all_players: false, describe: "Teleports player to coordinates, snapping to safe location.", fields: TELEPORT_FIELDS, build: BuildKind::Passthrough },
    CommandSpec { id: "TeleportToExact", label: "Teleport (exact)", category: Category::Movement, destructive: None, needs_player: true, allow_all_players: false, describe: "Teleports to exact coordinates with no safe-location snap.", fields: TELEPORT_FIELDS, build: BuildKind::Passthrough },
    CommandSpec { id: "SpawnVehicleAt", label: "Spawn vehicle", category: Category::Movement, destructive: None, needs_player: true, allow_all_players: false, describe: "Spawns a vehicle at coordinates for the player.", fields: SPAWN_VEHICLE_FIELDS, build: BuildKind::Passthrough },
    CommandSpec { id: "JourneyCompleteStoryNode", label: "Journey: complete node", category: Category::Journey, destructive: None, needs_player: true, allow_all_players: true, describe: "Marks a story node as completed.", fields: PLAYER_AND_STORY, build: BuildKind::Passthrough },
    CommandSpec { id: "JourneyRevealStoryNode", label: "Journey: reveal node", category: Category::Journey, destructive: None, needs_player: true, allow_all_players: true, describe: "Reveals a story node to the player.", fields: PLAYER_AND_STORY, build: BuildKind::Passthrough },
    CommandSpec { id: "JourneyResetStoryNode", label: "Journey: reset node", category: Category::Journey, destructive: Some(true), needs_player: true, allow_all_players: true, describe: "Resets a story node's progress.", fields: PLAYER_AND_STORY, build: BuildKind::Passthrough },
    CommandSpec { id: "JourneyDeleteStoryNode", label: "Journey: delete node", category: Category::Journey, destructive: Some(true), needs_player: true, allow_all_players: true, describe: "Removes a node from the player save.", fields: PLAYER_AND_STORY, build: BuildKind::Passthrough },
    CommandSpec { id: "ServerExec", label: "Server exec", category: Category::Exec, destructive: None, needs_player: true, allow_all_players: true, describe: "Runs an Exec console command on the targeted player.", fields: SERVER_EXEC_FIELDS, build: BuildKind::Passthrough },
    CommandSpec { id: "CheatScript", label: "Cheat script", category: Category::Exec, destructive: None, needs_player: true, allow_all_players: true, describe: "Runs a named cheat script (looked up in runtime data table).", fields: SCRIPT_NAME_FIELDS, build: BuildKind::Passthrough },
    CommandSpec { id: "RunLuaScriptFile", label: "Run Lua script (no-op)", category: Category::Exec, destructive: None, needs_player: true, allow_all_players: true, describe: "Lua subsystem is not wired up in shipping. Publishes successfully but the server ignores it.", fields: SCRIPT_NAME_FIELDS, build: BuildKind::Passthrough },
];

const fn json_const_i(n: i64) -> serde_json::Value {
    // const fn body: we can't actually call serde_json::json! macro inside const yet;
    // workaround: rely on a Lazy at first use. Inline construction below.
    let _ = n;
    serde_json::Value::Null
}
const fn json_const_f(n: f64) -> serde_json::Value {
    let _ = n;
    serde_json::Value::Null
}
const fn json_const_s(s: &'static str) -> serde_json::Value {
    let _ = s;
    serde_json::Value::Null
}
