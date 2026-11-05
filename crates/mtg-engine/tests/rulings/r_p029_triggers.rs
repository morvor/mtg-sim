//! Rulings batch P029 — when the copying abilities trigger: spells that target only one
//! object, even several times (CR 115.9c), "could target" (CR 115.10b), intervening "if"
//! clauses (CR 603.4), abilities that trigger once each turn or once per spell (CR 603.2h),
//! "first instant spell each turn" (CR 603.2), triggering after casting is complete
//! (CR 601.2i), and delayed triggers from mana (CR 106.6, 603.7).

use crate::r_p028_common::*;
use crate::r_p029_common::*;
use crate::r_p030_common::resolve_until_copies;
use crate::r_s01_common::supported;
use crate::r_s06_common::{activate_containing, give_control};
use crate::r_s17_common::{enter_transformed, put_onto_battlefield};
use crate::r_s20_common::tap_for_mana;
use crate::r_s25_common::*;
use mtg_engine::decision::Answer;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

/// The triggered abilities on the stack whose source is `id` (followed across zone
/// changes).
fn triggered_from(t: &TestGame, id: ObjectId) -> usize {
    abilities_from(t, t.g.current(id)).len()
}

/// `p` casts Electrolyze ("Electrolyze deals 2 damage divided as you choose among one or
/// two targets. Draw a card.") at `a` and `b`, 1 damage each.
fn electrolyze(t: &mut TestGame, p: PlayerId, a: Entity, b: Entity) -> ObjectId {
    t.answer(p, DecisionKind::Divide, Answer::Numbers(vec![1, 1]));
    lands_for_cost(t, p, "Electrolyze");
    let card = t.hand(p, "Electrolyze");
    t.cast(p, card).targets(&[a, b]).go()
}

// --- "targets only ~" -------------------------------------------------------------------------

#[test]
fn mirrorwing_dragon_triggers_on_any_player_s_spell_targeting_only_it() {
    cr!("115.9c", "603.2", "707.10d");
    ruling!(
        "Mirrorwing Dragon",
        "The ability triggers whenever a player casts an instant or sorcery spell that targets only Mirrorwing Dragon and no other object or player."
    );
    supported("Mirrorwing Dragon");
    // P0's Giant Growth at it: copied for the Bears.
    let mut t = TestGame::new(2);
    let dragon = t.battlefield(P0, "Mirrorwing Dragon");
    t.battlefield(P0, "Grizzly Bears");
    cast_new(&mut t, P0, "Giant Growth", &[obj(dragon)]);
    t.settle();
    assert_eq!(triggered_from(&t, dragon), 1);
    // P1's Lightning Bolt at it: triggers too (P1 controls no other creature: no copy).
    let mut t = TestGame::new(2);
    let dragon = t.battlefield(P0, "Mirrorwing Dragon");
    t.set_step(P1, Step::PrecombatMain);
    cast_new(&mut t, P1, "Lightning Bolt", &[obj(dragon)]);
    t.settle();
    assert_eq!(triggered_from(&t, dragon), 1);
    // Electrolyze at it and at P1: another target, no trigger.
    let mut t = TestGame::new(2);
    let dragon = t.battlefield(P0, "Mirrorwing Dragon");
    t.battlefield(P0, "Grizzly Bears");
    electrolyze(&mut t, P0, obj(dragon), Entity::Player(P1));
    t.settle();
    assert_eq!(triggered_from(&t, dragon), 0);
}

/// P0's Beamsplitter Mage and (if `other`) Grizzly Bears.
fn mage_game(other: bool) -> (TestGame, ObjectId, Option<ObjectId>) {
    supported("Beamsplitter Mage");
    let mut t = TestGame::new(2);
    let mage = t.battlefield(P0, "Beamsplitter Mage");
    let bears = other.then(|| t.battlefield(P0, "Grizzly Bears"));
    (t, mage, bears)
}

