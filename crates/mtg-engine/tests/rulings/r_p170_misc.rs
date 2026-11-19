//! Rulings batch P170 — the rest of the noncreature-spell payoffs: blocks that stand
//! (CR 509.1h, 506.4), damage replacements and their sources (CR 614.1a, 120.3),
//! characteristic-changing effects that keep what they don't mention (CR 613.1d, 205.1b),
//! card types (CR 205.2a), control exchanges (CR 701.12b), "may pay" on resolution
//! (CR 608.2c, 118.12), last known information (CR 608.2h), reflexive triggers
//! (CR 603.12), and mana abilities (CR 605.3).

use crate::r_p076_common::is_blocked;
use crate::r_p170_common::*;
use crate::r_s01_common::attack_with;
use crate::r_s06_common::give_control;
use mtg_engine::decision::{Answer, Decision};
use mtg_engine::events::Event;
use mtg_engine::keywords::KeywordKind;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

/// P0's `attacker` attacks P1 and P1 blocks it with `blocker`; stops in the declare
/// blockers step with P0 holding priority.
fn blocked_by(t: &mut TestGame, attacker: ObjectId, blocker: ObjectId) {
    attack_with(t, &[(attacker, pl(P1))]);
    t.answer(
        P1,
        DecisionKind::Blockers,
        Answer::Blockers(vec![(blocker, attacker)]),
    );
    t.advance_to(P0, Step::DeclareBlockers);
    t.settle();
    assert!(is_blocked(t, attacker));
}

#[test]
fn becoming_unblockable_after_blocks_doesnt_undo_them() {
    cr!("509.1h", "506.4a", "509.1b");
    ruling!(
        "Ichor Synthesizer",
        "Putting a fourth oil counter on Ichor Synthesizer after it has become blocked will not cause it to become unblocked."
    );
    ruling!(
        "Elusive Otter // Grove's Bounty",
        "You compare Elusive Otter's power to the power of any creature trying to block it only as blockers are assigned. Once Elusive Otter has been legally blocked by a creature, changing the power of either creature won't change or undo the block."
    );
    supported("Ichor Synthesizer");
    supported("Elusive Otter // Grove's Bounty");
    // Ichor Synthesizer with three oil counters is blocked; Opt gives it the fourth.
    let mut t = TestGame::new(2);
    let syn = t.battlefield(P0, "Ichor Synthesizer");
    add_counters(&mut t, syn, "oil", 3);
    let bears = t.battlefield(P1, "Grizzly Bears");
    blocked_by(&mut t, syn, bears);
    cast_card(&mut t, P0, "Opt", &[]);
    t.resolve_all();
    assert_eq!(t.counters(syn, "oil"), 4);
    assert_eq!(t.pt(syn), (3, 3));
    assert!(is_blocked(&t, syn));
    t.advance_to(P0, Step::EndOfCombat);
    assert_eq!(t.life(P1), 20);
    assert!(t.in_graveyard(P1, "Grizzly Bears"));
    // Elusive Otter (1/1) blocked by a 2/2 Memnite; then the Otter grows (5/5) so the
    // Memnite's power is less than its: still blocked.
    let mut t = TestGame::new(2);
    let otter = t.battlefield(P0, "Elusive Otter // Grove's Bounty");
    let thopter = t.battlefield(P1, "Memnite");
    add_counters(&mut t, thopter, "+1/+1", 1);
    blocked_by(&mut t, otter, thopter);
    cast_card(&mut t, P0, "Giant Growth", &[obj(otter)]);
    t.resolve_all();
    assert_eq!(t.pt(otter), (5, 5), "prowess and Giant Growth");
    assert!(is_blocked(&t, otter));
    t.advance_to(P0, Step::EndOfCombat);
    assert_eq!(t.life(P1), 20);
}

#[test]
fn soul_scar_mage_replaces_damage_that_cant_be_prevented() {
    cr!("614.1a", "615.12", "122.1a");
    ruling!(
        "Soul-Scar Mage",
        "Soul-Scar Mage’s effect isn’t a prevention effect. Damage that can’t be prevented will be replaced with -1/-1 counters."
    );
    supported("Soul-Scar Mage");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Soul-Scar Mage");
    let giant = t.battlefield(P1, "Hill Giant");
    // Skullcrack: "Damage can't be prevented this turn." 3 to P1.
    cast_card(&mut t, P0, "Skullcrack", &[pl(P1)]);
    t.resolve_all();
    assert_eq!(t.life(P1), 17);
    cast_card(&mut t, P0, "Shock", &[obj(giant)]);
    t.resolve_all();
    assert_eq!(t.counters(giant, "-1/-1"), 2);
    assert_eq!(t.obj_now(giant).damage, 0);
    assert_eq!(t.pt(giant), (1, 1));
}

