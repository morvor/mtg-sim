//! Rulings batch P122 — "deals damage to target creature; that creature can't block":
//! a spell or ability whose targets are all illegal doesn't resolve, and one with some
//! illegal targets doesn't affect those (CR 608.2b); the "can't block" part applies even
//! if the damage is prevented (CR 608.2c, 615.1); a spell that targets more than one
//! thing isn't a spell "with a single target" (CR 115.7).

use crate::r_p122_common::*;
use crate::r_s01_common::supported;
use crate::r_s02_common::destroy;
use crate::r_s09_common::{legal_attack, to_combat};
use crate::r_s28_common::cast_card;
use mtg_engine::mana::ManaType;
use mtg_engine::object::CastMethod;
use mtg_engine::testing::*;
use mtg_engine::*;

/// Whether `id` could block now (not counting what's attacking).
fn can_block(t: &mut TestGame, id: ObjectId) -> bool {
    t.g.recompute();
    let id = t.g.current(id);
    t.g.can_block_at_all(id)
}

/// P0 casts Healing Salve's prevention mode on `target` ("prevent the next 3 damage that
/// would be dealt to any target this turn").
fn salve(t: &mut TestGame, target: ObjectId) {
    supported("Healing Salve");
    crate::r_s25_common::lands_for_cost(t, P0, "Healing Salve");
    let c = t.hand(P0, "Healing Salve");
    t.cast(P0, c).modes(&[1]).target(obj(target)).go();
    t.resolve_all();
}

#[test]
fn intimidation_bolt_with_an_illegal_target() {
    cr!("608.2b");
    ruling!(
        "Intimidation Bolt",
        "If the targeted creature has become an illegal target by the time Intimidation Bolt resolves, Intimidation Bolt doesn't resolve. Creatures will still be able to attack that turn."
    );
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P0, "Hill Giant");
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.answer_targets(P0, &[obj(bears)]);
    cast_card(&mut t, P0, "Intimidation Bolt");
    destroy(&mut t, bears);
    t.resolve_all();
    to_combat(&mut t, P0);
    assert!(legal_attack(&mut t, &[(giant, Entity::Player(P1))]));
}

#[test]
fn intimidation_bolt_survivor_attacks_alone() {
    cr!("608.2c", "611.2c");
    ruling!(
        "Intimidation Bolt",
        "If the targeted creature survives the damage, it can attack this turn, but no other creatures can. If the targeted creature is destroyed, no creatures can attack this turn, including creatures that enter later on."
    );
    // The target survives.
    let mut t = TestGame::new(2);
    let wurm = t.battlefield(P0, "Craw Wurm");
    let giant = t.battlefield(P0, "Hill Giant");
    t.answer_targets(P0, &[obj(wurm)]);
    cast_card(&mut t, P0, "Intimidation Bolt");
    t.resolve_all();
    assert_eq!(t.obj_now(wurm).damage, 3);
    to_combat(&mut t, P0);
    assert!(legal_attack(&mut t, &[(wurm, Entity::Player(P1))]));
    assert!(!legal_attack(&mut t, &[(giant, Entity::Player(P1))]));
    // The target is destroyed: nothing can attack, not even a creature that enters later.
    let mut t = TestGame::new(2);
    let target = t.battlefield(P0, "Grizzly Bears");
    let giant = t.battlefield(P0, "Hill Giant");
    t.answer_targets(P0, &[obj(target)]);
    cast_card(&mut t, P0, "Intimidation Bolt");
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Grizzly Bears"));
    let goblin = t.enter(P0, "Raging Goblin");
    to_combat(&mut t, P0);
    assert!(!legal_attack(&mut t, &[(giant, Entity::Player(P1))]));
    assert!(!legal_attack(&mut t, &[(goblin, Entity::Player(P1))]));
}

#[test]
fn sparkmages_gambit_with_one_or_both_targets_illegal() {
    cr!("608.2b");
    ruling!(
        "Sparkmage's Gambit",
        "If Sparkmage's Gambit has two targets, and one of them is illegal as Sparkmage's Gambit resolves, only the remaining legal target will be affected."
    );
    supported("Sparkmage's Gambit");
    let mut t = TestGame::new(2);
    let a = t.battlefield(P1, "Grizzly Bears");
    let b = t.battlefield(P1, "Hill Giant");
    t.answer_targets(P0, &[obj(a), obj(b)]);
    cast_card(&mut t, P0, "Sparkmage's Gambit");
    // Hill Giant becomes an illegal target (it leaves and returns as a new object).
    let b_new = flicker(&mut t, b);
    t.resolve_all();
    assert_eq!(t.obj_now(a).damage, 1);
    assert!(!can_block(&mut t, a));
    assert_eq!(t.g.obj(b_new).damage, 0);
    assert!(can_block(&mut t, b_new), "the illegal target can block");
    // Both targets illegal: it doesn't resolve.
    let mut t = TestGame::new(2);
    let a = t.battlefield(P1, "Grizzly Bears");
    let b = t.battlefield(P1, "Hill Giant");
    t.answer_targets(P0, &[obj(a), obj(b)]);
    let spell = cast_card(&mut t, P0, "Sparkmage's Gambit");
    destroy(&mut t, a);
    destroy(&mut t, b);
    t.resolve_all();
    assert!(!crate::r_s07_common::resolved(&t, spell));
    assert!(t.in_graveyard(P0, "Sparkmage's Gambit"));
}

