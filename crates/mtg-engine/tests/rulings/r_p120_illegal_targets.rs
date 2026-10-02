//! Rulings batch P120 — spells and abilities about +1/+1 counters whose targets become
//! illegal before they resolve (CR 608.2b): with every target illegal they don't resolve
//! at all; with some illegal, the rest are affected, a distribution chosen on casting
//! stays as chosen (CR 601.2d) and the share of an illegal target is lost; a fight with
//! an illegal target doesn't happen (CR 701.14b). Abilities on the stack are independent
//! of their source (CR 113.7a).

use crate::r_p120_common::*;
use crate::r_s29_common::damage_marked;
use mtg_engine::decision::Answer;
use mtg_engine::keywords::KeywordKind;
use mtg_engine::testing::*;
use mtg_engine::*;

/// Exiles the permanent (making it an illegal target) and settles.
fn exile_now(t: &mut TestGame, id: ObjectId) {
    let id = t.g.current(id);
    t.g.exile_object(id, None);
    t.g.flush_events();
    t.settle();
}

/// Removes every +1/+1 counter from the permanent and settles.
fn remove_plus1(t: &mut TestGame, id: ObjectId) {
    let id = t.g.current(id);
    let n = plus1(t, id);
    t.g.remove_counters(Entity::Object(id), PLUS1, n);
    t.g.recompute();
    t.g.flush_events();
    t.settle();
}

// --- Skarrgan Hellkite ---------------------------------------------------------------------

/// P0's Skarrgan Hellkite with a +1/+1 counter and mana for one activation.
fn hellkite(t: &mut TestGame) -> ObjectId {
    let h = t.battlefield(P0, "Skarrgan Hellkite");
    give_plus1(t, h, 1);
    t.lands(P0, "Mountain", 4);
    h
}

#[test]
fn skarrgan_hellkite_deals_only_its_share_to_the_remaining_target() {
    cr!("608.2b", "601.2d");
    ruling!(
        "Skarrgan Hellkite",
        "If Skarrgan Hellkite's ability has two targets and one becomes illegal, the remaining target is dealt 1 damage, not 2."
    );
    supported("Skarrgan Hellkite");
    let mut t = TestGame::new(2);
    let h = hellkite(&mut t);
    let a = t.battlefield(P1, "Hill Giant");
    let b = t.battlefield(P1, "Hill Giant");
    t.answer_targets(P0, &[obj(a), obj(b)]);
    t.answer(P0, DecisionKind::Divide, Answer::Numbers(vec![1, 1]));
    t.activate(P0, h, 0, &[]).expect("activate");
    exile_now(&mut t, a);
    t.resolve_all();
    assert_eq!(damage_marked(&t, b), 1);
}

#[test]
fn skarrgan_hellkite_ability_resolves_without_its_source() {
    cr!("113.7a");
    ruling!(
        "Skarrgan Hellkite",
        "Once Skarrgan Hellkite's ability has been activated, it resolves even if all +1/+1 counters are removed from it or if Skarrgan Hellkite leaves the battlefield."
    );
    for leave in [false, true] {
        let mut t = TestGame::new(2);
        let h = hellkite(&mut t);
        t.answer_targets(P0, &[Entity::Player(P1)]);
        t.answer(P0, DecisionKind::Divide, Answer::Numbers(vec![2]));
        t.activate(P0, h, 0, &[]).expect("activate");
        remove_plus1(&mut t, h);
        if leave {
            exile_now(&mut t, h);
        }
        t.resolve_all();
        assert_eq!(t.life(P1), 18, "leave = {leave}");
    }
}

// --- The Crowd Goes Wild -------------------------------------------------------------------

/// P0 casts The Crowd Goes Wild with X = the number of targets.
fn crowd(t: &mut TestGame, targets: &[ObjectId]) -> ObjectId {
    lands_for_cost(t, P0, "The Crowd Goes Wild");
    t.lands(P0, "Forest", targets.len());
    let card = t.hand(P0, "The Crowd Goes Wild");
    let es: Vec<Entity> = targets.iter().map(|id| obj(*id)).collect();
    t.answer_targets(P0, &es);
    // Nobody assists.
    t.answer_yes(P1, false);
    t.cast(P0, card).x(targets.len() as i64).go()
}

