//! Item `rulings-residue-misc-2`, pass A — combat and ability-loss rulings (CR 506–511,
//! 613.1f, 702.111): Tifa, Aurelia, Wyll's Reversal, Drop of Honey, Island Sanctuary,
//! Boros Battleshaper, Dog Umbra, Spara's Adjudicators, Immovable Rod, Heliod's
//! Punishment, Volrath's Curse, Stun, and Trove of Temptation.

use crate::r_residue_misc_2a_common::*;
use crate::r_s09_common::legal_attack;
use mtg_engine::decision::{Action, SpecialAction};
use mtg_engine::keywords::KeywordKind;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::Color;
use mtg_engine::*;

fn has_kw(t: &TestGame, id: ObjectId, kw: KeywordKind) -> bool {
    t.obj_now(id).has_keyword(kw)
}

fn attackers(t: &TestGame) -> Vec<ObjectId> {
    t.g.combat
        .as_ref()
        .map(|c| c.attackers.iter().map(|a| a.id).collect())
        .unwrap_or_default()
}

// --- Tifa, Martial Artist --------------------------------------------------------------------

#[test]
fn tifa_melee_counts_each_opponent_attacked_with_any_creature() {
    cr!("702.121a", "702.121b");
    ruling!(
        "Tifa, Martial Artist",
        "It doesn't matter how many creatures you attacked a player with, only that you attacked a player with at least one creature. For example, if you attack one player with Tifa and another player with five creatures, Tifa will get +2/+2 until end of turn."
    );
    supported("Tifa, Martial Artist");
    let mut t = TestGame::new(3);
    let tifa = t.battlefield(P0, "Tifa, Martial Artist");
    let mut atk = vec![(tifa, pl(P1))];
    for _ in 0..5 {
        atk.push((t.battlefield(P0, "Grizzly Bears"), pl(P2)));
    }
    attack_with(&mut t, &atk);
    t.resolve_all();
    assert_eq!(t.pt(tifa), (6, 6));
}

/// P0's Tifa attacks P1 along with `others`; combat damage is dealt and the triggers
/// resolve. Returns whether P0's creatures were untapped afterwards.
fn tifa_combat(others: &[(&str, u32)]) -> (TestGame, Vec<ObjectId>) {
    let mut t = TestGame::new(2);
    let tifa = t.battlefield(P0, "Tifa, Martial Artist");
    let mut ids = vec![tifa];
    for (n, c) in others {
        let id = t.battlefield(P0, n);
        plus1(&mut t, id, *c);
        ids.push(id);
    }
    let atk: Vec<_> = ids.iter().map(|x| (*x, pl(P1))).collect();
    attack_with(&mut t, &atk);
    t.resolve_all();
    t.advance_to(P0, Step::CombatDamage);
    t.resolve_all();
    (t, ids)
}

#[test]
fn tifa_looks_at_each_creature_s_power_not_their_total() {
    cr!("603.2c", "510.2");
    ruling!(
        "Tifa, Martial Artist",
        "Tifa's last ability cares about the power of each individual creature that dealt combat damage to a player, not their total power."
    );
    // Tifa (5/5 with melee) and a 6/4 Craw Wurm: 11 power in all, neither 7 or more.
    let (t, ids) = tifa_combat(&[("Craw Wurm", 0)]);
    assert!(ids.iter().all(|x| t.obj_now(*x).tapped));
    // A Craw Wurm with a +1/+1 counter (7/5): everything untaps.
    let (t, ids) = tifa_combat(&[("Craw Wurm", 1)]);
    assert!(
        ids.iter().all(|x| !t.obj_now(*x).tapped),
        "{}",
        t.dump_log()
    );
}

#[test]
fn tifa_s_extra_combat_comes_without_a_main_phase() {
    cr!("505.1a", "500.8");
    ruling!(
        "Tifa, Martial Artist",
        "Tifa's last ability doesn't give you any additional main phases. This means that you will move directly from the end of combat step of one combat phase to the beginning of combat step of the next one."
    );
    let (mut t, _) = tifa_combat(&[("Craw Wurm", 1)]);
    t.advance_to(P0, Step::EndOfCombat);
    let ok = t.g.run_until(1000, |g| g.turn.step != Step::EndOfCombat);
    assert!(ok);
    assert_eq!(t.g.turn.step, Step::BeginningOfCombat);
}

