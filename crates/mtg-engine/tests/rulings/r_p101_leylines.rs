//! Rulings batch P101 — the Leylines on the battlefield: Leyline of Hope's replacement per
//! life-gain event (CR 614.1a, 119.10) and its +2/+2 against marked damage (704.5g);
//! Leyline of Punishment against prevention (615.12) but not other replacements (614.9),
//! and life totals that would rise (119.5); Leyline of Vitality entering with a creature
//! (603.6a); Leyline of Combustion's trigger (603.3); Leyline of the Meek and copies of
//! tokens (111.1, 707.2); Leyline of Transformation's chosen type and enters replacements
//! (614.12, 205.3m); Leyline of Resonance's copy (707.10); Leyline of the Void and
//! discarded cards (614.6, 701.9).

use crate::r_p101_common::*;
use crate::r_p152_common::gain;
use crate::r_s25_common::{change_copy_targets, keep_copy_targets};
use mtg_engine::decision::{Answer, Decision};
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

#[test]
fn leyline_of_hope_makes_marked_damage_lethal_when_it_leaves_or_life_drops() {
    cr!("704.5g", "613.4c");
    ruling!(
        "Leyline of Hope",
        "Because damage remains marked on creatures until the damage is removed as the turn ends, nonlethal damage dealt to creatures you control may become lethal if Leyline of Hope leaves the battlefield or if your life total drops below 7 life more than your starting life total."
    );
    supported("Leyline of Hope");
    for leaves in [true, false] {
        let mut t = TestGame::new(2);
        let hope = t.battlefield(P0, "Leyline of Hope");
        set_life(&mut t, P0, 27);
        let bears = t.battlefield(P0, "Grizzly Bears");
        assert_eq!(t.pt(bears), (4, 4));
        let src = t.battlefield(P1, "Raging Goblin");
        damage(&mut t, src, bears, 3);
        assert!(t.on_battlefield(bears));
        if leaves {
            destroy(&mut t, hope);
        } else {
            t.g.lose_life(P0, 1);
            t.g.flush_events();
            t.settle();
        }
        assert!(!t.on_battlefield(bears), "leaves = {leaves}");
    }
}

#[test]
fn leyline_of_hope_applies_once_per_life_gain_event() {
    cr!("614.1a", "702.15b", "510.2", "119.10", "702.19b", "702.4b");
    ruling!(
        "Leyline of Hope",
        "Each creature with lifelink dealing combat damage causes a separate life-gaining event. For example, if two creatures you control with lifelink deal combat damage at the same time, Leyline of Hope's second ability will apply to those two events separately. However, if a single creature you control with lifelink deals combat damage to multiple permanents and/or players at the same time (perhaps because it has trample or was blocked by more than one creature), the ability will apply only once."
    );
    ruling!(
        "Leyline of Hope",
        "The second ability of Leyline of Hope applies just once to each life-gaining event, no matter how much life is gained. If you gain an amount of life \"for each\" of something or \"equal to the number\" of something, that life is gained as one event and Leyline of Hope's second ability applies only once."
    );
    ruling!(
        "Leyline of Hope",
        "If you control two Leylines of Hope and you would gain life, you gain that much life plus 2. A third Leyline of Hope has you gain that much life plus 3, and so on."
    );
    // Two Vampire Nighthawks (2/3 lifelink) unblocked: 2 + 1 twice.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Leyline of Hope");
    let a = t.battlefield(P0, "Vampire Nighthawk");
    let b = t.battlefield(P0, "Vampire Nighthawk");
    t.set_step(P0, Step::BeginningOfCombat);
    t.attack(&[(a, pl(P1)), (b, pl(P1))], &[]);
    assert_eq!(t.life(P0), 26);
    // One Child of Night (2/1 lifelink) blocked by two Grizzly Bears, dealing 1 to each:
    // 2 + 1 once.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Leyline of Hope");
    let a = t.battlefield(P0, "Child of Night");
    let x = t.battlefield(P1, "Grizzly Bears");
    let y = t.battlefield(P1, "Grizzly Bears");
    t.answer(P0, DecisionKind::Damage, Answer::Numbers(vec![1, 1]));
    t.set_step(P0, Step::BeginningOfCombat);
    t.attack(&[(a, pl(P1))], &[(x, a), (y, a)]);
    assert_eq!((dmg(&t, x), dmg(&t, y)), (1, 1), "{}", t.dump_log());
    assert_eq!(t.life(P0), 23);
    // Child of Night with Leyline Axe (3/2 double strike, trample) blocked by Grizzly
    // Bears: in the first-strike step 2 to the Bears and 1 to P1 (one event: 3 + 1), then
    // 3 to P1 (3 + 1).
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Leyline of Hope");
    let a = t.battlefield(P0, "Child of Night");
    crate::r_s06_common::attach_new(&mut t, P0, "Leyline Axe", obj(a));
    assert_eq!(t.pt(a), (3, 2));
    let x = t.battlefield(P1, "Grizzly Bears");
    t.answer(P0, DecisionKind::Damage, Answer::Numbers(vec![2, 1]));
    t.set_step(P0, Step::BeginningOfCombat);
    t.attack(&[(a, pl(P1))], &[(x, a)]);
    assert!(!t.on_battlefield(x));
    assert_eq!(t.life(P1), 16);
    assert_eq!(t.life(P0), 28, "{}", t.dump_log());
    // Peach Garden Oath ("You gain 2 life for each creature you control.") with three
    // creatures: one event of 6, plus 1.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Leyline of Hope");
    for _ in 0..3 {
        t.battlefield(P0, "Grizzly Bears");
    }
    cast_resolve(&mut t, P0, "Peach Garden Oath", "W", &[]);
    assert_eq!(t.life(P0), 20 + 6 + 1);
    // Two, then three Leylines of Hope: 1 + 2, then 1 + 3.
    for (n, gained) in [(2, 3), (3, 4)] {
        let mut t = TestGame::new(2);
        for _ in 0..n {
            t.battlefield(P0, "Leyline of Hope");
        }
        gain(&mut t, P0, 1);
        assert_eq!(t.life(P0), 20 + gained, "{n} Leylines");
    }
}