#[test]
fn sparkmages_gambit_damage_prevented_still_cant_block() {
    cr!("608.2c", "615.1");
    ruling!(
        "Sparkmage's Gambit",
        "If Sparkmage's Gambit resolves, but the damage is prevented or redirected, the target creatures still won't be able to block that turn."
    );
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    salve(&mut t, bears);
    t.answer_targets(P0, &[obj(bears)]);
    cast_card(&mut t, P0, "Sparkmage's Gambit");
    t.resolve_all();
    assert_eq!(t.obj_now(bears).damage, 0);
    assert!(!can_block(&mut t, bears));
}

#[test]
fn wrap_in_flames_with_an_illegal_target() {
    cr!("608.2b");
    ruling!(
        "Wrap in Flames",
        "If any of the targeted creatures is an illegal target by the time Wrap in Flames resolves, it won’t be dealt damage and will be able to block. The other targeted creatures will still be affected."
    );
    supported("Wrap in Flames");
    let mut t = TestGame::new(2);
    let a = t.battlefield(P1, "Grizzly Bears");
    let b = t.battlefield(P1, "Hill Giant");
    let c = t.battlefield(P1, "Craw Wurm");
    t.answer_targets(P0, &[obj(a), obj(b), obj(c)]);
    cast_card(&mut t, P0, "Wrap in Flames");
    // Craw Wurm becomes an illegal target (it changes zones and returns as a new object).
    let c_new = flicker(&mut t, c);
    t.resolve_all();
    assert_eq!(t.obj_now(a).damage, 1);
    assert_eq!(t.obj_now(b).damage, 1);
    assert!(!can_block(&mut t, a) && !can_block(&mut t, b));
    assert_eq!(t.g.obj(c_new).damage, 0);
    assert!(can_block(&mut t, c_new));
}

/// Exiles a permanent and returns it to the battlefield as a new object (CR 400.7).
fn flicker(t: &mut TestGame, id: ObjectId) -> ObjectId {
    use mtg_engine::object::Zone;
    let ex = crate::r_s05_common::move_to(t, id, Zone::Exile).unwrap();
    let new = crate::r_s05_common::move_to(t, ex, Zone::Battlefield).unwrap();
    t.g.objects[new.0 as usize].summoning_sick = false;
    new
}

/// P0 casts `name` targeting P1's Grizzly Bears, which is destroyed in response: the spell
/// doesn't resolve and P0 doesn't draw.
fn no_draw_with_illegal_target(name: &str) {
    supported(name);
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.answer_targets(P0, &[obj(bears)]);
    cast_card(&mut t, P0, name);
    let hand = t.hand_size(P0);
    destroy(&mut t, bears);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand, "{name}: no card drawn");
    assert!(t.in_graveyard(P0, name));
    // With a legal target, a card is drawn.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.answer_targets(P0, &[obj(bears)]);
    cast_card(&mut t, P0, name);
    let hand = t.hand_size(P0);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand + 1, "{name}");
}

#[test]
fn blindblast_with_an_illegal_target() {
    cr!("608.2b");
    ruling!(
        "Blindblast",
        "If the target creature is an illegal target by the time Blindblast tries to resolve, the spell doesn’t resolve. You won’t draw a card."
    );
    no_draw_with_illegal_target("Blindblast");
}

#[test]
fn renegade_tactics_with_an_illegal_target() {
    cr!("608.2b");
    ruling!(
        "Renegade Tactics",
        "If the target creature is an illegal target by the time Renegade Tactics tries to resolve, the spell doesn't resolve. You don't draw a card."
    );
    no_draw_with_illegal_target("Renegade Tactics");
}

