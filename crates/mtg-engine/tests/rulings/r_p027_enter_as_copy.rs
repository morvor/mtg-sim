//! Rulings batch P027 — a permanent that enters as a copy of a card or creature has the
//! copied "as this enters", "enters with" and "when this enters" abilities, which apply as
//! it enters (CR 707.2, 614.1c, 614.12, 603.6a), and what it may copy as it enters.

use crate::r_p020_common::{entered_as_riftwatcher, RIFTWATCHER};
use crate::r_p027_common::*;
use crate::r_s01_common::supported;
use crate::r_s24_common::choose_creature_type;
use crate::r_s25_common::cast_new;
use mtg_engine::testing::*;
use mtg_engine::types::*;
use mtg_engine::*;

/// `name` enters for P0 as a copy of `what` (a card or permanent); returns it.
fn enter_as(t: &mut TestGame, name: &str, what: ObjectId) -> ObjectId {
    supported(name);
    t.answer_choose(P0, &[obj(what)]);
    let c = t.enter(P0, name);
    t.resolve_all();
    c
}

/// The three kinds of enters abilities: as `name` enters as a copy of an Aven
/// Riftwatcher card, it gets time counters (vanishing, "enters with") and its "enters"
/// trigger gains P0 2 life; as a copy of an Adaptive Automaton card, P0 chooses a
/// creature type ("As this creature enters"). `card` puts the card to copy where `name`
/// can copy it; returns it.
fn copies_all_enters_abilities(name: &str, card: fn(&mut TestGame, &str) -> ObjectId) {
    let mut t = TestGame::new(2);
    let rift = card(&mut t, RIFTWATCHER);
    let life = t.life(P0);
    let c = enter_as(&mut t, name, rift);
    entered_as_riftwatcher(&t, c);
    assert_eq!(t.life(P0), life + 2, "{name}: no enters trigger");
    let mut t = TestGame::new(2);
    let auto = card(&mut t, "Adaptive Automaton");
    let elf = t.battlefield(P0, "Llanowar Elves");
    choose_creature_type(&mut t, P0, "Elf");
    let c = enter_as(&mut t, name, auto);
    assert_eq!(name_of(&t, c), "Adaptive Automaton");
    assert!(has_subtype(&t, c, "Elf"), "{name}: no type chosen");
    assert_eq!(t.pt(elf), (2, 2));
}

/// A creature card in P1's graveyard.
fn in_graveyard(t: &mut TestGame, name: &str) -> ObjectId {
    t.graveyard(P1, name)
}

/// A creature card put into P1's graveyard from the battlefield this turn.
fn died_this_turn(t: &mut TestGame, name: &str) -> ObjectId {
    let c = t.battlefield(P1, name);
    kill(t, c);
    t.resolve_all();
    t.g.current(c)
}

#[test]
fn body_double_has_the_copied_cards_enters_abilities() {
    cr!("707.2", "614.1c", "614.12", "603.6a");
    ruling!("Body Double", "Treat Body Double as though it were the chosen card entering. Any \"As this card enters,\" \"This card enters with,\" and \"When this card enters\" abilities of the chosen card will work.");
    copies_all_enters_abilities("Body Double", in_graveyard);
}

#[test]
fn activated_sleeper_has_the_copied_cards_enters_abilities() {
    cr!("707.2", "707.9b", "614.1c", "614.12", "603.6a");
    ruling!("Activated Sleeper", "Treat Activated Sleeper as though it were the chosen card entering the battlefield, except it's also a Phyrexian. Any \"As [this creature] enters the battlefield,\" \"[This creature] enters the battlefield with,\" and \"When [this creature] enters the battlefield\" abilities of the chosen card will work.");
    copies_all_enters_abilities("Activated Sleeper", died_this_turn);
    let mut t = TestGame::new(2);
    let c = died_this_turn(&mut t, "Grizzly Bears");
    let s = enter_as(&mut t, "Activated Sleeper", c);
    assert!(has_subtype(&t, s, "Bear") && has_subtype(&t, s, "Phyrexian"));
}

/// P0's The Mimeoplasm enters exiling `copy` and `other` (cards in P1's graveyard),
/// copying `copy`. Returns it.
fn mimeoplasm(t: &mut TestGame, copy: ObjectId, other: ObjectId) -> ObjectId {
    supported("The Mimeoplasm");
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[obj(copy), obj(other)]);
    t.answer_choose_moved(P0, &[obj(copy)]);
    let m = t.enter(P0, "The Mimeoplasm");
    t.resolve_all();
    m
}

