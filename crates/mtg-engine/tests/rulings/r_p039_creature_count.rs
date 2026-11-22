//! Rulings batch P039 — abilities that count creatures (Portcullis, Voice of Many, the
//! Offerings, Rally for the Throne, Deploy, Blunt the Assault, Fumigate, Muerra), and
//! Oketra the True, Protect the Negotiators and Darksteel Citadel.

use crate::r_p039_common::*;
use crate::r_s01_common::{creatures, give_mana_for, supported, triggers_on_stack};
use crate::r_s02_common::{can_attack, can_cast, can_play_land, destroy};
use mtg_engine::decision::{Answer, Decision};
use mtg_engine::mana::ManaType;
use mtg_engine::object::{CastMethod, Zone};
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

// ---------------------------------------------------------------------------
// Counted again on resolution
// ---------------------------------------------------------------------------

#[test]
fn portcullis_checks_the_creature_count_again_on_resolution() {
    cr!("603.4");
    interp!("other-than-trigger-object");
    ruling!(
        "Portcullis",
        "You check the count of the number of creatures on the battlefield again during resolution."
    );
    supported("Portcullis");
    for kill_one in [false, true] {
        let mut t = TestGame::new(2);
        t.battlefield(P0, "Portcullis");
        let a = t.battlefield(P1, "Grizzly Bears");
        t.battlefield(P1, "Grizzly Bears");
        let giant = enter(&mut t, P1, "Hill Giant");
        assert_eq!(triggers_on_stack(&t, "exile that creature"), 1);
        if kill_one {
            // Two creatures on the battlefield with the Giant: one other creature.
            destroy(&mut t, a);
        }
        t.resolve_all();
        assert_eq!(t.on_battlefield(giant), kill_one, "kill_one={kill_one}");
    }
}

#[test]
fn animated_portcullis_counts_itself_among_the_other_creatures() {
    cr!("603.4");
    interp!("other-than-trigger-object");
    supported("March of the Machines");
    // March of the Machines makes Portcullis a 4/4 artifact creature. With one Bears,
    // a Hill Giant entering sees two creatures other than itself: Portcullis and the
    // Bears ("other" still means other than the entering creature).
    for march in [false, true] {
        let mut t = TestGame::new(2);
        let p = t.battlefield(P0, "Portcullis");
        if march {
            t.battlefield(P0, "March of the Machines");
        }
        t.battlefield(P1, "Grizzly Bears");
        t.g.recompute();
        assert_eq!(creature_now(&t, p), march);
        let giant = enter(&mut t, P1, "Hill Giant");
        assert_eq!(triggers_on_stack(&t, "exile that creature"), march as usize);
        t.resolve_all();
        assert_eq!(t.on_battlefield(giant), !march, "march={march}");
    }
}

#[test]
fn voice_of_many_compares_on_resolution_and_counts_itself() {
    cr!("608.2h");
    ruling!(
        "Voice of Many",
        "You compare how many creatures opponents control only as Voice of Many’s triggered ability resolves. If it’s still on the battlefield, it’ll count among the creatures you control."
    );
    supported("Voice of Many");
    // P1 and P2 each have a creature as the ability triggers; P1's dies before it
    // resolves: P0 (Voice of Many) has more creatures than P1 only, and draws one card —
    // none if the counts were taken when it triggered or Voice of Many weren't counted.
    for both_die in [false, true] {
        let mut t = TestGame::new(3);
        let a = t.battlefield(P1, "Grizzly Bears");
        let b = t.battlefield(P2, "Grizzly Bears");
        let hand = t.hand_size(P0);
        enter(&mut t, P0, "Voice of Many");
        destroy(&mut t, a);
        if both_die {
            destroy(&mut t, b);
        }
        t.resolve_all();
        assert_eq!(t.hand_size(P0), hand + 1 + both_die as usize);
    }
}

// ---------------------------------------------------------------------------
// Create tokens, then gain life for each creature: one resolution
// ---------------------------------------------------------------------------

/// Resolves the spell on top of the stack, checking that the creatures' "enters"
/// triggers (Soul Warden) wait for it to finish: P0's life right after it resolves.
fn resolve_with_soul_warden(t: &mut TestGame) -> i32 {
    t.g.resolve_top();
    let life = t.life(P0);
    // The tokens' Soul Warden triggers are only now put on the stack.
    t.settle();
    assert_eq!(triggers_on_stack(t, "you gain 1 life"), 2);
    life
}

