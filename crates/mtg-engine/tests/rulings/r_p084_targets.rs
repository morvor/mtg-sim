//! Rulings batch P084 — choosing graveyard targets (CR 115, 601.2c, 602.2b, 603.3d):
//! what a spell can't target, abilities that need several targets in one graveyard,
//! "up to" targets, triggers of cards that are already in the graveyard, and Lochmere
//! Serpent's and Spelltwine's partly illegal targets.

use crate::r_p084_common::*;
use crate::r_s02_common::target_candidates;
use mtg_engine::ability::AbilityKind;
use mtg_engine::decision::Decision;
use mtg_engine::object::{CastMethod, Zone};
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

/// Whether `p` could activate the ability of `source` whose text contains `needle` now.
fn can_activate_containing(t: &mut TestGame, p: PlayerId, source: ObjectId, needle: &str) -> bool {
    t.g.turn.priority = Some(p);
    t.g.recompute();
    let s = t.g.current(source);
    let uid =
        t.g.obj(s)
            .chars
            .abilities
            .iter()
            .find(|a| matches!(a.kind, AbilityKind::Activated(_)) && a.text.contains(needle))
            .map(|a| a.uid)
            .expect("no such activated ability");
    t.g.legal_actions(p)
        .iter()
        .any(|a| matches!(a, Action::Activate { source: x, ability } if *x == s && *ability == uid))
}

/// Whether `p` could cast `card` now.
fn can_cast(t: &mut TestGame, p: PlayerId, card: ObjectId) -> bool {
    t.g.turn.priority = Some(p);
    t.g.recompute();
    t.g.legal_actions(p)
        .iter()
        .any(|a| matches!(a, Action::Cast { card: c, .. } if *c == card))
}

#[test]
fn a_spell_on_the_stack_cant_target_itself_in_a_graveyard() {
    cr!("601.2a", "601.2c", "115.1");
    ruling!(
        "Beckon Apparition",
        "Beckon Apparition will be on the stack when you choose its target. It can’t target itself."
    );
    ruling!(
        "Stream of Consciousness",
        "Stream of Consciousness can’t be one of the target cards you shuffle into your library."
    );
    // Beckon Apparition: the other cards in graveyards are candidates; the spell isn't.
    let mut t = TestGame::new(2);
    let other = t.graveyard(P0, "Beckon Apparition");
    let from = t.asked().len();
    let spell = cast_modes(&mut t, P0, "Beckon Apparition", "B", &[], &[&[obj(other)]]);
    let c = target_candidates(&t, P0, from);
    assert_eq!(c, vec![vec![obj(other)]]);
    assert!(!c[0].contains(&obj(spell)));
    assert!(matches!(t.zone(spell), Zone::Stack));
    // Stream of Consciousness, put into the graveyard as it resolves, isn't shuffled in.
    let mut t = TestGame::new(2);
    let bears = t.graveyard(P0, "Grizzly Bears");
    let from = t.asked().len();
    let spell = cast_modes(
        &mut t,
        P0,
        "Stream of Consciousness",
        "1U",
        &[],
        &[&[pl(P0)], &[obj(bears)]],
    );
    let c = target_candidates(&t, P0, from);
    assert!(c.iter().all(|v| !v.contains(&obj(spell))), "{c:?}");
    t.resolve_all();
    assert!(matches!(t.zone(bears), Zone::Library(P0)));
    assert_eq!(t.zone(spell), Zone::Graveyard(P0));
}