// --- Aurelia, Exemplar of Justice ------------------------------------------------------------

#[test]
fn aurelia_s_combat_trigger_resolves_before_attackers_are_declared() {
    cr!("507.1", "508.1", "702.20b");
    ruling!(
        "Aurelia, Exemplar of Justice",
        "Aurelia’s last ability resolves before attackers are chosen."
    );
    supported("Aurelia, Exemplar of Justice");
    let mut t = TestGame::new(2);
    let aurelia = t.battlefield(P0, "Aurelia, Exemplar of Justice");
    t.answer_targets(P0, &[obj(aurelia)]);
    t.advance_to(P0, Step::BeginningOfCombat);
    t.resolve_all();
    assert_eq!(t.pt(aurelia), (4, 5));
    assert!(has_kw(&t, aurelia, KeywordKind::Vigilance));
    // White: vigilance, so attacking doesn't tap it.
    attack_with(&mut t, &[(aurelia, pl(P1))]);
    assert!(!t.obj_now(aurelia).tapped);
}

// --- Wyll's Reversal ---------------------------------------------------------------------------

/// P0 controls a creature with power 14 or more and casts Wyll's Reversal targeting
/// `spell`, with a 20 loaded for its d20 (the copy happens).
fn reversal(t: &mut TestGame, spell: ObjectId) {
    supported("Wyll's Reversal");
    t.battlefield(P0, "Worldspine Wurm");
    t.g.dice.loaded.push_back(20);
    cast_new(t, P0, "Wyll's Reversal", &[obj(spell)]);
    t.resolve();
}

#[test]
fn wyll_s_reversal_copy_keeps_the_modes() {
    cr!("707.10", "700.2g");
    ruling!(
        "Wyll's Reversal",
        "If the target spell or ability is modal (that is, it has a bulleted list of choices), the copy will have the same mode(s). You can't choose different ones."
    );
    let mut t = TestGame::new(2);
    t.set_step(P1, Step::PrecombatMain);
    lands_for_cost(&mut t, P1, "Kolaghan's Command");
    let kc = t.hand(P1, "Kolaghan's Command");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let spell = t
        .cast(P1, kc)
        .modes(&[1, 3])
        .targets(&[pl(P0)])
        .target(obj(bears))
        .go();
    let modes = |t: &TestGame, id: ObjectId| -> Vec<Option<usize>> {
        t.g.obj(id)
            .stack
            .as_ref()
            .unwrap()
            .chosen
            .iter()
            .map(|c| c.mode)
            .collect()
    };
    let original = modes(&t, spell);
    reversal(&mut t, spell);
    let copies = crate::r_s25_common::spell_copies(&t);
    assert_eq!(copies.len(), 1, "{}", t.dump_log());
    assert_eq!(modes(&t, copies[0]), original);
    assert!(!t
        .asked()
        .iter()
        .any(|(p, d)| *p == P0 && matches!(d, mtg_engine::decision::Decision::ChooseModes { .. })));
}

#[test]
fn wyll_s_reversal_copy_has_the_additional_costs_paid_for_the_original() {
    cr!("707.10", "601.2b");
    ruling!(
        "Wyll's Reversal",
        "You can't choose to pay any additional costs for the copy. However, effects based on any additional costs that were paid for the original spell or ability are copied as though those same costs were paid for the copy. For example, if you sacrifice a 3/3 creature to cast Fling and then copy it with Wyll's Reversal, the copy of Fling will also deal 3 damage to its target."
    );
    let mut t = TestGame::new(2);
    t.set_step(P1, Step::PrecombatMain);
    let giant = t.battlefield(P1, "Hill Giant");
    lands_for_cost(&mut t, P1, "Fling");
    let fling = t.hand(P1, "Fling");
    t.answer_choose(P1, &[obj(giant)]);
    let spell = t.cast(P1, fling).target(pl(P0)).go();
    reversal(&mut t, spell);
    t.resolve_all();
    // The original and the copy (targets kept): 3 damage each.
    assert_eq!(t.life(P0), 14, "{}", t.dump_log());
}

// --- Drop of Honey ---------------------------------------------------------------------------------