#[test]
fn rally_for_the_throne_counts_the_tokens_it_just_created() {
    cr!("608.2c", "603.3");
    ruling!(
        "Rally for the Throne",
        "You create Humans and gain life all while Rally for the Throne is resolving. Nothing can happen between the two, and no player may choose to take actions."
    );
    supported("Rally for the Throne");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Soul Warden");
    t.lands(P0, "Plains", 3);
    let c = t.hand(P0, "Rally for the Throne");
    t.cast(P0, c).go();
    // Soul Warden + two Humans.
    assert_eq!(resolve_with_soul_warden(&mut t), 23);
}

#[test]
fn deploy_counts_the_thopters_it_just_created() {
    cr!("608.2c", "603.3", "709.3");
    ruling!(
        "Depose // Deploy",
        "You create Thopters and gain life all while Deploy is resolving. Nothing can happen between the two, and no player may choose to take actions."
    );
    supported("Depose // Deploy");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Soul Warden");
    t.battlefield(P0, "Grizzly Bears");
    t.lands(P0, "Plains", 2);
    t.lands(P0, "Island", 2);
    let c = t.hand(P0, "Depose // Deploy");
    t.cast(P0, c).method(CastMethod::Half(1)).go();
    // Soul Warden, Bears and two Thopters.
    assert_eq!(resolve_with_soul_warden(&mut t), 24);
}

#[test]
fn blunt_the_assault_counts_every_creature_on_the_battlefield() {
    cr!("608.2h");
    ruling!(
        "Blunt the Assault",
        "You gain 1 life for each creature on the battlefield, not just for each attacking and blocking creature."
    );
    let mut t = TestGame::new(2);
    let attacker = t.battlefield(P1, "Grizzly Bears");
    t.battlefield(P1, "Grizzly Bears");
    t.battlefield(P1, "Grizzly Bears");
    t.battlefield(P0, "Grizzly Bears");
    t.battlefield(P0, "Hill Giant");
    t.set_step(P1, Step::BeginningOfCombat);
    t.answer(
        P1,
        DecisionKind::Attackers,
        Answer::Attackers(vec![(attacker, pl(P0))]),
    );
    t.advance_to(P1, Step::DeclareAttackers);
    cast_named(&mut t, P0, "Blunt the Assault", &[]);
    t.resolve_all();
    // Five creatures, one of them attacking.
    assert_eq!(t.life(P0), 25);
}

#[test]
fn fumigate_counts_only_creatures_actually_destroyed() {
    cr!("701.8a", "702.12b");
    ruling!(
        "Fumigate",
        "You won't gain 1 life for a creature that isn't actually destroyed, such as a creature with indestructible."
    );
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Grizzly Bears");
    t.battlefield(P1, "Grizzly Bears");
    t.battlefield(P1, "Hill Giant");
    let myr = t.battlefield(P1, "Darksteel Myr");
    cast_named(&mut t, P0, "Fumigate", &[]);
    t.resolve_all();
    assert!(t.on_battlefield(myr));
    assert_eq!(t.life(P0), 23);
}

// ---------------------------------------------------------------------------
// The Offerings: choose an opponent for each effect as it resolves
// ---------------------------------------------------------------------------

/// The "choose an opponent" decisions asked since `from`: who was asked and the
/// candidates.
fn opponent_choices(t: &TestGame, from: usize) -> Vec<(PlayerId, Vec<Entity>)> {
    t.asked()[from..]
        .iter()
        .filter_map(|(p, d)| match d {
            Decision::ChooseEntities {
                candidates, prompt, ..
            } if prompt.contains("Choose a player") => Some((*p, candidates.clone())),
            _ => None,
        })
        .collect()
}

/// P0 casts the Offering `name` in a three-player game where P1 has a hexproof player
/// (Leyline of Sanctity), choosing `first` then `second` as it resolves. Checks that no
/// opponent was chosen while it was cast, and that P1 can be chosen though it can't be
/// targeted. Returns the game after resolution.
fn offering(name: &str, x: Option<i64>, first: PlayerId, second: PlayerId) -> TestGame {
    supported(name);
    let mut t = TestGame::new(3);
    t.battlefield(P1, "Leyline of Sanctity");
    t.set_step(P0, Step::PrecombatMain);
    mana_all(&mut t, P0, 6);
    let c = t.hand(P0, name);
    let from = mark(&t);
    let b = t.cast(P0, c);
    let b = match x {
        Some(x) => b.x(x),
        None => b,
    };
    b.go();
    assert!(opponent_choices(&t, from).is_empty(), "chosen while cast");
    t.answer_choose(P0, &[pl(first)]);
    t.answer_choose(P0, &[pl(second)]);
    let from = mark(&t);
    t.resolve_all();
    let asked = opponent_choices(&t, from);
    assert_eq!(asked.len(), 2, "{asked:?}");
    for (p, cands) in &asked {
        assert_eq!(*p, P0);
        assert!(cands.contains(&pl(P1)) && cands.contains(&pl(P2)));
        assert!(!cands.contains(&pl(P0)));
    }
    t
}

