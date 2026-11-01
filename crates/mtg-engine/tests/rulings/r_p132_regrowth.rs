//! Rulings batch P132 — spells and abilities that return creature cards from a graveyard
//! to hand: targets chosen as the spell or ability is put on the stack and "you may" /
//! payments decided as it resolves (CR 601.2c, 603.3d, 608.2), partial fizzling
//! (CR 608.2b), values taken from the card as it last existed in the graveyard
//! (CR 608.2h), modes (CR 700.2), reflexive triggers (CR 603.12) and cost reductions
//! (CR 601.2f, 202.3).

use crate::r_p006_common::targets_of;
use crate::r_p079_common::cast_card;
use crate::r_p132_common::*;
use crate::r_s01_common::{supported, tokens};
use crate::r_s02_common::{can_cast, destroy};
use crate::r_s06_common::attach_new;
use mtg_engine::decision::Decision;
use mtg_engine::keywords::KeywordKind;
use mtg_engine::object::{CastMethod, Zone};
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

fn yes_nos(t: &TestGame, p: PlayerId, from: usize) -> usize {
    t.asked()[from..]
        .iter()
        .filter(|(q, d)| *q == p && matches!(d, Decision::YesNo { .. }))
        .count()
}

#[test]
fn warren_pilferers_checks_for_a_goblin_as_it_resolves() {
    cr!("608.2", "603.3d");
    ruling!(
        "Warren Pilferers",
        "The ability checks whether the targeted card is a Goblin when the ability resolves."
    );
    for (card, haste) in [("Raging Goblin", true), ("Grizzly Bears", false)] {
        let mut t = TestGame::new(2);
        let c = t.graveyard(P0, card);
        t.answer_targets(P0, &[obj(c)]);
        let wp = t.enter(P0, "Warren Pilferers");
        t.resolve_all();
        assert_eq!(where_is(&t, c), "hand");
        assert_eq!(t.obj_now(wp).has_keyword(KeywordKind::Haste), haste, "{card}");
    }
}

#[test]
fn raise_dead_chooses_its_target_on_announcement() {
    cr!("601.2c");
    ruling!(
        "Raise Dead",
        "The creature being brought back is chosen on announcement and not on resolution of the spell because it is targeted."
    );
    let mut t = TestGame::new(2);
    let bears = t.graveyard(P0, "Grizzly Bears");
    let spell = cast_card(&mut t, P0, "Raise Dead", &[obj(bears)]).unwrap();
    assert_eq!(targets_of(&t, spell), vec![obj(bears)]);
    // A better card put into the graveyard afterwards doesn't change it.
    let giant = t.graveyard(P0, "Craw Wurm");
    t.resolve_all();
    assert_eq!(where_is(&t, bears), "hand");
    assert_eq!(where_is(&t, giant), "graveyard");
}

#[test]
fn woodland_sleuth_chooses_at_random_as_it_resolves() {
    cr!("608.2", "603.4");
    ruling!(
        "Woodland Sleuth",
        "The creature card isn’t chosen at random until the ability resolves."
    );
    let mut t = TestGame::new(2);
    let theirs = t.battlefield(P1, "Grizzly Bears");
    destroy(&mut t, theirs);
    t.enter(P0, "Woodland Sleuth");
    flush(&mut t);
    assert_eq!(t.stack_len(), 1);
    // The graveyard was empty as it triggered; a card put there in response is returned.
    let giant = t.graveyard(P0, "Hill Giant");
    t.resolve_all();
    assert_eq!(where_is(&t, giant), "hand");
}

#[test]
fn blood_for_bones_can_return_the_creature_sacrificed_to_cast_it() {
    cr!("601.2h", "608.2");
    ruling!(
        "Blood for Bones",
        "The creature sacrificed to cast Blood for Bones may be one of the cards chosen to be returned."
    );
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let giant = t.graveyard(P0, "Hill Giant");
    t.answer_choose_moved(P0, &[obj(bears)]);
    t.answer_choose_moved(P0, &[obj(bears)]);
    cast_card(&mut t, P0, "Blood for Bones", &[]).unwrap();
    assert_eq!(where_is(&t, bears), "graveyard");
    t.resolve_all();
    assert_eq!(where_is(&t, bears), "battlefield");
    assert_eq!(where_is(&t, giant), "hand");
}

