//! Rulings batch P194 — cost reductions that count what happened this turn (cards
//! discarded, permanents sacrificed, creatures that died): only generic mana is reduced
//! (CR 118.7a), cost increases apply first (CR 601.2f), the mana value doesn't change
//! (CR 202.3); and effects that count creatures that died this turn.

use crate::r_p189_common::triggers_of;
use crate::r_p194_common::*;
use crate::r_p216_common::sacrifice;
use crate::r_s01_common::{attack_with, supported, with_subtype};
use crate::r_s21_common::legal_blocks;
use mtg_engine::decision::{Answer, Decision};
use mtg_engine::events::MoveCause;
use mtg_engine::mana::ManaType;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

/// `p` discards `n` filler cards put into their hand.
fn discard_n(t: &mut TestGame, p: PlayerId, n: usize) {
    for _ in 0..n {
        let c = t.hand(p, "Grizzly Bears");
        t.g.discard(p, c, None);
    }
    t.g.flush_events();
    t.settle();
}

#[test]
fn hollow_one_costs_zero_after_three_discards_never_less() {
    cr!("118.7", "601.2f", "202.3");
    ruling!(
        "Hollow One",
        "Once you've discarded three cards during one turn, Hollow One costs {0} to cast. It won't stop at {1} or cost negative amounts of mana."
    );
    ruling!(
        "Hollow One",
        "To determine the total cost of a spell, start with the mana cost or alternative cost you're paying, add any cost increases, then apply any cost reductions (such as that of Hollow One). The mana value of the spell is determined by only its mana cost, no matter what the total cost to cast that spell was."
    );
    supported("Hollow One");
    supported("Sphere of Resistance");
    assert_total_cost(&|t| discard_n(t, P0, 1), P0, "Hollow One", &[], 3);
    for n in [3, 4] {
        let mut t = TestGame::new(2);
        discard_n(&mut t, P0, n);
        let h = t.hand(P0, "Hollow One");
        // Some mana in the pool that isn't spent.
        mana(&mut t, P0, ManaType::C, 1);
        let s = t.cast(P0, h).go();
        assert_eq!(pool(&t, P0), 1, "{n} discards");
        assert_eq!(t.g.mana_value_of(s), 5);
    }
    // Sphere of Resistance's {1} more is added before the reduction: with two discards
    // 5 + 1 - 4 = {2}; with three, {0}.
    assert_total_cost(
        &|t| {
            t.battlefield(P1, "Sphere of Resistance");
            discard_n(t, P0, 2);
        },
        P0,
        "Hollow One",
        &[],
        2,
    );
    assert_total_cost(
        &|t| {
            t.battlefield(P1, "Sphere of Resistance");
            discard_n(t, P0, 3);
        },
        P0,
        "Hollow One",
        &[],
        0,
    );
}

#[test]
fn dargo_counts_each_permanent_sacrificed_earlier_once() {
    cr!("118.7a", "601.2f", "202.3");
    ruling!(
        "Dargo, the Shipwrecker",
        "The cost reduction for artifacts and creatures sacrificed earlier in the turn will apply even if you don't sacrifice any as an additional cost."
    );
    ruling!(
        "Dargo, the Shipwrecker",
        "The cost reduction is {2} per sacrificed permanent, whether that permanent was an artifact, a creature, or both."
    );
    ruling!(
        "Dargo, the Shipwrecker",
        "To determine the total cost of a spell, start with the mana cost or alternative cost you're paying, add any cost increases (such as the commander tax), then apply any cost reductions (such as that of Dargo's first ability). Dargo's mana value is 7, no matter what the total cost to cast it was."
    );
    supported("Dargo, the Shipwrecker");
    // An artifact creature, an artifact and a land sacrificed earlier: {4} less; the
    // land doesn't count. An opponent's sacrifice doesn't count either.
    let setup = |t: &mut TestGame| {
        let o = t.battlefield(P0, "Ornithopter");
        let c = t.battlefield(P0, "Darksteel Citadel");
        let f = t.battlefield(P0, "Forest");
        let theirs = t.battlefield(P1, "Grizzly Bears");
        sacrifice(t, P0, o);
        sacrifice(t, P0, c);
        sacrifice(t, P0, f);
        sacrifice(t, P1, theirs);
    };
    assert_total_cost(&setup, P0, "Dargo, the Shipwrecker", &[(ManaType::R, 1)], 2);
    // A cost increase is added first: 6 + 1 - 4.
    let taxed = |t: &mut TestGame| {
        setup(t);
        t.battlefield(P1, "Sphere of Resistance");
    };
    assert_total_cost(&taxed, P0, "Dargo, the Shipwrecker", &[(ManaType::R, 1)], 3);
    let mut t = TestGame::new(2);
    setup(&mut t);
    let d = t.hand(P0, "Dargo, the Shipwrecker");
    mana(&mut t, P0, ManaType::R, 1);
    mana(&mut t, P0, ManaType::C, 2);
    t.answer(P0, DecisionKind::Entities, Answer::Entities(vec![]));
    let s = t.cast(P0, d).go();
    assert_eq!(t.g.mana_value_of(s), 7);
}

