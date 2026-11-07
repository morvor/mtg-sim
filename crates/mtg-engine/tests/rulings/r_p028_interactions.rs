//! Rulings batch P028 — copying instant and sorcery spells: any such spell can be copied,
//! targets or not, whoever controls it (CR 707.10); new targets for copies (CR 707.10c);
//! copies of a spell that has left the stack use its last known information (CR 707.10,
//! 608.2h); what is and isn't copied (kicker, divided damage, flashback, CR 707.10); when
//! the copying abilities trigger (CR 603.2, 603.7); cost reductions (CR 601.2f); mana
//! abilities (CR 605); copies of linked and modal abilities (CR 607, 707.10).

use crate::r_p028_common::*;
use crate::r_p030_common::resolve_until_copies;
use crate::r_s01_common::{attack_with, block_and_finish, supported};
use crate::r_s06_common::activate_containing;
use crate::r_s07_common::chosen_modes;
use crate::r_s17_common::enter_transformed;
use crate::r_s20_common::tap_for_mana;
use crate::r_s25_common::*;
use crate::r_s26_common::modify_until_eot;
use mtg_engine::ability::Modification;
use mtg_engine::decision::{Answer, Decision};
use mtg_engine::keywords::{Keyword, KeywordKind};
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

/// `caster` casts Divination ("Draw two cards."); `c` copies it for P0. The copy is P0's
/// and draws P0 two cards.
fn copies_divination(c: &Copier, caster: PlayerId) {
    supported(c.name);
    let mut t = TestGame::new(2);
    if caster != P0 {
        t.set_step(caster, Step::PrecombatMain);
    }
    (c.setup)(&mut t);
    let div = cast_new(&mut t, caster, "Divination", &[]);
    (c.start)(&mut t, div);
    let copies = resolve_until_copies(&mut t);
    assert_eq!(copies.len(), 1, "{}", c.name);
    assert_eq!(t.obj(copies[0]).controller, P0, "{}", c.name);
    let (h0, h1) = (t.hand_size(P0), t.hand_size(P1));
    t.resolve_all();
    if caster == P0 {
        assert_eq!(t.hand_size(P0), h0 + 4, "{}", c.name);
    } else {
        assert_eq!(t.hand_size(P0), h0 + 2, "{}", c.name);
        assert_eq!(t.hand_size(P1), h1 + 2, "{}", c.name);
    }
}

#[test]
fn any_instant_or_sorcery_spell_can_be_copied_not_just_one_with_targets() {
    cr!("707.10", "115.1");
    ruling!(
        "Bonus Round",
        "Bonus Round’s ability will copy any instant or sorcery spell, not just one with targets."
    );
    copies_divination(&BONUS_ROUND, P0);
    ruling!(
        "Chandra, the Firebrand",
        "Chandra's second ability creates a delayed triggered ability that will copy the next instant or sorcery spell you cast that turn, regardless of whether that spell has targets."
    );
    copies_divination(&CHANDRA, P0);
    ruling!(
        "Double Vision",
        "Double Vision's ability will copy any instant or sorcery spell, not just one with targets."
    );
    copies_divination(&DOUBLE_VISION, P0);
    ruling!(
        "Doublecast",
        "Doublecast’s ability will copy any instant or sorcery spell, not just one with targets."
    );
    copies_divination(&DOUBLECAST, P0);
    ruling!(
        "Galvanic Iteration",
        "Galvanic Iteration's ability will copy any instant or sorcery spell, not just one with targets."
    );
    copies_divination(&GALVANIC_ITERATION, P0);
    // Another player's spell: the copy is P0's.
    ruling!(
        "Dualcaster Mage",
        "Dualcaster Mage's triggered ability can target (and copy) any instant or sorcery spell, not just one with targets. It doesn't matter who controls it."
    );
    copies_divination(&DUALCASTER_MAGE, P1);
    ruling!(
        "Echo Mage",
        "Echo Mage’s last two activated abilities can target (and copy) any instant or sorcery spell, not just one with targets. It doesn’t matter who controls it."
    );
    copies_divination(&ECHO_MAGE, P1);
}

#[test]
fn a_copy_s_you_and_opponent_are_from_the_copy_s_controller_s_perspective() {
    cr!("707.10", "109.5");
    ruling!(
        "Echo Mage",
        "If a copy says that it affects “you,” it affects the controller of that copy, not the controller of the original spell. Similarly, if a copy says that it affects an “opponent,” it affects an opponent of that copy’s controller, not an opponent of the original spell’s controller."
    );
    supported("Sizzle");
    // P1 casts Sizzle ("Sizzle deals 3 damage to each opponent."); P0 copies it with Echo
    // Mage: the copy deals 3 damage to P1.
    let mut t = TestGame::new(2);
    t.set_step(P1, Step::PrecombatMain);
    (ECHO_MAGE.setup)(&mut t);
    let sizzle = cast_new(&mut t, P1, "Sizzle", &[]);
    (ECHO_MAGE.start)(&mut t, sizzle);
    t.resolve_all();
    assert_eq!(t.life(P0), 17);
    assert_eq!(t.life(P1), 17);
}

