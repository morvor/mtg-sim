//! Rulings batch P162 — strive and other "any number of / up to N / X target" spells:
//! what they can target (CR 115.3), the same object only once per word "target"
//! (CR 115.3, 601.2c), partial and total illegality on resolution (CR 608.2b), and
//! values counted on resolution.

use crate::r_p076_common::mana;
use crate::r_p162_common::*;
use crate::r_s01_common::{attack_with, supported, tokens};
use mtg_engine::keywords::KeywordKind;
use mtg_engine::mana::ManaType;
use mtg_engine::object::{CastMethod, Zone};
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

#[test]
fn strive_spells_can_target_your_own_and_untapped_creatures() {
    cr!("115.3", "608.2c");
    ruling!(
        "Blinding Flare",
        "Blinding Flare can target any creature, including those you control."
    );
    ruling!(
        "Harness by Force",
        "Harness by Force can target any creature, even one that's untapped or one you already control."
    );
    supported("Blinding Flare");
    supported("Harness by Force");
    // Blinding Flare on your own creature: it can't block.
    let mut t = TestGame::new(2);
    let mine = t.battlefield(P0, "Grizzly Bears");
    cast_resolve(&mut t, P0, "Blinding Flare", &[Entity::Object(mine)]);
    assert_eq!(t.zone(mine), Zone::Battlefield);
    assert!(t.in_graveyard(P0, "Blinding Flare"));
    t.set_step(P1, Step::BeginningOfCombat);
    let attacker = t.battlefield(P1, "Hill Giant");
    attack_with(&mut t, &[(attacker, Entity::Player(P0))]);
    assert!(!mtg_engine::combat::block_options(&t.g, &[P0])
        .iter()
        .any(|(b, can)| *b == mine && !can.is_empty()));
    // Harness by Force on your own tapped creature and an opponent's untapped one: both
    // untap and gain haste, and you control both.
    let mut t = TestGame::new(2);
    let mine = t.battlefield(P0, "Grizzly Bears");
    t.g.objects[mine.0 as usize].tapped = true;
    let theirs = t.battlefield(P1, "Hill Giant");
    mana(&mut t, P0, ManaType::R, 3);
    mana(&mut t, P0, ManaType::C, 3);
    let h = t.hand(P0, "Harness by Force");
    t.cast(P0, h)
        .targets(&[Entity::Object(mine), Entity::Object(theirs)])
        .go();
    t.resolve_all();
    for id in [mine, theirs] {
        assert_eq!(t.obj_now(id).controller, P0);
        assert!(!t.obj_now(id).tapped);
        assert!(crate::r_s06_common::has_kw(&t, id, KeywordKind::Haste));
    }
}

#[test]
fn up_to_n_targets_cant_be_the_same_creature_twice() {
    cr!("115.3", "601.2c");
    ruling!(
        "Reap What Is Sown",
        "Reap What Is Sown can’t target the same creature multiple times. It will put only one +1/+1 counter on each of its targets."
    );
    ruling!(
        "Gird for Battle",
        "You can't target the same creature twice to give it two +1/+1 counters."
    );
    ruling!(
        "Travel Preparations",
        "You can't target the same creature twice to put two +1/+1 counters on it."
    );
    ruling!(
        "Felidar Savior",
        "You can't target the same creature twice with Felidar Savior's last ability."
    );
    ruling!(
        "Basri's Acolyte",
        "You can't target the same creature twice with the triggered ability of Basri's Acolyte to give it two +1/+1 counters."
    );
    for card in [
        "Reap What Is Sown",
        "Gird for Battle",
        "Travel Preparations",
        "Felidar Savior",
        "Basri's Acolyte",
    ] {
        supported(card);
        let mut t = TestGame::new(2);
        let bears = t.battlefield(P0, "Grizzly Bears");
        let giant = t.battlefield(P0, "Hill Giant");
        // Trying to choose the bears twice (and the giant): the choice is invalid, so the
        // engine's default choice is used; nothing gets two counters.
        t.answer_targets(
            P0,
            &[
                Entity::Object(bears),
                Entity::Object(bears),
                Entity::Object(giant),
            ],
        );
        if card == "Felidar Savior" || card == "Basri's Acolyte" {
            crate::r_s05_common::enter(&mut t, P0, card);
        } else {
            crate::r_s01_common::give_mana_for(&mut t, P0, card);
            let c = t.hand(P0, card);
            t.cast(P0, c).go();
        }
        t.resolve_all();
        for id in [bears, giant] {
            assert!(t.counters(id, "+1/+1") <= 1, "{card}");
        }
        // Valid: each of two different creatures gets one.
        let mut t = TestGame::new(2);
        let bears = t.battlefield(P0, "Grizzly Bears");
        let giant = t.battlefield(P0, "Hill Giant");
        t.answer_targets(P0, &[Entity::Object(bears), Entity::Object(giant)]);
        if card == "Felidar Savior" || card == "Basri's Acolyte" {
            crate::r_s05_common::enter(&mut t, P0, card);
        } else {
            crate::r_s01_common::give_mana_for(&mut t, P0, card);
            let c = t.hand(P0, card);
            t.cast(P0, c).go();
        }
        t.resolve_all();
        for id in [bears, giant] {
            assert_eq!(t.counters(id, "+1/+1"), 1, "{card}");
        }
    }
}

