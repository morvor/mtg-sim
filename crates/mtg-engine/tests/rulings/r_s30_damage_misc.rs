//! Rulings batch S30 — miscellaneous damage rulings: the source of a painland's damage,
//! damage marked on a creature whose power and toughness are switched, abilities that
//! deal damage after their source left, amounts determined as an ability resolves,
//! choices made during resolution, reflexive triggers, prowl, and combat damage dealt to
//! several players at once.

use crate::r_s01_common::supported;
use crate::r_s02_common::{can_cast, destroy};
use crate::r_s03_common::to_blockers;
use crate::r_s05_common::colorless;
use crate::r_s06_common::attach_new;
use crate::r_s20_common::tap_for_mana;
use crate::r_s24_common::choose_creature_type;
use crate::r_s25_common::cast_new;
use crate::r_s29_common::damage_marked;
use crate::r_s30_common::damage_events;
use mtg_engine::decision::{Answer, Decision};
use mtg_engine::keywords::KeywordKind;
use mtg_engine::object::CastMethod;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

#[test]
fn a_painlands_damage_is_dealt_by_a_colorless_source() {
    cr!("105.2c", "120.3a");
    ruling!(
        "Shivan Reef",
        "Like most lands, each land in this cycle is colorless. The damage dealt to you is dealt by a colorless source."
    );
    supported("Shivan Reef");
    let mut t = TestGame::new(2);
    // "{T}: Add {U} or {R}. This land deals 1 damage to you."
    let reef = t.battlefield(P0, "Shivan Reef");
    assert!(tap_for_mana(&mut t, P0, reef, "{U} or {R}"));
    t.settle();
    assert_eq!(t.life(P0), 19);
    assert_eq!(
        damage_events(&t),
        vec![(reef, Entity::Player(P0), 1, false)]
    );
    assert!(colorless(&t, reef));
}

#[test]
fn switching_power_and_toughness_can_make_marked_damage_lethal() {
    cr!("613.4d", "120.6", "704.5g");
    ruling!(
        "Aeromoeba",
        "Because damage remains marked on a creature until the cleanup step or an effect removes that damage, nonlethal damage dealt to a creature may become lethal if you switch its power and toughness during that turn."
    );
    supported("Aeromoeba");
    let mut t = TestGame::new(2);
    // Aeromoeba (2/4): "Discard a card: Switch this creature's power and toughness until
    // end of turn."
    let moeba = t.battlefield(P0, "Aeromoeba");
    cast_new(&mut t, P1, "Shock", &[Entity::Object(moeba)]);
    t.resolve_all();
    assert!(t.on_battlefield(moeba));
    assert_eq!(damage_marked(&t, moeba), 2);
    let card = t.hand(P0, "Grizzly Bears");
    t.answer_choose(P0, &[Entity::Object(card)]);
    t.activate(P0, moeba, 0, &[]).unwrap();
    t.resolve_all();
    // A 4/2 with 2 damage marked on it.
    assert!(t.in_graveyard(P0, "Aeromoeba"));
}

#[test]
fn karplusan_yetis_ability_resolves_without_it_but_not_without_its_target() {
    cr!("113.7a", "608.2b");
    ruling!(
        "Karplusan Yeti",
        "If this leaves the battlefield before its activated ability resolves, it will still deal damage to the targeted creature. On the other hand, if the targeted creature leaves the battlefield before the ability resolves, the ability won't resolve and no damage will be dealt."
    );
    supported("Karplusan Yeti");
    // "{T}: This creature deals damage equal to its power to target creature. That
    // creature deals damage equal to its power to this creature."
    // The Yeti (3/3) leaves: it still deals 3 damage (its last known power).
    let mut t = TestGame::new(2);
    let yeti = t.battlefield(P0, "Karplusan Yeti");
    let giant = t.battlefield(P1, "Hill Giant");
    t.activate(P0, yeti, 0, &[Entity::Object(giant)]).unwrap();
    cast_new(&mut t, P1, "Unsummon", &[Entity::Object(yeti)]);
    t.resolve_all();
    assert!(t.in_hand(P0, "Karplusan Yeti"));
    assert!(t.in_graveyard(P1, "Hill Giant"));
    // The target leaves: no damage is dealt at all.
    let mut t = TestGame::new(2);
    let yeti = t.battlefield(P0, "Karplusan Yeti");
    let giant = t.battlefield(P1, "Hill Giant");
    t.activate(P0, yeti, 0, &[Entity::Object(giant)]).unwrap();
    cast_new(&mut t, P1, "Unsummon", &[Entity::Object(giant)]);
    t.resolve_all();
    assert!(t.in_hand(P1, "Hill Giant"));
    assert_eq!(damage_marked(&t, yeti), 0);
    assert!(damage_events(&t).is_empty());
}

