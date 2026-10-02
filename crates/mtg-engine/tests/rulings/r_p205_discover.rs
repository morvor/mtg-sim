//! Rulings batch P205 — discover (CR 701.57): "Exile cards from the top of your library
//! until you exile a nonland card with mana value N or less. Cast it without paying its
//! mana cost or put it into your hand."

use crate::r_s01_common::*;
use crate::r_s02_common::create_token;
use crate::r_s04_common::asked_of_since;
use crate::r_s05_common::move_to;
use crate::r_s06_common::activate_containing;
use mtg_engine::decision::{Answer, Decision};
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::*;

/// The options offered by each "choose what to cast" decision asked of P0 since `from`.
fn cast_choices(t: &TestGame, from: usize) -> Vec<Vec<String>> {
    t.asked()[from..]
        .iter()
        .filter_map(|(p, d)| match d {
            Decision::ChooseOption {
                prompt, options, ..
            } if *p == P0 && prompt.contains("what to cast") => Some(options.clone()),
            _ => None,
        })
        .collect()
}

/// P0 activates Long-Range Sensor ("{1}, Remove two charge counters from this artifact:
/// Discover 4. Activate only as a sorcery.") and everything resolves.
fn sensor_discovers(t: &mut TestGame) {
    supported("Long-Range Sensor");
    let sensor = t.battlefield(P0, "Long-Range Sensor");
    t.g.add_counters(Entity::Object(sensor), "charge", 2, None);
    t.lands(P0, "Wastes", 1);
    activate_containing(t, P0, sensor, "Discover").unwrap();
    t.resolve_all();
    assert_eq!(t.counters(sensor, "charge"), 0);
}

/// P0 controls Monstrous Vortex and casts the creature spell `name` (with the mana for
/// it); everything resolves.
fn vortex_cast(t: &mut TestGame, name: &str) {
    supported("Monstrous Vortex");
    t.battlefield(P0, "Monstrous Vortex");
    let card = t.hand(P0, name);
    give_mana_for(t, P0, name);
    t.cast(P0, card).go();
    t.resolve_all();
}

#[test]
fn monstrous_vortex_casts_a_discovered_x_spell_with_x_0() {
    cr!("701.57a", "107.3b");
    ruling!(
        "Monstrous Vortex",
        "If the discovered card has {X} in its mana cost, you must choose 0 as the value of X when casting it without paying its mana cost."
    );
    // Craw Wurm (6/4, mana value 6): discover 6 finds Walking Ballista.
    let mut t = TestGame::new(2);
    stack_library(&mut t, P0, &["Walking Ballista"]);
    t.answer_yes(P0, true);
    t.answer(P0, DecisionKind::X, Answer::Number(3));
    let from = t.asked().len();
    vortex_cast(&mut t, "Craw Wurm");
    assert_eq!(
        asked_of_since(&t, P0, from, |d| matches!(d, Decision::ChooseX { .. })),
        0
    );
    assert!(t.in_graveyard(P0, "Walking Ballista"));
    assert_eq!(t.named_on_battlefield("Craw Wurm").len(), 1);
}

#[test]
fn discovering_an_adventurer_card_depends_on_the_discover_value() {
    cr!("701.57a", "715.3", "715.4");
    ruling!(
        "Long-Range Sensor",
        "If you discover an adventurer card, split card, or modal double-faced card, you might be able to cast that card with either set of characteristics depending on the effect’s discover value. For example, if you discover 4 and reveal Galvanic Giant (an adventurer card from Wilds of Eldraine with a mana value of 4), you could cast Galvanic Giant, but not Storm Reading (its Adventure, which has a mana value of 7). If you discover 7 and reveal Galvanic Giant, you could cast either Galvanic Giant or Storm Reading."
    );
    ruling!(
        "Monstrous Vortex",
        "If you discover an adventurer card, split card, or modal double-faced card, you might be able to cast that card with either set of characteristics depending on the effect's discover value. For example, if you discover 4 and reveal Galvanic Giant (an adventurer card from Wilds of Eldraine™ with a mana value of 4), you could cast Galvanic Giant, but not Storm Reading (its Adventure, which has a mana value of 7). If you discover 7 and reveal Galvanic Giant, you could cast either Galvanic Giant or Storm Reading."
    );
    // Long-Range Sensor discovers 4: only Galvanic Giant can be cast.
    let mut t = TestGame::new(2);
    stack_library(&mut t, P0, &["Galvanic Giant"]);
    t.answer_yes(P0, true);
    let from = t.asked().len();
    sensor_discovers(&mut t);
    assert!(cast_choices(&t, from).is_empty());
    assert_eq!(t.named_on_battlefield("Galvanic Giant").len(), 1);
    // Monstrous Vortex with Ancient Brontodon (9/9, mana value 8): discover 8, either one.
    let mut t = TestGame::new(2);
    stack_library(&mut t, P0, &["Galvanic Giant"]);
    t.answer_yes(P0, true);
    t.answer(P0, DecisionKind::Option, Answer::Index(1));
    let from = t.asked().len();
    vortex_cast(&mut t, "Ancient Brontodon");
    assert_eq!(
        cast_choices(&t, from),
        vec![vec![
            "Cast Galvanic Giant".to_string(),
            "Cast Storm Reading".to_string()
        ]]
    );
    // Storm Reading was cast: the card is on an adventure.
    assert!(t.named_on_battlefield("Galvanic Giant").is_empty());
    assert!(t.in_exile("Galvanic Giant"));
}

