//! Rulings residue P129 and P130 (item rulings-residue-reflexive-refund): reflexive
//! triggered abilities ("When you do, ...", CR 603.12). The first ability goes on the stack
//! without targets; the payment, sacrifice, discard or exile happens as it resolves; the
//! reflexive ability then triggers, its targets (and modes) are chosen as it is put on the
//! stack, and players may respond before its effect happens.

use crate::r_p129_common::*;
use mtg_engine::decision::Answer;
use mtg_engine::mana::ManaType;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

pub use crate::r_p015_common::move_to;
pub use crate::r_p127_common::is_in;
pub use crate::r_s01_common::triggers_on_stack;
pub use crate::r_s04_common::can_cycle;
pub use crate::r_s04_common::untapped_lands;
pub use crate::r_s06_common::{activate_containing, attach_new};

/// Advances to the beginning of combat on `p`'s turn and puts its triggers on the stack.
pub fn to_beginning_of_combat(t: &mut TestGame, p: PlayerId) {
    t.set_step(p, Step::PrecombatMain);
    t.advance_to(p, Step::BeginningOfCombat);
    t.settle();
}

/// Advances to `p`'s end step and puts its triggers on the stack.
pub fn to_end_step(t: &mut TestGame, p: PlayerId) {
    t.advance_to(p, Step::End);
    t.settle();
}

#[test]
fn slumbering_walker_targets_after_the_counter_is_removed() {
    cr!("603.12", "603.3d");
    ruling!(
        "Slumbering Walker",
        "You don't choose a target for Slumbering Walker's ability at the time it triggers. Rather, a second \"reflexive\" ability triggers when you remove a counter from Slumbering Walker this way."
    );
    supported("Slumbering Walker");
    let mut t = TestGame::new(2);
    let walker = t.battlefield(P0, "Slumbering Walker");
    if t.counters(walker, counters::MINUS1) == 0 {
        put_counters(&mut t, walker, counters::MINUS1, 2);
    }
    let bears = t.graveyard(P0, "Grizzly Bears");
    let from = t.asked().len();
    t.answer_yes(P0, true);
    t.answer_targets(P0, &[obj(bears)]);
    to_end_step(&mut t, P0);
    untargeted_then_reflexive(&mut t, P0, from, &[obj(bears)]);
    assert_eq!(t.counters(walker, counters::MINUS1), 1);
    assert!(is_in(&t, bears, Zone::Graveyard(P0)));
    // P1 responds: the card leaves the graveyard, so the reflexive ability does nothing.
    move_to(&mut t, bears, Zone::Exile);
    t.resolve_all();
    assert!(is_in(&t, bears, Zone::Exile));
    // Declining: no counter is removed and nothing triggers.
    let mut t = TestGame::new(2);
    let walker = t.battlefield(P0, "Slumbering Walker");
    put_counters(&mut t, walker, counters::MINUS1, 2);
    let have = t.counters(walker, counters::MINUS1);
    t.graveyard(P0, "Grizzly Bears");
    t.answer_yes(P0, false);
    to_end_step(&mut t, P0);
    t.resolve();
    assert_eq!(t.stack_len(), 0);
    assert_eq!(t.counters(walker, counters::MINUS1), have);
}

