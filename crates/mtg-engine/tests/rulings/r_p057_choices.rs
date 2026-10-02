//! Rulings batch P057 — choices made as a permanent enters (CR 614.12, 607.2d): naming a
//! card (CR 201.4) for Voidstone Gargoyle, Pithing Needle and Sorcerous Spyglass, and
//! choosing an opponent for Canker Abomination and Skyshroud War Beast.

use crate::r_s01_common::supported;
use crate::r_s12_common::MORPH;
use crate::r_s20_common::tap_for_mana;
use mtg_engine::decision::Answer;
use mtg_engine::object::{CastMethod, FaceState};
use mtg_engine::testing::*;
use mtg_engine::*;

fn name_card(t: &mut TestGame, p: PlayerId, name: &str) {
    t.answer(p, DecisionKind::Name, Answer::Text(name.to_string()));
}

fn chosen_name(t: &TestGame, id: ObjectId) -> String {
    t.obj_now(id)
        .choices
        .card_name
        .as_deref()
        .unwrap_or("")
        .to_string()
}

/// The ways `p` could begin to cast `card` now: (method, face).
fn cast_ways(t: &mut TestGame, p: PlayerId, card: ObjectId) -> Vec<(CastMethod, FaceState)> {
    t.g.recompute();
    t.g.turn.priority = Some(p);
    let card = t.g.current(card);
    t.g.cast_options(p, card)
        .into_iter()
        .filter(|o| t.g.can_begin_cast(p, card, o))
        .map(|o| (o.method, o.face))
        .collect()
}

#[test]
fn voidstone_gargoyle_doesnt_stop_casting_the_named_card_face_down() {
    cr!("708.4", "201.4");
    ruling!("Voidstone Gargoyle", "The named card can be cast face down.");
    supported("Voidstone Gargoyle");
    // "As this creature enters, choose a nonland card name. / Spells with the chosen name
    // can't be cast. ..." P1 names Akroma, Angel of Fury.
    let mut t = TestGame::new(2);
    name_card(&mut t, P1, "Akroma, Angel of Fury");
    let g = t.enter(P1, "Voidstone Gargoyle");
    assert_eq!(chosen_name(&t, g), "Akroma, Angel of Fury");
    t.lands(P0, "Mountain", 8);
    let akroma = t.hand(P0, "Akroma, Angel of Fury");
    let ways = cast_ways(&mut t, P0, akroma);
    assert!(ways.iter().all(|(m, _)| *m != CastMethod::Normal));
    assert!(ways.iter().any(|(m, _)| *m == MORPH));
    let spell = t.cast(P0, akroma).method(MORPH).go();
    assert!(t.obj(spell).face_down);
}

#[test]
fn voidstone_gargoyle_stops_mana_abilities() {
    cr!("605.1a", "602.5");
    ruling!(
        "Voidstone Gargoyle",
        "Voidstone Gargoyle stops activated abilities that are mana abilities from being activated, unlike Pithing Needle."
    );
    supported("Voidstone Gargoyle");
    supported("Pithing Needle");
    let mut t = TestGame::new(2);
    let elves = t.battlefield(P0, "Llanowar Elves");
    name_card(&mut t, P1, "Llanowar Elves");
    t.enter(P1, "Voidstone Gargoyle");
    assert!(!tap_for_mana(&mut t, P0, elves, "Add"));
    // Pithing Needle ("... can't be activated unless they're mana abilities") doesn't.
    let mut t = TestGame::new(2);
    let elves = t.battlefield(P0, "Llanowar Elves");
    name_card(&mut t, P1, "Llanowar Elves");
    t.enter(P1, "Pithing Needle");
    assert!(tap_for_mana(&mut t, P0, elves, "Add"));
}

#[test]
fn voidstone_gargoyle_naming_one_half_of_a_split_card() {
    cr!("709.3", "702.102d", "201.4b");
    ruling!(
        "Voidstone Gargoyle",
        "You can name either half of a split card, but not both."
    );
    supported("Voidstone Gargoyle");
    supported("Fire // Ice");
    supported("Turn // Burn");
    // Naming both halves isn't a name: nothing is named.
    let mut t = TestGame::new(2);
    name_card(&mut t, P1, "Fire // Ice");
    let g = t.enter(P1, "Voidstone Gargoyle");
    assert_eq!(chosen_name(&t, g), "");
    // Naming Fire: Fire can't be cast; Ice can.
    let mut t = TestGame::new(2);
    name_card(&mut t, P1, "Fire");
    t.enter(P1, "Voidstone Gargoyle");
    t.lands(P0, "Volcanic Island", 6);
    let fi = t.hand(P0, "Fire // Ice");
    let faces: Vec<FaceState> = cast_ways(&mut t, P0, fi).into_iter().map(|w| w.1).collect();
    assert!(!faces.contains(&FaceState::Half(0)), "{faces:?}");
    assert!(faces.contains(&FaceState::Half(1)), "{faces:?}");
    // Naming Turn (Turn // Burn has fuse): neither Turn nor both halves fused; Burn can.
    let mut t = TestGame::new(2);
    name_card(&mut t, P1, "Turn");
    t.enter(P1, "Voidstone Gargoyle");
    t.lands(P0, "Volcanic Island", 6);
    let tb = t.hand(P0, "Turn // Burn");
    let faces: Vec<FaceState> = cast_ways(&mut t, P0, tb).into_iter().map(|w| w.1).collect();
    assert!(!faces.contains(&FaceState::Half(0)), "{faces:?}");
    assert!(!faces.contains(&FaceState::Fused), "{faces:?}");
    assert!(faces.contains(&FaceState::Half(1)), "{faces:?}");
}

