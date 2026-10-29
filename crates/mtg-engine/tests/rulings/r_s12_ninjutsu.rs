//! Rulings batch S12 — ninjutsu (CR 702.49): "[Cost], Reveal this card from your hand,
//! Return an unblocked attacking creature you control to its owner's hand: Put this card
//! onto the battlefield from your hand tapped and attacking."

use crate::r_s01_common::*;
use crate::r_s03_common::to_blockers;
use crate::r_s12_common::*;
use mtg_engine::decision::Decision;
use mtg_engine::kw::ninjutsu::revealed_cards;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

/// `p` activates the ninjutsu ability of the card `ninja` in their hand, returning
/// `returned`.
fn ninjutsu(t: &mut TestGame, p: PlayerId, ninja: ObjectId, returned: ObjectId) {
    t.answer_choose(p, &[Entity::Object(returned)]);
    t.activate(p, ninja, 0, &[]).expect("ninjutsu");
}

/// Treasure tokens `p` controls.
fn treasures(t: &TestGame, p: PlayerId) -> usize {
    with_subtype(t, p, "Treasure").len()
}

/// Whether anyone was asked to choose what a creature entering attacking attacks.
fn asked_attack_choice(t: &TestGame, from: usize) -> bool {
    asked_since(t, from).iter().any(|(_, d)| {
        matches!(d, Decision::ChooseEntities { prompt, .. } if prompt.contains("attacking"))
    })
}

#[test]
fn ninjutsu_in_the_first_strike_step_deals_damage_in_the_regular_step_even_with_first_strike()
{
    cr!("702.49a", "510.4");
    ruling!(
        "Prosperous Thief",
        "If a creature in combat has first strike or double strike, you can activate the ninjutsu ability during the first-strike combat damage step. The Ninja will deal combat damage during the regular combat damage step, even if it has first strike."
    );
    supported("Prosperous Thief");
    supported("Knighthood");
    let mut t = TestGame::new(2);
    // Knighthood: "Creatures you control have first strike."
    t.battlefield(P0, "Knighthood");
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.lands(P0, "Island", 2);
    let thief = t.hand(P0, "Prosperous Thief");
    to_blockers(&mut t, &[(bears, Entity::Player(P1))], &[]);
    t.advance_to(P0, Step::FirstStrikeDamage);
    assert_eq!(t.life(P1), 18);
    // The Bears aren't a Ninja or Rogue: no Treasure.
    assert_eq!(treasures(&t, P0), 0);
    ninjutsu(&mut t, P0, thief, bears);
    t.resolve_all();
    let thief = t.g.current(thief);
    assert!(t.g.is_attacking(thief));
    assert!(t
        .obj(thief)
        .has_keyword(mtg_engine::keywords::KeywordKind::FirstStrike));
    t.advance_to(P0, Step::EndOfCombat);
    // The 3/2 Thief dealt its damage in the regular combat damage step.
    assert_eq!(t.life(P1), 15);
    t.resolve_all();
    assert_eq!(treasures(&t, P0), 1);
}

#[test]
fn the_ninja_attacks_what_the_returned_creature_attacked_other_effects_let_you_choose() {
    cr!("702.49c", "508.4");
    ruling!(
        "Prosperous Thief",
        "The creature with ninjutsu enters the battlefield attacking the same player or planeswalker that the returned creature was attacking. This is a rule specific to ninjutsu; in other cases, when a creature is put onto the battlefield attacking, that creature's controller chooses which player or planeswalker it's attacking."
    );
    ruling!(
        "Ingenious Infiltrator",
        "The creature with ninjutsu enters the battlefield attacking the same player or planeswalker that the returned creature was attacking. This is a rule specific to ninjutsu; in other cases, when a creature is put onto the battlefield attacking, that creature’s controller chooses which player or planeswalker it’s attacking."
    );
    ruling!(
        "Geist of Saint Traft",
        "You choose which player or planeswalker the Angel token is attacking. It doesn't have to be attacking the same player or planeswalker that Geist of Saint Traft is attacking."
    );
    supported("Ingenious Infiltrator");
    supported("Geist of Saint Traft");
    // Three players; the Bears attack P2's Jace Beleren. The Ninja attacks Jace too, with
    // no choice asked.
    for ninja in ["Prosperous Thief", "Ingenious Infiltrator"] {
        let mut t = TestGame::new(3);
        let jace = t.battlefield(P2, "Jace Beleren");
        let bears = t.battlefield(P0, "Grizzly Bears");
        t.lands(P0, "Island", 2);
        t.lands(P0, "Swamp", 1);
        let n = t.hand(P0, ninja);
        to_blockers(&mut t, &[(bears, Entity::Object(jace))], &[]);
        let from = t.asked().len();
        ninjutsu(&mut t, P0, n, bears);
        t.resolve_all();
        assert_eq!(attack_target(&t, n), Some(Entity::Object(jace)), "{ninja}");
        assert!(!asked_attack_choice(&t, from), "{ninja}");
    }
    // Other effects: Geist of Saint Traft attacks P1; its controller chooses what the Angel
    // token ("tapped and attacking") attacks: P2's Jace.
    let mut t = TestGame::new(3);
    let jace = t.battlefield(P2, "Jace Beleren");
    let geist = t.battlefield(P0, "Geist of Saint Traft");
    attack_with(&mut t, &[(geist, Entity::Player(P1))]);
    let from = t.asked().len();
    t.answer_choose(P0, &[Entity::Object(jace)]);
    t.resolve_all();
    assert!(asked_attack_choice(&t, from));
    let angel = with_subtype(&t, P0, "Angel");
    assert_eq!(angel.len(), 1);
    assert_eq!(attack_target(&t, angel[0]), Some(Entity::Object(jace)));
    assert_eq!(attack_target(&t, geist), Some(Entity::Player(P1)));
}

