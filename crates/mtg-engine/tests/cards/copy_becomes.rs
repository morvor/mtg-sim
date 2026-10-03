//! Permanents becoming copies of other objects (pattern in
//! `src/oracle/patterns/r707_becomes_copy.rs`): "[objects] become(s) a copy of
//! [object][ until end of turn]" (CR 707.2, 613.2a, 611.2).

use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

fn assert_compiles(names: &[&str]) {
    for n in names {
        let u = card(n).unsupported_text().join(" | ");
        assert!(u.is_empty(), "{n} has unsupported text: {u}");
    }
}

fn name(t: &TestGame, id: ObjectId) -> String {
    t.obj_now(id).chars.name.to_string()
}

#[test]
fn becomes_a_copy_cards_compile() {
    assert_compiles(&[
        "True Polymorph",
        "Mirrorweave",
        "Cytoshape",
        "Fleeting Reflection",
        "Mirrorform",
        "Scion of the Ur-Dragon",
    ]);
}

#[test]
fn true_polymorph_makes_a_permanent_copy() {
    cr!("707.2", "613.2a");
    // "Target artifact or creature becomes a copy of another target artifact or
    // creature."
    let mut t = TestGame::new(2);
    let bird = t.battlefield(P0, "Ornithopter");
    let giant = t.battlefield(P1, "Hill Giant");
    add_lands(&mut t, "True Polymorph");
    let tp = t.hand(P0, "True Polymorph");
    t.cast(P0, tp).target(bird).target(giant).go();
    t.resolve_all();
    assert_eq!(name(&t, bird), "Hill Giant");
    assert_eq!(t.pt(bird), (3, 3));
    assert!(!t.obj_now(bird).chars.is(types::CardType::Artifact));
    assert_eq!(name(&t, giant), "Hill Giant");
    // It lasts beyond this turn.
    t.advance_to(P1, Step::Upkeep);
    assert_eq!(name(&t, bird), "Hill Giant");
}

#[test]
fn mirrorweave_makes_each_other_creature_a_copy_until_end_of_turn() {
    cr!("707.2", "611.2a");
    // "Each other creature becomes a copy of target nonlegendary creature until end of
    // turn."
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let elves = t.battlefield(P0, "Llanowar Elves");
    let giant = t.battlefield(P1, "Hill Giant");
    let land = t.lands(P1, "Forest", 1)[0];
    add_lands(&mut t, "Mirrorweave");
    let mw = t.hand(P0, "Mirrorweave");
    t.cast(P0, mw).target(giant).go();
    t.resolve_all();
    for c in [bears, elves, giant] {
        assert_eq!(name(&t, c), "Hill Giant");
        assert_eq!(t.pt(c), (3, 3));
    }
    assert_eq!(name(&t, land), "Forest");
    // Until end of turn.
    t.advance_to(P1, Step::Upkeep);
    assert_eq!(name(&t, bears), "Grizzly Bears");
    assert_eq!(name(&t, elves), "Llanowar Elves");
}

#[test]
fn cytoshape_copies_the_chosen_creature_until_end_of_turn() {
    cr!("707.2", "611.2a");
    // "Choose a nonlegendary creature on the battlefield. Target creature becomes a copy
    // of that creature until end of turn."
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let giant = t.battlefield(P1, "Hill Giant");
    add_lands(&mut t, "Cytoshape");
    let cs = t.hand(P0, "Cytoshape");
    t.answer_choose(P0, &[Entity::Object(giant)]);
    t.cast(P0, cs).target(bears).go();
    t.resolve_all();
    assert_eq!(name(&t, bears), "Hill Giant");
    assert_eq!(t.pt(bears), (3, 3));
    t.advance_to(P1, Step::Upkeep);
    assert_eq!(name(&t, bears), "Grizzly Bears");
}

#[test]
fn fleeting_reflection_untaps_and_copies_up_to_one_other_target() {
    cr!("707.2", "611.2a");
    // "Target creature you control gains hexproof until end of turn. Untap that creature.
    // Until end of turn, it becomes a copy of up to one other target creature."
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.g.objects[bears.0 as usize].tapped = true;
    let giant = t.battlefield(P1, "Hill Giant");
    add_lands(&mut t, "Fleeting Reflection");
    let fr = t.hand(P0, "Fleeting Reflection");
    t.cast(P0, fr).target(bears).target(giant).go();
    t.resolve_all();
    assert!(!t.obj_now(bears).tapped);
    assert_eq!(name(&t, bears), "Hill Giant");
    assert_eq!(name(&t, giant), "Hill Giant");
    t.advance_to(P1, Step::Upkeep);
    assert_eq!(name(&t, bears), "Grizzly Bears");
}

#[test]
fn mirrorform_makes_each_nonland_permanent_you_control_a_copy() {
    cr!("707.2");
    // "Each nonland permanent you control becomes a copy of target non-Aura permanent."
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let bird = t.battlefield(P0, "Ornithopter");
    let theirs = t.battlefield(P1, "Llanowar Elves");
    let giant = t.battlefield(P1, "Hill Giant");
    add_lands(&mut t, "Mirrorform");
    let mf = t.hand(P0, "Mirrorform");
    t.cast(P0, mf).target(giant).go();
    t.resolve_all();
    assert_eq!(name(&t, bears), "Hill Giant");
    assert_eq!(name(&t, bird), "Hill Giant");
    assert_eq!(name(&t, theirs), "Llanowar Elves");
    for land in t.g.permanents().filter(|o| o.controller == P0 && o.chars.is_land()) {
        assert_ne!(land.chars.name.as_str(), "Hill Giant");
    }
}

#[test]
fn scion_of_the_ur_dragon_becomes_a_copy_of_the_dragon_it_put_into_the_graveyard() {
    cr!("707.2", "611.2a", "701.23a");
    // "{2}: Search your library for a Dragon permanent card and put it into your
    // graveyard. If you do, Scion of the Ur-Dragon becomes a copy of that card until end
    // of turn. Then shuffle."
    let mut t = TestGame::new(2);
    let scion = t.battlefield(P0, "Scion of the Ur-Dragon");
    t.lands(P0, "Wastes", 2);
    let dragon = t.library_top(P0, "Shivan Dragon");
    t.answer_choose(P0, &[Entity::Object(dragon)]);
    t.activate(P0, scion, 0, &[]).expect("Scion of the Ur-Dragon");
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Shivan Dragon"));
    assert_eq!(name(&t, scion), "Shivan Dragon");
    assert_eq!(t.pt(scion), (5, 5));
    t.advance_to(P1, Step::Upkeep);
    assert_eq!(name(&t, scion), "Scion of the Ur-Dragon");
}

/// Lands to pay for `spell`'s mana cost (Islands for {U}, Forests for {G}, Plains for
/// {W}, Wastes for generic; hybrid {W/U} is paid with an Island).
fn add_lands(t: &mut TestGame, spell: &str) {
    let cost = card(spell).front().chars.mana_cost.clone().unwrap_or_default();
    let text = format!("{cost}");
    for sym in text.split('}').filter_map(|s| s.strip_prefix('{')) {
        let land = match sym {
            "U" | "W/U" => "Island",
            "G" => "Forest",
            "W" => "Plains",
            n => {
                let k = n.parse::<usize>().unwrap_or(1);
                t.lands(P0, "Wastes", k);
                continue;
            }
        };
        t.lands(P0, land, 1);
    }
}