#[test]
fn the_crowd_goes_wild_with_all_targets_illegal_doesnt_resolve() {
    cr!("608.2b");
    ruling!(
        "The Crowd Goes Wild",
        "If The Crowd Goes Wild has any targets and each of those targets is illegal as it tries to resolve, the spell doesn’t resolve. No creatures gain trample."
    );
    supported("The Crowd Goes Wild");
    let mut t = TestGame::new(2);
    let target = t.battlefield(P0, "Grizzly Bears");
    let other = t.battlefield(P0, "Hill Giant");
    give_plus1(&mut t, other, 1);
    let spell = crowd(&mut t, &[target]);
    exile_now(&mut t, target);
    t.resolve_all();
    assert!(!resolved(&t, spell));
    assert!(!t.obj_now(other).has_keyword(KeywordKind::Trample));
}

#[test]
fn the_crowd_goes_wild_with_some_targets_illegal_affects_the_rest() {
    cr!("608.2b");
    ruling!(
        "The Crowd Goes Wild",
        "If some, but not all, targets for a spell become illegal, the remaining targets are affected as appropriate."
    );
    let mut t = TestGame::new(2);
    let a = t.battlefield(P0, "Grizzly Bears");
    let b = t.battlefield(P0, "Hill Giant");
    let other = t.battlefield(P0, "Elite Vanguard");
    give_plus1(&mut t, other, 1);
    let spell = crowd(&mut t, &[a, b]);
    exile_now(&mut t, a);
    t.resolve_all();
    assert!(resolved(&t, spell));
    assert_eq!(plus1(&t, b), 1);
    assert!(t.obj_now(b).has_keyword(KeywordKind::Trample));
    assert!(t.obj_now(other).has_keyword(KeywordKind::Trample));
}

// --- fights ----------------------------------------------------------------------------------

#[test]
fn titanic_brawl_with_an_illegal_target_deals_no_damage() {
    cr!("608.2b", "701.14b");
    ruling!(
        "Titanic Brawl",
        "If either target is an illegal target as Titanic Brawl resolves, neither creature will deal or be dealt damage."
    );
    supported("Titanic Brawl");
    for exile_mine in [false, true] {
        let mut t = TestGame::new(2);
        let mine = t.battlefield(P0, "Hill Giant");
        let theirs = t.battlefield(P1, "Hill Giant");
        cast_new(&mut t, P0, "Titanic Brawl", &[obj(mine), obj(theirs)]);
        let (gone, stays) = if exile_mine { (mine, theirs) } else { (theirs, mine) };
        exile_now(&mut t, gone);
        t.resolve_all();
        assert_eq!(damage_marked(&t, stays), 0, "exile_mine = {exile_mine}");
    }
}

#[test]
fn mutants_prey_first_target_needs_a_counter() {
    cr!("115.1", "601.2c");
    ruling!(
        "Mutant's Prey",
        "Only a creature with a +1/+1 counter on it can be chosen as the first target of Mutant’s Prey."
    );
    supported("Mutant's Prey");
    let mut t = TestGame::new(2);
    let plain = t.battlefield(P0, "Grizzly Bears");
    let countered = t.battlefield(P0, "Hill Giant");
    give_plus1(&mut t, countered, 1);
    let theirs = t.battlefield(P1, "Hill Giant");
    let from = t.asked().len();
    cast_new(&mut t, P0, "Mutant's Prey", &[obj(countered), obj(theirs)]);
    let first = &crate::r_s02_common::target_candidates(&t, P0, from)[0];
    assert!(first.contains(&obj(countered)));
    assert!(!first.contains(&obj(plain)));
}