#[test]
fn taeko_targets_an_attacking_creature_after_paying() {
    cr!("603.12", "509.1b", "107.4e");
    ruling!(
        "Taeko, the Patient Avalanche",
        "You don't choose a target for Taeko's last ability at the time it triggers."
    );
    ruling!(
        "Taeko, the Patient Avalanche",
        "You choose a target for that ability as it goes on the stack. Each player may respond to this triggered ability as normal."
    );
    // The ruling says the reflexive ability triggers "when you pay {U}{B} this way"; the
    // card's cost is the hybrid {U/B} ("you may pay {U/B}. When you do, ..."), paid with
    // one blue or one black mana (docs/RULING_DISCREPANCIES.md).
    interp!("ruling-contradicting-oracle-gives-way");
    supported("Taeko, the Patient Avalanche");
    for land in ["Island", "Swamp"] {
        let mut t = TestGame::new(2);
        let taeko = t.battlefield(P0, "Taeko, the Patient Avalanche");
        let bears = t.battlefield(P0, "Grizzly Bears");
        let wall = t.battlefield(P1, "Wall of Stone");
        t.lands(P0, land, 1);
        let from = t.asked().len();
        t.answer_yes(P0, true);
        t.answer_targets(P0, &[obj(bears)]);
        attack_with(
            &mut t,
            &[(taeko, Entity::Player(P1)), (bears, Entity::Player(P1))],
        );
        untargeted_then_reflexive(&mut t, P0, from, &[obj(bears)]);
        assert_eq!(untapped_lands(&t, P0), 0, "{land}: one mana paid");
        t.resolve_all();
        // The Wall can't block the Bears (an illegal block: no block at all).
        t.answer(
            P1,
            DecisionKind::Blockers,
            Answer::Blockers(vec![(wall, bears)]),
        );
        t.advance_to(P0, Step::EndOfCombat);
        assert_eq!(t.life(P1), 20 - 2 - t.pt(taeko).0, "{land}");
    }
}

#[test]
fn tester_of_the_tangential_moves_counters_to_the_reflexive_target() {
    cr!("603.12", "107.3", "122.5");
    ruling!(
        "Tester of the Tangential",
        "You don't choose a target for Tester of the Tangential's last ability at the time it triggers. Instead, a second \"reflexive\" ability triggers whenever you pay {X} this way."
    );
    supported("Tester of the Tangential");
    let mut t = TestGame::new(2);
    let tester = t.battlefield(P0, "Tester of the Tangential");
    put_counters(&mut t, tester, counters::PLUS1, 3);
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.lands(P0, "Island", 2);
    let from = t.asked().len();
    t.answer_yes(P0, true);
    t.answer(P0, DecisionKind::X, Answer::Number(2));
    t.answer(P0, DecisionKind::Number, Answer::Number(2));
    t.answer_targets(P0, &[obj(bears)]);
    to_beginning_of_combat(&mut t, P0);
    untargeted_then_reflexive(&mut t, P0, from, &[obj(bears)]);
    assert_eq!(untapped_lands(&t, P0), 0);
    assert_eq!(t.counters(tester, counters::PLUS1), 3);
    t.resolve_all();
    assert_eq!(t.counters(tester, counters::PLUS1), 1);
    assert_eq!(t.counters(bears, counters::PLUS1), 2);
}

#[test]
fn dire_blunderbuss_targets_after_the_artifact_is_sacrificed() {
    cr!("603.12", "701.21a");
    ruling!(
        "Dire Flail // Dire Blunderbuss",
        "You don't choose a target for the triggered ability granted by Dire Blunderbuss at the time it triggers. Rather, a second \"reflexive\" ability triggers when you sacrifice another artifact this way."
    );
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let flail = attach_new(&mut t, P0, "Dire Flail // Dire Blunderbuss", bears);
    assert!(mtg_engine::dfc::transform(&mut t.g, flail));
    t.g.recompute();
    let thopter = t.battlefield(P0, "Ornithopter");
    let wurm = t.battlefield(P1, "Craw Wurm");
    let from = t.asked().len();
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[obj(thopter)]);
    t.answer_targets(P0, &[obj(wurm)]);
    attack_with(&mut t, &[(bears, Entity::Player(P1))]);
    untargeted_then_reflexive(&mut t, P0, from, &[obj(wurm)]);
    assert!(!t.on_battlefield(thopter));
    assert_eq!(t.obj_now(wurm).damage, 0);
    t.resolve_all();
    assert!(t.in_graveyard(P1, "Craw Wurm"));
}

#[test]
fn motherlode_targets_a_nonbasic_land_after_paying_energy() {
    cr!("603.12", "107.14");
    ruling!(
        "The Motherlode, Excavator",
        "You don't choose a target nonbasic land at the time The Motherlode's last ability triggers. Rather, a second \"reflexive\" ability triggers when you pay {E}{E}{E}{E} this way."
    );
    let mut t = TestGame::new(2);
    let ml = t.battlefield(P0, "The Motherlode, Excavator");
    give_energy(&mut t, P0, 4);
    let vault = t.battlefield(P1, "Mutavault");
    let from = t.asked().len();
    t.answer_yes(P0, true);
    t.answer_targets(P0, &[obj(vault)]);
    attack_with(&mut t, &[(ml, Entity::Player(P1))]);
    untargeted_then_reflexive(&mut t, P0, from, &[obj(vault)]);
    assert_eq!(energy(&t, P0), 0);
    assert!(t.on_battlefield(vault));
    t.resolve_all();
    assert!(!t.on_battlefield(vault));
}