#[test]
fn dragon_tempest_counts_dragons_as_its_ability_resolves() {
    cr!("608.2h");
    ruling!(
        "Dragon Tempest",
        "The amount of damage dealt by the Dragon that entered the battlefield is based on the number of Dragons you control when the ability resolves."
    );
    supported("Dragon Tempest");
    let mut t = TestGame::new(2);
    // "Whenever a Dragon you control enters, it deals X damage to any target, where X is
    // the number of Dragons you control."
    t.battlefield(P0, "Dragon Tempest");
    let shivan = t.battlefield(P0, "Shivan Dragon");
    t.answer_targets(P0, &[Entity::Player(P1)]);
    t.enter(P0, "Furnace Whelp");
    t.settle();
    // Two Dragons as the ability triggered; one is destroyed in response.
    destroy(&mut t, shivan);
    t.resolve_all();
    assert_eq!(t.life(P1), 19);
}

#[test]
fn the_creature_type_is_chosen_after_players_could_respond() {
    cr!("608.2", "117.3c");
    ruling!(
        "Roar of the Crowd",
        "Spells and abilities that prevent damage or regenerate the targeted creature must be cast or activated before the creature type is chosen."
    );
    supported("Roar of the Crowd");
    let mut t = TestGame::new(2);
    for _ in 0..3 {
        t.battlefield(P0, "Grizzly Bears");
    }
    let giant = t.battlefield(P1, "Hill Giant");
    // "Choose a creature type. Roar of the Crowd deals damage to any target equal to the
    // number of permanents you control of the chosen type."
    choose_creature_type(&mut t, P0, "Bear");
    let from = t.asked().len();
    cast_new(&mut t, P0, "Roar of the Crowd", &[Entity::Object(giant)]);
    let cast_end = t.asked().len();
    t.answer(P0, DecisionKind::Priority, Answer::Action(Action::Pass));
    t.g.run_until(100, |g| g.stack.is_empty());
    t.settle();
    let asked = t.asked();
    let is_type_choice = |d: &Decision| matches!(d, Decision::ChooseOption { .. });
    // No type was chosen while casting the spell.
    assert!(!asked[from..cast_end].iter().any(|(_, d)| is_type_choice(d)));
    // P1's last chance to respond (priority) came before the type was chosen.
    let p1_priority = asked[cast_end..]
        .iter()
        .position(|(p, d)| *p == P1 && matches!(d, Decision::Priority { .. }))
        .expect("P1 got priority");
    let choice = asked[cast_end..]
        .iter()
        .position(|(p, d)| *p == P0 && is_type_choice(d))
        .expect("the type was chosen");
    assert!(p1_priority < choice);
    assert!(t.in_graveyard(P1, "Hill Giant"));
}