#[test]
fn leyline_of_hope_ignores_a_teammates_life_gain_in_two_headed_giant() {
    cr!("810.9", "614.1a", "119.10");
    ruling!(
        "Leyline of Hope",
        "In a Two-Headed Giant game, life gained by your teammate won't cause Leyline of Hope's second ability to apply, even though it will cause your team's life total to increase."
    );
    let mut t = two_headed_giant();
    t.battlefield(P0, "Leyline of Hope");
    let before = t.life(P0);
    gain(&mut t, P1, 3);
    assert_eq!(
        t.life(P0),
        before + 3,
        "the teammate gained 3; the team's total rose by 3"
    );
    gain(&mut t, P0, 3);
    assert_eq!(t.life(P0), before + 3 + 4);
}

#[test]
fn leyline_of_punishment_stops_prevention_but_not_its_other_effects() {
    cr!("615.12", "615.1a", "702.16e", "614.1a");
    ruling!(
        "Leyline of Punishment",
        "Damage prevention shields don't have any effect. If a prevention effect has an additional effect, the additional effect will still work (if possible). Spells that create prevention effects can still be cast, and abilities that create prevention effects can still be activated."
    );
    ruling!(
        "Leyline of Punishment",
        "Static abilities that prevent damage (including protection abilities) don't do so. If they have additional effects that don't depend on the amount of damage prevented, those additional effects will still work. Such effects are applied only once per source of damage."
    );
    supported("Leyline of Punishment");
    // Druid's Deliverance ("Prevent all combat damage that would be dealt to you this
    // turn. Populate.") can be cast; it populates. Samite Healer's ability ("Prevent the
    // next 1 damage that would be dealt to any target this turn.") can be activated. The
    // combat damage is dealt all the same.
    let mut t = TestGame::new(2);
    t.battlefield(P1, "Leyline of Punishment");
    create_token(&mut t, P0, "Thopter");
    let healer = t.battlefield(P0, "Samite Healer");
    let giant = t.battlefield(P1, "Hill Giant");
    t.set_step(P1, Step::PrecombatMain);
    mana(&mut t, P0, "1G");
    let dd = t.hand(P0, "Druid's Deliverance");
    t.g.turn.priority = Some(P0);
    t.cast(P0, dd).go();
    t.resolve_all();
    assert_eq!(tokens(&t, P0).len(), 2, "populated");
    t.g.turn.priority = Some(P0);
    act(&mut t, P0, healer, "Prevent the next 1 damage", &[pl(P0)]).expect("Samite Healer");
    t.resolve_all();
    assert!(t.obj_now(healer).tapped);
    t.set_step(P1, Step::BeginningOfCombat);
    t.attack(&[(giant, pl(P0))], &[]);
    assert_eq!(t.life(P0), 17);
    // Protection from red (Paladin en-Vec, 2/2) doesn't prevent Pyroclasm's 2 damage.
    // Unbreathing Horde (0/0: "If this creature would be dealt damage, prevent that damage
    // and remove a +1/+1 counter from it.") with four counters is dealt the 2 damage and
    // still loses one counter, once for the one source: a 3/3 with 2 damage.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Leyline of Punishment");
    let paladin = t.battlefield(P1, "Paladin en-Vec");
    let horde = t.battlefield(P1, "Unbreathing Horde");
    put(&mut t, horde, counters::PLUS1, 4);
    cast_resolve(&mut t, P0, "Pyroclasm", "1R", &[]);
    assert!(!t.on_battlefield(paladin));
    assert_eq!(t.counters(horde, counters::PLUS1), 3, "{}", t.dump_log());
    assert_eq!(dmg(&t, horde), 2);
    assert!(t.on_battlefield(horde));
    // With three counters, it dies: a 2/2 with 2 damage.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Leyline of Punishment");
    let horde = t.battlefield(P1, "Unbreathing Horde");
    put(&mut t, horde, counters::PLUS1, 3);
    cast_resolve(&mut t, P0, "Pyroclasm", "1R", &[]);
    assert!(t.in_graveyard(P1, "Unbreathing Horde"), "{}", t.dump_log());
    // Without the Leyline, the Horde prevents it and loses a counter.
    let mut t = TestGame::new(2);
    let horde = t.battlefield(P1, "Unbreathing Horde");
    put(&mut t, horde, counters::PLUS1, 3);
    cast_resolve(&mut t, P0, "Pyroclasm", "1R", &[]);
    assert_eq!(t.counters(horde, counters::PLUS1), 2);
    assert_eq!(dmg(&t, horde), 0);
}