/// The damage events since `from`: (source, target, amount).
fn damage_since(t: &TestGame, from: usize) -> Vec<(ObjectId, Entity, u32)> {
    t.g.turn_events[from..]
        .iter()
        .filter_map(|e| match e {
            Event::Damage {
                source,
                target,
                amount,
                ..
            } => Some((*source, *target, *amount)),
            _ => None,
        })
        .collect()
}

#[test]
fn the_additional_damage_is_dealt_by_the_original_source() {
    cr!("614.1a", "120.3");
    ruling!(
        "Artist's Talent",
        "The additional 2 damage is dealt by the same source as the original source of damage. The damage isn't dealt by Artist's Talent unless Artist's Talent is somehow the original source of that damage."
    );
    ruling!(
        "Invasion of Regatha // Disciples of the Inferno",
        "The additional 2 damage is dealt by the same source as the original source of damage. The damage isn’t dealt by Disciples of the Inferno."
    );
    supported("Artist's Talent");
    supported("Invasion of Regatha // Disciples of the Inferno");
    // Artist's Talent at level 3: P0's Shock deals 4 to P1, all of it by the Shock.
    let mut t = TestGame::new(2);
    let talent = t.battlefield(P0, "Artist's Talent");
    mtg_engine::classes::set_level(&mut t.g, talent, 3);
    t.g.flush_events();
    let from = t.g.turn_events.len();
    let shock = cast_card(&mut t, P0, "Shock", &[pl(P1)]);
    t.resolve_all();
    assert_eq!(damage_since(&t, from), vec![(shock, pl(P1), 4)]);
    assert_eq!(t.life(P1), 16);
    // Disciples of the Inferno (the battle transformed): the same, for a noncreature source.
    let mut t = TestGame::new(2);
    let inv = t.battlefield(P0, "Invasion of Regatha // Disciples of the Inferno");
    assert!(mtg_engine::dfc::transform(&mut t.g, inv));
    t.g.flush_events();
    t.settle();
    assert_eq!(t.obj_now(inv).chars.name, "Disciples of the Inferno");
    let giant = t.battlefield(P1, "Craw Wurm");
    let from = t.g.turn_events.len();
    let shock = cast_card(&mut t, P0, "Shock", &[obj(giant)]);
    t.resolve_all();
    assert_eq!(damage_since(&t, from), vec![(shock, obj(giant), 4)]);
}

#[test]
fn the_mechanists_construct_keeps_its_abilities_and_noncreature_types() {
    cr!("613.1d", "205.1b", "613.1f");
    ruling!(
        "The Mechanist, Aerial Artisan",
        "The Mechanist, Aerial Artisan's activated ability doesn't remove any abilities the target artifact token has. The artifact token also retains any types, subtypes, and supertypes other than creature types."
    );
    supported("The Mechanist, Aerial Artisan");
    // A Clue from its own trigger: a 3/1 Construct artifact creature with flying that's
    // still a Clue with its sacrifice ability.
    let mut t = TestGame::new(2);
    let mech = t.battlefield(P0, "The Mechanist, Aerial Artisan");
    cast_card(&mut t, P0, "Opt", &[]);
    t.resolve_all();
    let clue = tokens_with_subtype(&t, P0, "Clue")[0];
    t.activate(P0, mech, 0, &[obj(clue)]).unwrap();
    t.resolve_all();
    let c = t.obj_now(clue);
    assert!(c.is_creature() && c.is(CardType::Artifact));
    assert!(c.chars.has_subtype("Clue") && c.chars.has_subtype("Construct"));
    assert!(c.has_keyword(KeywordKind::Flying));
    assert_eq!(t.pt(clue), (3, 1));
    t.lands(P0, "Wastes", 2);
    crate::r_s06_common::activate_containing(&mut t, P0, clue, "Draw a card")
        .expect("the Clue keeps its ability");
    t.resolve_all();
    assert!(!t.on_battlefield(clue));
    // A Thopter token: its creature type Thopter is replaced by Construct.
    let mut t = TestGame::new(2);
    let mech = t.battlefield(P0, "The Mechanist, Aerial Artisan");
    t.enter(P0, "Whirler Rogue");
    t.resolve_all();
    let thopter = tokens_with_subtype(&t, P0, "Thopter")[0];
    t.activate(P0, mech, 0, &[obj(thopter)]).unwrap();
    t.resolve_all();
    let o = t.obj_now(thopter);
    assert!(o.chars.has_subtype("Construct"));
    assert!(!o.chars.has_subtype("Thopter"));
    assert_eq!(t.pt(thopter), (3, 1));
}

