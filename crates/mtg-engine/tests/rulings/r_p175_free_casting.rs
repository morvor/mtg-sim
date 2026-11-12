//! Rulings batch P175 — casting an opponent's cards (Mindclaw Shaman, Silent-Blade Oni,
//! Jace's Mindseeker, Reversal of Fortune, Sen Triplets) and creature cards from a
//! graveyard (Havengul Lich): what can be paid when casting without paying the mana cost
//! (CR 118.9), timing permissions (CR 307.1, 601.3), split card copies (CR 709.3), and
//! what "can't activate abilities" covers (CR 602.1).

use crate::r_p175_common::*;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

/// P0's Silent-Blade Oni deals combat damage to P1, whose hand holds `spell`; P0 chooses
/// it (and answers `more` for casting it). Everything resolves.
fn oni_casts(t: &mut TestGame, spell: &str, more: impl FnOnce(&mut TestGame)) -> ObjectId {
    let oni = t.battlefield(P0, "Silent-Blade Oni");
    let c = t.hand(P1, spell);
    t.answer_choose(P0, &[obj(c)]);
    t.answer_yes(P0, true);
    more(t);
    attack_with(t, &[(oni, pl(P1))]);
    t.advance_to(P0, Step::CombatDamage);
    t.resolve_all();
    c
}

#[test]
fn silent_blade_oni_x_is_zero_and_mandatory_costs_are_paid() {
    cr!("118.9", "107.3b", "601.2b", "601.2f");
    ruling!(
        "Silent-Blade Oni",
        "If it has {X} in its mana cost, X must be 0. However, you can pay optional additional costs, such as kicker, and you must still pay mandatory additional costs, such as the one on Fling."
    );
    supported("Silent-Blade Oni");
    // Blaze ({X}{R}: X damage): X is 0.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    oni_casts(&mut t, "Blaze", |t| {
        t.answer_targets(P0, &[obj(bears)]);
    });
    assert!(t.in_graveyard(P1, "Blaze"), "cast: {}", t.dump_log());
    assert_eq!(crate::r_p009_common::dmg(&t, bears), 0);
    // Fling: P0 sacrifices a creature (its power in damage to P1).
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P0, "Hill Giant");
    oni_casts(&mut t, "Fling", |t| {
        t.answer_choose(P0, &[obj(giant)]);
        t.answer_targets(P0, &[pl(P1)]);
    });
    assert!(t.in_graveyard(P0, "Hill Giant"));
    assert_eq!(t.life(P1), 20 - 6 - 3);
}

#[test]
fn silent_blade_oni_cant_cast_a_morph_face_down() {
    cr!("118.9a", "702.37c");
    ruling!(
        "Silent-Blade Oni",
        "If you cast a card without paying its mana cost, you can't choose to cast it for any alternative costs, such as evoke or the alternative cost provided by the morph ability."
    );
    supported("Exalted Angel");
    let mut t = TestGame::new(2);
    let from = t.asked().len();
    oni_casts(&mut t, "Exalted Angel", |_| {});
    assert!(!crate::r_p068_common::asked_method(&t, from));
    let angel = t.named_on_battlefield("Exalted Angel");
    assert_eq!(angel.len(), 1, "cast face up: {}", t.dump_log());
    assert!(!t.obj_now(angel[0]).face_down);
}

#[test]
fn jaces_mindseeker_casts_a_card_milled_into_exile() {
    cr!("614.1a", "608.2c", "400.7");
    ruling!(
        "Jace's Mindseeker",
        "If the five cards are put into a public zone such as exile instead of a graveyard (perhaps due to the ability of Rest in Peace), you can cast one of those instant or sorcery cards from that zone."
    );
    supported("Jace's Mindseeker");
    supported("Rest in Peace");
    let mut t = TestGame::new(2);
    t.battlefield(P1, "Rest in Peace");
    let bolt = t.library_top(P1, "Lightning Bolt");
    t.answer_targets(P0, &[pl(P1)]);
    t.answer_choose_moved(P0, &[obj(bolt)]);
    t.answer_yes(P0, true);
    t.answer_targets(P0, &[pl(P1)]);
    t.enter(P0, "Jace's Mindseeker");
    t.resolve_all();
    assert_eq!(t.life(P1), 17, "{}", t.dump_log());
}