#[test]
fn beamsplitter_mage_triggers_on_your_spell_targeting_only_it_with_another_creature() {
    cr!("115.9c", "603.4", "115.10b");
    ruling!(
        "Beamsplitter Mage",
        "The ability triggers whenever you cast an instant or sorcery spell that targets only Beamsplitter Mage and no other object or player, and only if you control another creature that spell could target. If you cast an instant or sorcery spell that has multiple targets and Beamsplitter Mage is chosen as the target in each instance, Beamsplitter Mage’s ability will trigger."
    );
    supported("Agony Warp");
    supported("Electrolyze");
    // Giant Growth at the Mage with the Bears: triggers, the Bears get a copy.
    let (mut t, mage, bears) = mage_game(true);
    cast_new(&mut t, P0, "Giant Growth", &[obj(mage)]);
    t.settle();
    assert_eq!(triggered_from(&t, mage), 1);
    t.answer_choose(P0, &[obj(bears.unwrap())]);
    t.resolve_all();
    assert_eq!(t.pt(bears.unwrap()), (5, 5));
    // No other creature: no trigger.
    let (mut t, mage, _) = mage_game(false);
    cast_new(&mut t, P0, "Giant Growth", &[obj(mage)]);
    t.settle();
    assert_eq!(triggered_from(&t, mage), 0);
    // Another creature the spell couldn't target (Blastoderm has shroud): no trigger.
    supported("Blastoderm");
    let (mut t, mage, _) = mage_game(false);
    t.battlefield(P0, "Blastoderm");
    cast_new(&mut t, P0, "Giant Growth", &[obj(mage)]);
    t.settle();
    assert_eq!(triggered_from(&t, mage), 0);
    // Electrolyze at the Mage and at P1: no trigger.
    let (mut t, mage, _) = mage_game(true);
    electrolyze(&mut t, P0, obj(mage), Entity::Player(P1));
    t.settle();
    assert_eq!(triggered_from(&t, mage), 0);
    // Agony Warp with the Mage as both targets: triggers; the copy targets the Bears twice.
    let (mut t, mage, bears) = mage_game(true);
    lands_for_cost(&mut t, P0, "Agony Warp");
    let card = t.hand(P0, "Agony Warp");
    t.cast(P0, card).target(mage).target(mage).go();
    t.settle();
    assert_eq!(triggered_from(&t, mage), 1);
    t.answer_choose(P0, &[obj(bears.unwrap())]);
    let copies = resolve_until_copies(&mut t);
    assert_eq!(
        targets_of(&t, copies[0]),
        vec![obj(bears.unwrap()), obj(bears.unwrap())]
    );
    // An opponent's spell at the Mage: "you cast", no trigger.
    let (mut t, mage, _) = mage_game(true);
    t.set_step(P1, Step::PrecombatMain);
    cast_new(&mut t, P1, "Lightning Bolt", &[obj(mage)]);
    t.settle();
    assert_eq!(triggered_from(&t, mage), 0);
}

#[test]
fn beamsplitter_mage_doesnt_copy_without_another_creature_on_resolution() {
    cr!("603.4", "115.10b");
    ruling!(
        "Beamsplitter Mage",
        "If you don’t control another creature the spell could target as Beamsplitter Mage’s triggered ability resolves, you don’t copy that spell."
    );
    let (mut t, mage, bears) = mage_game(true);
    let growth = cast_new(&mut t, P0, "Giant Growth", &[obj(mage)]);
    t.settle();
    assert_eq!(triggered_from(&t, mage), 1);
    // The Bears die before the ability resolves.
    t.g.destroy(bears.unwrap(), None);
    t.g.flush_events();
    t.settle();
    t.resolve();
    assert!(spell_copies(&t).is_empty());
    assert_eq!(t.g.stack, vec![growth]);
    t.resolve_all();
    assert_eq!(t.pt(mage), (5, 5));
}

#[test]
fn ink_treader_nephilim_triggers_on_a_spell_targeting_only_it_three_times() {
    cr!("115.9c", "707.10d", "603.4");
    ruling!(
        "Ink-Treader Nephilim",
        "Ink-Treader Nephilim's ability will trigger off a spell with multiple targets if all of those targets are Ink-Treader Nephilim. For example, Seeds of Strength cast with Ink-Treader Nephilim as each of the three targets will cause the ability to trigger."
    );
    supported("Ink-Treader Nephilim");
    supported("Seeds of Strength");
    // Seeds of Strength: "Target creature gets +1/+1 until end of turn. Target creature
    // gets +1/+1 until end of turn. Target creature gets +1/+1 until end of turn."
    let mut t = TestGame::new(2);
    let ink = t.battlefield(P0, "Ink-Treader Nephilim");
    let bears = t.battlefield(P1, "Grizzly Bears");
    lands_for_cost(&mut t, P0, "Seeds of Strength");
    let card = t.hand(P0, "Seeds of Strength");
    t.cast(P0, card).target(ink).target(ink).target(ink).go();
    t.settle();
    assert_eq!(triggered_from(&t, ink), 1);
    t.resolve_all();
    assert_eq!(t.pt(ink), (6, 6));
    assert_eq!(t.pt(bears), (5, 5));
    // Two targets, one of them another creature: no trigger.
    let mut t = TestGame::new(2);
    let ink = t.battlefield(P0, "Ink-Treader Nephilim");
    let bears = t.battlefield(P1, "Grizzly Bears");
    lands_for_cost(&mut t, P0, "Seeds of Strength");
    let card = t.hand(P0, "Seeds of Strength");
    t.cast(P0, card).target(ink).target(ink).target(bears).go();
    t.settle();
    assert_eq!(triggered_from(&t, ink), 0);
}