#[test]
fn quiet_contemplation_taps_and_tracks_the_creature() {
    cr!("608.2c", "118.12", "502.3", "611.2c");
    ruling!(
        "Quiet Contemplation",
        "You decide whether to pay {1} as the triggered ability resolves."
    );
    ruling!(
        "Quiet Contemplation",
        "The ability can target a creature that’s already tapped. It still won’t untap during its controller’s next untap step."
    );
    ruling!(
        "Quiet Contemplation",
        "The ability tracks the creature, but not its controller. If the creature changes controllers before its first controller’s next untap step has come around, then it won’t untap during its new controller’s next untap step."
    );
    supported("Quiet Contemplation");
    // An already tapped Hill Giant; the mana for {1} arrives after the trigger is on the
    // stack, and P0 is asked as it resolves.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Quiet Contemplation");
    let giant = t.battlefield(P1, "Hill Giant");
    tap(&mut t, giant);
    t.answer_targets(P0, &[obj(giant)]);
    let from = t.asked().len();
    cast_card(&mut t, P0, "Opt", &[]);
    assert!(trigger_on_top(&t));
    let yes_nos = |t: &TestGame, from: usize| {
        t.asked()[from..]
            .iter()
            .filter(|(_, d)| matches!(d, Decision::YesNo { .. } | Decision::OptionalCost { .. }))
            .count()
    };
    assert_eq!(yes_nos(&t, from), 0, "not asked yet");
    let wastes = t.lands(P0, "Wastes", 1)[0];
    t.answer_yes(P0, true);
    t.resolve();
    assert_eq!(yes_nos(&t, from), 1);
    assert!(t.obj_now(wastes).tapped, "{{1}} paid");
    t.advance_to(P1, Step::Upkeep);
    assert!(t.obj_now(giant).tapped, "it didn't untap");
    t.advance_to(P1, Step::End);
    t.advance_to(P0, Step::Upkeep);
    t.advance_to(P1, Step::Upkeep);
    assert!(!t.obj_now(giant).tapped, "only the next untap step");
    // P0 gains control of it first: it doesn't untap in P0's next untap step either.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Quiet Contemplation");
    let giant = t.battlefield(P1, "Hill Giant");
    t.lands(P0, "Wastes", 1);
    t.answer_targets(P0, &[obj(giant)]);
    t.answer_yes(P0, true);
    cast_card(&mut t, P0, "Opt", &[]);
    t.resolve_all();
    assert!(t.obj_now(giant).tapped);
    give_control(&mut t, giant, P0);
    assert_eq!(t.obj_now(giant).controller, P0);
    t.advance_to(P1, Step::Upkeep);
    t.advance_to(P0, Step::Upkeep);
    assert!(t.obj_now(giant).tapped, "not untapped in its new controller's untap step");
    t.advance_to(P1, Step::Upkeep);
    t.advance_to(P0, Step::Upkeep);
    assert!(!t.obj_now(giant).tapped);
}

#[test]
fn may_pay_triggers_pay_once_as_they_resolve() {
    cr!("608.2c", "118.12", "603.5");
    ruling!(
        "Goblinslide",
        "You make the decision whether to pay {1} as the ability resolves."
    );
    ruling!(
        "Skywise Teachings",
        "While resolving the triggered ability of Skywise Teachings, you can’t pay {1}{U} multiple times to create more tokens."
    );
    // (enchantment, token subtype, lands for one payment)
    for (ench, sub) in [("Goblinslide", "Goblin"), ("Skywise Teachings", "Djinn")] {
        supported(ench);
        let mut t = TestGame::new(2);
        t.battlefield(P0, ench);
        let from = t.asked().len();
        cast_card(&mut t, P0, "Opt", &[]);
        assert!(trigger_on_top(&t));
        assert!(
            !t.asked()[from..]
                .iter()
                .any(|(_, d)| matches!(d, Decision::YesNo { .. })),
            "{ench}: asked only on resolution"
        );
        // Plenty of mana, added after the trigger: still one token.
        t.lands(P0, "Island", 6);
        t.answer_yes(P0, true);
        t.answer_yes(P0, true);
        t.resolve();
        assert_eq!(tokens_with_subtype(&t, P0, sub).len(), 1, "{ench}");
        t.resolve_all();
        assert_eq!(tokens_with_subtype(&t, P0, sub).len(), 1, "{ench}");
    }
}