#[test]
fn motherlode_reflexive_ability_with_an_illegal_target_does_nothing() {
    cr!("608.2b", "603.12");
    ruling!(
        "The Motherlode, Excavator",
        "If the target nonbasic land is an illegal target as The Motherlode's reflexive triggered ability tries to resolve, it won't resolve and none of its effects happen. Creatures defending player controls without flying will still be able to block this turn."
    );
    let mut t = TestGame::new(2);
    let ml = t.battlefield(P0, "The Motherlode, Excavator");
    give_energy(&mut t, P0, 4);
    let vault = t.battlefield(P1, "Mutavault");
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.answer_yes(P0, true);
    t.answer_targets(P0, &[obj(vault)]);
    attack_with(&mut t, &[(ml, Entity::Player(P1))]);
    t.resolve();
    // In response, the land leaves the battlefield.
    destroy(&mut t, vault);
    t.resolve_all();
    t.answer(
        P1,
        DecisionKind::Blockers,
        Answer::Blockers(vec![(bears, ml)]),
    );
    t.advance_to(P0, Step::EndOfCombat);
    assert_eq!(t.life(P1), 20, "the Bears blocked");
}

#[test]
fn motherlode_cant_block_applies_to_creatures_that_lose_flying_or_arrive_later() {
    cr!("611.2c", "509.1b");
    ruling!(
        "The Motherlode, Excavator",
        "The Motherlode's reflexive triggered ability prevents all creatures without flying that player controls from blocking that turn, including creatures that lose flying after the ability resolves as well as creatures without flying that enter the battlefield or come under that player's control later in the turn."
    );
    // 0: the Drake keeps flying and blocks; 1: it loses flying (Gravity Sphere) after the
    // ability resolved; 2: Grizzly Bears enter later; 3: P1 gains control of P0's Bears.
    for case in 0..4 {
        let mut t = TestGame::new(2);
        let ml = t.battlefield(P0, "The Motherlode, Excavator");
        give_energy(&mut t, P0, 4);
        let vault = t.battlefield(P1, "Mutavault");
        let drake = t.battlefield(P1, "Wind Drake");
        let mine = t.battlefield(P0, "Grizzly Bears");
        t.answer_yes(P0, true);
        t.answer_targets(P0, &[obj(vault)]);
        attack_with(&mut t, &[(ml, Entity::Player(P1))]);
        t.resolve_all();
        assert!(!t.on_battlefield(vault));
        let blocker = match case {
            0 => drake,
            1 => {
                t.battlefield(P0, "Gravity Sphere");
                t.g.recompute();
                drake
            }
            2 => t.battlefield(P1, "Grizzly Bears"),
            _ => {
                crate::r_p015_common::gain_control(&mut t, P1, mine);
                mine
            }
        };
        t.answer(
            P1,
            DecisionKind::Blockers,
            Answer::Blockers(vec![(blocker, ml)]),
        );
        t.advance_to(P0, Step::EndOfCombat);
        let expect = if case == 0 { 20 } else { 20 - t.pt(ml).0 };
        assert_eq!(t.life(P1), expect, "case {case}");
    }
}

/// The index (in `t.asked()`, from `from` on) of the first decision `p` was asked of the
/// given kind.
pub fn first_asked(t: &TestGame, p: PlayerId, from: usize, kind: DecisionKind) -> Option<usize> {
    use mtg_engine::decision::Decision;
    t.asked()
        .iter()
        .enumerate()
        .skip(from)
        .find(|(_, (q, d))| {
            *q == p
                && match kind {
                    DecisionKind::Modes => matches!(d, Decision::ChooseModes { .. }),
                    DecisionKind::Targets => matches!(d, Decision::ChooseTargets { .. }),
                    DecisionKind::YesNo => matches!(d, Decision::YesNo { .. }),
                    DecisionKind::Name => matches!(d, Decision::NameCard { .. }),
                    DecisionKind::Entities => matches!(d, Decision::ChooseEntities { .. }),
                    _ => false,
                }
        })
        .map(|(i, _)| i)
}

