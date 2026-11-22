//! Rulings batch S31 — the Sieges of Fate Reforged: "As this enchantment enters, choose
//! Khans or Dragons", and the anchor words give it only the chosen ability (CR 614.12c,
//! 607.2m).

use crate::r_s01_common::supported;
use crate::r_s02_common::destroy;
use mtg_engine::decision::Answer;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

/// P0's Siege `name` enters with the anchor word at index `choice` chosen (0 = Khans,
/// 1 = Dragons).
fn siege(t: &mut TestGame, name: &str, choice: usize) -> ObjectId {
    supported(name);
    t.answer(P0, DecisionKind::Option, Answer::Index(choice));
    let s = t.enter(P0, name);
    t.settle();
    let word = ["Khans", "Dragons"][choice];
    assert_eq!(t.obj_now(s).choices.text.as_deref(), Some(word));
    s
}

/// Advances to P0's next upkeep and resolves what triggered.
fn next_upkeep(t: &mut TestGame) {
    t.advance_to(P1, Step::Upkeep);
    t.advance_to(P0, Step::Upkeep);
    t.resolve_all();
}

#[test]
fn outpost_siege_has_only_the_chosen_ability() {
    cr!("614.12c", "607.2m");
    ruling!(
        "Outpost Siege",
        "Each Siege will have one of the two listed abilities, depending on your choice as it enters the battlefield."
    );
    ruling!(
        "Outpost Siege",
        "The words \"Khans\" and \"Dragons\" are anchor words, connecting your choice to the appropriate ability. Anchor words are a new rules concept. \"[Anchor word] — [Ability]\" means \"As long as you chose [anchor word] as this permanent entered the battlefield, this permanent has [ability].\""
    );
    // "• Khans — At the beginning of your upkeep, exile the top card of your library.
    // Until end of turn, you may play that card. • Dragons — Whenever a creature you
    // control leaves the battlefield, this enchantment deals 1 damage to any target."
    let mut t = TestGame::new(2);
    siege(&mut t, "Outpost Siege", 0);
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.answer_targets(P0, &[Entity::Player(P1)]);
    destroy(&mut t, bears);
    t.resolve_all();
    assert_eq!(t.life(P1), 20);
    let top = t.library_top(P0, "Lightning Bolt");
    next_upkeep(&mut t);
    assert_eq!(t.zone(top), mtg_engine::object::Zone::Exile);

    let mut t = TestGame::new(2);
    siege(&mut t, "Outpost Siege", 1);
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.answer_targets(P0, &[Entity::Player(P1)]);
    destroy(&mut t, bears);
    t.resolve_all();
    assert_eq!(t.life(P1), 19);
    let top = t.library_top(P0, "Lightning Bolt");
    next_upkeep(&mut t);
    assert_ne!(t.zone(top), mtg_engine::object::Zone::Exile);
}

#[test]
fn the_anchor_word_dragons_has_nothing_to_do_with_dragons() {
    cr!("614.12c", "607.2m");
    ruling!(
        "Palace Siege",
        "The words “Khans” and “Dragons” are anchor words, connecting your choice to the appropriate ability. Anchor words are a new rules concept. “[Anchor word] — [Ability]” means “As long as you chose [anchor word] as this permanent entered the battlefield, this permanent has [ability].” Notably, the anchor word “Dragons” has no connection to the creature type Dragon."
    );
    // "• Khans — At the beginning of your upkeep, return target creature card from your
    // graveyard to your hand. • Dragons — At the beginning of your upkeep, each opponent
    // loses 2 life and you gain 2 life."
    // Dragons chosen, with no Dragon anywhere: it drains.
    let mut t = TestGame::new(2);
    siege(&mut t, "Palace Siege", 1);
    next_upkeep(&mut t);
    assert_eq!(t.life(P1), 18);
    assert_eq!(t.life(P0), 22);
    // Khans chosen, while controlling a Dragon: it doesn't.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Shivan Dragon");
    siege(&mut t, "Palace Siege", 0);
    next_upkeep(&mut t);
    assert_eq!(t.life(P1), 20);
    assert_eq!(t.life(P0), 20);
}