#[test]
fn swift_warkite_returns_the_creature_only_if_it_stayed() {
    cr!("603.7c", "400.7", "708.2");
    ruling!(
        "Swift Warkite",
        "The creature you put onto the battlefield will be returned to your hand only if it’s still on the battlefield as the delayed triggered ability resolves during the next end step."
    );
    ruling!(
        "Swift Warkite",
        "The delayed triggered ability that returns the creature to your hand will trigger even if Swift Warkite leaves the battlefield before the next end step."
    );
    ruling!(
        "Swift Warkite",
        "You can’t put creatures onto the battlefield face down this way."
    );
    // 0: stays; 1: the creature dies first; 2: the Warkite leaves first.
    for case in 0..3 {
        let mut t = TestGame::new(2);
        t.set_step(P0, Step::PrecombatMain);
        let bears = t.graveyard(P0, "Grizzly Bears");
        t.answer_choose(P0, &[obj(bears)]);
        let wk = t.enter(P0, "Swift Warkite");
        t.resolve_all();
        assert_eq!(where_is(&t, bears), "battlefield");
        assert!(t.obj_now(bears).has_keyword(KeywordKind::Haste));
        match case {
            1 => destroy(&mut t, bears),
            2 => destroy(&mut t, wk),
            _ => {}
        }
        t.advance_to(P0, Step::End);
        t.settle();
        t.resolve_all();
        let want = if case == 1 { "graveyard" } else { "hand" };
        assert_eq!(where_is(&t, bears), want, "case {case}");
    }
    // A morph creature card is put onto the battlefield face up.
    let mut t = TestGame::new(2);
    let seer = t.hand(P0, "Fathom Seer");
    t.answer_choose(P0, &[obj(seer)]);
    t.enter(P0, "Swift Warkite");
    t.resolve_all();
    assert_eq!(where_is(&t, seer), "battlefield");
    assert!(!t.obj_now(seer).face_down);
}

#[test]
fn virtue_of_strength_triples_basic_land_mana_and_stacks() {
    cr!("106.12", "605.1a", "616.1");
    ruling!(
        "Virtue of Strength // Garenbrig Growth",
        "The effects of multiple copies of Virtue of Strength are cumulative."
    );
    ruling!(
        "Virtue of Strength // Garenbrig Growth",
        "You're \"tapping a basic land for mana\" only if you're activating a mana ability of a basic land that includes the {T} symbol in its cost."
    );
    let tap = |t: &mut TestGame, name: &str| {
        let land = t.battlefield(P0, name);
        t.g.players[0].mana_pool = Default::default();
        t.activate(P0, land, 0, &[]).unwrap();
        t.g.player(P0).mana_pool.total()
    };
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Virtue of Strength // Garenbrig Growth");
    assert_eq!(tap(&mut t, "Forest"), 3);
    assert_eq!(tap(&mut t, "Breeding Pool"), 1, "not a basic land");
    t.battlefield(P0, "Virtue of Strength // Garenbrig Growth");
    assert_eq!(tap(&mut t, "Forest"), 9);
}

#[test]
fn iname_targets_on_trigger_and_exiles_on_resolution_even_with_no_targets() {
    cr!("603.3d", "608.2c");
    ruling!(
        "Iname, Life Aspect",
        "The targets of Iname, Life Aspect's ability are chosen when the ability is put onto the stack, but you don't choose whether to exile Iname, Life Aspect until the ability resolves."
    );
    ruling!(
        "Iname, Life Aspect",
        "You can exile the card even if you choose no targets."
    );
    for (target, exile) in [(true, true), (true, false), (false, true)] {
        let mut t = TestGame::new(2);
        let iname = t.battlefield(P0, "Iname, Life Aspect");
        let kami = t.graveyard(P0, "Kami of Ancient Law");
        let targets: &[Entity] = if target { &[obj(kami)] } else { &[] };
        t.answer_targets(P0, targets);
        destroy(&mut t, iname);
        assert_eq!(t.stack_len(), 1);
        let ab = *t.g.stack.last().unwrap();
        assert_eq!(targets_of(&t, ab).len(), usize::from(target));
        t.answer_yes(P0, exile);
        t.resolve_all();
        assert_eq!(where_is(&t, iname), if exile { "exile" } else { "graveyard" });
        let kami_back = target && exile;
        assert_eq!(
            where_is(&t, kami),
            if kami_back { "hand" } else { "graveyard" }
        );
    }
}