#[test]
fn mutants_prey_with_an_illegal_target_deals_no_damage() {
    cr!("608.2b", "701.14b");
    ruling!(
        "Mutant's Prey",
        "If one of the targets is illegal when Mutant’s Prey tries to resolve (for example, if the first target creature no longer has a +1/+1 counter on it), neither creature will deal or be dealt damage."
    );
    let mut t = TestGame::new(2);
    let mine = t.battlefield(P0, "Hill Giant");
    give_plus1(&mut t, mine, 1);
    let theirs = t.battlefield(P1, "Hill Giant");
    cast_new(&mut t, P0, "Mutant's Prey", &[obj(mine), obj(theirs)]);
    remove_plus1(&mut t, mine);
    t.resolve_all();
    assert_eq!(damage_marked(&t, mine), 0);
    assert_eq!(damage_marked(&t, theirs), 0);
}

#[test]
fn voracious_hydra_fight_with_an_illegal_target_or_without_the_hydra() {
    cr!("608.2b", "701.14b");
    ruling!(
        "Voracious Hydra",
        "If the target of Voracious Hydra's second mode isn't a legal target as the ability resolves, or if Voracious Hydra has left the battlefield, neither creature will deal or be dealt damage. The ability won't change to the first mode if the target is illegal."
    );
    supported("Voracious Hydra");
    for hydra_leaves in [false, true] {
        let mut t = TestGame::new(2);
        let theirs = t.battlefield(P1, "Hill Giant");
        lands_for_cost(&mut t, P0, "Voracious Hydra");
        t.lands(P0, "Forest", 2);
        let card = t.hand(P0, "Voracious Hydra");
        t.answer(P0, DecisionKind::Modes, Answer::Indices(vec![1]));
        t.answer_targets(P0, &[obj(theirs)]);
        t.cast(P0, card).x(2).go();
        t.resolve();
        let hydra = t.named_on_battlefield("Voracious Hydra")[0];
        assert_eq!(plus1(&t, hydra), 2);
        assert_eq!(t.stack_len(), 1, "the enters ability");
        if hydra_leaves {
            exile_now(&mut t, hydra);
        } else {
            exile_now(&mut t, theirs);
        }
        t.resolve_all();
        if hydra_leaves {
            assert_eq!(damage_marked(&t, theirs), 0);
        } else {
            assert_eq!(plus1(&t, hydra), 2, "not doubled");
            assert_eq!(damage_marked(&t, hydra), 0);
        }
    }
}

// --- distributions -------------------------------------------------------------------------

#[test]
fn the_earth_crystal_distribution_stays_when_a_target_is_illegal() {
    cr!("608.2b", "601.2d");
    ruling!(
        "The Earth Crystal",
        "If one of the creatures is an illegal target and the other one isn't as The Earth Crystal's last ability tries to resolve, the original distribution of counters applies and the counter that would have been put on the illegal target is lost."
    );
    supported("The Earth Crystal");
    let mut t = TestGame::new(2);
    let crystal = t.battlefield(P0, "The Earth Crystal");
    t.lands(P0, "Forest", 6);
    let a = t.battlefield(P0, "Grizzly Bears");
    let b = t.battlefield(P0, "Hill Giant");
    t.answer_targets(P0, &[obj(a), obj(b)]);
    t.answer(P0, DecisionKind::Divide, Answer::Numbers(vec![1, 1]));
    t.activate(P0, crystal, 0, &[]).expect("activate");
    exile_now(&mut t, a);
    t.resolve_all();
    // One counter, doubled by The Earth Crystal's replacement effect: two, not four.
    assert_eq!(plus1(&t, b), 2);
}