#[test]
fn death_rattle_oni_and_balrog_keep_their_mana_value_and_colored_cost() {
    cr!("118.7a", "601.2f", "202.3");
    ruling!(
        "Death-Rattle Oni",
        "The cost-reduction ability of Death-Rattle Oni doesn't change its mana cost or mana value, only the total cost you pay. Specifically, Death-Rattle Oni's mana value is always 7."
    );
    ruling!(
        "The Balrog, Durin's Bane",
        "The Balrog, Durin's Bane's first ability can't reduce its cost to less than {B}{R}."
    );
    ruling!(
        "The Balrog, Durin's Bane",
        "The Balrog, Durin's Bane's first ability counts every permanent that was sacrificed this turn, by any player."
    );
    supported("Death-Rattle Oni");
    supported("The Balrog, Durin's Bane");
    // Two creatures died: the Oni costs {2}{B} and its mana value is 7.
    let deaths = |t: &mut TestGame| {
        let a = t.battlefield(P0, "Grizzly Bears");
        let b = t.battlefield(P1, "Grizzly Bears");
        kill(t, a);
        kill(t, b);
    };
    assert_total_cost(&deaths, P0, "Death-Rattle Oni", &[(ManaType::B, 1)], 2);
    let mut t = TestGame::new(2);
    deaths(&mut t);
    let oni = t.hand(P0, "Death-Rattle Oni");
    mana(&mut t, P0, ManaType::B, 1);
    mana(&mut t, P0, ManaType::C, 2);
    let s = t.cast(P0, oni).go();
    assert_eq!(t.g.mana_value_of(s), 7);
    t.resolve_all();
    assert_eq!(t.g.mana_value_of(t.g.current(oni)), 7);
    // The Balrog: permanents sacrificed by both players (lands, a creature) all count.
    let sacrificed = |t: &mut TestGame| {
        for p in [P0, P1] {
            let land = t.battlefield(p, "Forest");
            sacrifice(t, p, land);
        }
        let b = t.battlefield(P1, "Grizzly Bears");
        sacrifice(t, P1, b);
    };
    assert_total_cost(
        &sacrificed,
        P0,
        "The Balrog, Durin's Bane",
        &[(ManaType::B, 1), (ManaType::R, 1)],
        2,
    );
    // Ten sacrificed: still {B}{R}.
    let ten = |t: &mut TestGame| {
        for _ in 0..10 {
            let land = t.battlefield(P1, "Forest");
            sacrifice(t, P1, land);
        }
    };
    assert_total_cost(
        &ten,
        P0,
        "The Balrog, Durin's Bane",
        &[(ManaType::B, 1), (ManaType::R, 1)],
        0,
    );
    assert!(!casts_for(
        &ten,
        P0,
        "The Balrog, Durin's Bane",
        &[(ManaType::B, 1)],
        1
    ));
}

#[test]
fn gadrak_stays_attacking_after_losing_artifacts() {
    cr!("506.4a", "508.1c");
    ruling!(
        "Gadrak, the Crown-Scourge",
        "Once Gadrak has attacked, it will remain an attacking creature even if you no longer control four or more artifacts."
    );
    supported("Gadrak, the Crown-Scourge");
    let mut t = TestGame::new(2);
    let gadrak = t.battlefield(P0, "Gadrak, the Crown-Scourge");
    let arts: Vec<ObjectId> = (0..4).map(|_| t.battlefield(P0, "Memnite")).collect();
    attack_with(&mut t, &[(gadrak, pl(P1))]);
    assert!(t.g.is_attacking(gadrak));
    kill(&mut t, arts[0]);
    assert!(t.g.is_attacking(gadrak));
    t.advance_to(P0, Step::EndOfCombat);
    assert_eq!(t.life(P1), 15);
    // With three artifacts, it can't be declared as an attacker.
    let mut t = TestGame::new(2);
    let gadrak = t.battlefield(P0, "Gadrak, the Crown-Scourge");
    for _ in 0..3 {
        t.battlefield(P0, "Memnite");
    }
    t.set_step(P0, Step::BeginningOfCombat);
    assert!(!crate::r_s09_common::legal_attack(
        &mut t,
        &[(gadrak, pl(P1))]
    ));
}