#[test]
fn basris_aegis_with_no_targets_or_partly_illegal_targets() {
    cr!("608.2b", "115.1a");
    ruling!(
        "Basri's Aegis",
        "If do you choose targets and each target creature is an illegal target by the time Basri's Aegis tries to resolve, the spell won't resolve. You won't search for Basri. If just one target is still legal, however, you do put a +1/+1 counter on it and search for Basri."
    );
    ruling!(
        "Basri's Aegis",
        "You can cast Basri's Aegis without any target creatures. You'll just search for Basri, Devoted Paladin."
    );
    supported("Basri's Aegis");
    let basri = "Basri, Devoted Paladin";
    // No targets: just the search.
    let mut t = TestGame::new(2);
    let b = t.library_top(P0, basri);
    t.answer_targets(P0, &[]);
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[Entity::Object(b)]);
    cast_resolve(&mut t, P0, "Basri's Aegis", &[]);
    assert_eq!(t.zone(b), Zone::Hand(P0));
    // Two targets, one becomes illegal: a counter on the other, and the search.
    // Both illegal: no search.
    for both in [false, true] {
        let mut t = TestGame::new(2);
        let b = t.library_top(P0, basri);
        let bears = t.battlefield(P0, "Grizzly Bears");
        let giant = t.battlefield(P0, "Hill Giant");
        crate::r_s01_common::give_mana_for(&mut t, P0, "Basri's Aegis");
        let c = t.hand(P0, "Basri's Aegis");
        t.cast(P0, c)
            .targets(&[Entity::Object(bears), Entity::Object(giant)])
            .go();
        crate::r_s02_common::destroy(&mut t, bears);
        if both {
            crate::r_s02_common::destroy(&mut t, giant);
        }
        t.answer_yes(P0, true);
        t.answer_choose(P0, &[Entity::Object(b)]);
        t.resolve_all();
        if both {
            assert_eq!(t.zone(b), Zone::Library(P0));
        } else {
            assert_eq!(t.counters(giant, "+1/+1"), 1);
            assert_eq!(t.zone(b), Zone::Hand(P0));
        }
    }
}

#[test]
fn jet_counts_creatures_on_resolution() {
    cr!("608.2h");
    ruling!(
        "Jet, Freedom Fighter",
        "The amount of damage Jet's first ability deals is calculated only on resolution of the ability. Only creatures you control when the ability resolves will be counted."
    );
    supported("Jet, Freedom Fighter");
    let mut t = TestGame::new(2);
    let wurm = t.battlefield(P1, "Colossal Dreadmaw");
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.answer_targets(P0, &[Entity::Object(wurm)]);
    crate::r_s05_common::enter(&mut t, P0, "Jet, Freedom Fighter");
    // Before it resolves, the bears leave and two creatures arrive: 1 + 2 = 3.
    crate::r_s02_common::destroy(&mut t, bears);
    t.battlefield(P0, "Hill Giant");
    t.battlefield(P0, "Hill Giant");
    t.resolve_all();
    assert_eq!(crate::r_s07_common::damage_on(&t, wurm), 3);
}

/// P0 casts Release the Gremlins with X = targets.len().
fn gremlins(t: &mut TestGame, targets: &[ObjectId]) -> ObjectId {
    let x = targets.len() as u32;
    mana(t, P0, ManaType::R, 1);
    mana(t, P0, ManaType::C, 2 * x);
    let c = t.hand(P0, "Release the Gremlins");
    let ents: Vec<Entity> = targets.iter().map(|i| Entity::Object(*i)).collect();
    t.cast(P0, c).x(x as i64).targets(&ents).go()
}

fn gremlin_tokens(t: &TestGame) -> usize {
    crate::r_s05_common::tokens_with_subtype(t, P0, "Gremlin").len()
}