#[test]
fn precursor_golem_triggers_on_a_spell_targeting_any_single_golem() {
    cr!("115.9c", "707.10d", "603.2");
    ruling!(
        "Precursor Golem",
        "The second ability triggers whenever a player casts an instant or sorcery spell that targets only one Golem and no other object or player. That Golem can be Precursor Golem itself, one of the Golem tokens it created, or any other Golem. It doesn't matter who controls the Golem."
    );
    supported("Precursor Golem");
    supported("Glass Golem");
    /// P0's Precursor Golem with its two tokens and P1's Glass Golem (a 6/2 Golem).
    fn golems() -> (TestGame, ObjectId, ObjectId, ObjectId) {
        let mut t = TestGame::new(2);
        let golem = t.enter(P0, "Precursor Golem");
        t.resolve_all();
        let token = t
            .g
            .permanents()
            .find(|o| o.is_token())
            .map(|o| o.id)
            .unwrap();
        let glass = t.battlefield(P1, "Glass Golem");
        (t, golem, token, glass)
    }
    // At Precursor Golem itself, at a token, at P1's Glass Golem: three copies each (for
    // the other three Golems).
    for which in 0..3 {
        let (mut t, golem, token, glass) = golems();
        let target = [golem, token, glass][which];
        cast_new(&mut t, P0, "Giant Growth", &[obj(target)]);
        t.settle();
        assert_eq!(triggered_from(&t, golem), 1, "{which}");
        let copies = resolve_until_copies(&mut t);
        assert_eq!(copies.len(), 3, "{which}");
        let mut targeted: Vec<Entity> = copies.iter().flat_map(|c| targets_of(&t, *c)).collect();
        targeted.sort();
        assert!(!targeted.contains(&obj(target)), "{which}");
    }
    // Electrolyze at a Golem and at P1: no trigger.
    let (mut t, golem, token, _) = golems();
    electrolyze(&mut t, P0, obj(token), Entity::Player(P1));
    t.settle();
    assert_eq!(triggered_from(&t, golem), 0);
    // Agony Warp at two different Golems: no trigger.
    let (mut t, golem, token, glass) = golems();
    lands_for_cost(&mut t, P0, "Agony Warp");
    let card = t.hand(P0, "Agony Warp");
    t.cast(P0, card).target(token).target(glass).go();
    t.settle();
    assert_eq!(triggered_from(&t, golem), 0);
}

#[test]
fn mirror_sheen_copies_a_spell_with_several_targets_one_of_them_you() {
    cr!("115.9b", "115.1", "707.10");
    ruling!(
        "Mirror Sheen",
        "The targeted spell can have any number of targets, as long as you are one of them."
    );
    supported("Mirror Sheen");
    // P1's Electrolyze at P0 and P0's Bears: Mirror Sheen can target it.
    let mut t = TestGame::new(2);
    t.set_step(P1, Step::PrecombatMain);
    (MIRROR_SHEEN.setup)(&mut t);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let e = electrolyze(&mut t, P1, Entity::Player(P0), obj(bears));
    (MIRROR_SHEEN.start)(&mut t, e);
    let copies = resolve_until_copies(&mut t);
    assert_eq!(copies.len(), 1);
    assert_eq!(t.obj(copies[0]).controller, P0);
    // A spell that doesn't target P0 can't be targeted.
    let mut t = TestGame::new(2);
    t.set_step(P1, Step::PrecombatMain);
    (MIRROR_SHEEN.setup)(&mut t);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let e = electrolyze(&mut t, P1, Entity::Player(P1), obj(bears));
    t.lands(P0, "Wastes", 1);
    t.lands(P0, "Island", 2);
    let sheen = t.named_on_battlefield("Mirror Sheen")[0];
    t.answer_targets(P0, &[obj(e)]);
    let r = activate_containing(&mut t, P0, sheen, "Copy target");
    assert!(r.is_err() || t.g.stack.len() == 1, "{r:?}");
}

