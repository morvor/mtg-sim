//! Rulings batch P158 — targets and resolution: a spell with every target illegal doesn't
//! resolve (CR 608.2b) while one with some illegal targets resolves without affecting
//! them (CR 608.2b, 608.2c); conditions, colors, toughness and X are checked as the spell
//! or ability resolves (CR 608.2h); a spell's target restriction holds whatever its
//! self-replacement effect does.

use crate::r_p158_common::*;
use mtg_engine::decision::Decision;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

#[test]
fn anticognition_targets_only_creature_or_planeswalker_spells_either_way() {
    cr!("115.1", "614.1a");
    ruling!(
        "Anticognition",
        "If an opponent has eight or more cards in their graveyard, Anticognition can still target only a creature or planeswalker spell."
    );
    supported("Anticognition");
    let mut t = TestGame::new(2);
    graveyard_n(&mut t, P1, "Grizzly Bears", 8);
    lots(&mut t, P1);
    let shock = t.hand(P1, "Shock");
    let shock = t.cast(P1, shock).target(pl(P0)).go();
    lots(&mut t, P0);
    let a = t.hand(P0, "Anticognition");
    assert!(t.cast(P0, a).target(obj(shock)).try_go().is_err());
    t.resolve_all();
    // A creature spell: countered outright (no paying {2}), then scry 2.
    let mut t = TestGame::new(2);
    t.set_step(P1, Step::PrecombatMain);
    graveyard_n(&mut t, P1, "Grizzly Bears", 8);
    let bears = cast_at(&mut t, P1, "Grizzly Bears", &[]);
    lots(&mut t, P1);
    let from = t.asked().len();
    cast_at(&mut t, P0, "Anticognition", &[obj(bears)]);
    t.resolve();
    assert_eq!(t.zone(bears), Zone::Graveyard(P1));
    assert!(
        !asked_since(&t, from).iter().any(|(p, d)| *p == P1 && matches!(d, Decision::YesNo { .. })),
        "{:?}",
        asked_since(&t, from)
    );
    assert!(asked_since(&t, from).iter().any(|(p, d)| *p == P0 && matches!(d, Decision::Scry { .. })));
}

#[test]
fn chocobo_kick_with_an_illegal_target_deals_no_damage() {
    cr!("608.2b", "608.2c");
    ruling!(
        "Chocobo Kick",
        "If either target is illegal as Chocobo Kick resolves, no damage will be dealt. If both targets are illegal, the spell won't resolve at all."
    );
    supported("Chocobo Kick");
    t_chocobo(false, false);
    t_chocobo(true, false);
    t_chocobo(false, true);
    t_chocobo(true, true);
}

/// Chocobo Kick: P0's Hill Giant kicks P1's Gray Ogre; `mine`/`theirs` make that target
/// illegal in response (the Giant dies, the Ogre gains hexproof).
fn t_chocobo(mine: bool, theirs: bool) {
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let giant = t.battlefield(P0, "Hill Giant");
    let ogre = t.battlefield(P1, "Gray Ogre");
    t.lands(P0, "Forest", 2);
    let k = t.hand(P0, "Chocobo Kick");
    let k = t.cast(P0, k).kicked(false).targets(&[obj(giant), obj(ogre)]).go();
    if mine {
        kill(&mut t, giant);
    }
    if theirs {
        hexproof(&mut t, ogre);
    }
    t.resolve_all();
    let hit = !mine && !theirs;
    assert_eq!(t.on_battlefield(ogre), !hit, "mine {mine} theirs {theirs}");
    assert_eq!(resolved(&t, k), !(mine && theirs), "mine {mine} theirs {theirs}");
    assert!(t.in_graveyard(P0, "Chocobo Kick"));
}