/// Resolves the whole stack; the copies of spells that were on it at some point.
fn resolve_collecting_copies(t: &mut TestGame) -> Vec<ObjectId> {
    t.settle();
    let mut seen = vec![];
    while !t.g.stack.is_empty() {
        for c in crate::r_s25_common::spell_copies(t) {
            if !seen.contains(&c) {
                seen.push(c);
            }
        }
        t.resolve();
    }
    seen
}

// --- New targets ---------------------------------------------------------------------------

#[test]
fn echo_mage_s_two_copies_may_get_different_new_targets() {
    cr!("707.10c", "115.7");
    ruling!(
        "Echo Mage",
        "If you activate Echo Mage’s last ability to copy a spell twice, you may choose new targets for each copy independently. They don’t have to have the same targets as one another."
    );
    ruling!(
        "Echo Mage",
        "Each copy will have the same targets as the spell it’s copying unless you choose new ones. You may change any number of the targets, including all of them or none of them. If, for one of the targets, you can’t choose a new legal target, then it remains unchanged (even if the current target is illegal)."
    );
    // Lightning Bolt at P1; the first copy keeps P1, the second gets the Bears.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    (ECHO_MAGE_4.setup)(&mut t);
    let bolt = cast_new(&mut t, P0, "Lightning Bolt", &[Entity::Player(P1)]);
    (ECHO_MAGE_4.start)(&mut t, bolt);
    keep_copy_targets(&mut t, P0);
    change_copy_targets(&mut t, P0, &[Some(obj(bears))]);
    let copies = resolve_until_copies(&mut t);
    assert_eq!(copies.len(), 2);
    let mut ts: Vec<Entity> = copies.iter().flat_map(|c| targets_of(&t, *c)).collect();
    ts.sort();
    let mut want = vec![Entity::Player(P1), obj(bears)];
    want.sort();
    assert_eq!(ts, want);
    t.resolve_all();
    assert_eq!(t.life(P1), 14);
    assert!(!t.on_battlefield(bears));
}

/// P0 casts Giant Growth at their Grizzly Bears; `copier` will copy it; the Bears die
/// before the copy is made, and no other creature is on the battlefield: the copy keeps
/// its (now illegal) target and is countered on resolution.
fn copy_keeps_an_illegal_target(c: &Copier) {
    supported(c.name);
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    (c.setup)(&mut t);
    let growth = cast_new(&mut t, P0, "Giant Growth", &[obj(bears)]);
    (c.start)(&mut t, growth);
    t.settle();
    t.g.destroy(bears, None);
    t.g.flush_events();
    t.settle();
    let copies = resolve_until_copies(&mut t);
    assert_eq!(copies.len(), 1, "{}", c.name);
    assert_eq!(targets_of(&t, copies[0]), vec![obj(bears)], "{}", c.name);
    t.resolve_all();
    assert!(t.g.stack.is_empty());
}

#[test]
fn a_copy_keeps_a_target_it_cant_change_even_if_illegal() {
    cr!("707.10c", "608.2b");
    ruling!(
        "Howl of the Horde",
        "Each copy will have the same targets as the spell it's copying unless you choose new ones. You may change any number of the targets, including all of them or none of them. If, for one of the targets, you can't choose a new legal target, then it remains unchanged (even if the current target is illegal). You can choose different targets for each copy."
    );
    ruling!(
        "Mirari",
        "If the copied spell requires a target and there are no legal targets, then the copy has the same target (which should be illegal) as the original."
    );
    let howl = Copier {
        name: "Howl of the Horde",
        setup: HOWL_OF_THE_HORDE.setup,
        start: |t, _| change_copy_targets(t, P0, &[None]),
    };
    copy_keeps_an_illegal_target(&howl);
    let mirari = Copier {
        name: "Mirari",
        setup: MIRARI.setup,
        start: |t, _| {
            t.answer_yes(P0, true);
            change_copy_targets(t, P0, &[None]);
        },
    };
    copy_keeps_an_illegal_target(&mirari);
}

#[test]
fn howl_of_the_horde_s_two_raid_copies_may_get_different_targets() {
    cr!("707.10c", "603.7");
    ruling!(
        "Howl of the Horde",
        "Each copy will have the same targets as the spell it's copying unless you choose new ones."
    );
    // Raid: P0 attacked this turn, so the next spell is copied twice.
    let mut t = TestGame::new(2);
    let attacker = t.battlefield(P0, "Hill Giant");
    let a = t.battlefield(P1, "Grizzly Bears");
    let b = t.battlefield(P1, "Grizzly Bears");
    attack_with(&mut t, &[(attacker, Entity::Player(P1))]);
    block_and_finish(&mut t, P1, &[]);
    t.advance_to(P0, Step::PostcombatMain);
    cast_new(&mut t, P0, "Howl of the Horde", &[]);
    t.resolve_all();
    let bolt = cast_new(&mut t, P0, "Lightning Bolt", &[Entity::Player(P1)]);
    change_copy_targets(&mut t, P0, &[Some(obj(a))]);
    change_copy_targets(&mut t, P0, &[Some(obj(b))]);
    let copies = resolve_collecting_copies(&mut t);
    assert_eq!(copies.len(), 2);
    assert!(!t.g.is_live(bolt) || !t.g.stack.contains(&bolt));
    let _ = bolt;
    t.resolve_all();
    assert_eq!(t.life(P1), 20 - 3 - 3);
    assert!(!t.on_battlefield(a) && !t.on_battlefield(b));
}

