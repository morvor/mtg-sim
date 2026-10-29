//! Rulings batch S14 — renown (CR 702.112): "When this creature deals combat damage to a
//! player, if it isn't renowned, put N +1/+1 counters on it and it becomes renowned."; and
//! repartee: "Whenever you cast an instant or sorcery spell that targets a creature, ..."
//! (an ability word, CR 207.2c), which triggers as the spell becomes cast (CR 601.2i) and
//! is then independent of it (CR 113.7a).

use crate::r_s01_common::*;
use crate::r_s03_common::to_blockers;
use crate::r_s14_common::*;
use mtg_engine::kw::renown::is_renowned;
use mtg_engine::testing::*;
use mtg_engine::turn::{Stage, Step};
use mtg_engine::types::counters;
use mtg_engine::*;

fn plus1(t: &TestGame, id: ObjectId) -> u32 {
    t.counters(id, counters::PLUS1)
}

#[test]
fn renown_triggers_on_combat_damage_redirected_to_its_controller() {
    cr!("702.112a", "510.2", "614.1a");
    ruling!(
        "Topan Freeblade",
        "If a creature with renown deals combat damage to its controller because that damage was redirected, renown will trigger."
    );
    supported("Topan Freeblade");
    supported("Mirror Strike");
    // Topan Freeblade (2/2, renown 1) attacks P1 and isn't blocked. P1 casts Mirror
    // Strike: "All combat damage that would be dealt to you this turn by target unblocked
    // creature is dealt to its controller instead."
    let mut t = TestGame::new(2);
    let blade = t.battlefield(P0, "Topan Freeblade");
    to_blockers(&mut t, &[(blade, Entity::Player(P1))], &[]);
    cast_from_hand(&mut t, P1, "Mirror Strike", &[Entity::Object(blade)]);
    t.resolve_all();
    t.advance_to(P0, Step::EndOfCombat);
    assert_eq!(t.life(P1), 20);
    assert_eq!(t.life(P0), 18);
    assert_eq!(plus1(&t, blade), 1);
    assert!(is_renowned(&t.g, blade));
}

#[test]
fn renown_doesnt_trigger_on_damage_to_a_planeswalker_or_creature_or_noncombat_damage() {
    cr!("702.112a", "120.2a");
    ruling!(
        "Relic Seeker",
        "Renown won't trigger when a creature deals combat damage to a planeswalker or another creature. It also won't trigger when a creature deals noncombat damage to a player."
    );
    supported("Relic Seeker");
    supported("Soul's Fire");
    // Relic Seeker (2/2, renown 1) attacks P1's Jace Beleren.
    let mut t = TestGame::new(2);
    let seeker = t.battlefield(P0, "Relic Seeker");
    let jace = t.battlefield(P1, "Jace Beleren");
    t.attack(&[(seeker, Entity::Object(jace))], &[]);
    assert_eq!(t.counters(jace, counters::LOYALTY), 1);
    assert!(!is_renowned(&t.g, seeker));
    // It attacks P1 and is blocked by Ornithopter (0/2), dealing it combat damage.
    let mut t = TestGame::new(2);
    let seeker = t.battlefield(P0, "Relic Seeker");
    let thopter = t.battlefield(P1, "Ornithopter");
    t.attack(&[(seeker, Entity::Player(P1))], &[(thopter, seeker)]);
    assert!(t.in_graveyard(P1, "Ornithopter"));
    assert!(!is_renowned(&t.g, seeker));
    // Soul's Fire: it deals 2 noncombat damage to P1.
    let mut t = TestGame::new(2);
    let seeker = t.battlefield(P0, "Relic Seeker");
    t.answer_targets(P0, &[Entity::Object(seeker)]);
    t.answer_targets(P0, &[Entity::Player(P1)]);
    crate::r_s01_common::give_mana_for(&mut t, P0, "Soul's Fire");
    let fire = t.hand(P0, "Soul's Fire");
    t.cast(P0, fire).go();
    t.resolve_all();
    assert_eq!(t.life(P1), 18);
    assert!(!is_renowned(&t.g, seeker));
    assert_eq!(plus1(&t, seeker), 0);
}