#[test]
fn once_and_future_with_one_illegal_target_still_exiles_itself() {
    cr!("608.2b", "608.2c");
    ruling!(
        "Once and Future",
        "If one of the two target cards is an illegal target by the time Once and Future resolves, you'll put the other in the appropriate zone and exile Once and Future. If both targets are illegal, the spell won't resolve and you won't exile Once and Future."
    );
    supported("Once and Future");
    for both in [false, true] {
        let mut t = TestGame::new(2);
        let a = t.graveyard(P0, "Grizzly Bears");
        let b = t.graveyard(P0, "Hill Giant");
        let o = cast_at(&mut t, P0, "Once and Future", &[obj(a), obj(b)]);
        crate::r_p198_common::exile(&mut t, a);
        if both {
            crate::r_p198_common::exile(&mut t, b);
        }
        t.resolve_all();
        assert_eq!(resolved(&t, o), !both, "both {both}");
        assert_eq!(t.in_exile("Once and Future"), !both, "both {both}");
        assert_eq!(t.in_graveyard(P0, "Once and Future"), both, "both {both}");
        if !both {
            assert_eq!(crate::r_s33_common::library_top_first(&t, P0)[0], t.g.current(b), "the Giant is on top");
        }
    }
}

#[test]
fn sundering_stroke_keeps_its_division_when_a_target_becomes_illegal() {
    cr!("608.2b", "608.2c", "601.2d");
    ruling!(
        "Sundering Stroke",
        "If some of the targets become illegal for Sundering Stroke, the original division of damage still applies, but the damage that would have been dealt to illegal targets isn't dealt at all."
    );
    supported("Sundering Stroke");
    let mut t = TestGame::new(3);
    let bears = t.battlefield(P1, "Grizzly Bears");
    lots(&mut t, P0);
    t.g.players[0].mana_pool.mana.clear();
    mana(&mut t, P0, mtg_engine::mana::ManaType::R, 1);
    mana(&mut t, P0, mtg_engine::mana::ManaType::C, 6);
    divide(&mut t, P0, &[4, 2, 1]);
    let s = t.hand(P0, "Sundering Stroke");
    t.cast(P0, s)
        .targets(&[pl(P1), pl(P2), obj(bears)])
        .go();
    kill(&mut t, bears);
    t.resolve_all();
    assert_eq!((t.life(P1), t.life(P2)), (16, 18));
}

#[test]
fn spell_crumple_with_an_illegal_target_goes_to_the_graveyard() {
    cr!("608.2b");
    ruling!(
        "Spell Crumple",
        "If the spell is an illegal target when Spell Crumple tries to resolve, it won’t resolve and none of its effects will happen. Spell Crumple will be put into its owner’s graveyard."
    );
    supported("Spell Crumple");
    let mut t = TestGame::new(2);
    lots(&mut t, P1);
    let shock = t.hand(P1, "Shock");
    let shock = t.cast(P1, shock).target(pl(P0)).go();
    let crumple = cast_at(&mut t, P0, "Spell Crumple", &[obj(shock)]);
    let library = t.library_size(P0);
    cast_at(&mut t, P0, "Counterspell", &[obj(shock)]);
    t.resolve(); // Counterspell
    assert_eq!(t.zone(shock), Zone::Graveyard(P1));
    t.resolve(); // Spell Crumple: no legal target
    assert!(!resolved(&t, crumple));
    assert!(t.in_graveyard(P0, "Spell Crumple"));
    assert_eq!(t.library_size(P0), library);
    // With a legal target, it goes to the bottom of its owner's library.
    let mut t = TestGame::new(2);
    lots(&mut t, P1);
    let shock = t.hand(P1, "Shock");
    let shock = t.cast(P1, shock).target(pl(P0)).go();
    let crumple = cast_at(&mut t, P0, "Spell Crumple", &[obj(shock)]);
    t.resolve();
    assert!(resolved(&t, crumple));
    assert_eq!(*crate::r_s33_common::library_top_first(&t, P0).last().unwrap(), t.g.current(crumple));
    assert_eq!(*crate::r_s33_common::library_top_first(&t, P1).last().unwrap(), t.g.current(shock));
}