#[test]
fn insidious_will_s_new_targets_must_be_legal() {
    cr!("707.10c", "115.7");
    ruling!(
        "Insidious Will",
        "If you choose new targets for the target spell or for the copy, the new targets must be legal."
    );
    // Giant Growth ("Target creature gets +3/+3") at the Bears; P0 copies it and tries to
    // retarget the copy to P1, which isn't a creature: the copy keeps the Bears.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let growth = cast_new(&mut t, P0, "Giant Growth", &[obj(bears)]);
    (INSIDIOUS_WILL.start)(&mut t, growth);
    t.clear_answers();
    change_copy_targets(&mut t, P0, &[Some(Entity::Player(P1))]);
    let copies = resolve_until_copies(&mut t);
    assert_eq!(targets_of(&t, copies[0]), vec![obj(bears)]);
    t.resolve_all();
    assert_eq!(t.pt(bears), (8, 8));
}

// --- Last known information ------------------------------------------------------------------

#[test]
fn thousand_year_storm_copies_a_countered_spell_and_the_copies_resolve_first() {
    cr!("707.10", "608.2h", "603.3");
    ruling!(
        "Thousand-Year Storm",
        "Copies are created even if the spell that caused Thousand-Year Storm's ability to trigger has been countered by the time that ability resolves. The copies resolve before the original spell."
    );
    supported("Thousand-Year Storm");
    // Countered: the copy (of Divination, the second spell this turn) is still made.
    let mut t = TestGame::new(2);
    (THOUSAND_YEAR_STORM.setup)(&mut t);
    let div = cast_new(&mut t, P0, "Divination", &[]);
    t.settle();
    cast_new(&mut t, P1, "Counterspell", &[obj(div)]);
    t.resolve();
    assert!(!t.g.stack.contains(&div));
    let copies = resolve_until_copies(&mut t);
    assert_eq!(copies.len(), 1);
    let hand = t.hand_size(P0);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand + 2);
    // Not countered: the copy is above the original.
    let mut t = TestGame::new(2);
    (THOUSAND_YEAR_STORM.setup)(&mut t);
    let div = cast_new(&mut t, P0, "Divination", &[]);
    let copies = resolve_until_copies(&mut t);
    assert_eq!(copies.len(), 1);
    let at = |id| t.g.stack.iter().position(|x| *x == id).unwrap();
    assert!(at(copies[0]) > at(div));
}

#[test]
fn mirari_copies_a_countered_spell_as_it_last_existed() {
    cr!("707.10", "608.2h");
    ruling!(
        "Mirari",
        "If the spell is countered before the triggered ability resolves, then the ability will make a copy of how the spell looked right before it was countered."
    );
    supported("Mirari");
    // Burst Lightning, kicked (4 damage), is countered; Mirari's copy is kicked too.
    let mut t = TestGame::new(2);
    (MIRARI.setup)(&mut t);
    t.lands(P0, "Mountain", 1);
    t.lands(P0, "Wastes", 4);
    let card = t.hand(P0, "Burst Lightning");
    let burst = t.cast(P0, card).kicked(true).target(P1).go();
    t.settle();
    cast_new(&mut t, P1, "Counterspell", &[obj(burst)]);
    t.resolve();
    assert!(!t.g.stack.contains(&burst));
    t.answer_yes(P0, true);
    keep_copy_targets(&mut t, P0);
    let copies = resolve_until_copies(&mut t);
    assert_eq!(copies.len(), 1);
    t.resolve_all();
    assert_eq!(t.life(P1), 16);
}