#[test]
fn noncreature_card_types_and_graveyard_card_types() {
    cr!("205.2a", "205.4a", "205.3a");
    ruling!(
        "Dragon's Rage Channeler",
        "The card types that can appear in a graveyard are artifact, battle, creature, enchantment, instant, kindred, land, planeswalker, and sorcery. Legendary, basic, and snow are supertypes, not card types; Human and Shaman are subtypes, not card types."
    );
    ruling!(
        "Hurkyl, Master Wizard",
        "The card types that a noncreature spell can have are artifact, battle, enchantment, instant, kindred, planeswalker, and sorcery. Subtypes, like Human and Equipment, and supertypes, like snow and legendary, aren't card types."
    );
    supported("Dragon's Rage Channeler");
    supported("Hurkyl, Master Wizard");
    // Delirium: a basic snow land, a legendary creature and an instant are three card
    // types; a battle card is the fourth.
    let mut t = TestGame::new(2);
    let drc = t.battlefield(P0, "Dragon's Rage Channeler");
    for c in ["Snow-Covered Forest", "Forest", "Kitsa, Otterball Elite", "Grizzly Bears", "Opt"] {
        t.graveyard(P0, c);
    }
    assert_eq!(t.pt(drc), (1, 1));
    t.graveyard(P0, "Invasion of Regatha // Disciples of the Inferno");
    assert_eq!(t.pt(drc), (3, 3));
    assert!(t.obj_now(drc).has_keyword(KeywordKind::Flying));
    // Hurkyl: Opt (instant) and Bonesplitter (artifact — Equipment) cast this turn; of
    // the revealed cards an instant and an artifact may be taken, not a sorcery or a
    // creature.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Hurkyl, Master Wizard");
    cast_card(&mut t, P0, "Opt", &[]);
    t.resolve_all();
    cast_card(&mut t, P0, "Bonesplitter", &[]);
    t.resolve_all();
    let top = crate::r_s01_common::stack_library(
        &mut t,
        P0,
        &["Lightning Bolt", "Divination", "Ornithopter", "Grizzly Bears", "Forest"],
    );
    let from = t.asked().len();
    t.advance_to(P0, Step::End);
    t.resolve_all();
    let offered: Vec<Entity> = t.asked()[from..]
        .iter()
        .filter_map(|(_, d)| match d {
            Decision::ChooseEntities { candidates, .. } => Some(candidates.clone()),
            _ => None,
        })
        .flatten()
        .collect();
    let offered_name = |n: usize| offered.iter().any(|e| *e == obj(top[n]));
    assert!(offered_name(0), "an instant: {offered:?}");
    assert!(offered_name(2), "an artifact");
    assert!(!offered_name(1), "not a sorcery");
    assert!(!offered_name(3), "not a creature");
    assert!(!offered_name(4), "not a land");
}