#[test]
fn desecrator_hag_picks_the_greatest_power_as_it_resolves() {
    cr!("608.2", "115.1");
    ruling!(
        "Desecrator Hag",
        "This ability doesn’t target a card. It will determine which card is returned to your hand as it resolves."
    );
    supported("Desecrator Hag");
    let mut t = TestGame::new(2);
    let giant = t.graveyard(P0, "Hill Giant");
    let armodon = t.graveyard(P0, "Trained Armodon");
    t.enter(P0, "Desecrator Hag");
    flush(&mut t);
    assert_eq!(t.stack_len(), 1);
    assert!(targets_of(&t, *t.g.stack.last().unwrap()).is_empty());
    // A bigger card put into the graveyard in response is the one returned.
    let wurm = t.graveyard(P0, "Craw Wurm");
    t.resolve_all();
    assert_eq!(where_is(&t, wurm), "hand");
    // Tied for greatest power: you choose.
    t.answer_choose(P0, &[obj(armodon)]);
    t.enter(P0, "Desecrator Hag");
    t.resolve_all();
    assert_eq!(where_is(&t, armodon), "hand");
    assert_eq!(where_is(&t, giant), "graveyard");
}

#[test]
fn cost_reductions_apply_after_increases_and_dont_change_mana_value() {
    cr!("601.2f", "202.3a", "118.7");
    ruling!(
        "Sanguine Indulgence",
        "To determine the total cost of a spell, start with the mana cost or alternative cost you're paying, add any cost increases, then apply any cost reductions (such as that of Sanguine Indulgence)."
    );
    ruling!(
        "Tale of Momo",
        "To determine the total cost of a spell, start with the mana cost or alternative cost you're paying, add any cost increases, then apply any cost reductions (such as that of Tale of Momo's first ability)."
    );
    // Thalia: noncreature spells cost {1} more; Goblin Electromancer: instants and
    // sorceries cost {1} less. Increase first: {3}{B} + {1} - {3} - {1} = {B} and
    // {2}{W} + {1} - {2} - {1} = {W} (one land). Reductions first would leave the
    // increase unreduced: {1}{B} / {1}{W} (two lands).
    for (name, land, mv) in [
        ("Sanguine Indulgence", "Swamp", 4),
        ("Tale of Momo", "Plains", 3),
    ] {
        let mut t = TestGame::new(2);
        t.battlefield(P1, "Thalia, Guardian of Thraben");
        t.battlefield(P0, "Goblin Electromancer");
        if name == "Sanguine Indulgence" {
            gain(&mut t, P0, 3);
        } else {
            let bears = t.battlefield(P0, "Grizzly Bears");
            destroy(&mut t, bears);
        }
        t.lands(P0, land, 1);
        t.lands(P0, "Wastes", 1);
        let c = t.hand(P0, name);
        let spell = t.cast(P0, c).go();
        assert_eq!(crate::r_s01_common::tapped_lands(&t, P0), 1, "{name}");
        assert_eq!(t.g.mana_value_of(spell), mv, "{name}");
    }
}