#[test]
fn mirari_s_copy_is_kicked_like_the_original_but_flashback_isnt_copied() {
    cr!("707.10", "702.33d", "702.34a");
    ruling!(
        "Mirari",
        "Everything about the original spell is copied, including any decisions made on announcement, such as whether it was kicked or its Buyback cost was paid. Effects on the spell, such as Sleight of Mind or Flashback, are not copied."
    );
    // Kicked Burst Lightning ("deals 2 damage to any target. If this spell was kicked, it
    // deals 4 damage instead."): the copy deals 4.
    let mut t = TestGame::new(2);
    (MIRARI.setup)(&mut t);
    t.lands(P0, "Mountain", 1);
    t.lands(P0, "Wastes", 4);
    let card = t.hand(P0, "Burst Lightning");
    t.cast(P0, card).kicked(true).target(P1).go();
    t.answer_yes(P0, true);
    keep_copy_targets(&mut t, P0);
    t.resolve_all();
    assert_eq!(t.life(P1), 12);
    // Think Twice cast with flashback: the original is exiled as it resolves; the copy
    // isn't a card and simply ceases to exist (no card goes to exile for it).
    let mut t = TestGame::new(2);
    (MIRARI.setup)(&mut t);
    t.graveyard(P0, "Think Twice");
    t.lands(P0, "Island", 1);
    t.lands(P0, "Wastes", 2);
    let tt = t.g.find_in_zone(mtg_engine::object::Zone::Graveyard(P0), "Think Twice")[0];
    let hand = t.hand_size(P0);
    t.cast(P0, tt)
        .method(mtg_engine::object::CastMethod::Keyword(KeywordKind::Flashback))
        .go();
    t.answer_yes(P0, true);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand + 2);
    assert!(t.in_exile("Think Twice"));
    assert!(!t.in_graveyard(P0, "Think Twice"));
}

// --- Divided damage ------------------------------------------------------------------------------

/// P0 casts Electrolyze ("deals 2 damage divided as you choose among one or two targets.
/// Draw a card.") with 1 damage to the Grizzly Bears and 1 to P1. `c` copies it; the copy
/// gets the Hill Giant instead of the Bears and keeps the division.
fn copy_keeps_the_division(c: &Copier) {
    supported(c.name);
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let giant = t.battlefield(P1, "Hill Giant");
    (c.setup)(&mut t);
    t.answer(P0, DecisionKind::Divide, Answer::Numbers(vec![1, 1]));
    lands_for_cost(&mut t, P0, "Electrolyze");
    let card = t.hand(P0, "Electrolyze");
    let e = t
        .cast(P0, card)
        .targets(&[obj(bears), Entity::Player(P1)])
        .go();
    (c.start)(&mut t, e);
    let copies = resolve_until_copies(&mut t);
    assert_eq!(copies.len(), 1, "{}", c.name);
    let copy = copies[0];
    let div = |id: ObjectId| -> Vec<u32> {
        t.g.obj(id)
            .stack
            .as_deref()
            .map(|si| {
                si.chosen
                    .iter()
                    .flat_map(|m| m.divided.iter().flatten().copied())
                    .collect()
            })
            .unwrap_or_default()
    };
    assert_eq!(div(copy), vec![1, 1], "{}", c.name);
    assert_eq!(div(copy), div(e), "{}", c.name);
    assert_eq!(
        targets_of(&t, copy),
        vec![obj(giant), Entity::Player(P1)],
        "{}",
        c.name
    );
    t.resolve_all();
    assert_eq!(t.life(P1), 18, "{}", c.name);
    assert_eq!(t.obj_now(giant).damage, 1, "{}", c.name);
    assert_eq!(t.obj_now(bears).damage, 1, "{}", c.name);
}

/// The copy gets the Giant for its first target, keeps P1 for its second.
fn retarget_to_giant(t: &mut TestGame) {
    let giant = t.named_on_battlefield("Hill Giant")[0];
    change_copy_targets(t, P0, &[Some(obj(giant)), None]);
}

#[test]
fn a_copy_keeps_the_division_of_damage_but_may_change_targets() {
    cr!("707.10", "707.10c", "601.2d");
    ruling!(
        "Display of Power",
        "If a spell has damage divided as it was put onto the stack, the division can't be changed, although the targets receiving that damage still can. The same is true of spells that distribute counters to targets."
    );
    copy_keeps_the_division(&Copier {
        name: "Display of Power",
        setup: nothing,
        start: |t, s| {
            cast_new(t, P0, "Display of Power", &[obj(s)]);
            retarget_to_giant(t);
        },
    });
    ruling!(
        "Naru Meha, Master Wizard",
        "If the spell has damage divided as it was cast (like Fight with Fire does when kicked), the division can't be changed (although the targets receiving that damage still can)."
    );
    copy_keeps_the_division(&Copier {
        name: "Naru Meha, Master Wizard",
        setup: nothing,
        start: |t, s| {
            cast_new(t, P0, "Naru Meha, Master Wizard", &[]);
            t.answer_targets(P0, &[obj(s)]);
            retarget_to_giant(t);
        },
    });
    ruling!(
        "The Mirari Conjecture",
        "If the spell has damage divided as it was cast (like Fight with Fire does when kicked), the division can’t be changed (although the targets receiving that damage still can)."
    );
    copy_keeps_the_division(&Copier {
        name: "The Mirari Conjecture",
        setup: MIRARI_CONJECTURE.setup,
        start: |t, _| retarget_to_giant(t),
    });
    ruling!(
        "Jin-Gitaxias, Progress Tyrant",
        "If the spell that's copied had damage divided to targets as it was cast, the division can't be changed, although the targets receiving that damage still can. The same is true of spells that distribute counters to targets as it was cast."
    );
    copy_keeps_the_division(&Copier {
        name: "Jin-Gitaxias, Progress Tyrant",
        setup: JIN_GITAXIAS.setup,
        start: |t, _| retarget_to_giant(t),
    });
    ruling!(
        "Kalamax, the Stormsire",
        "If the spell that's copied has damage divided as it was cast, the division can't be changed, although the targets receiving that damage still can. The same is true of spells that distribute counters."
    );
    copy_keeps_the_division(&Copier {
        name: "Kalamax, the Stormsire",
        setup: KALAMAX.setup,
        start: |t, _| retarget_to_giant(t),
    });
    ruling!(
        "Resonance Technician",
        "If a spell or ability has damage divided, the division can't be changed, although the targets receiving that damage still can. The same is true of spells and abilities that distribute counters."
    );
    // "{T}, Tap X untapped artifacts you control: Copy target instant or sorcery spell you
    // control with mana value X." Electrolyze has mana value 3.
    copy_keeps_the_division(&Copier {
        name: "Resonance Technician",
        setup: |t| {
            t.battlefield(P0, "Resonance Technician");
            for _ in 0..3 {
                t.battlefield(P0, "Ornithopter");
            }
        },
        start: |t, s| {
            let tech = t.named_on_battlefield("Resonance Technician")[0];
            let thopters: Vec<Entity> = t
                .named_on_battlefield("Ornithopter")
                .into_iter()
                .map(obj)
                .collect();
            t.answer(P0, DecisionKind::X, Answer::Number(3));
            t.answer(P0, DecisionKind::Entities, Answer::Entities(thopters));
            t.answer_targets(P0, &[obj(s)]);
            activate_containing(t, P0, tech, "Copy target").unwrap();
            retarget_to_giant(t);
        },
    });
}