#[test]
fn ninjutsu_in_the_combat_damage_step_comes_after_combat_damage() {
    cr!("702.49a", "510.2");
    ruling!(
        "Prosperous Thief",
        "The ninjutsu ability can be activated during the declare blockers step, combat damage step, or end of combat step. In most cases (see below), if you wait until the combatdamage step or end of combat step, it will be after combat damage has been dealt, so the Ninja won't deal combat damage."
    );
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.lands(P0, "Island", 2);
    let thief = t.hand(P0, "Prosperous Thief");
    to_blockers(&mut t, &[(bears, Entity::Player(P1))], &[]);
    t.advance_to(P0, Step::CombatDamage);
    assert_eq!(t.life(P1), 18);
    // The Bears are still an unblocked attacking creature: ninjutsu can be activated.
    ninjutsu(&mut t, P0, thief, bears);
    t.resolve_all();
    let thief = t.g.current(thief);
    assert!(t.g.is_attacking(thief));
    t.advance_to(P0, Step::PostcombatMain);
    // The Thief dealt no combat damage: no more life lost, no Treasure.
    assert_eq!(t.life(P1), 18);
    assert_eq!(treasures(&t, P0), 0);
    assert!(t.in_hand(P0, "Grizzly Bears"));
}

#[test]
fn ninjutsu_in_the_end_of_combat_step_deals_no_combat_damage() {
    cr!("702.49a", "511.1");
    ruling!(
        "Ingenious Infiltrator",
        "The ninjutsu ability can be activated during the declare blockers step, combat damage step, or end of combat step. If you wait until after the declare blockers step, because all combat damage is dealt at once, the Ninja won’t normally deal combat damage."
    );
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.lands(P0, "Island", 1);
    t.lands(P0, "Swamp", 1);
    let inf = t.hand(P0, "Ingenious Infiltrator");
    to_blockers(&mut t, &[(bears, Entity::Player(P1))], &[]);
    t.advance_to(P0, Step::EndOfCombat);
    assert_eq!(t.life(P1), 18);
    let hand = t.hand_size(P0);
    ninjutsu(&mut t, P0, inf, bears);
    t.resolve_all();
    let inf = t.g.current(inf);
    assert!(t.g.is_attacking(inf));
    t.advance_to(P0, Step::PostcombatMain);
    // No combat damage: the Infiltrator's "Whenever a Ninja you control deals combat damage
    // to a player, draw a card" didn't trigger. (The Bears returned; the Infiltrator left.)
    assert_eq!(t.life(P1), 18);
    assert_eq!(t.hand_size(P0), hand);
}

#[test]
fn the_ninja_is_revealed_and_enters_only_if_still_in_hand_as_the_ability_resolves() {
    cr!("702.49a", "702.49b");
    ruling!(
        "Ingenious Infiltrator",
        "As you activate a ninjutsu ability, you reveal the Ninja card in your hand and return the attacking creature. The Ninja isn’t put onto the battlefield until the ability resolves. If it leaves your hand before then, it won’t enter the battlefield at all."
    );
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.lands(P0, "Island", 2);
    t.lands(P0, "Swamp", 2);
    let inf = t.hand(P0, "Ingenious Infiltrator");
    to_blockers(&mut t, &[(bears, Entity::Player(P1))], &[]);
    ninjutsu(&mut t, P0, inf, bears);
    // The Bears were returned as a cost; the Infiltrator is revealed, still in hand.
    assert!(t.in_hand(P0, "Grizzly Bears"));
    assert_eq!(revealed_cards(&t.g), vec![inf]);
    assert!(t.in_hand(P0, "Ingenious Infiltrator"));
    assert_eq!(t.stack_len(), 1);
    // It's discarded in response: the ability does nothing.
    t.g.discard(P0, inf, None);
    t.resolve_all();
    assert!(t.named_on_battlefield("Ingenious Infiltrator").is_empty());
    assert!(t.in_graveyard(P0, "Ingenious Infiltrator"));
    // A second activation that resolves normally puts it onto the battlefield attacking.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.lands(P0, "Island", 1);
    t.lands(P0, "Swamp", 1);
    let inf = t.hand(P0, "Ingenious Infiltrator");
    to_blockers(&mut t, &[(bears, Entity::Player(P1))], &[]);
    ninjutsu(&mut t, P0, inf, bears);
    t.resolve_all();
    assert!(revealed_cards(&t.g).is_empty());
    let inf = t.g.current(inf);
    assert!(t.on_battlefield(inf) && t.obj(inf).tapped && t.g.is_attacking(inf));
}