#[test]
fn leyline_of_punishment_doesnt_stop_replacements_that_dont_prevent() {
    cr!("615.12", "614.1a", "614.9");
    ruling!(
        "Leyline of Punishment",
        "Effects that replace or redirect damage without using the word \"prevent\" are not affected by Leyline of Punishment."
    );
    // Furnace of Rath doubles the damage.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Leyline of Punishment");
    t.battlefield(P0, "Furnace of Rath");
    cast_resolve(&mut t, P0, "Shock", "R", &[pl(P1)]);
    assert_eq!(t.life(P1), 16);
    // Pariah ("All damage that would be dealt to you is dealt to enchanted creature
    // instead.") on P1's Hill Giant redirects P0's Shock at P1 to the Giant.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Leyline of Punishment");
    let giant = t.battlefield(P1, "Hill Giant");
    crate::r_s06_common::attach_new(&mut t, P1, "Pariah", obj(giant));
    cast_resolve(&mut t, P0, "Shock", "R", &[pl(P1)]);
    assert_eq!(t.life(P1), 20);
    assert_eq!(dmg(&t, giant), 2);
}

#[test]
fn leyline_of_punishment_stops_a_life_total_from_rising_but_not_from_falling() {
    cr!("119.5");
    ruling!(
        "Leyline of Punishment",
        "If an effect says to set a player's life total to a certain number, and that number is higher than the player's current life total, that part of the effect won't do anything. (If the number is lower than the player's current life total, the effect will work as normal.)"
    );
    for (from, to) in [(4, 4), (40, 26)] {
        let mut t = TestGame::new(2);
        t.battlefield(P1, "Leyline of Punishment");
        set_life(&mut t, P0, from);
        cast_resolve(&mut t, P0, "Providence", "5WW", &[]);
        assert_eq!(t.life(P0), to, "from {from}");
    }
}

#[test]
fn leyline_of_vitality_sees_a_creature_entering_with_it() {
    cr!("603.6a", "603.2");
    ruling!(
        "Leyline of Vitality",
        "If Leyline of Vitality and a creature enter under your control at the same time, Leyline of Vitality's third ability will trigger."
    );
    supported("Leyline of Vitality");
    let mut t = TestGame::new(2);
    crate::r_s21_common::genesis_wave(
        &mut t,
        &["Leyline of Vitality", "Grizzly Bears", "Island", "Island"],
        &[true],
    );
    assert_eq!(count_named(&t, "Leyline of Vitality"), 1);
    assert_eq!(t.life(P0), 21);
}