#[test]
fn the_mimeoplasm_has_the_copied_cards_enters_abilities() {
    cr!("707.2", "614.1c", "614.12", "603.6a");
    ruling!("The Mimeoplasm", "Treat The Mimeoplasm as though it were the creature card it's copying entering the battlefield. Any \"As [this card] enters,\" \"[This card] enters with,\" and \"When [this card] enters\" abilities of that creature card will work.");
    let mut t = TestGame::new(2);
    let rift = t.graveyard(P1, RIFTWATCHER);
    let giant = t.graveyard(P1, "Hill Giant");
    let life = t.life(P0);
    let m = mimeoplasm(&mut t, rift, giant);
    entered_as_riftwatcher(&t, m);
    assert_eq!(t.life(P0), life + 2);
    // And the counters for the other card's power.
    assert_eq!(t.counters(m, counters::PLUS1), 3);
    let mut t = TestGame::new(2);
    let auto = t.graveyard(P1, "Adaptive Automaton");
    let bears = t.graveyard(P1, "Grizzly Bears");
    let elf = t.battlefield(P0, "Llanowar Elves");
    choose_creature_type(&mut t, P0, "Elf");
    let m = mimeoplasm(&mut t, auto, bears);
    assert_eq!(name_of(&t, m), "Adaptive Automaton");
    assert!(has_subtype(&t, m, "Elf"));
    assert_eq!(t.pt(elf), (2, 2));
}

#[test]
fn dance_of_many_token_gets_its_enters_with_counters() {
    cr!("707.2", "614.1c", "111.10");
    ruling!("Dance of Many", "Treat the token as entering as a copy of the chosen creature. If that creature normally gets counters when entering, the token creature gets counters.");
    supported("Dance of Many");
    let mut t = TestGame::new(2);
    let rift = t.battlefield(P1, RIFTWATCHER);
    let before = crate::r_s01_common::tokens(&t, P0);
    cast_new(&mut t, P0, "Dance of Many", &[]);
    t.answer_targets(P0, &[obj(rift)]);
    t.resolve_all();
    let toks: Vec<ObjectId> = crate::r_s01_common::tokens(&t, P0)
        .into_iter()
        .filter(|x| !before.contains(x))
        .collect();
    assert_eq!(toks.len(), 1);
    entered_as_riftwatcher(&t, toks[0]);
}

#[test]
fn sakashima_of_a_thousand_faces_turns_off_the_legend_rule() {
    cr!("704.5j", "707.9b");
    ruling!("Sakashima of a Thousand Faces", "While the \"legend rule\" doesn't apply to permanents you control, you can control any number of legendary permanents with the same name and none of them will be put into the graveyard.");
    let mut t = TestGame::new(2);
    let isamaru = t.battlefield(P0, "Isamaru, Hound of Konda");
    let s = enter_as(&mut t, "Sakashima of a Thousand Faces", isamaru);
    t.enter(P0, "Isamaru, Hound of Konda");
    t.settle();
    assert_eq!(name_of(&t, s), "Isamaru, Hound of Konda");
    assert_eq!(t.named_on_battlefield("Isamaru, Hound of Konda").len(), 3);
    // It doesn't apply to P1's permanents.
    t.battlefield(P1, "Isamaru, Hound of Konda");
    t.answer_choose(P1, &[]);
    t.enter(P1, "Isamaru, Hound of Konda");
    t.settle();
    assert_eq!(t.named_on_battlefield("Isamaru, Hound of Konda").len(), 4);
    assert!(t.in_graveyard(P1, "Isamaru, Hound of Konda"));
}

#[test]
fn metamorphic_alteration_chooses_only_a_creature_on_the_battlefield() {
    cr!("614.12", "707.2");
    ruling!("Metamorphic Alteration", "You can choose only a creature on the battlefield.");
    // "Enchant creature. As this Aura enters, choose a creature. Enchanted creature is a
    // copy of the chosen creature."
    supported("Metamorphic Alteration");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let giant = t.battlefield(P1, "Hill Giant");
    let card = t.graveyard(P1, "Serra Angel");
    let hand = t.hand(P1, "Craw Wurm");
    let from = t.asked().len();
    t.answer_choose(P0, &[obj(giant)]);
    cast_new(&mut t, P0, "Metamorphic Alteration", &[obj(bears)]);
    t.resolve_all();
    let offered = choices_since(&t, from);
    assert_eq!(offered.len(), 1);
    assert!(offered[0].0.contains(&obj(giant)) && offered[0].0.contains(&obj(bears)));
    assert!(!offered[0].0.contains(&obj(card)) && !offered[0].0.contains(&obj(hand)));
    assert_eq!(name_of(&t, bears), "Hill Giant");
}