#[test]
fn end_step_counters_count_deaths_before_the_permanent_entered() {
    cr!("608.2h", "700.4");
    ruling!(
        "Asmira, Holy Avenger",
        "The ability counts creatures that went to the graveyard before this card entered the battlefield."
    );
    supported("Asmira, Holy Avenger");
    let mut t = TestGame::new(2);
    let a = t.battlefield(P0, "Grizzly Bears");
    let b = t.battlefield(P0, "Grizzly Bears");
    // An opponent's creature goes to their graveyard: not counted.
    let c = t.battlefield(P1, "Grizzly Bears");
    kill(&mut t, a);
    kill(&mut t, b);
    kill(&mut t, c);
    let asmira = t.battlefield(P0, "Asmira, Holy Avenger");
    end_step(&mut t);
    assert_eq!(t.counters(asmira, "+1/+1"), 2);
}

#[test]
fn feast_of_the_victorious_dead_counts_tokens_and_doesnt_target() {
    cr!("700.4", "115.1", "608.2d");
    ruling!(
        "Feast of the Victorious Dead",
        "The ability doesn't target any creatures. You choose how the counters are distributed as the ability resolves."
    );
    ruling!(
        "Feast of the Victorious Dead",
        "The ability will count any creature that died, including token creatures. For nontoken creatures that died, it doesn't matter if the cards are still in the graveyard."
    );
    supported("Feast of the Victorious Dead");
    supported("Lightning Greaves");
    supported("Raise the Alarm");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Feast of the Victorious Dead");
    // A Bears that died and was then exiled from the graveyard, and a token.
    let bears = t.battlefield(P1, "Grizzly Bears");
    kill(&mut t, bears);
    let card = t.g.current(bears);
    t.g.move_object(card, Zone::Exile, MoveCause::Effect, None);
    let rta = t.hand(P0, "Raise the Alarm");
    mana(&mut t, P0, ManaType::W, 2);
    t.cast(P0, rta).go();
    t.resolve_all();
    let soldiers = with_subtype(&t, P0, "Soldier");
    kill(&mut t, soldiers[0]);
    // The only creature P0 controls has shroud.
    let shrouded = soldiers[1];
    let greaves = t.battlefield(P0, "Lightning Greaves");
    assert!(t.g.attach(greaves, obj(shrouded)));
    t.g.recompute();
    assert!(t.obj_now(shrouded).chars.has_keyword(mtg_engine::keywords::KeywordKind::Shroud));
    let from = t.asked().len();
    end_step(&mut t);
    assert_eq!(t.life(P0), 22);
    assert_eq!(t.counters(shrouded, "+1/+1"), 2);
    assert!(target_choices(&t, P0, from, "").is_empty());
}

#[test]
fn death_counts_are_read_as_the_spell_resolves() {
    cr!("608.2h", "700.4");
    ruling!(
        "Rise of the Dread Marn",
        "The number of nontoken creatures that died that turn is counted as Rise of the Dread Marn is resolving. In other words, nontoken creatures that die in response to Rise of the Dread Marn will count."
    );
    ruling!(
        "Spoils of Blood",
        "Use the number of creatures that died that turn as Spoils of Blood resolves to determine the value of X. In other words, creatures that die in response to Spoils of Blood will count."
    );
    supported("Rise of the Dread Marn");
    supported("Spoils of Blood");
    // Rise: one Bears died before, one in response; the dying token doesn't count.
    let mut t = TestGame::new(2);
    let a = t.battlefield(P0, "Grizzly Bears");
    let b = t.battlefield(P1, "Grizzly Bears");
    kill(&mut t, a);
    let r = t.hand(P0, "Rise of the Dread Marn");
    mana(&mut t, P0, ManaType::B, 3);
    t.cast(P0, r).go();
    kill(&mut t, b);
    t.resolve_all();
    assert_eq!(with_subtype(&t, P0, "Zombie").len(), 2);
    // A dying token doesn't count for Rise, but does for Spoils.
    let mut t = TestGame::new(2);
    let rta = t.hand(P0, "Raise the Alarm");
    mana(&mut t, P0, ManaType::W, 2);
    t.cast(P0, rta).go();
    t.resolve_all();
    let soldiers = with_subtype(&t, P0, "Soldier");
    let bears = t.battlefield(P1, "Grizzly Bears");
    kill(&mut t, soldiers[0]);
    let s = t.hand(P0, "Spoils of Blood");
    mana(&mut t, P0, ManaType::B, 1);
    t.cast(P0, s).go();
    let r = t.hand(P0, "Rise of the Dread Marn");
    mana(&mut t, P0, ManaType::B, 3);
    t.cast(P0, r).go();
    kill(&mut t, soldiers[1]);
    kill(&mut t, bears);
    t.resolve_all();
    assert_eq!(with_subtype(&t, P0, "Zombie").len(), 1);
    let horror = with_subtype(&t, P0, "Horror");
    assert_eq!(horror.len(), 1);
    assert_eq!(t.pt(horror[0]), (3, 3));
}