/// P0's Mindclaw Shaman enters targeting P1, whose hand holds a Lightning Bolt; P0 would
/// cast the Bolt at P1. The trigger is left on the stack. Returns the Bolt.
fn mindclaw(t: &mut TestGame) -> ObjectId {
    let bolt = t.hand(P1, "Lightning Bolt");
    t.answer_targets(P0, &[pl(P1)]);
    t.enter(P0, "Mindclaw Shaman");
    t.settle();
    assert_eq!(t.stack_len(), 1);
    t.answer_choose(P0, &[obj(bolt)]);
    t.answer_yes(P0, true);
    t.answer_targets(P0, &[pl(P1)]);
    bolt
}

#[test]
fn mindclaw_shaman_can_be_responded_to_but_not_while_resolving() {
    cr!("608.2", "117.1", "608.2g");
    ruling!(
        "Mindclaw Shaman",
        "Any player can respond to Mindclaw Shaman’s triggered ability. However, once that ability starts resolving and the target opponent reveals their hand, players can’t respond until after the ability finishes resolving."
    );
    supported("Mindclaw Shaman");
    // P1 responds by casting the Bolt (at P0): it's gone when the ability resolves.
    let mut t = TestGame::new(2);
    let bolt = mindclaw(&mut t);
    crate::r_p194_common::mana(&mut t, P1, mtg_engine::mana::ManaType::R, 1);
    t.cast(P1, bolt).target(pl(P0)).go();
    t.resolve_all();
    assert_eq!(t.life(P0), 17);
    assert_eq!(t.life(P1), 20);
    // No response: P0 casts P1's Bolt during the resolution; P1 never had the chance to
    // cast it in between.
    let mut t = TestGame::new(2);
    let bolt = mindclaw(&mut t);
    let from = t.asked().len();
    t.resolve();
    let spell = t.g.current(bolt);
    assert!(on_stack(&t, spell), "P0 cast the Bolt: {}", t.dump_log());
    assert_eq!(t.obj(spell).controller, P0);
    assert!(t.asked()[from..]
        .iter()
        .all(|(p, d)| *p != P1 || !matches!(d, mtg_engine::decision::Decision::Priority { .. })));
    t.resolve_all();
    assert_eq!(t.life(P1), 17);
}

#[test]
fn havengul_lich_changes_the_zone_not_the_timing() {
    cr!("302.1", "601.3", "117.1a");
    ruling!(
        "Havengul Lich",
        "Havengul Lich’s ability changes the zone you are permitted to cast the card from, not the times you are permitted to cast it."
    );
    supported("Havengul Lich");
    let mut t = TestGame::new(2);
    let lich = t.battlefield(P0, "Havengul Lich");
    let bears = t.graveyard(P1, "Grizzly Bears");
    act(&mut t, P0, lich, 0, &[obj(bears)]);
    // In combat: not castable (no flash).
    t.set_step(P0, Step::BeginningOfCombat);
    lots(&mut t, P0);
    assert!(t.cast(P0, bears).try_go().is_err());
    // With a spell on the stack in the main phase: not castable either.
    t.set_step(P0, Step::PostcombatMain);
    let bolt = t.hand(P0, "Lightning Bolt");
    t.cast(P0, bolt).target(pl(P1)).go();
    assert!(t.cast(P0, bears).try_go().is_err());
    t.resolve_all();
    // Main phase, empty stack: castable from P1's graveyard.
    lots(&mut t, P0);
    t.cast(P0, bears).go();
    t.resolve_all();
    let b = t.g.current(bears);
    assert!(t.on_battlefield(b));
    assert_eq!(ctl(&t, b), P0);
}