#[test]
fn inferno_of_the_star_mounts_targets_only_when_its_power_becomes_20() {
    cr!("603.12", "602.2b");
    ruling!(
        "Inferno of the Star Mounts",
        "You don't choose a target when you activate the activated ability. You choose a target for the linked triggered ability only if it triggers because the power of Inferno of the Star Mounts became 20."
    );
    let mut t = TestGame::new(2);
    let inf = t.battlefield(P0, "Inferno of the Star Mounts");
    t.lands(P0, "Mountain", 14);
    let base = t.pt(inf).0;
    let from = t.asked().len();
    for _ in base..19 {
        activate_containing(&mut t, P0, inf, "+1/+0").unwrap();
        assert!(targets_of(&t, top(&t)).is_empty());
        t.resolve_all();
    }
    assert_eq!(t.pt(inf).0, 19);
    assert!(target_candidates(&t, P0, from).is_empty(), "no target yet");
    t.answer_targets(P0, &[Entity::Player(P1)]);
    activate_containing(&mut t, P0, inf, "+1/+0").unwrap();
    assert!(targets_of(&t, top(&t)).is_empty());
    resolve_into_reflexive(&mut t, &[Entity::Player(P1)]);
    assert_eq!(t.life(P1), 20);
    t.resolve_all();
    assert_eq!(t.life(P1), 0);
}

#[test]
fn caesar_chooses_modes_and_target_as_the_reflexive_ability_goes_on_the_stack() {
    cr!("603.12", "700.2a", "700.2b");
    ruling!(
        "Caesar, Legion's Emperor",
        "You don't choose modes or a target for Caesar, Legion's Emperor's ability at the time it triggers. Rather, a second \"reflexive\" ability triggers when you sacrifice another creature this way."
    );
    let mut t = TestGame::new(2);
    let caesar = t.battlefield(P0, "Caesar, Legion's Emperor");
    let thopter = t.battlefield(P0, "Ornithopter");
    create_token(&mut t, P0, "Soldier");
    let from = t.asked().len();
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[obj(thopter)]);
    t.answer(P0, DecisionKind::Modes, Answer::Indices(vec![1, 2]));
    t.answer_targets(P0, &[Entity::Player(P1)]);
    attack_with(&mut t, &[(caesar, Entity::Player(P1))]);
    assert!(first_asked(&t, P0, from, DecisionKind::Modes).is_none());
    untargeted_then_reflexive(&mut t, P0, from, &[Entity::Player(P1)]);
    assert!(!t.on_battlefield(thopter));
    let sac = first_asked(&t, P0, from, DecisionKind::Entities).unwrap();
    assert!(first_asked(&t, P0, from, DecisionKind::Modes).unwrap() > sac);
    let hand = t.hand_size(P0);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand + 1);
    assert_eq!(t.life(P0), 19);
    assert_eq!(t.life(P1), 19, "one creature token");
}

#[test]
fn miasma_demon_targets_up_to_the_number_discarded() {
    cr!("603.12", "115.1d");
    ruling!(
        "Miasma Demon",
        "You don't choose targets for Miasma Demon's last ability at the time it triggers. Rather, a second \"reflexive\" ability triggers when you discard cards this way."
    );
    supported("Miasma Demon");
    let mut t = TestGame::new(2);
    let a = t.hand(P0, "Island");
    let b = t.hand(P0, "Forest");
    let bears = t.battlefield(P1, "Grizzly Bears");
    let lion = t.battlefield(P1, "Savannah Lions");
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[obj(a), obj(b)]);
    t.answer_targets(P0, &[obj(bears), obj(lion)]);
    let from = t.asked().len();
    t.enter(P0, "Miasma Demon");
    t.settle();
    untargeted_then_reflexive(&mut t, P0, from, &[obj(bears), obj(lion)]);
    assert_eq!(t.graveyard_size(P0), 2);
    assert!(t.on_battlefield(bears));
    t.resolve_all();
    assert!(!t.on_battlefield(bears) && !t.on_battlefield(lion));
}