#[test]
fn sudden_substitutions_exchange_lasts_indefinitely() {
    cr!("701.12b", "611.2a", "702.61a");
    ruling!(
        "Sudden Substitution",
        "The exchange lasts indefinitely. It doesn't expire for the spell if the creature changes zones or leaves the game, or vice versa."
    );
    supported("Sudden Substitution");
    // P1 casts Glorious Anthem; P0 exchanges it with P0's Grizzly Bears.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let lions = t.battlefield(P0, "Savannah Lions");
    t.set_step(P1, Step::PrecombatMain);
    let anthem = cast_card(&mut t, P1, "Glorious Anthem", &[]);
    cast_card(&mut t, P0, "Sudden Substitution", &[obj(anthem), obj(bears)]);
    t.resolve();
    assert_eq!(t.obj_now(anthem).controller, P0);
    assert_eq!(t.obj_now(bears).controller, P1);
    // The spell resolves (it changes zones): P1 keeps the Bears; P0's Anthem.
    t.resolve_all();
    let anthem = t.named_on_battlefield("Glorious Anthem")[0];
    assert_eq!(t.obj_now(anthem).controller, P0);
    assert_eq!(t.pt(lions), (3, 2));
    // Then the Bears leave the battlefield: P0 keeps the Anthem.
    crate::r_s02_common::destroy(&mut t, bears);
    t.advance_to(P1, Step::Upkeep);
    t.advance_to(P0, Step::Upkeep);
    assert_eq!(t.obj_now(anthem).controller, P0);
    // Vice versa: the Bears stay with P1 across turns after the spell resolved.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.set_step(P1, Step::PrecombatMain);
    let anthem = cast_card(&mut t, P1, "Glorious Anthem", &[]);
    cast_card(&mut t, P0, "Sudden Substitution", &[obj(anthem), obj(bears)]);
    t.resolve_all();
    t.advance_to(P1, Step::Upkeep);
    t.advance_to(P0, Step::Upkeep);
    assert_eq!(t.obj_now(bears).controller, P1);
}

#[test]
fn goblin_wizardry_tokens_are_named_goblin_wizard_token() {
    cr!("111.4");
    ruling!(
        "Goblin Wizardry",
        "The tokens are named \"Goblin Wizard Token\" and have the creature types \"Goblin\" and \"Wizard.\""
    );
    supported("Goblin Wizardry");
    let mut t = TestGame::new(2);
    cast_card(&mut t, P0, "Goblin Wizardry", &[]);
    t.resolve_all();
    let toks = tokens_with_subtype(&t, P0, "Goblin");
    assert_eq!(toks.len(), 2);
    for tk in toks {
        let o = t.obj_now(tk);
        assert_eq!(o.chars.name, "Goblin Wizard Token");
        assert!(o.chars.has_subtype("Wizard"));
        assert!(o.has_keyword(KeywordKind::Prowess));
    }
}

#[test]
fn rediscover_the_ways_chapter_three_trigger_works_all_turn() {
    cr!("603.7a", "603.7c", "714.4");
    ruling!(
        "Rediscover the Way",
        "The triggered ability created by the chapter III ability may trigger multiple times during the turn, even though Rediscover the Way will likely no longer be on the battlefield."
    );
    supported("Rediscover the Way");
    let mut t = TestGame::new(2);
    let saga = t.battlefield(P0, "Rediscover the Way");
    add_counters(&mut t, saga, "lore", 3);
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Rediscover the Way"));
    let bears = t.battlefield(P0, "Grizzly Bears");
    let giant = t.battlefield(P0, "Hill Giant");
    for target in [bears, giant] {
        t.answer_targets(P0, &[obj(target)]);
        cast_card(&mut t, P0, "Opt", &[]);
        t.resolve_all();
        assert!(t.obj_now(target).has_keyword(KeywordKind::DoubleStrike));
    }
}

#[test]
fn power_as_it_last_existed_on_the_battlefield() {
    cr!("608.2h", "113.7a", "603.12");
    ruling!(
        "Blazing Bomb",
        "Use Blazing Bomb's power as it last existed on the battlefield to determine how much damage its last ability deals."
    );
    ruling!(
        "Cyclops Superconductor",
        "Use the power of Cyclops Superconductor as it last existed on the battlefield to determine the amount of damage Cyclops Superconductor deals due to its reflexive triggered ability."
    );
    ruling!(
        "Cyclops Superconductor",
        "You don't choose a target for Cyclops Superconductor's last ability at the time it triggers. Rather, a second \"reflexive\" ability triggers when you pay {E}{E}{E} this way. You choose a target for that ability as it goes on the stack. Each player may respond to this triggered ability as normal."
    );
    supported("Blazing Bomb");
    supported("Cyclops Superconductor");
    // Blazing Bomb with two +1/+1 counters (3/3) is sacrificed: 3 damage.
    let mut t = TestGame::new(2);
    let bomb = t.battlefield(P0, "Blazing Bomb");
    add_counters(&mut t, bomb, "+1/+1", 2);
    let giant = t.battlefield(P1, "Hill Giant");
    t.activate(P0, bomb, 0, &[obj(giant)]).unwrap();
    assert!(!t.on_battlefield(bomb));
    t.resolve_all();
    assert!(t.in_graveyard(P1, "Hill Giant"));
    // Cyclops Superconductor (with three energy) with a +1/+1 counter dies: the
    // reflexive trigger, targeted as it's put on the stack, deals 3.
    let mut t = TestGame::new(2);
    let cy = t.enter(P0, "Cyclops Superconductor");
    t.resolve_all();
    assert_eq!(crate::r_p015_common::energy(&t, P0), 3);
    add_counters(&mut t, cy, "+1/+1", 1);
    let from = t.asked().len();
    t.answer_yes(P0, true);
    crate::r_s02_common::destroy(&mut t, cy);
    // The dies trigger: no target chosen yet.
    assert!(trigger_on_top(&t));
    assert!(!t.asked()[from..]
        .iter()
        .any(|(_, d)| matches!(d, Decision::ChooseTargets { .. })));
    t.answer_targets(P0, &[pl(P1)]);
    t.resolve();
    assert_eq!(crate::r_p015_common::energy(&t, P0), 0);
    // The reflexive trigger is on the stack, targeting P1; players may respond.
    assert!(trigger_on_top(&t));
    assert!(t.asked()[from..]
        .iter()
        .any(|(_, d)| matches!(d, Decision::ChooseTargets { .. })));
    assert_eq!(t.life(P1), 20);
    t.resolve_all();
    assert_eq!(t.life(P1), 17);
}