#[test]
fn leyline_of_combustions_trigger_resolves_first_even_if_the_spell_is_countered() {
    cr!("603.3");
    ruling!(
        "Leyline of Combustion",
        "Leyline of Combustion's triggered ability resolves before the spell that caused it to trigger. It resolves even if that spell is countered."
    );
    supported("Leyline of Combustion");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Leyline of Combustion");
    t.set_step(P1, Step::PrecombatMain);
    cast_new(&mut t, P1, "Shock", &[pl(P0)]);
    t.settle();
    assert_eq!(t.stack_len(), 2);
    t.resolve();
    assert_eq!((t.life(P0), t.life(P1)), (20, 18), "the trigger first");
    t.resolve_all();
    assert_eq!(t.life(P0), 18);
    // Countered: the trigger still resolves.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Leyline of Combustion");
    t.set_step(P1, Step::PrecombatMain);
    let shock = cast_new(&mut t, P1, "Shock", &[pl(P0)]);
    t.settle();
    counter(&mut t, P0, shock);
    t.resolve_all();
    assert_eq!((t.life(P0), t.life(P1)), (20, 18));
}

#[test]
fn leyline_of_the_meek_pumps_tokens_but_not_a_clone_copying_a_token() {
    cr!("111.1", "707.2", "613.4c");
    ruling!(
        "Leyline of the Meek",
        "Leyline of the Meek gives a bonus to creatures only if they're tokens. Cards such as Clone that are copying tokens aren't tokens."
    );
    supported("Leyline of the Meek");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Leyline of the Meek");
    let thopter = create_token(&mut t, P0, "Thopter");
    assert_eq!(t.pt(thopter), (2, 2));
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[obj(thopter)]);
    let clone = t.enter(P0, "Clone");
    t.settle();
    assert_eq!(t.obj_now(clone).chars.name, "Thopter");
    assert_eq!(t.pt(clone), (1, 1));
}

/// `p` answers every "Choose a creature type" choice with `ty`, recording the options.
fn choose_type(t: &mut TestGame, p: PlayerId, ty: &str) {
    let i = subtype_lists()
        .creature
        .iter()
        .position(|t| t == ty)
        .unwrap();
    t.answer(p, DecisionKind::Option, Answer::Index(i));
}

#[test]
fn leyline_of_transformations_type_applies_before_enters_replacements_for_that_type() {
    cr!("614.12", "205.3m", "613.1d");
    ruling!(
        "Leyline of Transformation",
        "Replacement effects that modify creatures of a certain type as they enter will apply after you apply this effect. For example, if Warrior is the chosen creature type and you control Bramblewood Paragon (a creature with \"Each other Warrior creature you control enters with an additional +1/+1 counter on it.\"), a Runeclaw Bear would enter with an additional +1/+1 counter."
    );
    ruling!(
        "Leyline of Transformation",
        "To choose a creature type, you must choose an existing creature type, such as Human or Survivor. You can't choose multiple creature types, such as \"Human Survivor.\" Card types such as artifact can't be chosen, nor can subtypes that aren't creature types, such as Kaito, Vehicle, or Treasure."
    );
    let mut t = TestGame::new(2);
    let from = t.asked().len();
    choose_type(&mut t, P0, "Warrior");
    let leyline = t.enter(P0, "Leyline of Transformation");
    t.settle();
    let options: Vec<String> = t.asked()[from..]
        .iter()
        .find_map(|(_, d)| match d {
            Decision::ChooseOption {
                prompt, options, ..
            } if prompt == "Choose a creature type" => Some(options.clone()),
            _ => None,
        })
        .expect("asked for a creature type");
    for ok in ["Human", "Survivor", "Warrior"] {
        assert!(options.iter().any(|o| o == ok), "{ok}");
    }
    for no in [
        "Human Survivor",
        "Artifact",
        "artifact",
        "Kaito",
        "Vehicle",
        "Treasure",
    ] {
        assert!(!options.iter().any(|o| o == no), "{no}");
    }
    assert_eq!(
        t.obj_now(leyline).choices.creature_type.as_deref(),
        Some("Warrior")
    );
    t.battlefield(P0, "Bramblewood Paragon");
    cast_resolve(&mut t, P0, "Runeclaw Bear", "1G", &[]);
    let bear = t.named_on_battlefield("Runeclaw Bear")[0];
    assert!(t.obj_now(bear).chars.has_subtype("Warrior"));
    assert_eq!(t.counters(bear, counters::PLUS1), 1);
}