#[test]
fn biogenic_upgrade_distribution_stays_when_a_target_is_illegal() {
    cr!("608.2b", "601.2d");
    ruling!(
        "Biogenic Upgrade",
        "If some of the creatures are illegal targets as Biogenic Upgrade tries to resolve, the original distribution of counters still applies and the counters that would have been put on the illegal targets are lost."
    );
    supported("Biogenic Upgrade");
    let mut t = TestGame::new(2);
    let a = t.battlefield(P0, "Grizzly Bears");
    let b = t.battlefield(P0, "Hill Giant");
    let c = t.battlefield(P0, "Elite Vanguard");
    lands_for_cost(&mut t, P0, "Biogenic Upgrade");
    let card = t.hand(P0, "Biogenic Upgrade");
    t.answer_targets(P0, &[obj(a), obj(b), obj(c)]);
    t.answer(P0, DecisionKind::Divide, Answer::Numbers(vec![1, 1, 1]));
    t.cast(P0, card).go();
    exile_now(&mut t, a);
    t.resolve_all();
    // Each remaining target: its one counter, doubled.
    assert_eq!(plus1(&t, b), 2);
    assert_eq!(plus1(&t, c), 2);
}

#[test]
fn court_of_garenbrig_distribution_stays_when_a_target_is_illegal() {
    cr!("608.2b", "601.2d", "603.3d");
    ruling!(
        "Court of Garenbrig",
        "If some of the creatures are illegal targets as Court of Garenbrig's last ability tries to resolve, the original distribution of counters still applies and the counters that would have been put on illegal targets are lost."
    );
    supported("Court of Garenbrig");
    let mut t = TestGame::new(2);
    // On the battlefield without entering: nobody is the monarch, so nothing is doubled.
    t.battlefield(P0, "Court of Garenbrig");
    let a = t.battlefield(P0, "Grizzly Bears");
    let b = t.battlefield(P0, "Hill Giant");
    t.answer_targets(P0, &[obj(a), obj(b)]);
    t.answer(P0, DecisionKind::Divide, Answer::Numbers(vec![1, 1]));
    t.advance_to(P0, mtg_engine::turn::Step::Upkeep);
    t.settle();
    assert_eq!(t.stack_len(), 1);
    exile_now(&mut t, a);
    t.resolve_all();
    assert_eq!(plus1(&t, b), 1);
}

#[test]
fn quirion_beastcaller_distribution_stays_when_a_target_is_illegal() {
    cr!("608.2b", "601.2d", "603.3d");
    ruling!(
        "Quirion Beastcaller",
        "If some of the creatures are illegal targets as the second triggered ability tries to resolve, the original distribution of counters still applies and the counters that would have been put on illegal targets are lost."
    );
    supported("Quirion Beastcaller");
    let mut t = TestGame::new(2);
    let q = t.battlefield(P0, "Quirion Beastcaller");
    give_plus1(&mut t, q, 3);
    let a = t.battlefield(P0, "Grizzly Bears");
    let b = t.battlefield(P0, "Hill Giant");
    t.answer_targets(P0, &[obj(a), obj(b)]);
    t.answer(P0, DecisionKind::Divide, Answer::Numbers(vec![2, 1]));
    destroy(&mut t, q);
    assert_eq!(t.stack_len(), 1);
    exile_now(&mut t, a);
    t.resolve_all();
    assert_eq!(plus1(&t, b), 1);
}

// --- single targets ------------------------------------------------------------------------

#[test]
fn clockwork_hydra_keeps_its_counter_if_the_target_is_illegal() {
    cr!("608.2b");
    ruling!(
        "Clockwork Hydra",
        "If the chosen target is an illegal target by the time Clockwork Hydra's triggered ability tries to resolve, the ability doesn't resolve. You don't remove a counter from Clockwork Hydra."
    );
    supported("Clockwork Hydra");
    let mut t = TestGame::new(2);
    let hydra = t.battlefield(P0, "Clockwork Hydra");
    give_plus1(&mut t, hydra, 4);
    let target = t.battlefield(P1, "Hill Giant");
    t.answer_targets(P0, &[obj(target)]);
    attack_with(&mut t, &[(hydra, Entity::Player(P1))]);
    t.settle();
    assert_eq!(t.stack_len(), 1);
    exile_now(&mut t, target);
    t.resolve_all();
    assert_eq!(plus1(&t, hydra), 4);
}