// --- When the copying abilities trigger ---------------------------------------------------------

#[test]
fn double_vision_doesnt_trigger_if_a_spell_was_cast_before_it_entered_this_turn() {
    cr!("603.2", "603.4");
    ruling!(
        "Double Vision",
        "If you cast an instant or sorcery spell before Double Vision enters the battlefield during the same turn, its ability can't trigger that turn."
    );
    let mut t = TestGame::new(2);
    cast_new(&mut t, P0, "Opt", &[]);
    t.resolve_all();
    t.battlefield(P0, "Double Vision");
    cast_new(&mut t, P0, "Divination", &[]);
    assert!(resolve_until_copies(&mut t).is_empty());
}

#[test]
fn crackling_spellslinger_s_storm_needs_it_to_be_cast() {
    cr!("603.4", "702.40a");
    ruling!(
        "Crackling Spellslinger",
        "Crackling Spellslinger’s second ability triggers if you cast it from any zone. It doesn’t trigger if you put Crackling Spellslinger onto the battlefield without casting it."
    );
    supported("Crackling Spellslinger");
    supported("Yawgmoth's Will");
    // Cast (from the graveyard, with Yawgmoth's Will): the next instant (the third spell
    // this turn) has storm, two copies.
    let mut t = TestGame::new(2);
    cast_new(&mut t, P0, "Yawgmoth's Will", &[]);
    t.resolve_all();
    let card = t.graveyard(P0, "Crackling Spellslinger");
    lands_for_cost(&mut t, P0, "Crackling Spellslinger");
    t.cast(P0, card).go();
    t.resolve_all();
    assert!(t.on_battlefield(t.g.current(card)));
    cast_new(&mut t, P0, "Lightning Bolt", &[Entity::Player(P1)]);
    keep_copy_targets(&mut t, P0);
    keep_copy_targets(&mut t, P0);
    t.resolve_all();
    assert_eq!(t.life(P1), 20 - 9);
    // Put onto the battlefield: no storm.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Crackling Spellslinger");
    t.settle();
    cast_new(&mut t, P0, "Lightning Bolt", &[Entity::Player(P1)]);
    assert!(resolve_until_copies(&mut t).is_empty());
    t.resolve_all();
    assert_eq!(t.life(P1), 17);
}

#[test]
fn kalamax_triggers_when_tapped_to_pay_for_the_instant() {
    cr!("603.4", "702.51a", "601.2h");
    ruling!(
        "Kalamax, the Stormsire",
        "If Kalamax becomes tapped as a cost to cast your first instant spell in a turn, its first ability triggers."
    );
    // Stoke the Flames ("Convoke. Stoke the Flames deals 4 damage to any target."): Kalamax
    // (green, blue, red) is tapped for convoke.
    let mut t = TestGame::new(2);
    let k = t.battlefield(P0, "Kalamax, the Stormsire");
    t.lands(P0, "Mountain", 2);
    t.lands(P0, "Wastes", 1);
    t.answer(P0, DecisionKind::Entities, Answer::Entities(vec![obj(k)]));
    let card = t.hand(P0, "Stoke the Flames");
    t.cast(P0, card).target(P1).go();
    assert!(t.obj_now(k).tapped);
    keep_copy_targets(&mut t, P0);
    let copies = resolve_until_copies(&mut t);
    assert_eq!(copies.len(), 1);
    t.resolve_all();
    assert_eq!(t.life(P1), 12);
    // Its last ability: it copied an instant spell.
    assert_eq!(t.counters(k, "+1/+1"), 1);
}