#[test]
fn reversal_of_fortune_copy_of_a_split_card_casts_one_half() {
    cr!("709.3", "709.3c", "707.12", "702.102a");
    ruling!(
        "Reversal of Fortune",
        "If the copied card is a split card, you may choose to cast either side of the split card but not both."
    );
    supported("Reversal of Fortune");
    // Turn // Burn (fuse): the copy casts Burn (2 damage to P1), not fused.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let card = t.hand(P1, "Turn // Burn");
    t.answer_choose(P0, &[obj(card)]);
    t.answer_yes(P0, true);
    t.answer_yes(P0, true);
    let from = t.asked().len();
    t.answer(
        P0,
        DecisionKind::Option,
        mtg_engine::decision::Answer::Index(1),
    );
    t.answer_targets(P0, &[pl(P1)]);
    cast_at(&mut t, P0, "Reversal of Fortune", &[pl(P1)]);
    t.resolve_all();
    // The method offered: one half or the other (no fused option).
    let opts: Vec<Vec<String>> = t.asked()[from..]
        .iter()
        .filter_map(|(_, d)| match d {
            mtg_engine::decision::Decision::ChooseCastingMethod { options, .. } => {
                Some(options.clone())
            }
            mtg_engine::decision::Decision::ChooseOption { options, .. } => Some(options.clone()),
            _ => None,
        })
        .collect();
    assert_eq!(
        opts,
        vec![vec!["Cast Turn".to_string(), "Cast Burn".to_string()]]
    );
    assert_eq!(t.life(P1), 18, "Burn: {}", t.dump_log());
    assert_eq!(t.pt(bears), (2, 2), "not Turn");
    assert!(t.in_hand(P1, "Turn // Burn"), "the card stays in P1's hand");
}

/// P0's upkeep; with `triplets`, P0's Sen Triplets has targeted P1 this turn: P1 can't
/// cast spells or activate abilities.
fn sen_triplets(triplets: bool) -> TestGame {
    let mut t = TestGame::new(2);
    if triplets {
        t.battlefield(P0, "Sen Triplets");
        t.answer_targets(P0, &[pl(P1)]);
    }
    t.set_step(P1, Step::End);
    t.advance_to(P0, Step::Upkeep);
    t.resolve_all();
    t
}

/// P1's Triskelion and a Desert Cerodon in P1's hand, with mana and priority: whether P1
/// can activate each (Triskelion's ability, cycling).
fn p1_can_activate(t: &mut TestGame, trisk: ObjectId) -> (bool, bool) {
    let cycler = t.hand(P1, "Desert Cerodon");
    lots(t, P1);
    t.g.turn.priority = Some(P1);
    t.g.recompute();
    let acts = t.g.activatable_abilities(P1);
    (
        acts.iter().any(|(id, _)| *id == trisk),
        acts.iter().any(|(id, _)| *id == cycler),
    )
}

#[test]
fn sen_triplets_stops_activated_abilities_but_not_replacement_effects() {
    cr!("602.1", "614.1c", "614.1d", "702.29a");
    ruling!(
        "Sen Triplets",
        "Abilities that create replacement effects, such as a permanent entering the battlefield tapped or with counters on it, are unaffected by Sen Triplets. Abilities that apply “as [this creature] enters the battlefield” are also replacement effects and are unaffected."
    );
    ruling!(
        "Sen Triplets",
        "Activated abilities are written in the form “[Cost]: [Effect].” Some keyword abilities (such as cycling) are activated abilities and will have colons in their reminder texts."
    );
    supported("Sen Triplets");
    let mut t = sen_triplets(true);
    // P1's permanents' replacement effects still apply: Triskelion enters with counters,
    // Caged Sun's color is chosen as it enters, a Tapland enters tapped.
    let trisk = t.enter(P1, "Triskelion");
    assert_eq!(t.counters(trisk, "+1/+1"), 3);
    let from = t.asked().len();
    t.enter(P1, "Caged Sun");
    assert!(
        t.asked()[from..]
            .iter()
            .any(|(p, d)| *p == P1
                && matches!(d, mtg_engine::decision::Decision::ChooseOption { .. }))
    );
    let tapland = t.enter(P1, "Coastal Tower");
    assert!(tapped(&t, tapland));
    // P1 can't activate abilities: cycling (a keyword activated ability) or Triskelion's.
    assert_eq!(p1_can_activate(&mut t, trisk), (false, false));
    // Control: without the Triplets' effect, P1 could activate both.
    let mut t = sen_triplets(false);
    let trisk = t.enter(P1, "Triskelion");
    assert_eq!(p1_can_activate(&mut t, trisk), (true, true));
}
