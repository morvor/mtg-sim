//! Rulings batch S06 — enchant player: the Curses of Innistrad (CR 303.4, 702.5). An Aura
//! with "enchant player" targets the player it will enchant and stays attached to that
//! player like any other Aura.

use crate::r_s01_common::*;
use crate::r_s02_common::target_candidates;
use crate::r_s03_common::in_hand_with_mana;
use crate::r_s04_common::*;
use crate::r_s06_common::*;
use mtg_engine::decision::Answer;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

#[test]
fn a_curse_can_enchant_the_player_who_cast_it() {
    cr!("303.4a", "303.4d", "115.1");
    ruling!(
        "Curse of the Forsaken",
        "Each of the Curses can be attached to any player, including the player who cast the Curse."
    );
    supported("Curse of the Forsaken");
    // P0 enchants themself: "Whenever a creature attacks enchanted player, its controller
    // gains 1 life."
    let mut t = TestGame::new(2);
    let curse = in_hand_with_mana(&mut t, P0, "Curse of the Forsaken");
    let from = t.asked().len();
    t.cast(P0, curse).target(Entity::Player(P0)).go();
    let targets = target_candidates(&t, P0, from);
    assert!(targets[0].contains(&Entity::Player(P0)));
    assert!(targets[0].contains(&Entity::Player(P1)));
    t.resolve();
    let curse = t.named_on_battlefield("Curse of the Forsaken")[0];
    assert_eq!(t.g.obj(curse).attached_to, Some(Entity::Player(P0)));
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.set_step(P1, Step::PrecombatMain);
    attack_with(&mut t, &[(bears, Entity::Player(P0))]);
    t.resolve_all();
    assert_eq!(t.life(P1), 21);
}

#[test]
fn curse_is_an_enchantment_type_not_a_creature_type() {
    cr!("205.3h", "205.3m", "702.73a");
    ruling!(
        "Curse of Predation",
        "Curse is an enchantment type, not a creature type (or any other kind of subtype)."
    );
    supported("Curse of Predation");
    assert_eq!(subtype_kinds("Curse"), vec![SubtypeKind::Enchantment]);
    assert!(!is_creature_type("Curse"));
    let mut t = TestGame::new(2);
    let curse = t.battlefield(P0, "Curse of Predation");
    assert!(t.g.obj(curse).chars.has_subtype("Curse"));
    assert!(t.g.obj(curse).chars.has_subtype("Aura"));
    // A creature with every creature type (changeling) isn't a Curse.
    let colossus = t.battlefield(P0, "Chameleon Colossus");
    assert!(t.g.obj(colossus).chars.has_subtype("Elf"));
    assert!(!t.g.obj(colossus).chars.has_subtype("Curse"));
}

#[test]
fn a_curse_targets_its_player_and_falls_off_if_that_player_gains_protection_from_it() {
    cr!("303.4d", "702.16c", "702.16k", "704.5m");
    ruling!(
        "Curse of Predation",
        "A Curse spell targets the player it will enchant like any other Aura spell, and a Curse stays on the battlefield like any other Aura. If the enchanted player gains protection from the Curse’s color (or any other characteristic the Curse has), the Curse will be put into its owner’s graveyard."
    );
    supported("Runed Halo");
    // P1 is enchanted by Curse of Predation, then gets protection from its name.
    let mut t = TestGame::new(2);
    let curse = in_hand_with_mana(&mut t, P0, "Curse of Predation");
    t.cast(P0, curse).target(Entity::Player(P1)).go();
    t.resolve();
    let curse = t.named_on_battlefield("Curse of Predation")[0];
    assert_eq!(t.g.obj(curse).attached_to, Some(Entity::Player(P1)));
    t.answer(
        P1,
        DecisionKind::Name,
        Answer::Text("Curse of Predation".into()),
    );
    t.enter(P1, "Runed Halo");
    t.g.flush_events();
    t.settle();
    assert!(t.in_graveyard(P0, "Curse of Predation"));
    // A Curse spell can't target a player with protection from it.
    let curse = in_hand_with_mana(&mut t, P0, "Curse of Predation");
    let from = t.asked().len();
    t.cast(P0, curse).target(Entity::Player(P0)).go();
    let targets = target_candidates(&t, P0, from);
    assert!(targets[0].contains(&Entity::Player(P0)));
    assert!(!targets[0].contains(&Entity::Player(P1)));
}

#[test]
fn curse_of_predation_doesnt_trigger_for_attacks_on_planeswalkers() {
    cr!("506.2", "508.1b", "603.2");
    ruling!(
        "Curse of Predation",
        "The ability won’t trigger when a creature attacks a planeswalker controlled by the enchanted player."
    );
    // "Whenever a creature attacks enchanted player, put a +1/+1 counter on it."
    let mut t = TestGame::new(2);
    attach_new(&mut t, P0, "Curse of Predation", Entity::Player(P1));
    let lili = t.battlefield(P1, "Liliana of the Veil");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let giant = t.battlefield(P0, "Hill Giant");
    attack_with(
        &mut t,
        &[
            (bears, Entity::Object(lili)),
            (giant, Entity::Player(P1)),
        ],
    );
    assert_eq!(on_stack(&t, "+1/+1 counter"), 1);
    t.resolve_all();
    assert_eq!(t.counters(bears, "+1/+1"), 0);
    assert_eq!(t.counters(giant, "+1/+1"), 1);
}

#[test]
fn curse_of_the_forsaken_doesnt_trigger_for_attacks_on_planeswalkers() {
    cr!("506.2", "508.1b");
    ruling!(
        "Curse of the Forsaken",
        "The ability won’t trigger when a creature attacks a planeswalker controlled by the enchanted player."
    );
    // "Whenever a creature attacks enchanted player, its controller gains 1 life."
    let mut t = TestGame::new(2);
    attach_new(&mut t, P0, "Curse of the Forsaken", Entity::Player(P1));
    let lili = t.battlefield(P1, "Liliana of the Veil");
    let bears = t.battlefield(P0, "Grizzly Bears");
    attack_with(&mut t, &[(bears, Entity::Object(lili))]);
    t.resolve_all();
    assert_eq!(t.life(P0), 20);
}