#[test]
fn weapons_vendor_targets_after_paying() {
    cr!("603.12", "301.5c");
    ruling!(
        "Weapons Vendor",
        "You don't choose targets for Weapons Vendor's last ability at the time it triggers. Rather, a second \"reflexive\" ability triggers when you pay {1} this way."
    );
    supported("Weapons Vendor");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Weapons Vendor");
    let sword = t.battlefield(P0, "Short Sword");
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.lands(P0, "Plains", 1);
    let from = t.asked().len();
    t.answer_yes(P0, true);
    t.answer_targets(P0, &[obj(sword)]);
    t.answer_targets(P0, &[obj(bears)]);
    to_beginning_of_combat(&mut t, P0);
    untargeted_then_reflexive(&mut t, P0, from, &[obj(sword), obj(bears)]);
    assert_eq!(untapped_lands(&t, P0), 0);
    assert_eq!(t.obj_now(sword).attached_to, None);
    t.resolve_all();
    assert_eq!(t.obj_now(sword).attached_to, Some(obj(bears)));
}

#[test]
fn viviens_invocation_targets_after_the_creature_enters_and_uses_its_last_known_power() {
    cr!("603.12", "608.2h", "113.7a");
    ruling!(
        "Vivien's Invocation",
        "You don’t choose any targets as you cast Vivien’s Invocation. While that spell is resolving, you may put a creature card from among the cards you look at onto the battlefield. If you do, a separate ability triggers and you pick a target creature an opponent controls to be dealt damage."
    );
    ruling!(
        "Vivien's Invocation",
        "If the creature you put onto the battlefield leaves the battlefield before damage is dealt, its last known existence on the battlefield is used to determine how much damage it deals to the target creature."
    );
    supported("Vivien's Invocation");
    for leaves in [false, true] {
        let mut t = TestGame::new(2);
        let lib = stack_library(&mut t, P0, &["Craw Wurm", "Island", "Island"]);
        let wurm = lib[0];
        let bronto = t.battlefield(P1, "Ancient Brontodon");
        t.answer_choose(P0, &[obj(wurm)]);
        t.answer_targets(P0, &[obj(bronto)]);
        let from = t.asked().len();
        let spell = cast_new(&mut t, P0, "Vivien's Invocation", &[]);
        assert!(targets_of(&t, spell).is_empty());
        assert!(target_candidates(&t, P0, from).is_empty());
        resolve_into_reflexive(&mut t, &[obj(bronto)]);
        assert!(t.on_battlefield(wurm));
        // In response, the Wurm gets +3/+3 (9/7), then (`leaves`) is destroyed.
        let wurm = t.g.current(wurm);
        cast_new(&mut t, P0, "Giant Growth", &[obj(wurm)]);
        t.resolve();
        assert_eq!(t.pt(wurm), (9, 7));
        if leaves {
            destroy(&mut t, wurm);
            assert!(t.in_graveyard(P0, "Craw Wurm"));
        }
        t.resolve_all();
        assert!(!t.on_battlefield(bronto), "9 damage (leaves: {leaves})");
    }
}

#[test]
fn resplendent_marshal_players_respond_knowing_the_exiled_card() {
    cr!("603.12", "406.3");
    ruling!(
        "Resplendent Marshal",
        "You don’t choose which creature card you’re exiling, if any, until the triggered ability is resolving. When you do, the reflexive triggered ability triggers. Players may respond to this reflexive triggered ability, knowing the creature types of the card you exiled"
    );
    supported("Resplendent Marshal");
    let mut t = TestGame::new(2);
    let elf = t.graveyard(P0, "Llanowar Elves");
    let bears = t.graveyard(P0, "Grizzly Bears");
    let mystic = t.battlefield(P0, "Elvish Mystic");
    let lions = t.battlefield(P0, "Savannah Lions");
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[obj(elf)]);
    let from = t.asked().len();
    let marshal = t.enter(P0, "Resplendent Marshal");
    t.settle();
    assert!(first_asked(&t, P0, from, DecisionKind::Entities).is_none());
    resolve_into_reflexive(&mut t, &[]);
    // The exiled card is public: an Elf. Nothing has a counter yet.
    assert!(is_in(&t, elf, Zone::Exile));
    assert!(t.obj_now(elf).chars.has_subtype("Elf"));
    assert!(is_in(&t, bears, Zone::Graveyard(P0)));
    assert_eq!(t.counters(mystic, counters::PLUS1), 0);
    t.resolve_all();
    assert_eq!(t.counters(mystic, counters::PLUS1), 1);
    assert_eq!(t.counters(lions, counters::PLUS1), 0);
    assert_eq!(t.counters(marshal, counters::PLUS1), 0);
}