#[test]
fn several_targets_from_a_single_opponents_graveyard_are_required() {
    cr!("602.2b", "601.2c", "115.1");
    ruling!(
        "Deadeye Tracker",
        "You can't activate Deadeye Tracker's ability without targeting two cards in a single opponent's graveyard."
    );
    ruling!(
        "Lochmere Serpent",
        "You can't activate Lochmere Serpent's last ability without targeting five cards in a single opponent's graveyard."
    );
    // (source, in graveyard?, needle, mana, cards needed)
    for (name, from_gy, needle, cost, n) in [
        ("Deadeye Tracker", false, "Exile two target", "1B", 2),
        ("Lochmere Serpent", true, "Exile five target", "UB", 5),
    ] {
        supported(name);
        // Three players: enough cards among both opponents' graveyards, too few in each.
        let mut t = TestGame::new(3);
        let src = if from_gy {
            t.graveyard(P0, name)
        } else {
            t.battlefield(P0, name)
        };
        for _ in 0..n - 1 {
            t.graveyard(P1, "Grizzly Bears");
            t.graveyard(P2, "Grizzly Bears");
        }
        // Enough in P0's own graveyard doesn't count either.
        for _ in 0..n {
            t.graveyard(P0, "Lightning Bolt");
        }
        mana(&mut t, P0, cost);
        assert!(!can_activate_containing(&mut t, P0, src, needle), "{name}");
        // One more card in one opponent's graveyard: now it can be activated.
        t.graveyard(P2, "Hill Giant");
        assert!(can_activate_containing(&mut t, P0, src, needle), "{name}");
    }
}

#[test]
fn selesnya_eulogist_needs_a_creature_card_in_a_graveyard() {
    cr!("602.2b", "601.2c");
    ruling!(
        "Selesnya Eulogist",
        "You can't activate Selesnya Eulogist's ability without a target creature card in a graveyard."
    );
    let mut t = TestGame::new(2);
    let e = t.battlefield(P0, "Selesnya Eulogist");
    t.graveyard(P1, "Lightning Bolt");
    t.graveyard(P0, "Forest");
    mana(&mut t, P0, "2G");
    assert!(!can_activate_containing(
        &mut t,
        P0,
        e,
        "Exile target creature"
    ));
    t.graveyard(P1, "Grizzly Bears");
    assert!(can_activate_containing(
        &mut t,
        P0,
        e,
        "Exile target creature"
    ));
}

#[test]
fn spelltwine_needs_both_targets() {
    cr!("601.2c", "115.1");
    ruling!(
        "Spelltwine",
        "Spelltwine has two targets: the instant or sorcery card in your graveyard and the one in an opponent’s graveyard. You can’t cast Spelltwine unless you can choose both legal targets."
    );
    supported("Spelltwine");
    let mut t = TestGame::new(2);
    let st = t.hand(P0, "Spelltwine");
    mana(&mut t, P0, "5U");
    t.graveyard(P0, "Divination");
    t.graveyard(P1, "Grizzly Bears");
    assert!(!can_cast(&mut t, P0, st));
    // Two in P1's graveyard and none in P0's isn't enough either.
    let mut t2 = TestGame::new(2);
    let st2 = t2.hand(P0, "Spelltwine");
    mana(&mut t2, P0, "5U");
    t2.graveyard(P1, "Divination");
    t2.graveyard(P1, "Divination");
    assert!(!can_cast(&mut t2, P0, st2));
    t.graveyard(P1, "Divination");
    assert!(can_cast(&mut t, P0, st));
}

#[test]
fn spelltwine_with_one_illegal_target_still_exiles_and_copies_the_other() {
    cr!("608.2b", "707.10", "707.12");
    ruling!(
        "Spelltwine",
        "If one of Spelltwine’s targets is illegal when Spelltwine tries to resolve, you’ll still exile and copy the remaining legal target."
    );
    let mut t = TestGame::new(2);
    let mine = t.graveyard(P0, "Divination");
    let theirs = t.graveyard(P1, "Divination");
    cast_modes(
        &mut t,
        P0,
        "Spelltwine",
        "5U",
        &[],
        &[&[obj(mine)], &[obj(theirs)]],
    );
    let hand = t.hand_size(P0);
    exile_now(&mut t, theirs);
    t.resolve_all();
    assert_eq!(t.zone(mine), Zone::Exile);
    // One copy of Divination was cast: two cards.
    assert_eq!(t.hand_size(P0), hand + 2);
    assert!(t.in_exile("Spelltwine"));
}