#[test]
fn release_the_gremlins_counts_x_not_destroyed_artifacts() {
    cr!("608.2b", "107.3a");
    ruling!(
        "Release the Gremlins",
        "If an artifact becomes an illegal target for Release the Gremlins, it won’t change the number of tokens created. However, if every target artifact becomes an illegal target, Release the Gremlins doesn’t resolve and no tokens are created."
    );
    ruling!(
        "Release the Gremlins",
        "You can target an artifact with indestructible. It won’t be destroyed, and you’ll still create X Gremlin tokens."
    );
    supported("Release the Gremlins");
    // One of two targets illegal: two tokens anyway.
    let mut t = TestGame::new(2);
    let a = t.battlefield(P1, "Ornithopter");
    let b = t.battlefield(P1, "Mind Stone");
    gremlins(&mut t, &[a, b]);
    crate::r_s02_common::destroy(&mut t, a);
    t.resolve_all();
    assert!(t.in_graveyard(P1, "Mind Stone"));
    assert_eq!(gremlin_tokens(&t), 2);
    // All illegal: nothing.
    let mut t = TestGame::new(2);
    let a = t.battlefield(P1, "Ornithopter");
    gremlins(&mut t, &[a]);
    crate::r_s02_common::destroy(&mut t, a);
    t.resolve_all();
    assert_eq!(gremlin_tokens(&t), 0);
    // Indestructible: not destroyed, still X tokens.
    let mut t = TestGame::new(2);
    let myr = t.battlefield(P1, "Darksteel Myr");
    gremlins(&mut t, &[myr]);
    t.resolve_all();
    assert!(t.on_battlefield(myr));
    assert_eq!(gremlin_tokens(&t), 1);
}

#[test]
fn release_the_gremlins_x_is_limited_by_distinct_targets() {
    cr!("115.3", "601.2c");
    ruling!(
        "Release the Gremlins",
        "The value of X is limited by the number of artifacts that you can target. You can’t target the same artifact more than once."
    );
    let mut t = TestGame::new(2);
    let a = t.battlefield(P1, "Ornithopter");
    mana(&mut t, P0, ManaType::R, 1);
    mana(&mut t, P0, ManaType::C, 4);
    let c = t.hand(P0, "Release the Gremlins");
    // X = 2 with only one artifact to target (twice) can't be cast.
    let r = t
        .cast(P0, c)
        .x(2)
        .targets(&[Entity::Object(a), Entity::Object(a)])
        .try_go();
    assert!(r.is_err());
    assert_eq!(t.zone(c), Zone::Hand(P0));
    // X = 1 can.
    t.clear_answers();
    t.cast(P0, c).x(1).target(a).go();
    t.resolve_all();
    assert!(t.in_graveyard(P1, "Ornithopter"));
    assert_eq!(gremlin_tokens(&t), 1);
}

#[test]
fn silence_the_believers_exiles_a_bestowed_aura_with_the_creature() {
    cr!("702.103f", "608.2c");
    ruling!(
        "Silence the Believers",
        "The creatures and Auras are exiled at the same time. Specifically, if an Aura is a permanent with bestow, it will be exiled. It won't stop being an Aura and remain on the battlefield."
    );
    supported("Silence the Believers");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let paci = t.battlefield(P0, "Pacifism");
    t.g.attach(paci, Entity::Object(bears));
    mana(&mut t, P1, ManaType::U, 1);
    mana(&mut t, P1, ManaType::C, 4);
    t.set_step(P1, Step::PrecombatMain);
    let naiad = t.hand(P1, "Nimbus Naiad");
    t.cast(P1, naiad)
        .method(CastMethod::Keyword(KeywordKind::Bestow))
        .target(bears)
        .go();
    t.resolve_all();
    assert!(t.on_battlefield(naiad));
    t.set_step(P0, Step::PrecombatMain);
    cast_resolve(&mut t, P0, "Silence the Believers", &[Entity::Object(bears)]);
    assert_eq!(t.zone(bears), Zone::Exile);
    assert_eq!(t.zone(paci), Zone::Exile);
    assert_eq!(t.zone(naiad), Zone::Exile);
}

#[test]
fn launch_the_fleet_tokens_may_attack_a_different_player() {
    cr!("508.4");
    ruling!(
        "Launch the Fleet",
        "You declare which player or planeswalker each token is attacking as you put it onto the battlefield. It doesn't have to be the same player or planeswalker the original creature is attacking."
    );
    supported("Launch the Fleet");
    let mut t = TestGame::new(3);
    let bears = t.battlefield(P0, "Grizzly Bears");
    cast_resolve(&mut t, P0, "Launch the Fleet", &[Entity::Object(bears)]);
    t.answer_choose(P0, &[Entity::Player(P2)]);
    t.answer(
        P0,
        DecisionKind::Option,
        mtg_engine::decision::Answer::Index(1),
    );
    attack_with(&mut t, &[(bears, Entity::Player(P1))]);
    t.resolve_all();
    let soldiers = tokens(&t, P0);
    assert_eq!(soldiers.len(), 1);
    let target = crate::r_s12_common::attack_target(&t, soldiers[0]);
    assert_eq!(target, Some(Entity::Player(P2)));
    assert_eq!(
        crate::r_s12_common::attack_target(&t, bears),
        Some(Entity::Player(P1))
    );
}