// --- Once per spell, once per turn ---------------------------------------------------------

#[test]
fn mirari_triggers_once_per_spell_and_is_legendary() {
    cr!("603.2", "704.5j");
    ruling!(
        "Mirari",
        "It triggers once per spell, so you can’t get multiple copies of a spell using this card. And since this card is legendary, you can’t control more than one."
    );
    let mut t = TestGame::new(2);
    let mirari = t.battlefield(P0, "Mirari");
    t.lands(P0, "Wastes", 9);
    for _ in 0..3 {
        t.answer_yes(P0, true);
        keep_copy_targets(&mut t, P0);
    }
    cast_new(&mut t, P0, "Lightning Bolt", &[Entity::Player(P1)]);
    t.settle();
    assert_eq!(triggered_from(&t, mirari), 1);
    let copies = resolve_collecting_copies(&mut t);
    assert_eq!(copies.len(), 1);
    assert_eq!(t.life(P1), 14);
    // A second Mirari: the legend rule leaves one.
    t.answer_choose(P0, &[obj(mirari)]);
    t.battlefield(P0, "Mirari");
    t.settle();
    assert_eq!(t.named_on_battlefield("Mirari").len(), 1);
}

#[test]
fn cloven_casting_triggers_once_per_multicolored_instant_or_sorcery() {
    cr!("603.2", "105.4");
    ruling!(
        "Cloven Casting",
        "This ability triggers only once for each multicolored instant or sorcery spell you cast. You can’t use a single Cloven Casting to copy the same spell twice."
    );
    supported("Cloven Casting");
    // Electrolyze (blue and red) at P1, 2 damage: one trigger, one copy, though P0 could
    // pay {1} more than once.
    let mut t = TestGame::new(2);
    let cc = t.battlefield(P0, "Cloven Casting");
    t.lands(P0, "Wastes", 3);
    for _ in 0..3 {
        t.answer_yes(P0, true);
        keep_copy_targets(&mut t, P0);
    }
    t.answer(P0, DecisionKind::Divide, Answer::Numbers(vec![2]));
    lands_for_cost(&mut t, P0, "Electrolyze");
    let card = t.hand(P0, "Electrolyze");
    t.cast(P0, card).targets(&[Entity::Player(P1)]).go();
    t.settle();
    assert_eq!(triggered_from(&t, cc), 1);
    let copies = resolve_collecting_copies(&mut t);
    assert_eq!(copies.len(), 1);
    assert_eq!(t.life(P1), 16);
    // Monocolored instants and sorceries don't trigger it.
    let mut t = TestGame::new(2);
    let cc = t.battlefield(P0, "Cloven Casting");
    cast_new(&mut t, P0, "Lightning Bolt", &[Entity::Player(P1)]);
    t.settle();
    assert_eq!(triggered_from(&t, cc), 0);
    t.resolve_all();
    cast_new(&mut t, P0, "Divination", &[]);
    t.settle();
    assert_eq!(triggered_from(&t, cc), 0);
}

#[test]
fn jin_gitaxias_counters_only_the_first_opponent_spell_each_turn_in_all() {
    cr!("603.2h", "701.6a");
    ruling!(
        "Jin-Gitaxias, Progress Tyrant",
        "Jin-Gitaxias's last ability triggers once each turn, not once each turn per opponent."
    );
    // "Whenever an opponent casts an artifact, instant, or sorcery spell, counter that
    // spell. This ability triggers only once each turn."
    let mut t = TestGame::new(3);
    t.battlefield(P0, "Jin-Gitaxias, Progress Tyrant");
    cast_new(&mut t, P1, "Lightning Bolt", &[Entity::Player(P0)]);
    t.resolve_all();
    assert_eq!(t.life(P0), 20, "P1's Bolt is countered");
    cast_new(&mut t, P2, "Lightning Bolt", &[Entity::Player(P0)]);
    t.resolve_all();
    assert_eq!(t.life(P0), 17, "P2's isn't");
    // Next turn, it triggers again.
    t.advance_to(P1, Step::Upkeep);
    cast_new(&mut t, P2, "Lightning Bolt", &[Entity::Player(P0)]);
    t.resolve_all();
    assert_eq!(t.life(P0), 17);
}