#[test]
fn young_necromancer_players_may_respond_before_the_card_returns() {
    cr!("603.12", "117.3b");
    ruling!(
        "Young Necromancer",
        "Young Necromancer's reflexive triggered ability (the one that returns a card to the battlefield) triggers when you exile two cards from your graveyard. Players may respond to that ability before the card is returned."
    );
    supported("Young Necromancer");
    let mut t = TestGame::new(2);
    let a = t.graveyard(P0, "Island");
    let b = t.graveyard(P0, "Forest");
    let giant = t.graveyard(P0, "Hill Giant");
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[obj(a), obj(b)]);
    t.answer_targets(P0, &[obj(giant)]);
    let from = t.asked().len();
    t.enter(P0, "Young Necromancer");
    t.settle();
    untargeted_then_reflexive(&mut t, P0, from, &[obj(giant)]);
    assert!(is_in(&t, a, Zone::Exile) && is_in(&t, b, Zone::Exile));
    assert!(is_in(&t, giant, Zone::Graveyard(P0)));
    // In response, the Giant is exiled: nothing returns.
    move_to(&mut t, giant, Zone::Exile);
    t.resolve_all();
    assert!(is_in(&t, giant, Zone::Exile));
    // Without a response, it returns.
    let mut t = TestGame::new(2);
    let a = t.graveyard(P0, "Island");
    let b = t.graveyard(P0, "Forest");
    let giant = t.graveyard(P0, "Hill Giant");
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[obj(a), obj(b)]);
    t.answer_targets(P0, &[obj(giant)]);
    t.enter(P0, "Young Necromancer");
    t.settle();
    t.resolve_all();
    assert!(t.on_battlefield(giant));
}

#[test]
fn isareth_pays_x_then_targets_a_card_with_mana_value_x() {
    cr!("603.12", "107.3");
    ruling!(
        "Isareth the Awakener",
        "Isareth's triggered ability goes on the stack without a target. While that ability is resolving, you may pay {X}. When you do, a second ability triggers and you pick a target creature card to return."
    );
    let mut t = TestGame::new(2);
    let is = t.battlefield(P0, "Isareth the Awakener");
    let bears = t.graveyard(P0, "Grizzly Bears");
    let giant = t.graveyard(P0, "Hill Giant");
    t.lands(P0, "Swamp", 2);
    let from = t.asked().len();
    t.answer_yes(P0, true);
    t.answer(P0, DecisionKind::X, Answer::Number(2));
    t.answer(P0, DecisionKind::Number, Answer::Number(2));
    t.answer_targets(P0, &[obj(bears)]);
    attack_with(&mut t, &[(is, Entity::Player(P1))]);
    untargeted_then_reflexive(&mut t, P0, from, &[obj(bears)]);
    // Only the mana value 2 card was a candidate: the Giant (4) wasn't.
    let cands = target_candidates(&t, P0, from);
    assert!(cands[0].contains(&obj(bears)) && !cands[0].contains(&obj(giant)));
    assert_eq!(untapped_lands(&t, P0), 0);
    assert!(is_in(&t, bears, Zone::Graveyard(P0)));
    t.resolve_all();
    assert!(t.on_battlefield(bears));
}