#[test]
fn copying_a_card_isnt_copying_a_spell() {
    cr!("707.12", "707.10");
    ruling!(
        "Kalamax, the Stormsire",
        "If an effect copies a card rather than a spell (such as that of God-Eternal Kefnet), this doesn't cause Kalamax's last ability to trigger."
    );
    supported("Isochron Scepter");
    // Isochron Scepter: "{2}, {T}: You may copy the exiled card. If you do, you may cast the
    // copy without paying its mana cost." The copied card (Lightning Bolt) is cast: no
    // spell was copied.
    let mut t = TestGame::new(2);
    let k = t.battlefield(P0, "Kalamax, the Stormsire");
    let bolt = t.hand(P0, "Lightning Bolt");
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[obj(bolt)]);
    let scepter = t.enter(P0, "Isochron Scepter");
    t.resolve_all();
    t.lands(P0, "Wastes", 2);
    t.answer_yes(P0, true);
    t.answer_yes(P0, true);
    t.answer_targets(P0, &[Entity::Player(P1)]);
    activate_containing(&mut t, P0, scepter, "copy the exiled card").unwrap();
    t.resolve_all();
    assert_eq!(t.life(P1), 17);
    assert_eq!(t.counters(k, "+1/+1"), 0);
}

#[test]
fn tzaangor_shaman_dealing_combat_damage_twice_copies_the_next_spell_twice() {
    cr!("603.7", "702.4b", "707.10");
    ruling!(
        "Tzaangor Shaman",
        "If Tzaangor Shaman deals combat damage to a player more than one time in a turn, perhaps because it has gained double strike, its delayed triggered ability will trigger that many times when you cast your next instant or sorcery spell this turn."
    );
    supported("Tzaangor Shaman");
    let mut t = TestGame::new(2);
    let shaman = t.battlefield(P0, "Tzaangor Shaman");
    modify_until_eot(
        &mut t,
        shaman,
        vec![Modification::AddKeyword(Keyword::new(KeywordKind::DoubleStrike))],
    );
    attack_with(&mut t, &[(shaman, Entity::Player(P1))]);
    block_and_finish(&mut t, P1, &[]);
    t.resolve_all();
    // Tzaangor Shaman is a 3/3.
    assert_eq!(t.life(P1), 20 - 3 - 3);
    t.advance_to(P0, Step::PostcombatMain);
    cast_new(&mut t, P0, "Lightning Bolt", &[Entity::Player(P1)]);
    keep_copy_targets(&mut t, P0);
    keep_copy_targets(&mut t, P0);
    t.resolve_all();
    assert_eq!(t.life(P1), 14 - 9);
    // Only the next spell.
    cast_new(&mut t, P0, "Lightning Bolt", &[Entity::Player(P1)]);
    assert!(resolve_until_copies(&mut t).is_empty());
}

#[test]
fn doublecast_copied_by_doublecast_copies_the_next_spell_twice() {
    cr!("603.7", "707.10");
    ruling!(
        "Doublecast",
        "If you cast one Doublecast after another, you’ll copy the second Doublecast. Each resolves, creating two delayed triggered abilities. The next spell you cast after that will be copied twice. If that spell is also a Doublecast, the following spell will be copied three times, and so on."
    );
    let mut t = TestGame::new(2);
    cast_new(&mut t, P0, "Doublecast", &[]);
    t.resolve_all();
    // The second Doublecast is copied; both make delayed triggers.
    cast_new(&mut t, P0, "Doublecast", &[]);
    t.resolve_all();
    // A third Doublecast is copied twice: three delayed triggers.
    cast_new(&mut t, P0, "Doublecast", &[]);
    t.resolve_all();
    let hand = t.hand_size(P0);
    cast_new(&mut t, P0, "Divination", &[]);
    for _ in 0..3 {
        keep_copy_targets(&mut t, P0);
    }
    t.settle();
    let triggers = t
        .g
        .stack
        .iter()
        .filter(|x| {
            matches!(
                t.g.obj(**x).stack.as_ref().map(|s| &s.kind),
                Some(mtg_engine::object::StackKind::Triggered { .. })
            )
        })
        .count();
    assert_eq!(triggers, 3);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand + 8);
}

#[test]
fn thousand_year_storm_counts_spells_cast_one_at_a_time() {
    cr!("608.2g", "601.2", "702.40a");
    ruling!(
        "Thousand-Year Storm",
        "If an effect instructs you to cast multiple spells, they're cast one at a time in any order."
    );
    supported("Epic Experiment");
    // Epic Experiment (X = 2): "Exile the top X cards of your library. For each instant and
    // sorcery card with mana value X or less among them, you may cast that card without
    // paying its mana cost." Two Opts are cast one after another: the first is the second
    // spell this turn (one copy), the second is the third (two copies).
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Thousand-Year Storm");
    t.library_top(P0, "Opt");
    t.library_top(P0, "Opt");
    t.lands(P0, "Island", 1);
    t.lands(P0, "Mountain", 1);
    t.lands(P0, "Wastes", 2);
    let card = t.hand(P0, "Epic Experiment");
    crate::r_s22_common::choose_names_when_offered(&mut t, P0, &["Opt"]);
    t.cast(P0, card).x(2).go();
    let hand = t.hand_size(P0);
    t.resolve_all();
    // Two Opts and three copies: five cards drawn.
    assert_eq!(t.hand_size(P0), hand + 5);
}