#[test]
fn any_card_name_can_be_chosen_but_not_a_token_only_name() {
    cr!("201.4");
    ruling!(
        "Sorcerous Spyglass",
        "You can choose any card name, even if that card doesn't normally have an activated ability. You're not limited to the names of cards you saw in the opponent's hand."
    );
    ruling!(
        "Sorcerous Spyglass",
        "You can't choose the name of a token unless that token has the same name as a card."
    );
    ruling!(
        "Pithing Needle",
        "You can name any card, even if that card doesn't normally have an activated ability. You can't name a token unless that token has the same name as a card."
    );
    supported("Sorcerous Spyglass");
    supported("Pithing Needle");
    for chooser in ["Sorcerous Spyglass", "Pithing Needle"] {
        // A card with no activated abilities, not in P1's hand: fine.
        let mut t = TestGame::new(2);
        t.hand(P1, "Lightning Bolt");
        name_card(&mut t, P0, "Grizzly Bears");
        let s = t.enter(P0, chooser);
        assert_eq!(chosen_name(&t, s), "Grizzly Bears", "{chooser}");
        // A token-only name (Treasure) can't be chosen.
        let mut t = TestGame::new(2);
        name_card(&mut t, P0, "Treasure");
        let s = t.enter(P0, chooser);
        assert_eq!(chosen_name(&t, s), "", "{chooser}");
        // A token with the same name as a card: naming the card catches the token.
        let mut t = TestGame::new(2);
        let pyro = t.battlefield(P1, "Prodigal Pyromancer");
        let tok = crate::r_s17_common::token_copy(&mut t, P1, pyro)[0];
        t.g.objects[tok.0 as usize].summoning_sick = false;
        assert!(crate::r_s02_common::can_activate(&mut t, P1, tok));
        name_card(&mut t, P0, "Prodigal Pyromancer");
        t.enter(P0, chooser);
        assert!(
            !crate::r_s02_common::can_activate(&mut t, P1, tok),
            "{chooser}"
        );
    }
}

#[test]
fn canker_abomination_enters_with_its_counters_after_the_choice() {
    cr!("614.12", "614.1c", "122.6");
    ruling!(
        "Canker Abomination",
        "You choose an opponent while Canker Abomination is entering the battlefield. No player may take actions between the time you choose the opponent and the time Canker Abomination gets -1/-1 counters."
    );
    supported("Canker Abomination");
    // "As this creature enters, choose an opponent. This creature enters with a -1/-1
    // counter on it for each creature that player controls." (6/6)
    let mut t = TestGame::new(3);
    for _ in 0..3 {
        t.battlefield(P1, "Grizzly Bears");
    }
    t.battlefield(P2, "Grizzly Bears");
    t.answer_choose(P0, &[Entity::Player(P1)]);
    let from = t.asked().len();
    let c = t.enter(P0, "Canker Abomination");
    assert_eq!(t.counters(c, "-1/-1"), 3);
    assert_eq!(t.pt(c), (3, 3));
    assert!(t.asked()[from..]
        .iter()
        .all(|(_, d)| !matches!(d, mtg_engine::decision::Decision::Priority { .. })));
}

#[test]
fn skyshroud_war_beast_is_continuously_recalculated() {
    cr!("604.3", "611.3a");
    ruling!(
        "Skyshroud War Beast",
        "The power and toughness are continuously recalculated."
    );
    supported("Skyshroud War Beast");
    // "As this creature enters, choose an opponent. / Its power and toughness are each
    // equal to the number of nonbasic lands the chosen player controls."
    let mut t = TestGame::new(2);
    t.battlefield(P1, "Volcanic Island");
    t.battlefield(P1, "Mountain");
    t.answer_choose(P0, &[Entity::Player(P1)]);
    let beast = t.enter(P0, "Skyshroud War Beast");
    assert_eq!(t.pt(beast), (1, 1));
    let more = t.battlefield(P1, "Tropical Island");
    assert_eq!(t.pt(beast), (2, 2));
    crate::r_s02_common::destroy(&mut t, more);
    assert_eq!(t.pt(beast), (1, 1));
}