#[test]
fn drop_of_honey_doesnt_target() {
    cr!("115.1", "702.16b");
    ruling!(
        "Drop of Honey",
        "The first ability of Drop of Honey doesn't target. You can destroy a creature with shroud or protection from Green."
    );
    supported("Drop of Honey");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Drop of Honey");
    let crusader = t.battlefield(P1, "Mirran Crusader");
    let giant = t.battlefield(P1, "Hill Giant");
    t.set_step(P1, Step::End);
    t.advance_to(P0, Step::Upkeep);
    t.resolve_all();
    assert!(!t.on_battlefield(crusader), "{}", t.dump_log());
    assert!(t.on_battlefield(giant));
}

// --- Island Sanctuary --------------------------------------------------------------------------------

#[test]
fn island_sanctuary_lasts_until_your_next_turn_after_it_leaves() {
    cr!("614.1b", "611.2a");
    ruling!(
        "Island Sanctuary",
        "If the replacement effect is applied, the effect will continue until your next turn even if this card leaves the battlefield."
    );
    supported("Island Sanctuary");
    let mut t = TestGame::new(2);
    let sanctuary = t.battlefield(P0, "Island Sanctuary");
    let bears = t.battlefield(P1, "Grizzly Bears");
    let angel = t.battlefield(P1, "Serra Angel");
    // (Not the starting player's first turn, which has no draw, CR 103.8a.)
    t.g.turn.number = 3;
    t.set_step(P0, Step::Upkeep);
    t.answer_yes(P0, true);
    let hand = t.hand_size(P0);
    t.advance_to(P0, Step::PrecombatMain);
    assert_eq!(t.hand_size(P0), hand, "the draw was skipped");
    destroy(&mut t, sanctuary);
    t.advance_to(P1, Step::BeginningOfCombat);
    assert!(!legal_attack(&mut t, &[(bears, pl(P0))]));
    assert!(legal_attack(&mut t, &[(angel, pl(P0))]));
}

#[test]
fn island_sanctuary_one_skipped_draw_of_several_is_enough() {
    cr!("614.1b", "504.1");
    ruling!(
        "Island Sanctuary",
        "If you get multiple draws or you use a spell or ability during the draw step to draw extra cards, you can have the replacement effect apply to any one or all of those. You need only have it apply once to get the effect. If you skip more than one, there is no additional effect."
    );
    supported("Howling Mine");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Island Sanctuary");
    t.battlefield(P0, "Howling Mine");
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.g.turn.number = 3;
    t.set_step(P0, Step::Upkeep);
    // The normal draw is skipped, Howling Mine's isn't.
    t.answer_yes(P0, true);
    t.answer_yes(P0, false);
    let hand = t.hand_size(P0);
    t.advance_to(P0, Step::PrecombatMain);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand + 1);
    t.advance_to(P1, Step::BeginningOfCombat);
    assert!(!legal_attack(&mut t, &[(bears, pl(P0))]));
}

// --- Boros Battleshaper and Trove of Temptation -------------------------------------------------------

/// On P1's turn, P0's Boros Battleshaper's trigger targets `must` (attacks if able) and
/// `cant` (can't attack or block) and resolves; the game is in the beginning of combat
/// step.
fn battleshaper(t: &mut TestGame, must: Option<ObjectId>, cant: Option<ObjectId>) {
    supported("Boros Battleshaper");
    t.battlefield(P0, "Boros Battleshaper");
    t.answer_targets(P0, &must.map(obj).into_iter().collect::<Vec<_>>());
    t.answer_targets(P0, &cant.map(obj).into_iter().collect::<Vec<_>>());
    t.set_step(P1, Step::PrecombatMain);
    t.advance_to(P1, Step::BeginningOfCombat);
    t.settle();
}