#[test]
fn heart_piercer_manticore_sacrifices_only_one_creature() {
    cr!("603.12");
    ruling!(
        "Heart-Piercer Manticore",
        "You can't sacrifice multiple creatures to deal damage multiple times."
    );
    supported("Heart-Piercer Manticore");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let giant = t.battlefield(P0, "Hill Giant");
    // "When this creature enters, you may sacrifice another creature. When you do, this
    // creature deals damage equal to that creature's power to any target."
    t.answer_yes(P0, true);
    // An attempt to sacrifice both.
    t.answer_choose(P0, &[Entity::Object(giant), Entity::Object(bears)]);
    t.answer_targets(P0, &[Entity::Player(P1)]);
    t.enter(P0, "Heart-Piercer Manticore");
    t.resolve_all();
    let sacrificed = [bears, giant]
        .iter()
        .filter(|c| !t.on_battlefield(**c))
        .count();
    assert_eq!(sacrificed, 1);
    // One reflexive trigger, dealing damage once.
    let to_p1: Vec<u32> = damage_events(&t)
        .iter()
        .filter(|(_, e, _, _)| *e == Entity::Player(P1))
        .map(|(_, _, n, _)| *n)
        .collect();
    assert_eq!(to_p1.len(), 1);
    assert!(to_p1[0] == 2 || to_p1[0] == 3);
}

#[test]
fn prowl_stays_available_after_the_creature_and_the_player_are_gone() {
    cr!("702.76a");
    ruling!(
        "Hunting Velociraptor",
        "You can cast a spell for its prowl cost any time in a turn after a creature you control of a matching type has dealt combat damage to a player. It doesn't matter if that player left the game, if that creature left the battlefield or left your control, or if that creature no longer has a matching type."
    );
    supported("Hunting Velociraptor");
    const PROWL: CastMethod = CastMethod::Keyword(KeywordKind::Prowl);
    let mut t = TestGame::new(3);
    // "Dinosaur spells you cast have prowl {2}{R}."
    t.battlefield(P0, "Hunting Velociraptor");
    let dreadmaw = t.battlefield(P0, "Colossal Dreadmaw");
    t.lands(P0, "Mountain", 3);
    let card = t.hand(P0, "Colossal Dreadmaw");
    t.attack(&[(dreadmaw, Entity::Player(P1))], &[]);
    t.advance_to(P0, Step::PostcombatMain);
    assert_eq!(t.life(P1), 14);
    // The Dinosaur that dealt the damage leaves the battlefield, and the player it dealt
    // damage to leaves the game.
    destroy(&mut t, dreadmaw);
    t.g.lose_game(P1);
    t.settle();
    assert!(t.in_graveyard(P0, "Colossal Dreadmaw"));
    assert!(!t.g.player(P1).in_game());
    assert!(can_cast(&mut t, P0, card, PROWL));
    t.cast(P0, card).method(PROWL).go();
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Colossal Dreadmaw").len(), 1);
}

#[test]
fn combat_damage_to_two_players_at_once_triggers_for_each() {
    cr!("603.2c", "510.2", "614.9");
    ruling!(
        "Erdwal Ripper",
        "If this creature deals combat damage to multiple players simultaneously, perhaps because some combat damage was redirected, its ability will trigger for each of those players."
    );
    supported("Erdwal Ripper");
    supported("Sivvi's Valor");
    let mut t = TestGame::new(3);
    // Erdwal Ripper (2/1): "Whenever this creature deals combat damage to a player, put a
    // +1/+1 counter on it." With Rancor ("+2/+0 and has trample"): 4/1 trample.
    let ripper = t.battlefield(P0, "Erdwal Ripper");
    attach_new(&mut t, P0, "Rancor", ripper);
    // Ornithopter: a 0/2 blocker.
    let thopter = t.battlefield(P1, "Ornithopter");
    to_blockers(&mut t, &[(ripper, Entity::Player(P1))], &[(thopter, ripper)]);
    // P2's Sivvi's Valor: "All damage that would be dealt to target creature this turn is
    // dealt to you instead."
    t.lands(P2, "Plains", 1);
    t.lands(P2, "Wastes", 2);
    let valor = t.hand(P2, "Sivvi's Valor");
    t.cast_with(P2, valor, &[Entity::Object(thopter)]).unwrap();
    t.resolve_all();
    // 2 damage assigned to Ornithopter (dealt to P2) and 2 to P1, at the same time.
    t.advance_to(P0, Step::EndOfCombat);
    t.resolve_all();
    assert_eq!(t.life(P1), 18);
    assert_eq!(t.life(P2), 18);
    assert_eq!(t.counters(ripper, "+1/+1"), 2);
}