#[test]
fn canopy_stalker_needs_one_blocker_chosen_by_the_defending_player() {
    cr!("509.1c");
    ruling!(
        "Canopy Stalker",
        "Only one creature is required to block Canopy Stalker. Other creatures may also block it, and are free to block other creatures or not block at all."
    );
    ruling!(
        "Canopy Stalker",
        "The defending player, not you, chooses which creature blocks Canopy Stalker."
    );
    supported("Canopy Stalker");
    let mut t = TestGame::new(2);
    let stalker = t.battlefield(P0, "Canopy Stalker");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let a = t.battlefield(P1, "Grizzly Bears");
    let b = t.battlefield(P1, "Runeclaw Bear");
    attack_with(&mut t, &[(stalker, pl(P1)), (bears, pl(P1))]);
    assert!(!legal_blocks(&mut t, P1, &[]));
    assert!(!legal_blocks(&mut t, P1, &[(a, bears)]));
    assert!(legal_blocks(&mut t, P1, &[(a, stalker)]));
    assert!(legal_blocks(&mut t, P1, &[(b, stalker)]));
    assert!(legal_blocks(&mut t, P1, &[(a, stalker), (b, stalker)]));
    assert!(legal_blocks(&mut t, P1, &[(b, stalker), (a, bears)]));
    // P1 declares the blocks.
    let from = t.asked().len();
    crate::r_s01_common::block_and_finish(&mut t, P1, &[(b, stalker)]);
    assert!(t.asked()[from..]
        .iter()
        .any(|(p, d)| *p == P1 && matches!(d, Decision::DeclareBlockers { .. })));
    assert!(!t.asked()[from..]
        .iter()
        .any(|(p, d)| *p == P0 && matches!(d, Decision::DeclareBlockers { .. })));
    assert!(t.on_battlefield(a));
    assert_eq!(t.life(P1), 18);
}

#[test]
fn flesh_allergy_sacrificing_its_own_target_makes_it_fizzle() {
    cr!("601.2c", "601.2h", "608.2b");
    ruling!(
        "Flesh Allergy",
        "You can target a creature you control, then sacrifice that creature to pay the additional cost. However, if you do, Flesh Allergy won't resolve for having an illegal target. You won't lose any life."
    );
    supported("Flesh Allergy");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.battlefield(P0, "Runeclaw Bear");
    let fa = t.hand(P0, "Flesh Allergy");
    mana(&mut t, P0, ManaType::B, 4);
    t.answer_choose(P0, &[obj(bears)]);
    let s = t.cast(P0, fa).target(bears).go();
    assert!(!t.on_battlefield(bears));
    assert_eq!(t.g.obj(s).zone, Zone::Stack);
    t.resolve_all();
    assert_eq!(t.life(P0), 20);
    assert_eq!(t.life(P1), 20);
    assert!(t.in_graveyard(P0, "Flesh Allergy"));
}