#[test]
fn a_spell_targeting_only_fblthp_doesnt_resolve() {
    cr!("608.2b", "603.2", "701.20a");
    ruling!(
        "Fblthp, the Lost",
        "If the spell that targets Fblthp has no other targets, it won't resolve (because it no longer has a legal target after Fblthp has gotten totally lost in your library)."
    );
    supported("Fblthp, the Lost");
    let mut t = TestGame::new(2);
    let f = t.battlefield(P0, "Fblthp, the Lost");
    let library = t.library_size(P0);
    lots(&mut t, P1);
    let shock = t.hand(P1, "Shock");
    let shock = t.cast(P1, shock).target(obj(f)).go();
    t.resolve(); // Fblthp's trigger
    assert_eq!(t.library_size(P0), library + 1);
    t.resolve(); // Shock: no legal target
    assert!(!resolved(&t, shock));
    assert!(t.in_graveyard(P1, "Shock"));
}

#[test]
fn urzas_rage_with_an_illegal_target_doesnt_resolve_even_kicked() {
    cr!("608.2b", "702.33a");
    ruling!(
        "Urza's Rage",
        "If the target for Urza's Rage becomes an illegal target, the spell won't resolve and will be put into its owner's graveyard with no effect. This is true even if its damage can't be prevented and even though it can't be countered."
    );
    supported("Urza's Rage");
    for kicked in [false, true] {
        let mut t = TestGame::new(2);
        let bears = t.battlefield(P1, "Grizzly Bears");
        lots(&mut t, P0);
        let r = t.hand(P0, "Urza's Rage");
        let r = t.cast(P0, r).kicked(kicked).target(obj(bears)).go();
        hexproof(&mut t, bears);
        t.resolve_all();
        assert!(!resolved(&t, r), "kicked {kicked}");
        assert!(t.in_graveyard(P0, "Urza's Rage"), "kicked {kicked}");
        assert!(t.on_battlefield(bears) && t.obj_now(bears).damage == 0, "kicked {kicked}");
    }
}

/// P0's Mythos of Illuna (with {R}{G} spent) copying `target`; returns the token (its
/// fight trigger, targeting `fight`, left on the stack).
fn mythos_token(t: &mut TestGame, target: ObjectId, fight: Option<ObjectId>) -> ObjectId {
    supported("Mythos of Illuna");
    let before = t.g.battlefield.clone();
    mana(t, P0, mtg_engine::mana::ManaType::U, 2);
    mana(t, P0, mtg_engine::mana::ManaType::R, 1);
    mana(t, P0, mtg_engine::mana::ManaType::G, 1);
    let m = t.hand(P0, "Mythos of Illuna");
    t.cast(P0, m).target(obj(target)).go();
    if let Some(f) = fight {
        t.answer_targets(P0, &[obj(f)]);
    }
    t.resolve();
    t.settle();
    let toks = new_tokens(t, P0, &before);
    assert_eq!(toks.len(), 1);
    toks[0]
}

