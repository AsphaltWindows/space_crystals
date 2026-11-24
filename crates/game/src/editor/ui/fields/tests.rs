use super::*;
use sc_sim::{MapType, Spawn, SpawnAt, TilePos};

fn map(width: u32, height: u32) -> MapDef {
    MapSettings {
        map_type: MapType::Custom,
        symmetry: Symmetry::None,
        width,
        height,
        slots: 2,
        name: "empty".into(),
        ..MapSettings::default()
    }
    .create()
    .unwrap()
}

#[test]
fn fields_commit_valid_changes_only() {
    let map = map(20, 16);
    let dialog = MapSettings::default();
    let playtest = PlayTestSetup::default();
    let ctx = FieldContext {
        map: &map,
        dialog: &dialog,
        playtest: &playtest,
    };
    let commit = |field: TextField, text: &str| field.commit(text, ctx);
    assert_eq!(commit(TextField::MapName, " empty "), Ok(None), "unchanged");
    assert_eq!(
        commit(TextField::MapName, "Ridge"),
        Ok(Some(EditorAction::SetName("Ridge".into())))
    );
    assert_eq!(
        commit(TextField::Description, "Two bases"),
        Ok(Some(EditorAction::SetDescription("Two bases".into())))
    );
    assert_eq!(
        commit(TextField::Seed, "42"),
        Ok(Some(EditorAction::SetPlayTestSeed(42)))
    );
    assert!(commit(TextField::Seed, "").is_err());
    assert_eq!(
        commit(TextField::Width, "24"),
        Ok(Some(EditorAction::Resize { width: 24, height: 16 }))
    );
    assert_eq!(
        commit(TextField::Height, "30"),
        Ok(Some(EditorAction::Resize { width: 20, height: 30 }))
    );
    assert!(commit(TextField::Height, "99999999999").is_err());
    assert_eq!(
        commit(TextField::PresetName(TilePresetId(5)), "Ice"),
        Ok(Some(EditorAction::SetPresetName(TilePresetId(5), "Ice".into())))
    );
    assert_eq!(TextField::PresetName(TilePresetId(99)).value(ctx), None);
    assert_eq!(
        commit(TextField::NewMapWidth, "96"),
        Ok(Some(EditorAction::EditNewMap(NewMapField::Width(96))))
    );
    assert_eq!(
        commit(TextField::NewMapName, "untitled"),
        Ok(None),
        "the dialog's value"
    );

    // A map with 90 degree rotation stays square.
    let square = MapSettings {
        symmetry: Symmetry::Rotate90,
        width: 32,
        height: 32,
        slots: 4,
        ..MapSettings::default()
    }
    .create()
    .unwrap();
    let ctx = FieldContext { map: &square, ..ctx };
    assert_eq!(
        TextField::Height.commit("40", ctx),
        Ok(Some(EditorAction::Resize { width: 40, height: 40 }))
    );
}

#[test]
fn spawn_value_fields_show_and_commit_the_spawns_values() {
    let mut map = map(20, 16);
    map.spawns = vec![
        Spawn::new(ObjectKind::CrystalPatch, None, SpawnAt::Tile(TilePos::new(1, 1))),
        Spawn::new(
            ObjectKind::SupplyDeliveryStation,
            None,
            SpawnAt::Tile(TilePos::new(4, 4)),
        ),
    ];
    let dialog = MapSettings::default();
    let playtest = PlayTestSetup::default();
    let ctx = FieldContext {
        map: &map,
        dialog: &dialog,
        playtest: &playtest,
    };
    let field = |spawn, value| TextField::SpawnValue { spawn, value };
    assert_eq!(
        field(0, SpawnValue::StartingAmount).value(ctx),
        Some(sc_sim::scenario::DEFAULT_STARTING_AMOUNT.to_string())
    );
    assert_eq!(field(1, SpawnValue::DeliveryInterval).value(ctx), Some("60".into()));
    assert_eq!(
        field(0, SpawnValue::DeliverySize).value(ctx),
        None,
        "patches have no deliveries"
    );
    assert_eq!(field(5, SpawnValue::StartingAmount).value(ctx), None);
    assert_eq!(
        field(0, SpawnValue::StartingAmount).commit("900", ctx),
        Ok(Some(EditorAction::SetSpawnSettings {
            spawn: 0,
            settings: SpawnSettings::CrystalPatch { starting_amount: 900 }
        }))
    );
    // Seconds become ticks, rounded to the nearest tick.
    let Ok(Some(EditorAction::SetSpawnSettings { settings, .. })) =
        field(1, SpawnValue::DeliveryInterval).commit("7.5", ctx)
    else {
        panic!("the interval should commit");
    };
    assert_eq!(
        settings,
        SpawnSettings::SupplyDeliveryStation {
            starting_supplies: sc_sim::scenario::DEFAULT_STARTING_SUPPLIES,
            delivery_size: sc_sim::scenario::DEFAULT_DELIVERY_SIZE,
            delivery_interval: 120,
        }
    );
    assert_eq!(SpawnValue::DeliveryInterval.get(settings), Some("7.5".into()));
    assert!(field(1, SpawnValue::DeliverySize).commit("lots", ctx).is_err());
    assert!(field(1, SpawnValue::DeliveryInterval).commit("1e99", ctx).is_err());
    assert_eq!(
        field(1, SpawnValue::DeliveryInterval).commit("60", ctx),
        Ok(None),
        "unchanged"
    );
}