#[test]
fn blood_aspirant_illegal_target_or_damage_prevented() {
    cr!("608.2b", "608.2c", "615.1");
    ruling!(
        "Blood Aspirant",
        "If the target creature is an illegal target by the time Blood Aspirant's last ability tries to resolve, the ability won't resolve. The creature can block as normal. If the target is legal but not dealt damage (most likely because of a prevention effect), it can't block this turn."
    );
    supported("Blood Aspirant");
    // Illegal target (it left and came back as a new object): it can block.
    let mut t = TestGame::new(2);
    let aspirant = t.battlefield(P0, "Blood Aspirant");
    t.battlefield(P0, "Llanowar Elves");
    let bears = t.battlefield(P1, "Grizzly Bears");
    mana(&mut t, P0, ManaType::R, 2);
    t.activate(P0, aspirant, 0, &[obj(bears)]).unwrap();
    let bears_new = flicker(&mut t, bears);
    t.resolve_all();
    assert!(can_block(&mut t, bears_new));
    assert_eq!(t.g.obj(bears_new).damage, 0);
    // Damage prevented: it can't block.
    let mut t = TestGame::new(2);
    let aspirant = t.battlefield(P0, "Blood Aspirant");
    t.battlefield(P0, "Llanowar Elves");
    let bears = t.battlefield(P1, "Grizzly Bears");
    salve(&mut t, bears);
    mana(&mut t, P0, ManaType::R, 2);
    t.activate(P0, aspirant, 0, &[obj(bears)]).unwrap();
    t.resolve_all();
    assert_eq!(t.obj_now(bears).damage, 0);
    assert!(!can_block(&mut t, bears));
}

#[test]
fn beat_a_path_with_one_or_both_targets_illegal() {
    cr!("608.2b", "715.3d");
    ruling!(
        "Bellowing Bruiser // Beat a Path",
        "If two targets are chosen for Beat a Path and one of them becomes an illegal target but not the other, the legal target will be unable to block and Beat a Path will be exiled. If both targets are illegal, Beat a Path will be put into its owner's graveyard."
    );
    supported("Bellowing Bruiser // Beat a Path");
    let cast_beat = |t: &mut TestGame, a: ObjectId, b: ObjectId| {
        let card = t.hand(P0, "Bellowing Bruiser // Beat a Path");
        mana(t, P0, ManaType::R, 3);
        t.cast(P0, card)
            .method(CastMethod::Half(1))
            .targets(&[obj(a), obj(b)])
            .go();
    };
    let mut t = TestGame::new(2);
    let a = t.battlefield(P1, "Grizzly Bears");
    let b = t.battlefield(P1, "Hill Giant");
    cast_beat(&mut t, a, b);
    let b_new = flicker(&mut t, b);
    t.resolve_all();
    assert!(!can_block(&mut t, a));
    assert!(can_block(&mut t, b_new));
    assert!(t.in_exile("Bellowing Bruiser // Beat a Path") || t.in_exile("Bellowing Bruiser"));
    let mut t = TestGame::new(2);
    let a = t.battlefield(P1, "Grizzly Bears");
    let b = t.battlefield(P1, "Hill Giant");
    cast_beat(&mut t, a, b);
    destroy(&mut t, a);
    destroy(&mut t, b);
    t.resolve_all();
    assert_eq!(t.graveyard_size(P0), 1);
    assert!(!t.in_exile("Bellowing Bruiser // Beat a Path") && !t.in_exile("Bellowing Bruiser"));
}

/// The modes P0 may choose for Untimely Malfunction now (asked as it's cast).
fn malfunction_modes(t: &mut TestGame) -> Vec<usize> {
    use mtg_engine::decision::Decision;
    let malfunction = t.hand(P0, "Untimely Malfunction");
    crate::r_s25_common::lands_for_cost(t, P0, "Untimely Malfunction");
    let from = t.asked().len();
    let first = t.g.battlefield[0];
    let _ = t.cast(P0, malfunction).modes(&[2]).target(obj(first)).try_go();
    t.asked()[from..]
        .iter()
        .find_map(|(_, d)| match d {
            Decision::ChooseModes { available, .. } => Some(available.clone()),
            _ => None,
        })
        .expect("modes asked")
}

#[test]
fn untimely_malfunction_cant_target_a_spell_with_two_targets() {
    cr!("115.7");
    ruling!(
        "Untimely Malfunction",
        "If a spell or ability targets multiple things, you can't target it with Untimely Malfunction's second mode, even if all but one of those targets have become illegal."
    );
    supported("Untimely Malfunction");
    // Sparkmage's Gambit targeting two creatures, one of which has since died: the second
    // mode ("change the target of target spell or ability with a single target") has no
    // legal target.
    let mut t = TestGame::new(2);
    let a = t.battlefield(P0, "Grizzly Bears");
    let b = t.battlefield(P0, "Hill Giant");
    t.set_step(P1, mtg_engine::turn::Step::PrecombatMain);
    t.answer_targets(P1, &[obj(a), obj(b)]);
    cast_card(&mut t, P1, "Sparkmage's Gambit");
    destroy(&mut t, b);
    assert!(!malfunction_modes(&mut t).contains(&1));
    // Targeting only one creature, it has a single target.
    let mut t = TestGame::new(2);
    let a = t.battlefield(P0, "Grizzly Bears");
    t.set_step(P1, mtg_engine::turn::Step::PrecombatMain);
    t.answer_targets(P1, &[obj(a)]);
    cast_card(&mut t, P1, "Sparkmage's Gambit");
    assert!(malfunction_modes(&mut t).contains(&1));
}
