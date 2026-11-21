//! cards5-referents-statics, family `cost-referents`: what a cost did, read by the effect
//! (CR 400.7j, 608.2h).

use mtg_engine::testing::*;
use mtg_engine::*;

fn supported(name: &str) {
    let def = card(name);
    assert!(
        def.unsupported_text().is_empty(),
        "{name} has unsupported text: {:?}",
        def.unsupported_text()
    );
}

#[test]
fn synthesis_pod_reveals_a_card_with_one_more_than_the_exiled_spells_mana_value() {
    cr!("608.2h");
    ruling!(
        "Synthesis Pod",
        "you must exile a spell you control on the stack"
    );
    ruling!(
        "Synthesis Pod",
        "The spell you exiled as part of the cost to activate Synthesis Pod's ability will remain in exile"
    );
    supported("Synthesis Pod");
    let mut t = TestGame::new(2);
    let pod = t.battlefield(P0, "Synthesis Pod");
    t.lands(P0, "Forest", 2);
    t.lands(P0, "Island", 2);
    // P1's library, top first: Island, Hill Giant (4), Centaur Courser (3), Island.
    t.library_top(P1, "Island");
    t.library_top(P1, "Centaur Courser");
    t.library_top(P1, "Hill Giant");
    t.library_top(P1, "Island");
    let bears = t.hand(P0, "Grizzly Bears");
    t.cast(P0, bears).go();
    let lib = t.library_size(P1);
    t.answer_yes(P0, true);
    t.activate(P0, pod, 0, &[Entity::Player(P1)])
        .expect("activates exiling the Bears spell");
    assert!(t.in_exile("Grizzly Bears"));
    t.resolve_all();
    // Centaur Courser (1 + 2) was exiled and cast; the others went back.
    assert_eq!(t.named_on_battlefield("Centaur Courser").len(), 1);
    let courser = t.named_on_battlefield("Centaur Courser")[0];
    assert_eq!(t.obj_now(courser).controller, P0);
    assert_eq!(t.library_size(P1), lib - 1);
    assert!(t.in_exile("Grizzly Bears"));
    assert!(t.named_on_battlefield("Hill Giant").is_empty());
}

/// Shuriken equipped to `holder` (P0's); the granted ability targets P1's Hill Giant.
fn shuriken(holder: &str) -> (TestGame, ObjectId) {
    let mut t = TestGame::new(2);
    let h = t.battlefield(P0, holder);
    let s = t.battlefield(P0, "Shuriken");
    t.g.attach(s, Entity::Object(h));
    let giant = t.battlefield(P1, "Hill Giant");
    t.g.recompute();
    t.activate(P0, h, 0, &[Entity::Object(giant)])
        .expect("the granted ability");
    t.resolve_all();
    assert_eq!(t.g.obj(giant).damage, 2);
    assert_eq!(t.obj_now(s).attached_to, None);
    (t, s)
}

#[test]
fn shuriken_goes_to_the_damaged_creatures_controller_unless_unattached_from_a_ninja() {
    cr!("608.2h");
    supported("Shuriken");
    let (t, s) = shuriken("Grizzly Bears");
    assert_eq!(t.obj_now(s).controller, P1);
    let (t, s) = shuriken("Ninja of the Deep Hours");
    assert_eq!(t.obj_now(s).controller, P0);
}

#[test]
fn captain_americas_throw_reads_the_unattached_equipments_mana_value() {
    cr!("608.2h");
    // "{3}, Unattach an Equipment from ~: ~ deals damage equal to that Equipment's mana
    // value to any target." (the single-target form of Captain America's ability): the
    // Equipment's mana value, not the creature's.
    let text = "{3}, Unattach an Equipment from ~: ~ deals damage equal to that Equipment's mana value to any target.";
    let tl = mtg_engine::types::TypeLine::parse("Creature — Human Soldier");
    let ctx = mtg_engine::oracle::CompileContext {
        card_name: "Thrower",
        full_name: "Thrower",
        type_line: &tl,
        layout: mtg_engine::card::Layout::Normal,
        face_index: 0,
        keywords: &[],
        power: Some("2"),
        toughness: Some("2"),
    };
    let compiled = mtg_engine::oracle::compile(text, &ctx);
    assert!(compiled.unsupported.is_empty(), "{:?}", compiled.unsupported);
    let def = CardDef::custom(mtg_engine::object::Characteristics {
        name: "Thrower".into(),
        mana_cost: mtg_engine::mana::ManaCost::parse("{1}{W}"),
        card_types: tl.card_types,
        abilities: compiled.abilities,
        rules_text: std::sync::Arc::from(text),
        power: Some(2),
        toughness: Some(2),
        ..Default::default()
    });
    let mut t = TestGame::new(2);
    let thrower = t.custom(P0, def, mtg_engine::object::Zone::Battlefield);
    let sword = t.battlefield(P0, "Bonesplitter");
    t.g.attach(sword, Entity::Object(thrower));
    t.lands(P0, "Plains", 3);
    t.activate(P0, thrower, 0, &[Entity::Player(P1)])
        .expect("activates");
    t.resolve_all();
    assert_eq!(t.life(P1), 19);
    assert_eq!(t.obj_now(sword).attached_to, None);
}