#[test]
fn esper_ramuh_counts_the_graveyard_as_chapter_one_resolves() {
    cr!("608.2h", "714.2b");
    ruling!(
        "Summon: Esper Ramuh",
        "Use the number of noncreature, nonland cards in your graveyard as Summon: Esper Ramuh's first chapter ability resolves to determine how much damage is dealt."
    );
    supported("Summon: Esper Ramuh");
    let mut t = TestGame::new(2);
    t.graveyard(P0, "Divination");
    t.graveyard(P0, "Grizzly Bears");
    t.graveyard(P0, "Forest");
    let wurm = t.battlefield(P1, "Craw Wurm");
    t.answer_targets(P0, &[obj(wurm)]);
    t.enter(P0, "Summon: Esper Ramuh");
    t.settle();
    assert!(trigger_on_top(&t));
    // In response, two more instants go to the graveyard: 3 damage, not 1.
    cast_card(&mut t, P0, "Opt", &[]);
    t.resolve();
    cast_card(&mut t, P0, "Opt", &[]);
    t.resolve();
    t.resolve();
    assert_eq!(t.obj_now(wurm).damage, 3);
}

#[test]
fn vivi_ornitiers_mana_ability_doesnt_use_the_stack() {
    cr!("605.3a", "605.3b");
    ruling!(
        "Vivi Ornitier",
        "Vivi Ornitier's first ability is a mana ability. It doesn't use the stack and it can't be responded to."
    );
    supported("Vivi Ornitier");
    let mut t = TestGame::new(2);
    let vivi = t.battlefield(P0, "Vivi Ornitier");
    add_counters(&mut t, vivi, "+1/+1", 2);
    let r = t.activate(P0, vivi, 0, &[]).expect("the mana ability");
    assert!(r.is_none(), "nothing put on the stack");
    assert!(t.g.stack.is_empty());
    assert_eq!(t.g.players[0].mana_pool.total(), 2);
}