#[test]
fn power_of_the_returned_card_is_as_it_last_existed_in_the_graveyard() {
    cr!("608.2h");
    ruling!(
        "Morgue Burst",
        "Use the power of the card as it last existed in your graveyard to determine how much damage will be dealt."
    );
    ruling!(
        "Lie in Wait",
        "Use the power of the card in the graveyard to determine how much damage Lie in Wait deals to the target creature."
    );
    supported("Morgue Burst");
    // Tarmogoyf is 2/3 in the graveyard (a creature card there and an instant), 1/2 once
    // it's in hand.
    let mut t = TestGame::new(2);
    let goyf = t.graveyard(P0, "Tarmogoyf");
    t.graveyard(P1, "Lightning Bolt");
    t.g.recompute();
    assert_eq!(t.pt(goyf).0, 2);
    cast_card(&mut t, P0, "Morgue Burst", &[obj(goyf), pl(P1)]).unwrap();
    t.resolve_all();
    assert_eq!(where_is(&t, goyf), "hand");
    assert_eq!(t.life(P1), 18);

    let mut t = TestGame::new(2);
    let goyf = t.graveyard(P0, "Tarmogoyf");
    t.graveyard(P1, "Lightning Bolt");
    let wurm = t.battlefield(P1, "Craw Wurm");
    cast_card(&mut t, P0, "Lie in Wait", &[obj(goyf), obj(wurm)]).unwrap();
    t.resolve_all();
    assert_eq!(where_is(&t, goyf), "hand");
    assert_eq!(t.obj_now(wurm).damage, 2);
}

#[test]
fn resourceful_return_checks_for_an_artifact_as_it_resolves() {
    cr!("608.2c");
    ruling!(
        "Resourceful Return",
        "Whether or not you control an artifact is checked as Resourceful Return resolves."
    );
    let mut t = TestGame::new(2);
    let bears = t.graveyard(P0, "Grizzly Bears");
    cast_card(&mut t, P0, "Resourceful Return", &[obj(bears)]).unwrap();
    t.battlefield(P0, "Ornithopter");
    let h = t.hand_size(P0);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), h + 2);
}

#[test]
fn garna_returns_itself_put_into_the_graveyard_by_the_legend_rule() {
    cr!("704.5j", "608.2");
    ruling!(
        "Garna, the Bloodflame",
        "Which creature cards to return to your hand is determined as Garna's triggered ability resolves. If Garna somehow finds its way into your graveyard before that, perhaps due to the \"legend rule,\" it will be returned to your hand."
    );
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Garna, the Bloodflame");
    t.enter(P0, "Garna, the Bloodflame");
    flush(&mut t);
    assert_eq!(t.named_on_battlefield("Garna, the Bloodflame").len(), 1);
    t.resolve_all();
    assert!(t.in_hand(P0, "Garna, the Bloodflame"));
}

#[test]
fn spells_that_need_targets_cant_be_cast_without_them() {
    cr!("601.2c", "115.1");
    ruling!(
        "Emergency Weld",
        "You can't cast Emergency Weld without an artifact or creature card in your graveyard to target."
    );
    ruling!(
        "Lie in Wait",
        "You can’t cast Lie in Wait unless you choose a creature card in your graveyard and a creature on the battlefield as targets."
    );
    ruling!(
        "Resourceful Return",
        "You can’t cast Resourceful Return without a target creature card in your graveyard."
    );
    ruling!(
        "Clear the Stage",
        "You can’t cast Clear the Stage without a target creature to get -3/-3."
    );
    ruling!(
        "Foul Renewal",
        "You must target a creature card in your graveyard and a creature on the battlefield to cast Foul Renewal."
    );
    // (card, needs a creature card in your graveyard, needs a creature on the battlefield)
    for (name, gy, bf) in [
        ("Emergency Weld", true, false),
        ("Lie in Wait", true, true),
        ("Resourceful Return", true, false),
        ("Clear the Stage", false, true),
        ("Foul Renewal", true, true),
    ] {
        for (have_gy, have_bf) in [(false, false), (true, false), (false, true), (true, true)] {
            let mut t = TestGame::new(2);
            crate::r_s25_common::lands_for_cost(&mut t, P0, name);
            t.lands(P0, "Wastes", 2);
            let c = t.hand(P0, name);
            if have_gy {
                t.graveyard(P0, "Grizzly Bears");
            }
            if have_bf {
                t.battlefield(P1, "Grizzly Bears");
            }
            let want = (!gy || have_gy) && (!bf || have_bf);
            assert_eq!(
                can_cast(&mut t, P0, c, CastMethod::Normal),
                want,
                "{name}: graveyard {have_gy}, battlefield {have_bf}"
            );
        }
    }
}