// --- Costs and mana ------------------------------------------------------------------------------

#[test]
fn primal_amulet_doesnt_reduce_colored_mana() {
    cr!("601.2f", "118.7");
    ruling!(
        "Primal Amulet // Primal Wellspring",
        "Effects that reduce the generic mana cost of a spell can't reduce that spell's colored mana requirements."
    );
    // Counterspell ({U}{U}) with an Island and a Wastes: still not castable. Divination
    // ({2}{U}) costs {1}{U}.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Primal Amulet // Primal Wellspring");
    t.set_step(P1, Step::PrecombatMain);
    let div = cast_new(&mut t, P1, "Divination", &[]);
    t.lands(P0, "Island", 1);
    t.lands(P0, "Wastes", 1);
    let cs = t.hand(P0, "Counterspell");
    assert!(t.cast_with(P0, cs, &[obj(div)]).is_err());
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Primal Amulet // Primal Wellspring");
    t.lands(P0, "Island", 1);
    t.lands(P0, "Wastes", 1);
    let d = t.hand(P0, "Divination");
    assert!(t.cast_with(P0, d, &[]).is_ok());
}

#[test]
fn two_primal_wellspring_mana_spent_make_two_copies() {
    cr!("106.6", "707.10", "603.7");
    ruling!(
        "Primal Amulet // Primal Wellspring",
        "If more than one mana produced by a Primal Wellspring is spent to cast a single instant or sorcery spell, the delayed triggered ability associated with each mana spent will trigger. That many copies will be created. It doesn't matter if this mana was produced by one Primal Wellspring or by multiple Primal Wellsprings."
    );
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P1, "Hill Giant");
    let w1 = enter_transformed(&mut t, P0, "Primal Amulet // Primal Wellspring");
    let w2 = enter_transformed(&mut t, P0, "Primal Amulet // Primal Wellspring");
    // {U} and {R}: Index(1) is blue, Index(3) red.
    t.answer(P0, DecisionKind::Option, Answer::Index(1));
    assert!(tap_for_mana(&mut t, P0, w1, "Add one mana"));
    t.answer(P0, DecisionKind::Option, Answer::Index(3));
    assert!(tap_for_mana(&mut t, P0, w2, "Add one mana"));
    let card = t.hand(P0, "Izzet Charm");
    t.cast(P0, card).modes(&[1]).target(giant).go();
    keep_copy_targets(&mut t, P0);
    keep_copy_targets(&mut t, P0);
    t.settle();
    let mut modes = vec![];
    let mut seen = vec![];
    while !t.g.stack.is_empty() {
        for c in crate::r_s25_common::spell_copies(&t) {
            if !seen.contains(&c) {
                seen.push(c);
                modes.push(chosen_modes(&t, c));
            }
        }
        t.resolve();
    }
    assert_eq!(modes, vec![vec![1], vec![1]]);
}

#[test]
fn ether_s_ability_is_a_mana_ability() {
    cr!("605.1a", "605.3b", "603.7");
    ruling!(
        "Ether",
        "Ether's ability is a mana ability. It doesn't use the stack and can't be responded to."
    );
    supported("Ether");
    let mut t = TestGame::new(2);
    let ether = t.battlefield(P0, "Ether");
    assert!(tap_for_mana(&mut t, P0, ether, "Add {U}"));
    assert!(t.g.stack.is_empty());
    assert!(t.in_exile("Ether"));
    // The delayed trigger copies the next instant or sorcery spell.
    t.lands(P0, "Wastes", 2);
    let div = t.hand(P0, "Divination");
    t.cast(P0, div).go();
    keep_copy_targets(&mut t, P0);
    assert_eq!(resolve_until_copies(&mut t).len(), 1);
}

// --- Damage on Wizards ----------------------------------------------------------------------------

#[test]
fn nonlethal_damage_on_a_wizard_becomes_lethal_when_naru_meha_leaves() {
    cr!("704.5g", "613.4c", "120.6");
    ruling!(
        "Naru Meha, Master Wizard",
        "Because damage remains marked on a creature until it's removed as the turn ends, nonlethal damage dealt to a Wizard you control may become lethal if Naru leaves the battlefield during that turn."
    );
    supported("Prodigal Sorcerer");
    let mut t = TestGame::new(2);
    let naru = t.battlefield(P0, "Naru Meha, Master Wizard");
    let wizard = t.battlefield(P0, "Prodigal Sorcerer");
    assert_eq!(t.pt(wizard), (2, 2));
    crate::r_s06_common::damage(&mut t, naru, 1, wizard);
    assert!(t.on_battlefield(wizard));
    t.g.destroy(naru, None);
    t.g.flush_events();
    t.settle();
    assert!(!t.on_battlefield(wizard));
}