#[test]
fn a_renown_creature_that_leaves_before_its_ability_resolves_doesnt_become_renowned() {
    cr!("702.112a", "702.112b", "400.7");
    ruling!(
        "Relic Seeker",
        "If a renown ability triggers, but the creature leaves the battlefield before that ability resolves, the creature doesn't become renowned. Any ability that triggers \"whenever a creature becomes renowned\" won't trigger."
    );
    supported("Valeron Wardens");
    // Relic Seeker ("When this creature becomes renowned, you may search your library for
    // an Equipment card ...") deals combat damage to P1 with P0's Valeron Wardens
    // ("Whenever a creature you control becomes renowned, draw a card.") on the
    // battlefield. P1 destroys Relic Seeker with Lightning Bolt in response to renown.
    let mut t = TestGame::new(2);
    let seeker = t.battlefield(P0, "Relic Seeker");
    t.battlefield(P0, "Valeron Wardens");
    to_blockers(&mut t, &[(seeker, Entity::Player(P1))], &[]);
    let ok = t.g.run_until(1000, |g| {
        g.turn.step == Step::CombatDamage && g.turn.stage == Stage::Priority && !g.stack.is_empty()
    });
    assert!(ok);
    assert_eq!(t.life(P1), 18);
    assert_eq!(triggers_from(&t, seeker), 1);
    let hand = t.hand_size(P0);
    cast_from_hand(&mut t, P1, "Lightning Bolt", &[Entity::Object(seeker)]);
    t.resolve();
    assert!(t.in_graveyard(P0, "Relic Seeker"));
    t.resolve_all();
    assert_eq!(t.stack_len(), 0);
    assert_eq!(
        t.hand_size(P0),
        hand,
        "no ability triggered on becoming renowned"
    );
    assert!(!t.obj_now(seeker).renowned);
    // Without the Bolt, Relic Seeker becomes renowned and both abilities trigger.
    let mut t = TestGame::new(2);
    let seeker = t.battlefield(P0, "Relic Seeker");
    t.battlefield(P0, "Valeron Wardens");
    let hand = t.hand_size(P0);
    t.attack(&[(seeker, Entity::Player(P1))], &[]);
    assert!(is_renowned(&t.g, seeker));
    assert_eq!(t.hand_size(P0), hand + 1);
}

#[test]
fn repartee_resolves_even_if_the_targeted_creature_leaves() {
    cr!("601.2i", "603.2", "113.7a");
    ruling!(
        "Stirring Hopesinger",
        "It doesn't matter if the creature or creatures targeted by the spell that caused the repartee ability to trigger leave the battlefield or stop being creatures before that spell resolves. As long as that spell targeted one or more creatures when it was cast, the repartee ability will trigger and later resolve."
    );
    supported("Stirring Hopesinger");
    // Stirring Hopesinger (1/3): "Repartee — ... put a +1/+1 counter on each creature you
    // control." P0 casts Shock at P1's Grizzly Bears; P1 returns the Bears to its hand
    // with Unsummon while the repartee trigger is on the stack.
    let mut t = TestGame::new(2);
    let singer = t.battlefield(P0, "Stirring Hopesinger");
    let bears = t.battlefield(P1, "Grizzly Bears");
    cast_from_hand(&mut t, P0, "Shock", &[Entity::Object(bears)]);
    t.settle();
    assert_eq!(triggers_from(&t, singer), 1);
    cast_from_hand(&mut t, P1, "Unsummon", &[Entity::Object(bears)]);
    t.resolve();
    assert!(t.in_hand(P1, "Grizzly Bears"));
    t.resolve_all();
    assert_eq!(plus1(&t, singer), 1);
    assert_eq!(t.pt(singer), (2, 4));
}

#[test]
fn repartee_resolves_first_and_even_if_the_spell_is_countered() {
    cr!("601.2i", "603.3", "701.6a");
    ruling!(
        "Lecturing Scornmage",
        "A repartee ability resolves before the spell that caused it to trigger. It resolves even if that spell is countered or otherwise leaves the stack."
    );
    supported("Lecturing Scornmage");
    // Lecturing Scornmage (1/1): "Repartee — ... put a +1/+1 counter on this creature."
    let mut t = TestGame::new(2);
    let mage = t.battlefield(P0, "Lecturing Scornmage");
    let bears = t.battlefield(P1, "Grizzly Bears");
    let shock = cast_from_hand(&mut t, P0, "Shock", &[Entity::Object(bears)]);
    t.settle();
    // The trigger is above the spell: it resolves first.
    assert_eq!(t.g.stack.first(), Some(&shock));
    assert_eq!(triggers_from(&t, mage), 1);
    t.resolve();
    assert_eq!(plus1(&t, mage), 1);
    assert!(t.g.stack.contains(&shock));
    // Again, with the spell countered while the trigger waits above it.
    let mut t = TestGame::new(2);
    let mage = t.battlefield(P0, "Lecturing Scornmage");
    let bears = t.battlefield(P1, "Grizzly Bears");
    let shock = cast_from_hand(&mut t, P0, "Shock", &[Entity::Object(bears)]);
    t.settle();
    cast_from_hand(&mut t, P1, "Counterspell", &[Entity::Object(shock)]);
    t.resolve();
    assert!(!t.g.stack.contains(&shock));
    t.resolve_all();
    assert_eq!(plus1(&t, mage), 1);
    assert!(t.on_battlefield(bears));
}