#[test]
fn groves_bounty_divides_counters_as_its_cast() {
    cr!("601.2d", "122.1a");
    ruling!(
        "Elusive Otter // Grove's Bounty",
        "You choose how the counters will be distributed as you cast Grove's Bounty. Each target must receive at least one +1/+1 counter."
    );
    let mut t = TestGame::new(2);
    let a = t.battlefield(P0, "Grizzly Bears");
    let b = t.battlefield(P0, "Hill Giant");
    t.lands(P0, "Forest", 4);
    let card = t.hand(P0, "Elusive Otter // Grove's Bounty");
    t.answer_targets(P0, &[obj(a), obj(b)]);
    t.answer(P0, DecisionKind::Divide, Answer::Numbers(vec![2, 1]));
    let from = t.asked().len();
    let spell = t
        .cast(P0, card)
        .method(mtg_engine::object::CastMethod::Half(1))
        .x(3)
        .go();
    let divides: Vec<Decision> = t.asked()[from..]
        .iter()
        .filter(|(_, d)| matches!(d, Decision::Divide { .. }))
        .map(|(_, d)| d.clone())
        .collect();
    assert_eq!(divides.len(), 1, "divided while casting");
    if let Decision::Divide { min_each, .. } = &divides[0] {
        assert_eq!(*min_each, 1);
    }
    assert_eq!(t.zone(spell), Zone::Stack);
    t.resolve_all();
    assert_eq!(t.counters(a, "+1/+1"), 2);
    assert_eq!(t.counters(b, "+1/+1"), 1);
    // A division leaving a target with nothing isn't accepted: the engine's default
    // division (as even as possible) is used instead.
    let mut t = TestGame::new(2);
    let a = t.battlefield(P0, "Grizzly Bears");
    let b = t.battlefield(P0, "Hill Giant");
    t.lands(P0, "Forest", 4);
    let card = t.hand(P0, "Elusive Otter // Grove's Bounty");
    t.answer_targets(P0, &[obj(a), obj(b)]);
    t.answer(P0, DecisionKind::Divide, Answer::Numbers(vec![3, 0]));
    t.cast(P0, card)
        .method(mtg_engine::object::CastMethod::Half(1))
        .x(3)
        .go();
    t.resolve_all();
    assert_eq!((t.counters(a, "+1/+1"), t.counters(b, "+1/+1")), (2, 1));
}

#[test]
fn loot_effects_happen_within_one_resolution() {
    cr!("608.2c", "117.1");
    ruling!(
        "Jeskai Elder",
        "You draw a card and discard a card all while Jeskai Elder's ability is resolving. Nothing can happen between the two, and no player may choose to take actions."
    );
    ruling!(
        "Kasmina, Enigmatic Mentor",
        "You can't do anything in between drawing a card and discarding a card, including casting the card you drew."
    );
    ruling!(
        "Pako, Arcane Retriever",
        "You exile cards and put +1/+1 counters on Pako all while Pako's last ability is resolving. Nothing can happen between the two, and no player may choose to take actions."
    );
    supported("Jeskai Elder");
    supported("Kasmina, Enigmatic Mentor");
    supported("Pako, Arcane Retriever");
    // Jeskai Elder hits P1: P0 draws the Bolt on top and must discard it at once.
    let mut t = TestGame::new(2);
    let elder = t.battlefield(P0, "Jeskai Elder");
    let bolt = t.library_top(P0, "Lightning Bolt");
    t.answer_yes(P0, true);
    t.answer_choose_moved(P0, &[obj(bolt)]);
    attack_with(&mut t, &[(elder, pl(P1))]);
    t.advance_to(P0, Step::CombatDamage);
    t.settle();
    assert!(trigger_on_top(&t));
    // One resolution draws the Bolt and discards it: it's never castable in between.
    t.g.resolve_top();
    assert!(t.in_graveyard(P0, "Lightning Bolt"));
    assert_eq!(t.hand_size(P0), 0);
    // Kasmina's −2: the same.
    let mut t = TestGame::new(2);
    let k = t.battlefield(P0, "Kasmina, Enigmatic Mentor");
    let bolt = t.library_top(P0, "Lightning Bolt");
    t.answer_choose_moved(P0, &[obj(bolt)]);
    t.activate(P0, k, 0, &[]).unwrap();
    t.g.resolve_top();
    assert!(t.in_graveyard(P0, "Lightning Bolt"));
    assert_eq!(t.hand_size(P0), 0);
    assert_eq!(tokens_with_subtype(&t, P0, "Wizard").len(), 1);
    // Pako attacks: two cards exiled (Opt is noncreature) and a counter, in one go.
    let mut t = TestGame::new(2);
    let pako = t.battlefield(P0, "Pako, Arcane Retriever");
    t.library_top(P0, "Opt");
    t.library_top(P1, "Grizzly Bears");
    attack_with(&mut t, &[(pako, pl(P1))]);
    assert!(trigger_on_top(&t));
    t.g.resolve_top();
    assert!(t.in_exile("Opt") && t.in_exile("Grizzly Bears"));
    assert_eq!(t.counters(pako, "+1/+1"), 1);
}