#[test]
fn up_to_targets_can_be_zero_and_all_illegal_targets_fizzle() {
    cr!("601.2c", "608.2b");
    ruling!(
        "Urborg Uprising",
        "You can cast Urborg Uprising with no targets if you want to only draw a card."
    );
    ruling!(
        "Mourner's Surprise",
        "You don't have to choose a target for Mourner's Surprise. However, if you do, and that creature card is an illegal target at the time Mourner's Surprise tries to resolve, it won't resolve and none of its effects will happen."
    );
    ruling!(
        "Pull from the Grave",
        "You don't have to choose any targets for Pull from the Grave. However, if you do and all of the targets are illegal as Pull from the Grave tries to resolve, it won't resolve and none of its effects will happen."
    );
    ruling!(
        "Macabre Waltz",
        "You may cast Macabre Waltz targeting one or zero creature cards. You'll still discard a card, even if you target no creature cards."
    );
    supported("Mourner's Surprise");
    // Urborg Uprising with no targets: draws.
    let mut t = TestGame::new(2);
    let h = t.hand_size(P0);
    cast_card(&mut t, P0, "Urborg Uprising", &[]).unwrap();
    t.resolve_all();
    assert_eq!(t.hand_size(P0), h + 1);
    // Mourner's Surprise: no target, a token; an illegal target, nothing.
    for target in [false, true] {
        let mut t = TestGame::new(2);
        let bears = t.graveyard(P0, "Grizzly Bears");
        let targets: &[Entity] = if target { &[obj(bears)] } else { &[] };
        t.answer_targets(P0, targets);
        let c = t.hand(P0, "Mourner's Surprise");
        crate::r_s25_common::lands_for_cost(&mut t, P0, "Mourner's Surprise");
        t.cast(P0, c).go();
        if target {
            move_to(&mut t, bears, Zone::Exile);
        }
        t.resolve_all();
        assert_eq!(tokens(&t, P0).len(), usize::from(!target), "target {target}");
    }
    // Pull from the Grave: both targets illegal: no life; no targets: life.
    for target in [false, true] {
        let mut t = TestGame::new(2);
        let a = t.graveyard(P0, "Grizzly Bears");
        let b = t.graveyard(P0, "Hill Giant");
        let targets: Vec<Entity> = if target {
            vec![obj(a), obj(b)]
        } else {
            vec![]
        };
        t.answer_targets(P0, &targets);
        let c = t.hand(P0, "Pull from the Grave");
        crate::r_s25_common::lands_for_cost(&mut t, P0, "Pull from the Grave");
        t.cast(P0, c).go();
        if target {
            move_to(&mut t, a, Zone::Exile);
            move_to(&mut t, b, Zone::Exile);
        }
        t.resolve_all();
        assert_eq!(t.life(P0), if target { 20 } else { 22 });
    }
    // Macabre Waltz with no targets still discards.
    let mut t = TestGame::new(2);
    let forest = t.hand(P0, "Forest");
    t.answer_targets(P0, &[]);
    let c = t.hand(P0, "Macabre Waltz");
    crate::r_s25_common::lands_for_cost(&mut t, P0, "Macabre Waltz");
    t.cast(P0, c).go();
    t.resolve_all();
    assert_eq!(where_is(&t, forest), "graveyard");
}

#[test]
fn edgars_awakening_returns_a_creature_when_an_effect_discards_it() {
    cr!("701.9a", "603.12");
    ruling!(
        "Edgar's Awakening",
        "You can't discard Edgar's Awakening just because you want to."
    );
    let mut t = TestGame::new(2);
    t.lands(P0, "Swamp", 1);
    let ea = t.hand(P0, "Edgar's Awakening");
    // No action discards it.
    t.g.turn.priority = Some(P0);
    assert!(t.g.legal_actions(P0).iter().all(|a| match a {
        Action::Cast { card, .. } => *card != ea,
        Action::Activate { source, .. } => *source != ea,
        _ => true,
    }));
    let bears = t.graveyard(P0, "Grizzly Bears");
    let opp = t.battlefield(P1, "Mountain");
    t.g.discard(P0, ea, Some(opp));
    flush(&mut t);
    t.answer_yes(P0, true);
    t.answer_targets(P0, &[obj(bears)]);
    t.resolve_all();
    assert_eq!(where_is(&t, bears), "hand");
}