#[test]
fn boros_battleshaper_forces_only_a_creature_able_to_attack_without_a_cost() {
    cr!("508.1d", "508.1h");
    ruling!(
        "Boros Battleshaper",
        "A creature forced to attack does so only if it’s able to do so as attackers are declared. If, at that time, the creature is tapped, is affected by a spell or ability that says it can’t attack, or hasn’t been under that player’s control continuously since the turn began (and doesn’t have haste), then it doesn’t attack. If there’s a cost associated with having the creature attack, the creature’s controller isn’t forced to pay that cost, so it doesn’t attack in that case either."
    );
    supported("Ghostly Prison");
    // Untapped: it must attack.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    battleshaper(&mut t, Some(bears), None);
    t.resolve_all();
    assert!(!legal_attack(&mut t, &[]));
    // Tapped: not.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    battleshaper(&mut t, Some(bears), None);
    t.resolve_all();
    t.g.tap(bears);
    assert!(legal_attack(&mut t, &[]));
    // Summoning sick: not.
    let mut t = TestGame::new(2);
    let bears = t.battlefield_sick(P1, "Grizzly Bears");
    battleshaper(&mut t, Some(bears), None);
    t.resolve_all();
    assert!(legal_attack(&mut t, &[]));
    // With a cost to attack (Ghostly Prison): not.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Ghostly Prison");
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.lands(P1, "Wastes", 2);
    battleshaper(&mut t, Some(bears), None);
    t.resolve_all();
    assert!(legal_attack(&mut t, &[]));
}

#[test]
fn boros_battleshaper_with_one_illegal_target_affects_the_other() {
    cr!("608.2b");
    ruling!(
        "Boros Battleshaper",
        "If you choose two targets for Boros Battleshaper’s ability, and only one of those targets is legal when the ability tries to resolve, only the legal target will be affected."
    );
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let lions = t.battlefield(P1, "Savannah Lions");
    battleshaper(&mut t, Some(bears), Some(lions));
    // The Lions leaves and comes back (a new object) before the ability resolves.
    cast_new(&mut t, P1, "Unsummon", &[obj(lions)]);
    t.resolve();
    t.resolve_all();
    assert!(!legal_attack(&mut t, &[]), "the Bears must still attack");
    assert!(legal_attack(&mut t, &[(bears, pl(P0))]));
}

#[test]
fn trove_of_temptation_requires_only_creatures_able_to_attack() {
    cr!("508.1d", "508.1h");
    ruling!(
        "Trove of Temptation",
        "If a creature isn't able to attack you or a planeswalker you control for any reason (such as being tapped as its controller's declare attackers step begins or being affected by \"summoning sickness\"), that creature doesn't have to attack. If no creatures a player controls are able to attack you or a planeswalker you control, Trove of Temptation's requirement has no effect during that combat. If there's a cost associated with having a creature attack, its controller isn't forced to pay that cost, so it doesn't have to attack in that case either."
    );
    supported("Trove of Temptation");
    for case in ["able", "tapped", "sick", "cost"] {
        let mut t = TestGame::new(2);
        t.battlefield(P0, "Trove of Temptation");
        if case == "cost" {
            t.battlefield(P0, "Ghostly Prison");
            t.lands(P1, "Wastes", 2);
        }
        let bears = if case == "sick" {
            t.battlefield_sick(P1, "Grizzly Bears")
        } else {
            t.battlefield(P1, "Grizzly Bears")
        };
        if case == "tapped" {
            t.g.tap(bears);
        }
        t.set_step(P1, Step::BeginningOfCombat);
        assert_eq!(legal_attack(&mut t, &[]), case != "able", "{case}");
    }
}

// --- Dog Umbra ------------------------------------------------------------------------------------

#[test]
fn dog_umbra_on_an_attacking_creature_doesnt_remove_it_from_combat() {
    cr!("506.4", "508.1c");
    ruling!(
        "Dog Umbra",
        "Attaching Dog Umbra to a creature another player controls that's already attacking or blocking won't remove that creature from combat."
    );
    supported("Dog Umbra");
    let mut t = TestGame::new(2);
    t.set_step(P1, Step::BeginningOfCombat);
    let bears = t.battlefield(P1, "Grizzly Bears");
    attack_with(&mut t, &[(bears, pl(P0))]);
    cast_new(&mut t, P0, "Dog Umbra", &[obj(bears)]);
    t.resolve_all();
    assert!(attackers(&t).contains(&bears));
    block_and_finish(&mut t, P0, &[]);
    assert_eq!(t.life(P0), 18);
}

// --- Spara's Adjudicators ---------------------------------------------------------------------------------