#[test]
fn mythos_tokens_fight_trigger_checks_its_target_and_the_token_on_resolution() {
    cr!("608.2b", "603.4", "608.2h", "701.12b");
    ruling!(
        "Mythos of Illuna",
        "If the target creature is an illegal target or if the token is no longer a creature when the token's granted ability tries to resolve, the ability doesn't resolve. If it's a legal target but the token is no longer on the battlefield when the ability resolves, the target creature won't deal or be dealt damage."
    );
    supported("Song of the Dryads");
    // The target gains hexproof: no fight.
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P1, "Hill Giant");
    let ogre = t.battlefield(P1, "Gray Ogre");
    let tok = mythos_token(&mut t, giant, Some(ogre));
    assert_eq!(t.stack_len(), 1);
    hexproof(&mut t, ogre);
    t.resolve_all();
    assert!(t.on_battlefield(ogre) && t.obj_now(ogre).damage == 0);
    assert_eq!(t.obj_now(tok).damage, 0);
    // The token stops being a creature (Song of the Dryads): no fight.
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P1, "Hill Giant");
    let ogre = t.battlefield(P1, "Gray Ogre");
    let tok = mythos_token(&mut t, giant, Some(ogre));
    crate::r_s06_common::attach_new(&mut t, P1, "Song of the Dryads", obj(tok));
    assert!(!t.obj_now(tok).is(CardType::Creature));
    t.resolve_all();
    assert!(t.on_battlefield(ogre) && t.obj_now(ogre).damage == 0);
    // The token leaves the battlefield: the ability resolves, but no damage either way.
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P1, "Hill Giant");
    let ogre = t.battlefield(P1, "Gray Ogre");
    let tok = mythos_token(&mut t, giant, Some(ogre));
    kill(&mut t, tok);
    t.resolve_all();
    assert!(t.on_battlefield(ogre) && t.obj_now(ogre).damage == 0);
    // All legal: the Ogre dies, the token is dealt 2.
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P1, "Hill Giant");
    let ogre = t.battlefield(P1, "Gray Ogre");
    let tok = mythos_token(&mut t, giant, Some(ogre));
    t.resolve_all();
    assert!(!t.on_battlefield(ogre));
    assert_eq!(t.obj_now(tok).damage, 2);
}

#[test]
fn mythos_tokens_fight_trigger_needs_a_creature_as_it_enters() {
    cr!("603.4", "603.2");
    ruling!(
        "Mythos of Illuna",
        "If the token isn't a creature immediately after it enters the battlefield, the granted ability won't trigger. You can't take actions to make that token into a creature after it has entered the battlefield."
    );
    supported("Sunglasses of Urza");
    let mut t = TestGame::new(2);
    let glasses = t.battlefield(P1, "Sunglasses of Urza");
    t.battlefield(P1, "Gray Ogre");
    let tok = mythos_token(&mut t, glasses, None);
    assert!(has_ability(&t, tok, "fights up to one target creature"));
    assert_eq!(t.stack_len(), 0, "no trigger");
    assert_eq!(triggers_on_stack(&t, "fights"), 0);
}

#[test]
fn welcome_to_the_fold_checks_toughness_only_as_it_resolves() {
    cr!("608.2h", "611.2a");
    ruling!(
        "Welcome to the Fold",
        "The toughness of the creature is checked only as Welcome to the Fold resolves. It doesn't matter if it has higher toughness as you cast Welcome to the Fold, or if its toughness becomes higher after Welcome to the Fold has resolved."
    );
    supported("Welcome to the Fold");
    supported("Disfigure");
    // A 3/3 as it's cast, a 1/1 (Disfigure) as it resolves: gained.
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P1, "Hill Giant");
    cast_at(&mut t, P0, "Welcome to the Fold", &[obj(giant)]);
    cast_at(&mut t, P0, "Disfigure", &[obj(giant)]);
    t.resolve_all();
    assert_eq!(ctl(&t, giant), P0);
    // A 2/2 as it resolves, a 5/5 afterwards: still gained.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    cast_at(&mut t, P0, "Welcome to the Fold", &[obj(bears)]);
    t.resolve_all();
    assert_eq!(ctl(&t, bears), P0);
    cast_at(&mut t, P1, "Giant Growth", &[obj(bears)]);
    t.resolve_all();
    assert_eq!(t.pt(bears), (5, 5));
    assert_eq!(ctl(&t, bears), P0);
    // A 3/3 as it resolves: not gained.
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P1, "Hill Giant");
    cast_at(&mut t, P0, "Welcome to the Fold", &[obj(giant)]);
    t.resolve_all();
    assert_eq!(ctl(&t, giant), P1);
}