#[test]
fn season_of_loss_sacrifices_are_chosen_in_turn_order_then_simultaneous() {
    cr!("101.4", "608.2e", "603.10a");
    ruling!(
        "Season of Loss",
        "When you're following the instructions from the first mode, first the player whose turn it is chooses which creature they're going to sacrifice, then each other player in turn order does the same. Players will know the choices of previous players when making their choices. Then all chosen creatures are sacrificed by their controllers simultaneously. Repeat this process for each additional time the first mode was chosen."
    );
    supported("Season of Loss");
    fn look(g: &mtg_engine::game::Game) -> (usize, Vec<PlayerId>) {
        (
            g.permanents().count(),
            g.known_apnap_choices(PlayerId(0))
                .iter()
                .filter(|(_, c)| c.as_ref().is_some_and(|c| !c.is_empty()))
                .map(|(p, _)| *p)
                .collect(),
        )
    }
    let is_choose = |d: &Decision| matches!(d, Decision::ChooseEntities { .. });
    let mut t = TestGame::new(3);
    let mut seen = vec![];
    let mut first = vec![];
    for p in [P0, P1, P2] {
        let bears = t.battlefield(p, "Grizzly Bears");
        let giant = t.battlefield(p, "Hill Giant");
        // P1 sacrifices its Hill Giant first, the others their Bears.
        let pick = if p == P1 { giant } else { bears };
        first.push(pick);
        t.answer_choose(p, &[obj(pick)]);
        seen.push(crate::r_s01_common::watch(&mut t, p, is_choose, look));
    }
    let s = t.hand(P0, "Season of Loss");
    mana(&mut t, P0, ManaType::B, 5);
    // The first mode twice.
    t.cast(P0, s).modes(&[0, 0]).go();
    let from = t.asked().len();
    t.resolve_all();
    let order: Vec<PlayerId> = t.asked()[from..]
        .iter()
        .filter(|(_, d)| is_choose(d))
        .map(|(p, _)| *p)
        .collect();
    assert_eq!(order[..3], [P0, P1, P2]);
    // Each chooser saw all six creatures still there, and the earlier choices.
    assert_eq!(seen[0].lock().unwrap()[0], (6, vec![]));
    assert_eq!(seen[1].lock().unwrap()[0], (6, vec![P0]));
    assert_eq!(seen[2].lock().unwrap()[0], (6, vec![P0, P1]));
    // The first round's choices went first; then the second round took the rest.
    for (i, p) in [P0, P1, P2].into_iter().enumerate() {
        assert_eq!(crate::r_s01_common::creatures(&t, p).len(), 0, "{p:?}");
        let gy = &t.g.player(p).graveyard;
        let names: Vec<&str> = gy.iter().map(|c| t.g.obj(*c).chars.name.as_str()).collect();
        let want = t.g.obj(first[i]).chars.name.as_str();
        assert_eq!(names[0], want, "{p:?}: {names:?}");
    }
    // The chosen creatures are sacrificed at once: the active player's Blood Artist,
    // chosen first in turn order, still sees the other two die with it (CR 603.10a).
    let mut t = TestGame::new(3);
    let artist = t.battlefield(P0, "Blood Artist");
    t.battlefield(P1, "Grizzly Bears");
    t.battlefield(P2, "Grizzly Bears");
    let s = t.hand(P0, "Season of Loss");
    mana(&mut t, P0, ManaType::B, 5);
    t.cast(P0, s).modes(&[0]).go();
    t.resolve();
    t.settle();
    assert!(!t.on_battlefield(artist));
    assert_eq!(triggers_of(&t, artist), 3);
}

#[test]
fn structural_assault_counts_artifacts_put_into_graveyards_any_way() {
    cr!("608.2h", "702.12b");
    ruling!(
        "Structural Assault",
        "Structural Assault counts the number of artifacts that were put into graveyards from the battlefield this turn for any reason, not just because they were destroyed by Structural Assault."
    );
    supported("Structural Assault");
    let mut t = TestGame::new(2);
    let o = t.battlefield(P1, "Ornithopter");
    sacrifice(&mut t, P1, o);
    // Destroyed by Structural Assault: one Memnite; the Citadel is indestructible.
    t.battlefield(P1, "Memnite");
    t.battlefield(P1, "Darksteel Citadel");
    let big = t.battlefield(P0, "Ancient Brontodon");
    let sa = t.hand(P0, "Structural Assault");
    mana(&mut t, P0, ManaType::R, 5);
    t.cast(P0, sa).go();
    t.resolve_all();
    assert_eq!(t.obj_now(big).damage, 2);
}

#[test]
fn mongrel_pack_triggers_when_it_dies_in_any_combat_phase() {
    cr!("506.1", "603.6c");
    ruling!(
        "Mongrel Pack",
        "The Hound tokens are put onto the battlefield if it dies during the combat phase, even if it was not an attacker or blocker."
    );
    supported("Mongrel Pack");
    // On the opponent's turn, in the beginning of combat step, not in combat.
    let mut t = TestGame::new(2);
    let pack = t.battlefield(P0, "Mongrel Pack");
    t.set_step(P1, Step::BeginningOfCombat);
    kill(&mut t, pack);
    t.resolve_all();
    assert_eq!(with_subtype(&t, P0, "Dog").len(), 4);
    // In a main phase: nothing.
    let mut t = TestGame::new(2);
    let pack = t.battlefield(P0, "Mongrel Pack");
    t.set_step(P1, Step::PostcombatMain);
    kill(&mut t, pack);
    t.resolve_all();
    assert!(with_subtype(&t, P0, "Dog").is_empty());
}