#[test]
fn long_range_sensor_puts_a_discovered_card_it_cant_cast_into_hand() {
    cr!("701.57a", "601.2c");
    ruling!(
        "Long-Range Sensor",
        "If you can’t cast the discovered card (perhaps because there are no legal targets for the spell), you’ll put it into your hand."
    );
    // Murder ("Destroy target creature") with no creature on the battlefield.
    let mut t = TestGame::new(2);
    stack_library(&mut t, P0, &["Murder"]);
    t.answer_yes(P0, true);
    sensor_discovers(&mut t);
    assert!(t.in_hand(P0, "Murder"));
    assert!(t.g.exile.is_empty());
}

#[test]
fn long_range_sensor_discovers_a_split_card_and_casts_one_half() {
    cr!("701.57a", "709.4", "709.3");
    ruling!(
        "Long-Range Sensor",
        "The mana value of a split card is determined by the combined mana cost of its two halves. If discover allows you to cast a split card, you may cast either half (as long as its mana value is less than or equal to the effect’s discover value) but not both halves."
    );
    // Fire // Ice ({1}{R} // {1}{U}): mana value 4. Ice ("Tap target permanent. Draw a
    // card.") is cast; Fire isn't.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    stack_library(&mut t, P0, &["Fire // Ice"]);
    t.answer_yes(P0, true);
    t.answer(P0, DecisionKind::Option, Answer::Index(1));
    t.answer_targets(P0, &[Entity::Object(bears)]);
    let from = t.asked().len();
    let hand = t.hand_size(P0);
    sensor_discovers(&mut t);
    assert_eq!(
        cast_choices(&t, from),
        vec![vec!["Cast Fire".to_string(), "Cast Ice".to_string()]]
    );
    assert!(t.obj(bears).tapped);
    assert!(t.on_battlefield(bears));
    assert_eq!(t.life(P1), 20);
    assert_eq!(t.hand_size(P0), hand + 1);
    assert!(t.in_graveyard(P0, "Fire // Ice"));
}

#[test]
fn pantlaza_stops_triggering_once_you_discovered_this_turn() {
    cr!("603.2", "701.57a");
    ruling!(
        "Pantlaza, Sun-Favored",
        "Once you decide to discover using Pantlaza's ability, that ability will stop triggering for the duration of that turn."
    );
    supported("Pantlaza, Sun-Favored");
    // Discovered: a later Dinosaur (Colossal Dreadmaw) doesn't trigger it.
    let mut t = TestGame::new(2);
    stack_library(&mut t, P0, &["Grizzly Bears"]);
    t.answer_yes(P0, true);
    t.answer_yes(P0, true);
    t.enter(P0, "Pantlaza, Sun-Favored");
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Grizzly Bears").len(), 1);
    t.enter(P0, "Colossal Dreadmaw");
    t.settle();
    assert_eq!(triggers_on_stack(&t, "discover"), 0);
    // Declined: it triggers again.
    let mut t = TestGame::new(2);
    stack_library(&mut t, P0, &["Grizzly Bears"]);
    t.answer_yes(P0, false);
    t.enter(P0, "Pantlaza, Sun-Favored");
    t.resolve_all();
    assert!(t.named_on_battlefield("Grizzly Bears").is_empty());
    assert!(!t.in_hand(P0, "Grizzly Bears") && t.g.exile.is_empty());
    t.enter(P0, "Colossal Dreadmaw");
    t.settle();
    assert_eq!(triggers_on_stack(&t, "discover"), 1);
}