#[test]
fn kalamax_considers_instants_cast_before_it_was_tapped() {
    cr!("603.2", "603.4");
    ruling!(
        "Kalamax, the Stormsire",
        "Kalamax's first ability considers the entire turn. If you cast an instant spell before Kalamax becomes tapped, its first ability won't trigger when you cast your second."
    );
    supported("Kalamax, the Stormsire");
    let mut t = TestGame::new(2);
    let k = t.battlefield(P0, "Kalamax, the Stormsire");
    cast_new(&mut t, P0, "Lightning Bolt", &[Entity::Player(P1)]);
    t.resolve_all();
    t.g.tap(k);
    cast_new(&mut t, P0, "Lightning Bolt", &[Entity::Player(P1)]);
    t.settle();
    assert_eq!(triggered_from(&t, k), 0);
    t.resolve_all();
    assert_eq!(t.life(P1), 14);
    // Tapped before the first instant: copied (positive control).
    let mut t = TestGame::new(2);
    let k = t.battlefield(P0, "Kalamax, the Stormsire");
    t.g.tap(k);
    cast_new(&mut t, P0, "Lightning Bolt", &[Entity::Player(P1)]);
    keep_copy_targets(&mut t, P0);
    t.settle();
    assert_eq!(triggered_from(&t, k), 1);
    t.resolve_all();
    assert_eq!(t.life(P1), 14);
}

// --- Primal Amulet // Primal Wellspring ----------------------------------------------------------

#[test]
fn primal_amulet_transforms_after_the_spell_is_cast_so_wellspring_cant_pay_for_it() {
    cr!("601.2i", "603.3", "712.10");
    ruling!(
        "Primal Amulet // Primal Wellspring",
        "Primal Amulet's last ability triggers once you've completed casting a spell. Notably, you can't use Primal Wellspring to pay for the spell that gives Primal Amulet its fourth counter."
    );
    // Three charge counters; Opt ({U}, {1} less: still {U}) is the fourth spell.
    let mut t = TestGame::new(2);
    let amulet = t.battlefield(P0, "Primal Amulet // Primal Wellspring");
    t.g.add_counters(obj(amulet), "charge", 3, None);
    t.lands(P0, "Island", 1);
    let card = t.hand(P0, "Opt");
    t.answer_yes(P0, true); // transform
    let opt = t.cast(P0, card).go();
    // The trigger is on the stack above Opt: the card is still Primal Amulet.
    t.settle();
    assert_eq!(triggered_from(&t, amulet), 1);
    assert_eq!(t.obj_now(amulet).chars.name, "Primal Amulet");
    t.resolve();
    let w = t.g.current(amulet);
    assert_eq!(t.obj_now(w).chars.name, "Primal Wellspring");
    assert!(!t.obj_now(w).tapped);
    // Opt was paid for with the Island; no Wellspring mana was spent: no copy.
    assert_eq!(t.g.stack, vec![opt]);
    let hand = t.hand_size(P0);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand + 1);
}

#[test]
fn primal_wellspring_s_trigger_happens_after_it_leaves_and_its_mana_buys_anything() {
    cr!("106.6", "603.7c", "106.4");
    ruling!(
        "Primal Amulet // Primal Wellspring",
        "The delayed triggered ability from Primal Wellspring's mana ability will trigger even if Primal Wellspring leaves the battlefield before that mana is spent."
    );
    ruling!(
        "Primal Amulet // Primal Wellspring",
        "The mana produced by Primal Wellspring can be spent on anything, not just an instant or sorcery spell."
    );
    // {U} from the Wellspring; the Wellspring is destroyed; then Divination is cast with
    // that mana: copied.
    let mut t = TestGame::new(2);
    let w = enter_transformed(&mut t, P0, "Primal Amulet // Primal Wellspring");
    t.answer(P0, DecisionKind::Option, Answer::Index(1));
    assert!(tap_for_mana(&mut t, P0, w, "Add one mana"));
    t.g.destroy(w, None);
    t.g.flush_events();
    t.settle();
    assert!(!t.on_battlefield(w));
    t.lands(P0, "Wastes", 2);
    let card = t.hand(P0, "Divination");
    t.cast(P0, card).go();
    keep_copy_targets(&mut t, P0);
    assert_eq!(resolve_until_copies(&mut t).len(), 1);
    // {G} spent on Grizzly Bears: castable, nothing copied.
    let mut t = TestGame::new(2);
    let w = enter_transformed(&mut t, P0, "Primal Amulet // Primal Wellspring");
    t.answer(P0, DecisionKind::Option, Answer::Index(4));
    assert!(tap_for_mana(&mut t, P0, w, "Add one mana"));
    t.lands(P0, "Wastes", 1);
    let card = t.hand(P0, "Grizzly Bears");
    t.cast(P0, card).go();
    assert!(t.g.player(P0).mana_pool.mana.is_empty());
    t.settle();
    assert_eq!(t.stack_len(), 1);
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Grizzly Bears").len(), 1);
}

