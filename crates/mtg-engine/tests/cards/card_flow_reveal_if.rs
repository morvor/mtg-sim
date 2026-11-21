//! "Reveal the top card of your library. If it's a [kind of] card, put it [somewhere].
//! Otherwise, put it [somewhere else]." (Skyward Eye Prophets, Zoologist, Garruk, Savage
//! Herald; Nissa, Sage Animist).

use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::*;

fn assert_supported(name: &str) {
    let c = card(name);
    assert!(
        c.unsupported_text().is_empty(),
        "{name} has unsupported text: {:?}",
        c.unsupported_text()
    );
}

/// Activates the `index`th activated ability of `source` and resolves it.
fn activate_and_resolve(t: &mut TestGame, source: ObjectId, index: usize) {
    t.activate(P0, source, index, &[]).expect("can't activate");
    t.resolve_all();
}

#[test]
fn skyward_eye_prophets_a_land_onto_the_battlefield_otherwise_into_your_hand() {
    cr!("701.20a", "608.2c");
    assert_supported("Skyward Eye Prophets");
    // A land: onto the battlefield.
    let mut t = TestGame::new(2);
    let prophets = t.battlefield(P0, "Skyward Eye Prophets");
    let forest = t.library_top(P0, "Forest");
    activate_and_resolve(&mut t, prophets, 0);
    assert_eq!(t.zone(forest), Zone::Battlefield);
    assert_eq!(t.obj_now(forest).controller, P0);
    // Anything else: into your hand.
    let mut t = TestGame::new(2);
    let prophets = t.battlefield(P0, "Skyward Eye Prophets");
    let bolt = t.library_top(P0, "Lightning Bolt");
    let before = t.library_size(P0);
    activate_and_resolve(&mut t, prophets, 0);
    assert_eq!(t.zone(bolt), Zone::Hand(P0));
    assert_eq!(t.library_size(P0), before - 1);
}

#[test]
fn zoologist_a_creature_onto_the_battlefield_otherwise_into_your_graveyard() {
    cr!("701.20a", "608.2c");
    assert_supported("Zoologist");
    let mut t = TestGame::new(2);
    t.lands(P0, "Forest", 4);
    let zoologist = t.battlefield(P0, "Zoologist");
    let bears = t.library_top(P0, "Grizzly Bears");
    activate_and_resolve(&mut t, zoologist, 0);
    assert_eq!(t.zone(bears), Zone::Battlefield);
    let mut t = TestGame::new(2);
    t.lands(P0, "Forest", 4);
    let zoologist = t.battlefield(P0, "Zoologist");
    let forest = t.library_top(P0, "Forest");
    activate_and_resolve(&mut t, zoologist, 0);
    assert_eq!(t.zone(forest), Zone::Graveyard(P0));
}

#[test]
fn garruk_a_creature_into_your_hand_otherwise_on_the_bottom() {
    cr!("701.20a", "608.2c");
    assert_supported("Garruk, Savage Herald");
    let mut t = TestGame::new(2);
    let garruk = t.battlefield(P0, "Garruk, Savage Herald");
    let bears = t.library_top(P0, "Grizzly Bears");
    activate_and_resolve(&mut t, garruk, 0);
    assert_eq!(t.zone(bears), Zone::Hand(P0));
    let mut t = TestGame::new(2);
    let garruk = t.battlefield(P0, "Garruk, Savage Herald");
    let forest = t.library_top(P0, "Forest");
    activate_and_resolve(&mut t, garruk, 0);
    assert_eq!(t.zone(forest), Zone::Library(P0));
    assert_eq!(t.g.player(P0).library[0], t.g.current(forest));
}

/// P0's Nissa, Sage Animist, put onto the battlefield transformed.
fn animist(t: &mut TestGame) -> ObjectId {
    use mtg_engine::ability::{Destination, Effect, Sel};
    let card = t.graveyard(P0, "Nissa, Vastwood Seer // Nissa, Sage Animist");
    let mut ctx = mtg_engine::eval::Ctx::new(None, P0);
    ctx.targets = vec![vec![Entity::Object(card)]];
    let mut to = Destination::battlefield();
    to.transformed = true;
    t.g.exec(
        &Effect::Move {
            what: Sel::Target(0),
            to,
        },
        &mut ctx,
    );
    t.g.flush_events();
    t.resolve_all();
    let nissa = t.g.current(card);
    assert_eq!(t.obj(nissa).chars.name, "Nissa, Sage Animist");
    nissa
}

#[test]
fn nissa_sage_animist_a_land_onto_the_battlefield_otherwise_into_your_hand() {
    cr!("701.20a", "608.2c");
    assert_supported("Nissa, Vastwood Seer // Nissa, Sage Animist");
    let mut t = TestGame::new(2);
    let nissa = animist(&mut t);
    let forest = t.library_top(P0, "Forest");
    activate_and_resolve(&mut t, nissa, 0);
    assert_eq!(t.zone(forest), Zone::Battlefield);
    assert_eq!(t.counters(nissa, mtg_engine::types::counters::LOYALTY), 4);
    let mut t = TestGame::new(2);
    let nissa = animist(&mut t);
    let bolt = t.library_top(P0, "Lightning Bolt");
    activate_and_resolve(&mut t, nissa, 0);
    assert_eq!(t.zone(bolt), Zone::Hand(P0));
}