#[test]
fn daring_discovery_with_no_targets_or_one_legal_target_discovers() {
    cr!("608.2b", "115.1", "701.57a");
    ruling!(
        "Daring Discovery",
        "You can cast Daring Discovery with no targets and just discover 4. However, if you choose any targets, and all of those targets are illegal by the time Daring Discovery tries to resolve, it won't resolve and none of its effects will happen. You won't discover 4. As long as one target remains legal, any legal targets won't be able to block this turn, any illegal targets won't be affected, and you'll discover 4."
    );
    // No targets: just discover 4 (Hill Giant is put into hand).
    let mut t = TestGame::new(2);
    stack_library(&mut t, P0, &["Hill Giant"]);
    give_mana_for(&mut t, P0, "Daring Discovery");
    let spell = t.hand(P0, "Daring Discovery");
    t.answer_yes(P0, false);
    t.cast(P0, spell).targets(&[]).go();
    t.resolve_all();
    assert!(t.in_hand(P0, "Hill Giant"));
    // Two targets, one gone: the other can't block, and P0 discovers.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let goblin = t.battlefield(P1, "Raging Goblin");
    stack_library(&mut t, P0, &["Hill Giant"]);
    give_mana_for(&mut t, P0, "Daring Discovery");
    let spell = t.hand(P0, "Daring Discovery");
    t.answer_yes(P0, false);
    t.cast(P0, spell)
        .targets(&[Entity::Object(bears), Entity::Object(goblin)])
        .go();
    let bounced = move_to(&mut t, bears, Zone::Hand(P1)).unwrap();
    t.resolve_all();
    assert!(t.in_hand(P0, "Hill Giant"));
    t.battlefield(P1, "Grizzly Bears");
    let attacker = t.battlefield(P0, "Hill Giant");
    crate::r_s20_common::to_beginning_of_combat(&mut t, P0);
    attack_with(&mut t, &[(attacker, Entity::Player(P1))]);
    assert!(!t.g.can_block(goblin, attacker));
    // The bounced Bears is a new object; a new Bears can block.
    let new_bears = t.named_on_battlefield("Grizzly Bears")[0];
    assert!(t.g.can_block(new_bears, attacker));
    assert_eq!(t.zone(bounced), Zone::Hand(P1));
}

#[test]
fn hit_the_mother_lodes_treasures_come_after_discovering() {
    cr!("608.2c", "701.57a", "601.2b");
    ruling!(
        "Hit the Mother Lode",
        "You won't create any Treasure tokens until you finish discovering. For example, if the card you discover has \"As an additional cost to cast this spell, sacrifice an artifact,\" you can't sacrifice one of the Treasures from Hit the Mother Lode to pay that additional cost."
    );
    supported("Shrapnel Blast");
    // Shrapnel Blast ({1}{R}, "As an additional cost to cast this spell, sacrifice an
    // artifact.") can't be cast without an artifact: it goes to hand, then eight
    // Treasures are created.
    let mut t = TestGame::new(2);
    stack_library(&mut t, P0, &["Shrapnel Blast"]);
    give_mana_for(&mut t, P0, "Hit the Mother Lode");
    let lode = t.hand(P0, "Hit the Mother Lode");
    t.answer_yes(P0, true);
    t.answer_targets(P0, &[Entity::Player(P1)]);
    t.cast(P0, lode).go();
    t.resolve_all();
    assert!(t.in_hand(P0, "Shrapnel Blast"));
    assert_eq!(t.life(P1), 20);
    assert_eq!(with_subtype(&t, P0, "Treasure").len(), 8);
    // With an artifact already there, it can be cast (sacrificing that artifact).
    let mut t = TestGame::new(2);
    let food = create_token(&mut t, P0, "Food");
    stack_library(&mut t, P0, &["Shrapnel Blast"]);
    give_mana_for(&mut t, P0, "Hit the Mother Lode");
    let lode = t.hand(P0, "Hit the Mother Lode");
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[Entity::Object(food)]);
    t.answer_targets(P0, &[Entity::Player(P1)]);
    t.cast(P0, lode).go();
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Shrapnel Blast"));
    assert_eq!(t.life(P1), 15);
    assert_eq!(with_subtype(&t, P0, "Treasure").len(), 8);
}