fn spirits(t: &TestGame, p: PlayerId) -> usize {
    named(t, p, "Spirit Token").len()
}

#[test]
fn benevolent_offering_chooses_opponents_on_resolution() {
    cr!("608.2d", "115.10");
    ruling!(
        "Benevolent Offering",
        "You choose the opponents for each effect as the spell resolves."
    );
    ruling!(
        "Benevolent Offering",
        "You may choose the same opponent for each of the effects, or you may choose different opponents. None of the affected players are targets of the spell."
    );
    // Different opponents: P1 gets the Spirits, P2 the life.
    let t = offering("Benevolent Offering", None, P1, P2);
    assert_eq!((spirits(&t, P0), spirits(&t, P1), spirits(&t, P2)), (3, 3, 0));
    assert_eq!((t.life(P0), t.life(P1), t.life(P2)), (26, 20, 20));
    // The same opponent (with a hexproof player) for both.
    let t = offering("Benevolent Offering", None, P1, P1);
    assert_eq!((t.life(P0), t.life(P1), t.life(P2)), (26, 26, 20));
}

#[test]
fn infernal_offering_chooses_opponents_on_resolution() {
    cr!("608.2d", "115.10");
    ruling!(
        "Infernal Offering",
        "You choose the opponents for each effect as the spell resolves."
    );
    ruling!(
        "Infernal Offering",
        "You may choose the same opponent for each of the effects, or you may choose different opponents. None of the affected players are targets of the spell."
    );
    let t = offering("Infernal Offering", None, P2, P1);
    // (Nobody had a creature: no one sacrificed or drew; P0 drew nothing.)
    assert_eq!(t.hand_size(P2), 0);
}

#[test]
fn intellectual_offering_chooses_opponents_on_resolution() {
    cr!("608.2d", "115.10");
    ruling!(
        "Intellectual Offering",
        "You choose the opponents for each effect as the spell resolves."
    );
    ruling!(
        "Intellectual Offering",
        "You may choose the same opponent for each of the effects, or you may choose different opponents. None of the affected players are targets of the spell."
    );
    let t = offering("Intellectual Offering", None, P1, P2);
    assert_eq!((t.hand_size(P1), t.hand_size(P2)), (3, 0));
    let t = offering("Intellectual Offering", None, P2, P2);
    assert_eq!((t.hand_size(P1), t.hand_size(P2)), (0, 3));
}

#[test]
fn sylvan_offering_chooses_opponents_on_resolution() {
    cr!("608.2d", "115.10");
    ruling!(
        "Sylvan Offering",
        "You choose the opponents for each effect as the spell resolves."
    );
    ruling!(
        "Sylvan Offering",
        "You may choose the same opponent for each of the effects, or you may choose different opponents. None of the affected players are targets of the spell."
    );
    let t = offering("Sylvan Offering", Some(2), P1, P2);
    // P1 gets a 2/2 Treefolk, P2 two Elf Warriors.
    assert_eq!(creatures(&t, P1).len(), 1);
    assert_eq!(creatures(&t, P2).len(), 2);
    assert_eq!(creatures(&t, P0).len(), 3);
}

// ---------------------------------------------------------------------------
// Muerra, Trash Tactician
// ---------------------------------------------------------------------------

#[test]
fn muerra_adds_a_chosen_color_for_each_raccoon() {
    cr!("603.2");
    ruling!(
        "Muerra, Trash Tactician",
        "You choose {R} or {G} for each Raccoon you control. You aren’t limited to only one color of mana."
    );
    supported("Muerra, Trash Tactician");
    let mut t = TestGame::new(2);
    old(&mut t, P0, "Muerra, Trash Tactician");
    old(&mut t, P0, "Bark-Knuckle Boxer");
    old(&mut t, P0, "Bark-Knuckle Boxer");
    t.set_step(P0, Step::Upkeep);
    // R for the first Raccoon, G for the other two.
    for i in [0usize, 1, 1] {
        t.answer(P0, DecisionKind::Option, Answer::Index(i));
    }
    t.advance_to(P0, Step::PrecombatMain);
    t.resolve_all();
    let pool = &t.g.players[0].mana_pool;
    assert_eq!(
        (pool.count(ManaType::R), pool.count(ManaType::G)),
        (1, 2)
    );
}