#[test]
fn spara_s_adjudicators_leaving_exile_uncast_leaves_the_land_its_ability() {
    cr!("611.2b", "400.7");
    ruling!(
        "Spara's Adjudicators",
        "If Spara's Adjudicators is removed from exile without being cast, the land will continue to have this ability."
    );
    supported("Spara's Adjudicators");
    let mut t = TestGame::new(2);
    let land = t.battlefield(P0, "Wastes");
    let before = n_abilities(&mut t, land);
    let card = t.hand(P0, "Spara's Adjudicators");
    t.lands(P0, "Plains", 2);
    t.activate(P0, card, 0, &[obj(land)])
        .expect("exiled from hand");
    t.resolve_all();
    assert_eq!(n_abilities(&mut t, land), before + 1);
    // Moved from exile to its owner's graveyard by some effect.
    let exiled = t.g.current(card);
    assert_eq!(t.zone(exiled), Zone::Exile);
    t.g.move_object_ev(mtg_engine::replacement::MoveEv {
        obj: exiled,
        to: Zone::Graveyard(P0),
        pos: mtg_engine::ability::LibraryPosition::Top,
        cause: mtg_engine::events::MoveCause::Effect,
        by: Some(P0),
        etb: Default::default(),
        source: None,
    });
    t.g.flush_events();
    t.settle();
    assert!(t.in_graveyard(P0, "Spara's Adjudicators"));
    assert_eq!(n_abilities(&mut t, land), before + 1);
}

// --- Immovable Rod ------------------------------------------------------------------------------------------

/// P0's Immovable Rod activates targeting `target`, and it resolves.
fn rod(t: &mut TestGame, target: ObjectId) -> ObjectId {
    supported("Immovable Rod");
    let r = t.battlefield(P0, "Immovable Rod");
    t.lands(P0, "Plains", 1);
    t.lands(P0, "Wastes", 3);
    t.activate(P0, r, 0, &[obj(target)]).expect("Immovable Rod");
    t.resolve_all();
    r
}

#[test]
fn immovable_rod_a_creature_losing_its_pt_defining_ability_is_0() {
    cr!("604.3", "613.1f", "613.4a");
    interp!("star-without-defining-ability-is-zero");
    ruling!(
        "Immovable Rod",
        "If a creature has a characteristic-defining ability that sets its power and/or toughness loses that ability, the power and/or toughness (as appropriate) become 0."
    );
    // Nightmare (*/*, Swamps P1 controls) with two +1/+1 counters: a 2/2.
    let mut t = TestGame::new(2);
    t.lands(P1, "Swamp", 3);
    let nightmare = t.battlefield(P1, "Nightmare");
    plus1(&mut t, nightmare, 2);
    assert_eq!(t.pt(nightmare), (5, 5));
    rod(&mut t, nightmare);
    assert_eq!(t.pt(nightmare), (2, 2));
    // Tarmogoyf (*/1+*): only the star is 0 (as other cards' rulings put it), 0/1 and the
    // counters.
    let mut t = TestGame::new(2);
    let goyf = t.battlefield(P1, "Tarmogoyf");
    t.graveyard(P1, "Grizzly Bears");
    t.graveyard(P1, "Forest");
    plus1(&mut t, goyf, 2);
    assert_eq!(t.pt(goyf), (4, 5));
    rod(&mut t, goyf);
    assert_eq!(t.pt(goyf), (2, 3));
}

#[test]
fn immovable_rod_on_an_aura_or_equipment() {
    cr!("303.4d", "704.5m", "301.5c");
    ruling!(
        "Immovable Rod",
        "If you target an Aura with Immovable Rod's last ability, it will lose its enchant ability and be put into its owner's graveyard because it can't legally enchant anything. An Equipment that loses its abilities loses its equip ability, so you won't be able to move it that way, but it will stay attached to the creature it's equipping, if any."
    );
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let pacifism = crate::r_s06_common::attach_new(&mut t, P1, "Pacifism", bears);
    rod(&mut t, pacifism);
    assert!(t.in_graveyard(P1, "Pacifism"));
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let lions = t.battlefield(P1, "Savannah Lions");
    let splitter = crate::r_s06_common::attach_new(&mut t, P1, "Bonesplitter", bears);
    rod(&mut t, splitter);
    assert_eq!(t.obj_now(splitter).attached_to, Some(obj(bears)));
    assert_eq!(t.pt(bears), (2, 2), "its static ability is gone too");
    t.set_step(P1, Step::PrecombatMain);
    t.lands(P1, "Wastes", 1);
    assert!(!can_activate(&mut t, P1, splitter));
    let _ = lions;
}