#[test]
fn clear_the_stage_checks_power_after_the_shrink_and_on_resolution() {
    cr!("608.2b", "608.2c", "704.3");
    ruling!(
        "Clear the Stage",
        "If either target is illegal when Clear the Stage tries to resolve, the other is still affected as appropriate."
    );
    ruling!(
        "Clear the Stage",
        "You check whether you control a creature with power 4 or greater after the target creature gets -3/-3 but before it dies, if appropriate."
    );
    ruling!(
        "Clear the Stage",
        "You may always choose a target creature card in your graveyard, even if you don’t control a creature with power 4 or greater."
    );
    // (your 6/4 Craw Wurm is: absent / the target / not the target, the opponent's
    // target is destroyed first, the Wurm enters before it resolves, returned)
    for (case, wurm_target, kill, late_wurm, want) in [
        ("illegal creature target", false, true, false, "hand"),
        ("own Wurm shrunk", true, false, false, "graveyard"),
        ("no power 4", false, false, false, "graveyard"),
        ("Wurm arrives", false, false, true, "hand"),
    ] {
        let mut t = TestGame::new(2);
        let wurm = if case == "illegal creature target" || wurm_target {
            Some(t.battlefield(P0, "Craw Wurm"))
        } else {
            None
        };
        let theirs = t.battlefield(P1, "Grizzly Bears");
        let giant = t.graveyard(P0, "Hill Giant");
        let target = if wurm_target { wurm.unwrap() } else { theirs };
        cast_card(&mut t, P0, "Clear the Stage", &[obj(target), obj(giant)]).unwrap();
        if kill {
            destroy(&mut t, theirs);
        }
        if late_wurm {
            t.battlefield(P0, "Craw Wurm");
        }
        t.answer_yes(P0, true);
        t.resolve_all();
        assert_eq!(where_is(&t, giant), want, "{case}");
    }
}

#[test]
fn carrion_thrash_targets_on_trigger_and_pays_on_resolution() {
    cr!("603.3d", "608.2");
    ruling!(
        "Carrion Thrash",
        "You choose a target when the ability triggers. You decide whether to pay {2} when the ability resolves."
    );
    for pay in [false, true] {
        let mut t = TestGame::new(2);
        t.lands(P0, "Wastes", 2);
        let thrash = t.battlefield(P0, "Carrion Thrash");
        let bears = t.graveyard(P0, "Grizzly Bears");
        t.answer_targets(P0, &[obj(bears)]);
        destroy(&mut t, thrash);
        let ab = *t.g.stack.last().unwrap();
        assert_eq!(targets_of(&t, ab), vec![obj(bears)]);
        let from = t.asked().len();
        t.answer_yes(P0, pay);
        t.resolve_all();
        assert_eq!(yes_nos(&t, P0, from), 1);
        assert_eq!(where_is(&t, bears), if pay { "hand" } else { "graveyard" });
    }
}

#[test]
fn angel_of_serenity_targets_up_to_three_and_exiles_on_resolution() {
    cr!("603.3d", "601.2c", "608.2c");
    ruling!(
        "Angel of Serenity",
        "You choose the targets as part of putting the \"enters\" trigger on the stack."
    );
    ruling!(
        "Angel of Serenity",
        "You choose up to three total targets from among other creatures on the battlefield and creature cards in graveyards. You may choose zero targets."
    );
    supported("Angel of Serenity");
    for exile in [false, true] {
        let mut t = TestGame::new(2);
        let bears = t.battlefield(P1, "Grizzly Bears");
        let a = t.graveyard(P1, "Hill Giant");
        let b = t.graveyard(P0, "Craw Wurm");
        t.answer_targets(P0, &[obj(bears), obj(a), obj(b)]);
        let angel = t.enter(P0, "Angel of Serenity");
        flush(&mut t);
        let ab = *t.g.stack.last().unwrap();
        assert_eq!(targets_of(&t, ab).len(), 3);
        assert!(!targets_of(&t, ab).contains(&obj(angel)));
        t.answer_yes(P0, exile);
        t.resolve_all();
        let want = if exile { "exile" } else { "graveyard" };
        assert_eq!(where_is(&t, a), want);
        assert_eq!(where_is(&t, b), want);
        assert_eq!(
            where_is(&t, bears),
            if exile { "exile" } else { "battlefield" }
        );
    }
    // Zero targets.
    let mut t = TestGame::new(2);
    t.graveyard(P1, "Hill Giant");
    t.answer_targets(P0, &[]);
    t.enter(P0, "Angel of Serenity");
    flush(&mut t);
    assert_eq!(t.stack_len(), 1);
}