#[test]
fn muerra_cards_stay_playable_after_muerra_leaves() {
    cr!("611.2a");
    ruling!(
        "Muerra, Trash Tactician",
        "You may play the exiled cards even if Muerra is no longer on the battlefield or under your control."
    );
    let mut t = TestGame::new(2);
    let muerra = t.battlefield(P0, "Muerra, Trash Tactician");
    stack_two(&mut t);
    // Spend eight mana on spells: Craw Wurm (6) and Grizzly Bears (2).
    for name in ["Craw Wurm", "Grizzly Bears"] {
        cast_named(&mut t, P0, name, &[]);
        t.resolve_all();
    }
    let forest = t.g.find_in_zone(Zone::Exile, "Forest");
    let bears = t.g.find_in_zone(Zone::Exile, "Llanowar Elves");
    assert_eq!((forest.len(), bears.len()), (1, 1));
    destroy(&mut t, muerra);
    assert!(can_play_land(&mut t, P0, forest[0]));
    give_mana_for(&mut t, P0, "Llanowar Elves");
    assert!(can_cast(&mut t, P0, bears[0], CastMethod::Normal));
}

/// Puts Llanowar Elves and a Forest on top of P0's library.
fn stack_two(t: &mut TestGame) {
    t.library_top(P0, "Llanowar Elves");
    t.library_top(P0, "Forest");
}

// ---------------------------------------------------------------------------
// Oketra, Protect the Negotiators, Darksteel Citadel
// ---------------------------------------------------------------------------

#[test]
fn oketra_can_attack_and_block_alone_with_three_other_creatures() {
    cr!("508.1c", "509.1b");
    ruling!(
        "Oketra the True",
        "You don't have to attack with three other creatures for Oketra to be able to attack. The same is true of blocking."
    );
    supported("Oketra the True");
    for others in [2usize, 3] {
        let mut t = TestGame::new(2);
        let oketra = old(&mut t, P0, "Oketra the True");
        for _ in 0..others {
            old(&mut t, P0, "Grizzly Bears");
        }
        t.set_step(P0, Step::BeginningOfCombat);
        assert_eq!(can_attack(&mut t, oketra), others == 3);
        if others == 3 {
            t.attack(&[(oketra, pl(P1))], &[]);
            // Double strike: 6 damage.
            assert_eq!(t.life(P1), 14);
        }
        // Blocking alone.
        let mut t2 = TestGame::new(2);
        let oketra = old(&mut t2, P0, "Oketra the True");
        for _ in 0..others {
            old(&mut t2, P0, "Grizzly Bears");
        }
        let ogre = old(&mut t2, P1, "Hill Giant");
        t2.set_step(P1, Step::BeginningOfCombat);
        t2.attack(&[(ogre, pl(P0))], &[(oketra, ogre)]);
        assert_eq!(t2.on_battlefield(ogre), others != 3, "others={others}");
        assert_eq!(t2.life(P0), if others == 3 { 20 } else { 17 });
    }
}

#[test]
fn protect_the_negotiators_needs_a_spell_to_target() {
    cr!("601.2c", "115.1");
    ruling!(
        "Protect the Negotiators",
        "You may not cast Protect the Negotiators without a spell to target."
    );
    supported("Protect the Negotiators");
    let mut t = TestGame::new(2);
    mana_all(&mut t, P0, 2);
    let c = t.hand(P0, "Protect the Negotiators");
    assert!(!can_cast(&mut t, P0, c, CastMethod::Normal));
    // With a spell on the stack, it can.
    let s = t.hand(P1, "Shock");
    mana(&mut t, P1, ManaType::R, 1);
    t.cast(P1, s).target(pl(P0)).go();
    assert!(can_cast(&mut t, P0, c, CastMethod::Normal));
}

#[test]
fn darksteel_citadel_is_played_as_a_land_not_cast() {
    cr!("305.1", "305.2", "305.9");
    ruling!(
        "Darksteel Citadel",
        "Although Darksteel Citadel is an artifact, it can't be cast as a spell. Playing it follows the normal rules for playing a land."
    );
    let mut t = TestGame::new(2);
    mana_all(&mut t, P0, 3);
    let c = t.hand(P0, "Darksteel Citadel");
    assert!(!can_cast(&mut t, P0, c, CastMethod::Normal));
    assert!(can_play_land(&mut t, P0, c));
    // It uses the land play: after another land, it can't be played this turn.
    let f = t.hand(P0, "Forest");
    t.play_land(P0, f).unwrap();
    assert!(!can_play_land(&mut t, P0, c));
    // Nor at instant speed in the opponent's turn.
    let mut t = TestGame::new(2);
    let c = t.hand(P0, "Darksteel Citadel");
    t.set_step(P1, Step::PrecombatMain);
    assert!(!can_play_land(&mut t, P0, c));
}