#[test]
fn immovable_rod_type_and_color_defining_abilities_still_apply() {
    cr!("613.1d", "613.1e", "613.8a", "604.3");
    ruling!(
        "Immovable Rod",
        "Some abilities still work even if they have been removed. Specifically, static abilities that change an object's types or change an object's colors will still apply even if those abilities are lost."
    );
    let mut t = TestGame::new(2);
    let courier = t.battlefield(P1, "Transguild Courier");
    rod(&mut t, courier);
    assert_eq!(n_abilities(&mut t, courier), 0);
    for c in Color::ALL {
        assert!(t.obj_now(courier).chars.colors.contains(c), "{c:?}");
    }
    let outcast = t.battlefield(P1, "Changeling Outcast");
    rod(&mut t, outcast);
    assert_eq!(n_abilities(&mut t, outcast), 0);
    let o = t.obj_now(outcast);
    assert!(o.chars.has_subtype("Elf") && o.chars.has_subtype("Dragon"));
}

// --- Heliod's Punishment --------------------------------------------------------------------------------

#[test]
fn heliod_s_punishment_doesnt_remove_abilities_gained_later() {
    cr!("613.7", "613.1f");
    ruling!(
        "Heliod's Punishment",
        "If the affected creature gains an ability after Heliod's Punishment becomes attached to it, it will keep that ability."
    );
    supported("Heliod's Punishment");
    let mut t = TestGame::new(2);
    let angel = t.battlefield(P1, "Serra Angel");
    crate::r_s06_common::attach_new(&mut t, P0, "Heliod's Punishment", angel);
    assert!(!has_kw(&t, angel, KeywordKind::Flying));
    cast_new(&mut t, P1, "Jump", &[obj(angel)]);
    t.resolve_all();
    assert!(has_kw(&t, angel, KeywordKind::Flying));
}

// --- Volrath's Curse ------------------------------------------------------------------------------------

#[test]
fn volrath_s_curse_whose_creature_is_sacrificed_to_it_goes_before_it_can_be_returned() {
    cr!("116.2d", "704.5m");
    ruling!(
        "Volrath's Curse",
        "If the creature this card is on is sacrificed to this card, this card is put into the graveyard before the ability to unsummon it can be used."
    );
    supported("Volrath's Curse");
    let mut t = TestGame::new(2);
    t.set_step(P1, Step::PrecombatMain);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let curse = crate::r_s06_common::attach_new(&mut t, P0, "Volrath's Curse", bears);
    t.lands(P0, "Island", 2);
    t.g.turn.priority = Some(P1);
    t.g.recompute();
    let special = t
        .g
        .legal_actions(P1)
        .into_iter()
        .find(|a| matches!(a, Action::Special(SpecialAction::Static { source, .. }) if *source == curse))
        .expect("the special action");
    t.answer_choose(P1, &[obj(bears)]);
    t.g.perform_action(P1, special)
        .expect("sacrifice the Bears");
    t.g.flush_events();
    // The state-based action puts the Curse into the graveyard before anyone has priority.
    t.settle();
    assert!(t.in_graveyard(P1, "Grizzly Bears"));
    assert!(t.in_graveyard(P0, "Volrath's Curse"));
    assert!(!can_activate(&mut t, P0, curse));
}

// --- Stun ------------------------------------------------------------------------------------------------

#[test]
fn stun_after_blocks_doesnt_remove_the_blocker() {
    cr!("509.1", "506.4");
    ruling!(
        "Stun",
        "Must be used before blockers are declared in order to affect blocking decisions. You can’t wait to see what your opponent declares and then try to stop them."
    );
    supported("Stun");
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P0, "Hill Giant");
    let bears = t.battlefield(P1, "Grizzly Bears");
    attack_with(&mut t, &[(giant, pl(P1))]);
    crate::r_p148_common::declare_blocks(&mut t, P1, &[(bears, giant)]);
    cast_new(&mut t, P0, "Stun", &[obj(bears)]);
    t.resolve_all();
    assert!(crate::r_s10_common::blocking(&t, bears));
    // Before blocks it works.
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P0, "Hill Giant");
    let bears = t.battlefield(P1, "Grizzly Bears");
    attack_with(&mut t, &[(giant, pl(P1))]);
    cast_new(&mut t, P0, "Stun", &[obj(bears)]);
    t.resolve_all();
    assert!(!legal_blocks(&mut t, P1, &[(bears, giant)]));
}