#[test]
fn one_or_both_modes_are_fixed_as_the_spell_is_cast() {
    cr!("700.2a", "601.2b");
    ruling!(
        "Remember the Fallen",
        "You choose which mode you're using — or whether to use both modes — as you're casting the spell."
    );
    ruling!(
        "Reviving Melody",
        "You choose which mode you're using—or whether to use both modes—as you're casting the spell."
    );
    for (name, other) in [
        ("Remember the Fallen", "Ornithopter"),
        ("Reviving Melody", "Pacifism"),
    ] {
        let mut t = TestGame::new(2);
        let bears = t.graveyard(P0, "Grizzly Bears");
        let o = t.graveyard(P0, other);
        let c = t.hand(P0, name);
        crate::r_s25_common::lands_for_cost(&mut t, P0, name);
        t.cast(P0, c).modes(&[0]).target(obj(bears)).go();
        t.resolve_all();
        assert_eq!(where_is(&t, bears), "hand", "{name}");
        assert_eq!(where_is(&t, o), "graveyard", "{name}");
    }
}

#[test]
fn loamcrafter_faun_targets_with_a_reflexive_trigger() {
    cr!("603.12", "603.3d");
    ruling!(
        "Loamcrafter Faun",
        "You don't choose targets for Loamcrafter Faun's ability at the time it triggers."
    );
    supported("Loamcrafter Faun");
    let mut t = TestGame::new(2);
    let forest = t.hand(P0, "Forest");
    let bears = t.graveyard(P0, "Grizzly Bears");
    t.enter(P0, "Loamcrafter Faun");
    flush(&mut t);
    let ab = *t.g.stack.last().unwrap();
    assert!(targets_of(&t, ab).is_empty());
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[obj(forest)]);
    t.answer_targets(P0, &[obj(bears)]);
    t.resolve();
    // The reflexive ability is on the stack: players may respond.
    assert_eq!(where_is(&t, forest), "graveyard");
    assert_eq!(t.stack_len(), 1);
    let refl = *t.g.stack.last().unwrap();
    assert_eq!(targets_of(&t, refl), vec![obj(bears)]);
    t.resolve_all();
    assert_eq!(where_is(&t, bears), "hand");
}

#[test]
fn soul_manipulation_modes_need_legal_targets() {
    cr!("700.2a", "601.2c");
    ruling!(
        "Soul Manipulation",
        "You can’t choose a mode unless there’s a legal target for it."
    );
    let mut t = TestGame::new(2);
    let bears = t.graveyard(P0, "Grizzly Bears");
    crate::r_s25_common::lands_for_cost(&mut t, P0, "Soul Manipulation");
    let c = t.hand(P0, "Soul Manipulation");
    // No creature spell to counter: only the second mode is available.
    let from = t.asked().len();
    let spell = t.cast(P0, c).modes(&[0, 1]).target(obj(bears)).go();
    let available: Vec<Vec<usize>> = t.asked()[from..]
        .iter()
        .filter_map(|(_, d)| match d {
            Decision::ChooseModes { available, .. } => Some(available.clone()),
            _ => None,
        })
        .collect();
    assert_eq!(available, vec![vec![1]]);
    assert_eq!(targets_of(&t, spell), vec![obj(bears)]);
    t.resolve_all();
    assert_eq!(where_is(&t, bears), "hand");
}