#[test]
fn borborygmos_and_fblthp_targets_after_lands_are_discarded() {
    cr!("603.12", "701.9a");
    ruling!(
        "Borborygmos and Fblthp",
        "The first ability of Borborygmos and Fblthp triggers and goes on the stack without a target. If you discard one or more land cards, the second “reflexive” triggered ability will trigger."
    );
    ruling!(
        "Borborygmos and Fblthp",
        "You can’t discard nonland cards, even if you want to."
    );
    supported("Borborygmos and Fblthp");
    let mut t = TestGame::new(2);
    stack_library(&mut t, P0, &["Island"]);
    let forest = t.hand(P0, "Forest");
    let bolt = t.hand(P0, "Lightning Bolt");
    let wurm = t.battlefield(P1, "Craw Wurm");
    t.answer_choose(P0, &[obj(forest)]);
    t.answer_targets(P0, &[obj(wurm)]);
    let from = t.asked().len();
    t.enter(P0, "Borborygmos and Fblthp");
    t.settle();
    untargeted_then_reflexive(&mut t, P0, from, &[obj(wurm)]);
    assert!(t.in_graveyard(P0, "Forest"));
    // The discard choice offered only land cards.
    let offered: Vec<Vec<Entity>> = t.asked()[from..]
        .iter()
        .filter_map(|(_, d)| match d {
            mtg_engine::decision::Decision::ChooseEntities { candidates, .. } => {
                Some(candidates.clone())
            }
            _ => None,
        })
        .collect();
    assert!(offered.iter().all(|c| !c.contains(&obj(bolt))));
    assert!(offered.iter().any(|c| c.contains(&obj(forest))));
    t.resolve_all();
    assert_eq!(t.obj_now(wurm).damage, 2);
    assert!(t.in_hand(P0, "Lightning Bolt"));
    // Trying to discard the Bolt anyway: it stays in hand and nothing triggers.
    let mut t = TestGame::new(2);
    stack_library(&mut t, P0, &["Island"]);
    let bolt = t.hand(P0, "Lightning Bolt");
    t.answer_choose(P0, &[obj(bolt)]);
    t.enter(P0, "Borborygmos and Fblthp");
    t.settle();
    t.resolve();
    assert!(t.in_hand(P0, "Lightning Bolt"));
    assert_eq!(t.stack_len(), 0);
}

#[test]
fn espers_to_magicite_targets_a_card_exiled_this_way() {
    cr!("603.12", "707.9b");
    ruling!(
        "Espers to Magicite",
        "You don't choose a target for Espers to Magicite at the time you cast it. Rather, a second \"reflexive\" ability triggers when you exile each opponent's graveyard this way."
    );
    ruling!(
        "Espers to Magicite",
        "The token is an artifact instead of its other types. This is a copiable value of the token that other effects may copy."
    );
    supported("Espers to Magicite");
    let mut t = TestGame::new(2);
    let giant = t.graveyard(P1, "Hill Giant");
    let from = t.asked().len();
    let spell = cast_new(&mut t, P0, "Espers to Magicite", &[]);
    assert!(targets_of(&t, spell).is_empty());
    assert!(target_candidates(&t, P0, from).is_empty());
    t.g.resolve_top();
    let exiled = t.g.current(giant);
    assert!(is_in(&t, giant, Zone::Exile));
    t.answer_targets(P0, &[obj(exiled)]);
    t.settle();
    assert_eq!(targets_of(&t, top(&t)), vec![obj(exiled)]);
    assert!(is_trigger(&t, top(&t)));
    assert!(tokens(&t, P0).is_empty());
    t.resolve_all();
    let tok = tokens(&t, P0);
    assert_eq!(tok.len(), 1);
    let o = t.obj_now(tok[0]);
    assert!(o.is(CardType::Artifact) && !o.is(CardType::Creature));
    // A copy of the token is an artifact too, not a creature (copiable values, CR 707.2).
    let tok = tok[0];
    t.answer_choose(P0, &[obj(tok)]);
    let copy = cast_new(&mut t, P0, "Sculpting Steel", &[]);
    let _ = copy;
    t.resolve_all();
    let steel = t.named_on_battlefield("Hill Giant");
    assert_eq!(
        steel.len(),
        2,
        "the token and the Sculpting Steel copying it"
    );
    for s in steel {
        let o = t.obj_now(s);
        assert!(o.is(CardType::Artifact) && !o.is(CardType::Creature));
    }
}
