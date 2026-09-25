//! Intervening "if" clauses of triggered abilities (CR 603.4) that depend on what happened
//! this turn or last turn.

use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

fn assert_supported(names: &[&str]) {
    for n in names {
        let c = card(n);
        assert!(
            c.unsupported_text().is_empty(),
            "{n} has unsupported text: {:?}",
            c.unsupported_text()
        );
    }
}

fn enter(t: &mut TestGame, p: PlayerId, name: &str) -> ObjectId {
    let id = t.hand(p, name);
    t.g.move_object(
        id,
        mtg_engine::object::Zone::Battlefield,
        mtg_engine::events::MoveCause::Effect,
        Some(p),
    )
    .expect("failed to enter the battlefield")
}

fn count_subtype(t: &TestGame, p: PlayerId, subtype: &str) -> usize {
    t.g.battlefield
        .iter()
        .filter(|id| {
            let o = t.g.obj(**id);
            o.controller == p && o.chars.has_subtype(subtype)
        })
        .count()
}

#[test]
fn werewolf_transforms_after_a_turn_without_spells_and_back_after_two() {
    cr!("603.4", "701.27a");
    assert_supported(&["Gatstaf Shepherd // Gatstaf Howler"]);
    let mut t = TestGame::new(2);
    let wolf = t.battlefield(P0, "Gatstaf Shepherd // Gatstaf Howler");
    assert_eq!(t.pt(wolf), (2, 2));
    // No spells this turn: at the next upkeep it transforms.
    t.advance_to(P1, Step::Draw);
    assert_eq!(t.pt(wolf), (3, 3));
    // P1 casts two spells this turn; at the next upkeep it transforms back.
    t.set_step(P1, Step::PrecombatMain);
    t.lands(P1, "Mountain", 2);
    for _ in 0..2 {
        let bolt = t.hand(P1, "Lightning Bolt");
        t.cast(P1, bolt).target(P0).go();
        t.resolve_all();
    }
    t.advance_to(P0, Step::Draw);
    assert_eq!(t.pt(wolf), (2, 2));
}

#[test]
fn if_you_attacked_this_turn() {
    cr!("603.4", "508.1");
    assert_supported(&["Nightsquad Commando"]);
    // Without attacking: no token.
    let mut t = TestGame::new(2);
    enter(&mut t, P0, "Nightsquad Commando");
    t.resolve_all();
    assert_eq!(count_subtype(&t, P0, "Human"), 1);
    // After attacking this turn: a 1/1 Human token.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.set_step(P0, Step::BeginningOfCombat);
    t.attack(&[(bears, Entity::Player(P1))], &[]);
    t.advance_to(P0, Step::PostcombatMain);
    enter(&mut t, P0, "Nightsquad Commando");
    t.resolve_all();
    assert_eq!(count_subtype(&t, P0, "Human"), 2);
}

#[test]
fn morbid_if_a_creature_died_this_turn() {
    cr!("603.4");
    assert_supported(&["Ulvenwald Bear"]);
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let a = enter(&mut t, P0, "Ulvenwald Bear");
    t.answer_targets(P0, &[Entity::Object(bears)]);
    t.resolve_all();
    assert_eq!(t.counters(bears, "+1/+1"), 0);
    let victim = t.battlefield(P1, "Grizzly Bears");
    t.g.destroy(victim, None);
    t.resolve_all();
    t.answer_targets(P0, &[Entity::Object(bears)]);
    enter(&mut t, P0, "Ulvenwald Bear");
    t.resolve_all();
    assert_eq!(t.counters(bears, "+1/+1"), 2);
    let _ = a;
}

#[test]
fn if_this_is_tapped() {
    cr!("603.4");
    assert_supported(&["Savior of the Small"]);
    // "At the beginning of your second main phase, if this creature is tapped, return
    // target creature card with mana value 3 or less from your graveyard to your hand."
    for tapped in [false, true] {
        let mut t = TestGame::new(2);
        let savior = t.battlefield(P0, "Savior of the Small");
        let bears = t.graveyard(P0, "Grizzly Bears");
        if tapped {
            t.g.tap(savior);
        }
        t.answer_targets(P0, &[Entity::Object(bears)]);
        t.set_step(P0, Step::EndOfCombat);
        t.advance_to(P0, Step::PostcombatMain);
        t.resolve_all();
        assert_eq!(t.in_hand(P0, "Grizzly Bears"), tapped, "tapped: {tapped}");
    }
}