#[test]
fn elvish_healer_checks_the_creatures_color_as_the_ability_resolves() {
    cr!("608.2h", "615.1");
    ruling!(
        "Elvish Healer",
        "It checks the color of the creature on resolution of the ability."
    );
    supported("Elvish Healer");
    // (the creature, green as the ability resolves, damage left after a Shock)
    for (creature, green, left) in [("Gray Ogre", true, 0), ("Grizzly Bears", false, 1)] {
        let mut t = TestGame::new(2);
        let healer = t.battlefield(P0, "Elvish Healer");
        let c = t.battlefield(P0, creature);
        act_only(&mut t, P0, healer, 0, &[obj(c)]);
        make_color(&mut t, c, if green { Color::Green } else { Color::Blue });
        t.resolve_all();
        crate::r_p194_common::shock(&mut t, P1, obj(c));
        assert_eq!(t.obj_now(c).damage, left, "{creature}");
    }
}

#[test]
fn ether_well_chooses_the_destination_on_resolution() {
    cr!("608.2h", "608.2d");
    ruling!(
        "Ether Well",
        "The choice of where to put the creature is made on resolution."
    );
    supported("Ether Well");
    // Gray Ogre (red) turned blue in response: no choice, on top. Grizzly Bears turned
    // red: the choice is offered; taken, the Bears go to the bottom.
    for (creature, red_then) in [("Gray Ogre", false), ("Grizzly Bears", true)] {
        let mut t = TestGame::new(2);
        let c = t.battlefield(P1, creature);
        cast_at(&mut t, P0, "Ether Well", &[obj(c)]);
        make_color(&mut t, c, if red_then { Color::Red } else { Color::Blue });
        let from = t.asked().len();
        t.answer_yes(P0, true);
        t.resolve_all();
        let asked = asked_since(&t, from)
            .iter()
            .any(|(p, d)| *p == P0 && matches!(d, Decision::YesNo { .. }));
        assert_eq!(asked, red_then, "{creature}");
        let lib = crate::r_s33_common::library_top_first(&t, P1);
        if red_then {
            assert_eq!(*lib.last().unwrap(), t.g.current(c), "{creature}: bottom");
        } else {
            assert_eq!(lib[0], t.g.current(c), "{creature}: top");
        }
    }
}

#[test]
fn x_of_consult_the_star_charts_and_the_final_days_is_computed_as_they_resolve() {
    cr!("608.2h");
    ruling!(
        "Consult the Star Charts",
        "The value of X is calculated only once, as Consult the Star Charts resolves."
    );
    ruling!(
        "The Final Days",
        "The value of X is calculated only once, as The Final Days resolves."
    );
    supported("Consult the Star Charts");
    supported("The Final Days");
    // Three lands as it's cast, two as it resolves: two cards are looked at.
    let mut t = TestGame::new(2);
    let lands = t.lands(P0, "Island", 3);
    let c = t.hand(P0, "Consult the Star Charts");
    t.cast(P0, c).kicked(false).go();
    kill(&mut t, lands[0]);
    let from = t.asked().len();
    let library = t.library_size(P0);
    t.resolve_all();
    let offered: Vec<usize> = asked_since(&t, from)
        .iter()
        .filter_map(|(p, d)| match d {
            Decision::ChooseEntities { candidates, .. } if *p == P0 => Some(candidates.len()),
            _ => None,
        })
        .collect();
    assert_eq!(offered, vec![2], "{:?}", asked_since(&t, from));
    assert_eq!(t.hand_size(P0), 1);
    assert_eq!(t.library_size(P0), library - 1);
    // The Final Days flashed back with one creature card in the graveyard as it's cast
    // and two as it resolves: two Horrors.
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    t.graveyard(P0, "Grizzly Bears");
    let fd = t.graveyard(P0, "The Final Days");
    cast_from_graveyard(&mut t, P0, fd, &[]);
    t.graveyard(P0, "Hill Giant");
    t.resolve_all();
    assert_eq!(with_subtype(&t, P0, "Horror").len(), 2);
}