#[test]
fn lochmere_serpent_partly_and_wholly_illegal_targets() {
    cr!("608.2b", "602.2b");
    ruling!(
        "Lochmere Serpent",
        "If each target card is an illegal target by the time Lochmere Serpent's last ability resolves, the entire ability won't resolve. Lochmere Serpent won't be returned."
    );
    ruling!(
        "Lochmere Serpent",
        "If up to four of the target cards are illegal targets by the time Lochmere Serpent's last ability resolves, the remaining legal targets are exiled and Lochmere Serpent is returned to your hand."
    );
    supported("Lochmere Serpent");
    for gone in [0usize, 4, 5] {
        let mut t = TestGame::new(2);
        let serpent = t.graveyard(P0, "Lochmere Serpent");
        let cards: Vec<ObjectId> = (0..5).map(|_| t.graveyard(P1, "Grizzly Bears")).collect();
        let es: Vec<Entity> = cards.iter().map(|c| obj(*c)).collect();
        t.answer_targets(P0, &es);
        activate(&mut t, P0, serpent, "Exile five", "UB", &[]);
        t.settle();
        for c in &cards[..gone] {
            exile_now(&mut t, *c);
        }
        t.resolve_all();
        for c in &cards[gone..] {
            assert_eq!(t.zone(*c), Zone::Exile, "gone={gone}");
        }
        let back = gone < 5;
        assert_eq!(t.in_hand(P0, "Lochmere Serpent"), back, "gone={gone}");
        assert_eq!(t.in_graveyard(P0, "Lochmere Serpent"), !back, "gone={gone}");
    }
}

#[test]
fn skullsnatcher_targets_zero_one_or_two_cards() {
    cr!("115.1", "603.3d", "510.3a");
    ruling!(
        "Skullsnatcher",
        "Skullsnatcher’s ability can target zero, one, or two cards in that player’s graveyard."
    );
    supported("Skullsnatcher");
    for n in [0usize, 1, 2] {
        let mut t = TestGame::new(2);
        let s = t.battlefield(P0, "Skullsnatcher");
        let cards: Vec<ObjectId> = (0..3).map(|_| t.graveyard(P1, "Grizzly Bears")).collect();
        let es: Vec<Entity> = cards[..n].iter().map(|c| obj(*c)).collect();
        t.answer_targets(P0, &es);
        let from = t.asked().len();
        t.attack(&[(s, pl(P1))], &[]);
        t.advance_to(P0, Step::EndOfCombat);
        let asked: Vec<(u32, u32)> = t.asked()[from..]
            .iter()
            .filter_map(|(_, d)| match d {
                Decision::ChooseTargets { min, max, .. } => Some((*min, *max)),
                _ => None,
            })
            .collect();
        assert_eq!(asked, vec![(0, 2)], "n={n}");
        assert_eq!(t.graveyard_size(P1), 3 - n, "n={n}");
    }
}

#[test]
fn faerie_macabre_from_hand_at_instant_speed_any_graveyards() {
    cr!("602.2", "116.1", "115.1");
    ruling!(
        "Faerie Macabre",
        "Faerie Macabre's second ability can be activated only while it's in your hand. You can do this any time you could cast an instant."
    );
    ruling!(
        "Faerie Macabre",
        "The targeted cards may be in the same graveyard or in different graveyards."
    );
    supported("Faerie Macabre");
    // In P1's upkeep with a spell on the stack, P0 discards it from hand.
    let mut t = TestGame::new(2);
    t.set_step(P1, Step::Upkeep);
    let fm = t.hand(P0, "Faerie Macabre");
    let a = t.graveyard(P0, "Grizzly Bears");
    let b = t.graveyard(P1, "Hill Giant");
    let bolt = t.hand(P1, "Lightning Bolt");
    mana(&mut t, P1, "R");
    t.answer_targets(P1, &[pl(P0)]);
    t.g.turn.priority = Some(P1);
    t.g.cast_spell(P1, bolt, CastMethod::Normal).unwrap();
    assert!(can_activate_containing(&mut t, P0, fm, "Exile up to two"));
    t.answer_targets(P0, &[obj(a), obj(b)]);
    activate(&mut t, P0, fm, "Exile up to two", "", &[]);
    assert!(t.in_graveyard(P0, "Faerie Macabre"));
    t.resolve_all();
    assert_eq!(t.zone(a), Zone::Exile);
    assert_eq!(t.zone(b), Zone::Exile);
    // On the battlefield it can't be activated.
    let mut t = TestGame::new(2);
    let fm = t.battlefield(P0, "Faerie Macabre");
    t.graveyard(P1, "Hill Giant");
    assert!(!can_activate_containing(&mut t, P0, fm, "Exile up to two"));
}