#[test]
fn rite_of_the_serpent_with_an_illegal_target_makes_no_snake() {
    cr!("608.2b");
    ruling!(
        "Rite of the Serpent",
        "If the creature is an illegal target as Rite of the Serpent tries to resolve, Rite of the Serpent won’t resolve and none of its effects will occur. You won’t get a Snake token."
    );
    supported("Rite of the Serpent");
    let mut t = TestGame::new(2);
    let target = t.battlefield(P1, "Hill Giant");
    give_plus1(&mut t, target, 1);
    cast_new(&mut t, P0, "Rite of the Serpent", &[obj(target)]);
    exile_now(&mut t, target);
    t.resolve_all();
    assert_eq!(tokens_named(&t, P0, "Snake"), 0);
}

#[test]
fn lifecrafters_gift_with_an_illegal_target_puts_no_counters() {
    cr!("608.2b");
    ruling!(
        "Lifecrafter's Gift",
        "If the target creature is an illegal target by the time Lifecrafter's Gift tries to resolve, the spell doesn't resolve. No creatures receive +1/+1 counters."
    );
    supported("Lifecrafter's Gift");
    let mut t = TestGame::new(2);
    let target = t.battlefield(P0, "Grizzly Bears");
    let other = t.battlefield(P0, "Hill Giant");
    give_plus1(&mut t, other, 1);
    cast_new(&mut t, P0, "Lifecrafter's Gift", &[obj(target)]);
    exile_now(&mut t, target);
    t.resolve_all();
    assert_eq!(plus1(&t, other), 1);
}

#[test]
fn lifecrafters_gift_skips_noncreature_permanents_with_counters() {
    cr!("608.2c");
    ruling!(
        "Lifecrafter's Gift",
        "If there is a +1/+1 counter on a noncreature permanent you control, such as a Vehicle that isn't crewed, it won't get another one from Lifecrafter's Gift."
    );
    let mut t = TestGame::new(2);
    let vehicle = t.battlefield(P0, "Smuggler's Copter");
    give_plus1(&mut t, vehicle, 1);
    let target = t.battlefield(P0, "Grizzly Bears");
    cast_new(&mut t, P0, "Lifecrafter's Gift", &[obj(target)]);
    t.resolve_all();
    assert_eq!(plus1(&t, vehicle), 1);
    // The target got its counter, then another for having one.
    assert_eq!(plus1(&t, target), 2);
}

#[test]
fn oblivions_hunger_with_an_illegal_target_draws_nothing() {
    cr!("608.2b");
    ruling!(
        "Oblivion's Hunger",
        "If the target creature is an illegal target by the time Oblivion's Hunger tries to resolve, the spell doesn't resolve. You don't draw a card if the target has or had a +1/+1 counter on it."
    );
    supported("Oblivion's Hunger");
    let mut t = TestGame::new(2);
    let target = t.battlefield(P0, "Hill Giant");
    give_plus1(&mut t, target, 1);
    cast_new(&mut t, P0, "Oblivion's Hunger", &[obj(target)]);
    let hand = t.hand_size(P0);
    exile_now(&mut t, target);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand);
}

#[test]
fn cytoplast_manipulator_leaving_before_its_ability_resolves_does_nothing() {
    cr!("611.2b");
    ruling!(
        "Cytoplast Manipulator",
        "If the ability is activated and Cytoplast Manipulator leaves the battlefield before it resolves, the ability does nothing. The target creature will remain under its current controller's control."
    );
    supported("Cytoplast Manipulator");
    let mut t = TestGame::new(2);
    let m = t.battlefield(P0, "Cytoplast Manipulator");
    give_plus1(&mut t, m, 2);
    t.lands(P0, "Island", 1);
    let target = t.battlefield(P1, "Hill Giant");
    give_plus1(&mut t, target, 1);
    t.activate(P0, m, 0, &[obj(target)]).expect("activate");
    exile_now(&mut t, m);
    t.resolve_all();
    assert_eq!(t.obj_now(target).controller, P1);
}