// --- Gandalf the Grey ---------------------------------------------------------------------------

/// The modes that may be chosen in the last mode choice for Gandalf's trigger.
fn gandalf_mode_choice(t: &TestGame, from: usize) -> Option<(PlayerId, Vec<usize>)> {
    t.asked()[from..].iter().rev().find_map(|(p, d)| match d {
        mtg_engine::decision::Decision::ChooseModes { available, .. } => {
            Some((*p, available.clone()))
        }
        _ => None,
    })
}

#[test]
fn gandalf_s_chosen_modes_stay_chosen_for_a_new_controller() {
    cr!("700.2", "603.3a");
    ruling!(
        "Gandalf the Grey",
        "It doesn't matter who has chosen any particular mode. For example, say you control Gandalf the Grey and have chosen the first two modes. If an opponent gains control of Gandalf the Grey, that player can choose only the third or fourth mode."
    );
    supported("Gandalf the Grey");
    let mut t = TestGame::new(2);
    let gandalf = t.battlefield(P0, "Gandalf the Grey");
    let bears = t.battlefield(P1, "Grizzly Bears");
    // First mode: tap the Bears; second: 3 damage to P1.
    t.answer(P0, DecisionKind::Modes, Answer::Indices(vec![0]));
    t.answer_targets(P0, &[obj(bears)]);
    t.answer_yes(P0, true);
    cast_new(&mut t, P0, "Opt", &[]);
    t.resolve_all();
    assert!(t.obj_now(bears).tapped);
    t.answer(P0, DecisionKind::Modes, Answer::Indices(vec![1]));
    cast_new(&mut t, P0, "Opt", &[]);
    t.resolve_all();
    assert_eq!(t.life(P1), 17);
    // P1 gains control and casts an instant: only the third and fourth modes.
    give_control(&mut t, gandalf, P1);
    t.answer(P1, DecisionKind::Modes, Answer::Indices(vec![1]));
    let from = t.asked().len();
    cast_new(&mut t, P1, "Opt", &[]);
    t.settle();
    let (p, available) = gandalf_mode_choice(&t, from).expect("a mode choice");
    assert_eq!(p, P1);
    assert_eq!(available, vec![2, 3]);
    t.resolve_all();
    // P1 asked for the second mode; it wasn't available: P0 took no damage.
    assert_eq!(t.life(P0), 20);
}

#[test]
fn a_gandalf_that_returns_is_a_new_object_with_no_modes_chosen() {
    cr!("400.7", "700.2");
    ruling!(
        "Gandalf the Grey",
        "The phrase \"that hasn't been chosen\" refers only to that specific Gandalf the Grey. If Gandalf the Grey leaves the battlefield (possibly because of the fourth mode of its ability) and then returns to the battlefield later, it will be a new object with no memory of the modes chosen when it was last on the battlefield."
    );
    let mut t = TestGame::new(2);
    let gandalf = t.battlefield(P0, "Gandalf the Grey");
    // Second mode, then the fourth: Gandalf goes on top of its owner's library.
    t.answer(P0, DecisionKind::Modes, Answer::Indices(vec![1]));
    cast_new(&mut t, P0, "Opt", &[]);
    t.resolve_all();
    assert_eq!(t.life(P1), 17);
    t.answer(P0, DecisionKind::Modes, Answer::Indices(vec![3]));
    cast_new(&mut t, P0, "Lightning Bolt", &[Entity::Player(P1)]);
    t.resolve_all();
    assert_eq!(t.life(P1), 14);
    let card = t.g.current(gandalf);
    assert_eq!(t.zone(card), mtg_engine::object::Zone::Library(P0));
    // It returns: the second mode can be chosen again.
    let g2 = put_onto_battlefield(&mut t, P0, card, false).expect("it returns");
    assert_ne!(g2, gandalf);
    t.answer(P0, DecisionKind::Modes, Answer::Indices(vec![1]));
    let from = t.asked().len();
    cast_new(&mut t, P0, "Opt", &[]);
    t.settle();
    let (_, available) = gandalf_mode_choice(&t, from).expect("a mode choice");
    assert!(available.contains(&1));
    t.resolve_all();
    assert_eq!(t.life(P1), 11);
}