#[test]
fn foul_renewal_partial_fizzle() {
    cr!("608.2b");
    ruling!(
        "Foul Renewal",
        "If the creature becomes an illegal target before Foul Renewal resolves, you’ll still return the creature card to your hand."
    );
    supported("Foul Renewal");
    // The creature on the battlefield is gone: the card is still returned.
    let mut t = TestGame::new(2);
    let giant = t.graveyard(P0, "Hill Giant");
    let bears = t.battlefield(P1, "Grizzly Bears");
    cast_card(&mut t, P0, "Foul Renewal", &[obj(giant), obj(bears)]).unwrap();
    destroy(&mut t, bears);
    t.resolve_all();
    assert_eq!(where_is(&t, giant), "hand");
    // The card is gone: no card returned this way, so the creature is unaffected.
    let mut t = TestGame::new(2);
    let giant = t.graveyard(P0, "Hill Giant");
    let wurm = t.battlefield(P1, "Craw Wurm");
    cast_card(&mut t, P0, "Foul Renewal", &[obj(giant), obj(wurm)]).unwrap();
    move_to(&mut t, giant, Zone::Exile);
    t.resolve_all();
    assert_eq!(t.pt(wurm), (6, 4));
    // Both legal: -3/-3 (Hill Giant's toughness).
    let mut t = TestGame::new(2);
    let giant = t.graveyard(P0, "Hill Giant");
    let wurm = t.battlefield(P1, "Craw Wurm");
    cast_card(&mut t, P0, "Foul Renewal", &[obj(giant), obj(wurm)]).unwrap();
    t.resolve_all();
    assert_eq!(t.pt(wurm), (3, 1));
}

#[test]
fn aphetto_dredging_chooses_the_type_before_the_targets() {
    cr!("601.2c", "601.2b");
    ruling!(
        "Aphetto Dredging",
        "You need to choose the creature type before choosing the targets."
    );
    let mut t = TestGame::new(2);
    let gob = t.graveyard(P0, "Raging Goblin");
    t.graveyard(P0, "Grizzly Bears");
    crate::r_s35_common::pick_options(&mut t, P0, &["Goblin"]);
    t.answer_targets(P0, &[obj(gob)]);
    let from = t.asked().len();
    let c = t.hand(P0, "Aphetto Dredging");
    crate::r_s25_common::lands_for_cost(&mut t, P0, "Aphetto Dredging");
    t.cast(P0, c).go();
    let kinds: Vec<&str> = t.asked()[from..]
        .iter()
        .filter_map(|(_, d)| match d {
            Decision::ChooseOption { .. } => Some("type"),
            Decision::ChooseTargets { .. } => Some("targets"),
            _ => None,
        })
        .collect();
    assert_eq!(kinds.first(), Some(&"type"), "{kinds:?}");
    assert!(kinds.contains(&"targets"));
    t.resolve_all();
    assert_eq!(where_is(&t, gob), "hand");
}

#[test]
fn puppet_master_offers_the_payment_only_if_the_card_was_returned() {
    cr!("603.10a", "608.2c");
    ruling!(
        "Puppet Master",
        "You only get the opportunity to pay {U}{U}{U} (and thus to return Puppet Master to its owner's hand) if the creature card actually gets returned to its owner's hand."
    );
    supported("Puppet Master");
    for removed in [false, true] {
        let mut t = TestGame::new(2);
        t.lands(P0, "Island", 3);
        let bears = t.battlefield(P0, "Grizzly Bears");
        let pm = attach_new(&mut t, P0, "Puppet Master", obj(bears));
        destroy(&mut t, bears);
        assert_eq!(t.stack_len(), 1);
        if removed {
            move_to(&mut t, bears, Zone::Exile);
        }
        let from = t.asked().len();
        t.answer_yes(P0, true);
        t.resolve_all();
        assert_eq!(yes_nos(&t, P0, from), usize::from(!removed));
        if removed {
            assert_eq!(where_is(&t, pm), "graveyard");
        } else {
            assert_eq!(where_is(&t, bears), "hand");
            assert_eq!(where_is(&t, pm), "hand");
        }
    }
}