// --- Lithoform Engine: copies of abilities ---------------------------------------------------------

#[test]
fn a_copied_linked_ability_is_linked_too() {
    cr!("607.2a", "707.10", "610.3");
    ruling!(
        "Lithoform Engine",
        "If an ability is linked to a second ability, copies of that first ability are also linked to that second ability. If the second ability refers to “the exiled card,” it refers to all cards exiled by the first ability and the copy."
    );
    supported("Lithoform Engine");
    supported("Fiend Hunter");
    // Fiend Hunter: "When this creature enters, you may exile another target creature an
    // opponent controls ... When this creature leaves the battlefield, return the exiled
    // card to the battlefield under its owner's control."
    let mut t = TestGame::new(2);
    let engine = t.battlefield(P0, "Lithoform Engine");
    let a = t.battlefield(P1, "Grizzly Bears");
    let b = t.battlefield(P1, "Hill Giant");
    let hunter = cast_new(&mut t, P0, "Fiend Hunter", &[]);
    t.answer_targets(P0, &[obj(a)]);
    t.resolve();
    let trigger = *t.g.stack.last().unwrap();
    t.lands(P0, "Wastes", 2);
    t.answer_targets(P0, &[obj(trigger)]);
    activate_containing(&mut t, P0, engine, "activated or triggered ability").unwrap();
    change_copy_targets(&mut t, P0, &[Some(obj(b))]);
    t.answer_yes(P0, true);
    t.answer_yes(P0, true);
    t.resolve_all();
    assert!(t.in_exile("Grizzly Bears") && t.in_exile("Hill Giant"));
    let hunter = t.g.current(hunter);
    t.g.destroy(hunter, None);
    t.g.flush_events();
    t.settle();
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Grizzly Bears").len(), 1);
    assert_eq!(t.named_on_battlefield("Hill Giant").len(), 1);
}

#[test]
fn a_copied_modal_ability_keeps_its_mode_but_a_copied_permanent_spell_chooses_anew() {
    cr!("707.10", "700.2", "707.10f");
    ruling!(
        "Lithoform Engine",
        "If the spell or ability that's copied is modal (that is, it says “Choose one —” or the like), the copy will have the same mode. A different mode can't be chosen. This doesn't apply to copying a permanent spell with a modal enters-the-battlefield triggered ability, but it does apply to copying that ability."
    );
    supported("Charming Prince");
    // Charming Prince: "When this creature enters, choose one — • Scry 2. • You gain 3
    // life. • Exile another target creature you own. Return it ..."
    // Its enters trigger with "You gain 3 life", copied: the copy gains 3 life too.
    let mut t = TestGame::new(2);
    let engine = t.battlefield(P0, "Lithoform Engine");
    cast_new(&mut t, P0, "Charming Prince", &[]);
    t.answer(P0, DecisionKind::Modes, Answer::Indices(vec![1]));
    t.resolve();
    let trigger = *t.g.stack.last().unwrap();
    assert_eq!(chosen_modes(&t, trigger), vec![1]);
    t.lands(P0, "Wastes", 2);
    t.answer_targets(P0, &[obj(trigger)]);
    activate_containing(&mut t, P0, engine, "activated or triggered ability").unwrap();
    let from = t.asked().len();
    t.resolve();
    let copy = *t.g.stack.last().unwrap();
    assert_ne!(copy, trigger);
    assert_eq!(chosen_modes(&t, copy), vec![1]);
    t.resolve_all();
    assert_eq!(t.life(P0), 26);
    assert!(!t.asked()[from..]
        .iter()
        .any(|(_, d)| matches!(d, Decision::ChooseModes { .. })));
    // The permanent spell, copied: the token's own enters trigger has its mode chosen as
    // it's put on the stack (here "You gain 3 life" for the Prince, "Scry 2" for the
    // token).
    let mut t = TestGame::new(2);
    let engine = t.battlefield(P0, "Lithoform Engine");
    let prince = cast_new(&mut t, P0, "Charming Prince", &[]);
    t.lands(P0, "Wastes", 4);
    t.answer_targets(P0, &[obj(prince)]);
    activate_containing(&mut t, P0, engine, "permanent spell").unwrap();
    t.answer(P0, DecisionKind::Modes, Answer::Indices(vec![0]));
    t.answer(P0, DecisionKind::Modes, Answer::Indices(vec![1]));
    let from = t.asked().len();
    t.resolve_all();
    let modes_chosen = t.asked()[from..]
        .iter()
        .filter(|(_, d)| matches!(d, Decision::ChooseModes { .. }))
        .count();
    assert_eq!(modes_chosen, 2);
    assert_eq!(t.named_on_battlefield("Charming Prince").len(), 2);
    assert_eq!(t.life(P0), 23);
}