#[test]
fn emeria_captains_counters_come_from_a_trigger_players_can_respond_to() {
    cr!("603.3", "603.6a", "704.5f");
    ruling!(
        "Emeria Captain",
        "Because Emeria Captain receives its +1/+1 counters from a triggered ability rather than a replacement effect, players may take actions while that triggered ability is on the stack and Emeria Captain’s toughness is just 1."
    );
    supported("Emeria Captain");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Merfolk Looter");
    let cap = t.enter(P0, "Emeria Captain");
    t.settle();
    assert!(trigger_on_top(&t));
    assert_eq!(t.pt(cap), (1, 1));
    // P1 responds with Shock: the Captain dies before getting its counters.
    cast_card(&mut t, P1, "Shock", &[obj(cap)]);
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Emeria Captain"));
    // Without the response it gets two (a Warrior — itself — and a Rogue).
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Merfolk Looter");
    let cap = t.enter(P0, "Emeria Captain");
    t.resolve_all();
    assert_eq!(t.counters(cap, "+1/+1"), 2);
}

#[test]
fn party_sizes_are_locked_in_as_the_effect_resolves() {
    cr!("700.8", "608.2h", "611.2c");
    ruling!(
        "Grotag Bug-Catcher",
        "The bonus is determined only as Grotag Bug-Catcher’s ability resolves. Once that happens, the size of the bonus won’t change later in the turn even if the number of creatures in your party changes."
    );
    ruling!(
        "Kabira Outrider",
        "The bonus is determined only as Kabira Outrider’s ability resolves. Once that happens, the size of the bonus won’t change later in the turn even if the number of creatures in your party changes."
    );
    ruling!(
        "Allied Assault",
        "The value of X is determined only as Allied Assault resolves. Once that happens, the value of X won’t change later in the turn even if the number of creatures in your party changes."
    );
    ruling!(
        "Drana's Silencer",
        "The value of X is determined only as Drana's Silencer's ability resolves. Once that happens, the value of X won't change later in the turn even if the number of creatures in your party changes."
    );
    for card in [
        "Grotag Bug-Catcher",
        "Kabira Outrider",
        "Allied Assault",
        "Drana's Silencer",
    ] {
        supported(card);
    }
    // P0's party: a Rogue (Merfolk Looter) plus whatever the card adds. After the effect,
    // a Cleric arrives (Cleric of the Forward Order) and the Rogue leaves.
    let after = |t: &mut TestGame, looter: ObjectId| {
        t.battlefield(P0, "Cleric of the Forward Order");
        t.battlefield(P0, "Silvergill Adept");
        crate::r_s02_common::destroy(t, looter);
    };
    // Grotag (a Warrior): Rogue + Warrior = 2 as it attacks: 3/2 for the turn.
    let mut t = TestGame::new(2);
    let looter = t.battlefield(P0, "Merfolk Looter");
    let grotag = t.battlefield(P0, "Grotag Bug-Catcher");
    attack_with(&mut t, &[(grotag, pl(P1))]);
    t.resolve_all();
    assert_eq!(t.pt(grotag), (3, 2));
    after(&mut t, looter);
    assert_eq!(t.pt(grotag), (3, 2));
    // Kabira Outrider (a Warrior) targets the Hill Giant: +2/+2.
    let mut t = TestGame::new(2);
    let looter = t.battlefield(P0, "Merfolk Looter");
    let giant = t.battlefield(P0, "Hill Giant");
    t.answer_targets(P0, &[obj(giant)]);
    t.enter(P0, "Kabira Outrider");
    t.resolve_all();
    assert_eq!(t.pt(giant), (5, 5));
    after(&mut t, looter);
    assert_eq!(t.pt(giant), (5, 5));
    // Allied Assault: the Rogue alone, X = 1.
    let mut t = TestGame::new(2);
    let looter = t.battlefield(P0, "Merfolk Looter");
    let giant = t.battlefield(P0, "Hill Giant");
    cast_card(&mut t, P0, "Allied Assault", &[obj(giant)]);
    t.resolve_all();
    assert_eq!(t.pt(giant), (4, 4));
    after(&mut t, looter);
    assert_eq!(t.pt(giant), (4, 4));
    // Drana's Silencer (a Rogue itself): the Looter's a Rogue too, X = 1.
    let mut t = TestGame::new(2);
    let looter = t.battlefield(P0, "Merfolk Looter");
    let wurm = t.battlefield(P1, "Craw Wurm");
    t.answer_targets(P0, &[obj(wurm)]);
    t.enter(P0, "Drana's Silencer");
    t.resolve_all();
    assert_eq!(t.pt(wurm), (5, 3));
    after(&mut t, looter);
    assert_eq!(t.pt(wurm), (5, 3));
}