#[test]
fn leyline_of_resonances_copy_isnt_cast_and_may_keep_or_change_its_target() {
    cr!("707.10", "707.10c", "115.7d", "601.2");
    ruling!(
        "Leyline of Resonance",
        "The copy created by Leyline of Resonance's last ability will have the same targets (all the same creature you control) as the spell it's copying unless you choose new ones. You may change any number of the targets, including all of them or none of them. The new targets must be legal."
    );
    ruling!(
        "Leyline of Resonance",
        "The copy made by Leyline of Resonance's last ability is created on the stack, so it's not \"cast.\""
    );
    supported("Leyline of Resonance");
    for change in [false, true] {
        let mut t = TestGame::new(2);
        t.battlefield(P0, "Leyline of Resonance");
        let bears = t.battlefield(P0, "Grizzly Bears");
        let goblin = t.battlefield(P1, "Raging Goblin");
        cast(&mut t, P0, "Giant Growth", "G", &[obj(bears)]);
        if change {
            change_copy_targets(&mut t, P0, &[Some(obj(goblin))]);
        } else {
            keep_copy_targets(&mut t, P0);
        }
        t.resolve_all();
        if change {
            assert_eq!(t.pt(bears), (5, 5));
            assert_eq!(
                t.pt(goblin),
                (4, 4),
                "an opponent's creature is a legal new target"
            );
        } else {
            assert_eq!(t.pt(bears), (8, 8));
        }
        assert_eq!(t.g.history.spells_cast.len(), 1, "the copy wasn't cast");
    }
}

#[test]
fn leyline_of_the_void_still_lets_discard_abilities_see_the_card_in_exile() {
    cr!("614.6", "701.9a", "702.35a");
    ruling!(
        "Leyline of the Void",
        "If your opponent discards a card while you control Leyline of the Void, abilities that check whether a card is discarded (such as Hollow One, or a madness ability of the discarded card) still work, even though that card never reaches that player's graveyard. In addition, spells or abilities that check the characteristics of the discarded card can find that card in exile."
    );
    supported("Leyline of the Void");
    // Megrim ("Whenever an opponent discards a card, Megrim deals 2 damage to that
    // player.") triggers for each card of P1's that Mind Rot exiles instead.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Leyline of the Void");
    t.battlefield(P0, "Megrim");
    t.hand(P1, "Hill Giant");
    t.hand(P1, "Grizzly Bears");
    cast_resolve(&mut t, P0, "Mind Rot", "2B", &[pl(P1)]);
    assert_eq!(t.graveyard_size(P1), 0);
    assert!(t.in_exile("Hill Giant") && t.in_exile("Grizzly Bears"));
    assert_eq!(t.life(P1), 16);
    // Hollow Marauder sees the mana value of the card P1 discarded into exile: Hill Giant
    // (4) draws nothing; Grizzly Bears (2) draws a card.
    for (card, drawn) in [("Hill Giant", 0), ("Grizzly Bears", 1)] {
        let mut t = TestGame::new(2);
        t.battlefield(P0, "Leyline of the Void");
        let c = t.hand(P1, card);
        t.answer_choose(P1, &[obj(c)]);
        let hand = t.hand_size(P0);
        cast_resolve(&mut t, P0, "Hollow Marauder", "6B", &[pl(P1)]);
        assert_eq!(t.zone(c), Zone::Exile, "{card}");
        assert!(t.in_exile(card));
        assert_eq!(t.hand_size(P0), hand + drawn, "{card}");
    }
    // Fiery Temper's madness still works when P1 discards it: P1 casts it from exile.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Leyline of the Void");
    t.hand(P1, "Fiery Temper");
    t.hand(P1, "Hill Giant");
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.g.players[P1.idx()]
        .mana_pool
        .add_type(mtg_engine::mana::ManaType::R, 1);
    t.answer_yes(P1, true);
    t.answer_targets(P1, &[obj(bears)]);
    cast_resolve(&mut t, P0, "Mind Rot", "2B", &[pl(P1)]);
    assert!(!t.on_battlefield(bears), "{}", t.dump_log());
    assert!(
        t.in_exile("Fiery Temper"),
        "a resolved spell goes to the graveyard, so is exiled"
    );
}