#[test]
fn grixis_sojourners_dies_trigger_may_target_itself() {
    cr!("603.10a", "603.3d", "700.4");
    ruling!(
        "Grixis Sojourners",
        "Grixis Sojourner's triggered ability may target Grixis Sojourners itself. By the time the ability triggers, Grixis Sojourner will already be in your graveyard."
    );
    supported("Grixis Sojourners");
    let mut t = TestGame::new(2);
    let gs = t.battlefield(P0, "Grixis Sojourners");
    t.g.destroy(gs, None);
    t.g.flush_events();
    let card = t.g.current(gs);
    assert_eq!(t.zone(card), Zone::Graveyard(P0));
    t.answer_yes(P0, true);
    t.answer_targets(P0, &[obj(card)]);
    let from = t.asked().len();
    t.resolve_all();
    let c = target_candidates(&t, P0, from);
    assert!(c.iter().any(|v| v.contains(&obj(card))), "{c:?}");
    assert!(t.in_exile("Grixis Sojourners"));
}

#[test]
fn ruin_rat_can_exile_a_creature_that_died_with_it() {
    cr!("603.10a", "603.3d", "510.2", "702.2b");
    ruling!(
        "Ruin Rat",
        "If Ruin Rat dies at the same time as another creature, most likely because they were in combat together, its triggered ability can exile that other card."
    );
    supported("Ruin Rat");
    let mut t = TestGame::new(2);
    let rat = t.battlefield(P0, "Ruin Rat");
    let bears = t.battlefield(P1, "Grizzly Bears");
    // P1's only graveyard card is the Bears that died with the Rat: the default target.
    let from = t.asked().len();
    t.attack(&[(rat, pl(P1))], &[(bears, rat)]);
    let card = t.g.current(bears);
    let c = target_candidates(&t, P0, from);
    let [ref only] = c[..] else { panic!("{c:?}") };
    let [Entity::Object(target)] = only[..] else {
        panic!("{c:?}")
    };
    // The target was the Bears card in the graveyard, now exiled.
    assert_eq!(t.g.current(target), card);
    assert_eq!(t.zone(card), Zone::Exile);
    assert!(t.in_graveyard(P0, "Ruin Rat"));
}

#[test]
fn rotfeaster_maggot_must_target_your_own_creature_card_if_it_is_the_only_one() {
    cr!("603.3d", "115.1");
    ruling!(
        "Rotfeaster Maggot",
        "Rotfeaster Maggot’s ability is mandatory. If you are the only player with creature cards in your graveyard, you must target one of them."
    );
    supported("Rotfeaster Maggot");
    let mut t = TestGame::new(2);
    let bears = t.graveyard(P0, "Grizzly Bears");
    t.graveyard(P1, "Lightning Bolt");
    let from = t.asked().len();
    t.enter(P0, "Rotfeaster Maggot");
    t.resolve_all();
    let asked: Vec<(Vec<Entity>, u32)> = t.asked()[from..]
        .iter()
        .filter_map(|(_, d)| match d {
            Decision::ChooseTargets {
                candidates, min, ..
            } => Some((candidates.clone(), *min)),
            _ => None,
        })
        .collect();
    assert_eq!(asked, vec![(vec![obj(bears)], 1)]);
    assert_eq!(t.zone(bears), Zone::Exile);
    // You gain life equal to its toughness.
    assert_eq!(t.life(P0), 22);
}
