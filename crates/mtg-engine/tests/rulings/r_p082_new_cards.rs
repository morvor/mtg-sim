//! Rulings batch P082 — cards the batch made compile: Steadfast Armasaur and Vortex
//! Elemental ("blocking or blocked by it", with the source's last known combat information
//! once it has left, CR 113.7a, 608.2h), Ride Down ("creatures that were blocked by that
//! creature this combat") and Outmaneuver ("assign their combat damage this turn as though
//! they weren't blocked", CR 510.1c).

use crate::r_p082_common::*;
use crate::r_s01_common::{attack_with, supported};
use crate::r_s02_common::destroy;
use crate::r_s04_common::add_mana;
use crate::r_s21_common::legal_blocks;
use crate::r_s25_common::cast_new;
use mtg_engine::keywords::KeywordKind;
use mtg_engine::mana::ManaType;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

fn obj(id: ObjectId) -> Entity {
    Entity::Object(id)
}

// ---------------------------------------------------------------------------------
// Steadfast Armasaur: "{1}{W}, {T}: This creature deals damage equal to its toughness to
// target creature blocking or blocked by it."
// ---------------------------------------------------------------------------------

fn armasaur_activate(t: &mut TestGame, a: ObjectId, target: ObjectId) {
    add_mana(t, P0, ManaType::W, 1);
    add_mana(t, P0, ManaType::C, 1);
    t.activate(P0, a, 0, &[obj(target)]).unwrap();
}

#[test]
fn armasaur_uses_its_last_known_toughness() {
    cr!("113.7a", "608.2h");
    ruling!(
        "Steadfast Armasaur",
        "If Steadfast Armasaur is no longer on the battlefield as its ability resolves, use its toughness as it last existed on the battlefield to determine how damage is dealt."
    );
    // Armasaur (2/3, pumped to 5/6) blocked by Palace Guard (1/4); it's destroyed in
    // response: the Guard is dealt 6 damage.
    let mut t = TestGame::new(2);
    let (a, bs) = block_with(&mut t, "Steadfast Armasaur", &["Palace Guard"]);
    armasaur_activate(&mut t, a, bs[0]);
    cast_new(&mut t, P0, "Giant Growth", &[obj(a)]);
    t.resolve();
    assert_eq!(t.pt(a), (5, 6));
    destroy(&mut t, a);
    t.resolve_all();
    assert!(!t.on_battlefield(bs[0]), "dealt 6 damage");
}

#[test]
fn armasaur_tapping_doesnt_remove_from_combat() {
    cr!("506.4", "510.1");
    ruling!(
        "Steadfast Armasaur",
        "Tapping an attacking or blocking creature doesn't remove it from combat. If the target of Steadfast Armasaur's ability survives the damage, Steadfast Armasaur will deal combat damage to and be dealt combat damage by that creature as normal."
    );
    let mut t = TestGame::new(2);
    let (a, bs) = block_with(&mut t, "Steadfast Armasaur", &["Palace Guard"]);
    armasaur_activate(&mut t, a, bs[0]);
    t.resolve_all();
    assert!(t.obj_now(a).tapped);
    assert_eq!(t.obj_now(bs[0]).damage, 3);
    finish_combat(&mut t);
    // 3 + 2 combat damage: the Guard dies; it dealt 1 back.
    assert!(!t.on_battlefield(bs[0]));
    assert_eq!(t.obj_now(a).damage, 1);
}

// ---------------------------------------------------------------------------------
// Vortex Elemental.
// ---------------------------------------------------------------------------------

fn vortex_first(t: &mut TestGame, v: ObjectId) {
    add_mana(t, P0, ManaType::U, 1);
    t.activate(P0, v, 0, &[]).unwrap();
}

#[test]
fn vortex_elemental_alone_or_outside_combat() {
    cr!("701.24a", "602.5");
    ruling!(
        "Vortex Elemental",
        "If no creature is blocking or being blocked by Vortex Elemental as its first ability resolves, it alone will be put on top of its owner’s library, and that library will be shuffled. You can activate the first ability outside of combat."
    );
    supported("Vortex Elemental");
    let mut t = TestGame::new(2);
    let v = t.battlefield(P0, "Vortex Elemental");
    let bears = t.battlefield(P1, "Grizzly Bears");
    let lib = (t.library_size(P0), t.library_size(P1));
    vortex_first(&mut t, v);
    t.resolve_all();
    assert!(matches!(t.zone(v), Zone::Library(_)));
    assert!(t.on_battlefield(bears));
    assert_eq!((t.library_size(P0), t.library_size(P1)), (lib.0 + 1, lib.1));
}

#[test]
fn vortex_elemental_gone_still_takes_its_blockers() {
    cr!("113.7a", "608.2h", "701.24a");
    ruling!(
        "Vortex Elemental",
        "If Vortex Elemental isn’t on the battlefield when its first ability resolves, any creatures blocking or blocked by it when it left the battlefield will be put on top of their owners’ libraries and those libraries will be shuffled."
    );
    let mut t = TestGame::new(2);
    let (v, bs) = block_with(&mut t, "Vortex Elemental", &["Grizzly Bears", "Hill Giant"]);
    let other = t.battlefield(P1, "Grizzly Bears");
    // On the battlefield: everything goes.
    vortex_first(&mut t, v);
    destroy(&mut t, v);
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Vortex Elemental"));
    for b in &bs {
        assert!(matches!(t.zone(*b), Zone::Library(_)));
    }
    assert!(t.on_battlefield(other));
}

#[test]
fn vortex_elemental_forced_block_only_if_able() {
    cr!("509.1c", "509.1d");
    ruling!(
        "Vortex Elemental",
        "The creature blocks Vortex Elemental only if it’s able to do so as the declare blockers step begins. If, at that time, the creature is tapped, it’s affected by a spell or ability that says it can’t block, or Vortex Elemental isn’t attacking its controller or a planeswalker controlled by that player, then it doesn’t block. If there’s a cost associated with having the creature block, the player isn’t forced to pay that cost. If that cost isn’t paid, the creature won’t block."
    );
    /// Activates the second ability targeting `target`.
    fn lure(t: &mut TestGame, v: ObjectId, target: ObjectId) {
        add_mana(t, P0, ManaType::U, 2);
        add_mana(t, P0, ManaType::C, 3);
        t.activate(P0, v, 1, &[obj(target)]).unwrap();
        t.resolve_all();
    }
    // Able: it must block.
    let mut t = TestGame::new(2);
    let v = t.battlefield(P0, "Vortex Elemental");
    let bears = t.battlefield(P1, "Grizzly Bears");
    lure(&mut t, v, bears);
    attack_with(&mut t, &[(v, Entity::Player(P1))]);
    assert!(!legal_blocks(&mut t, P1, &[]));
    assert!(legal_blocks(&mut t, P1, &[(bears, v)]));
    // Tapped: it doesn't.
    let mut t = TestGame::new(2);
    let v = t.battlefield(P0, "Vortex Elemental");
    let bears = t.battlefield(P1, "Grizzly Bears");
    lure(&mut t, v, bears);
    t.g.objects[bears.0 as usize].tapped = true;
    attack_with(&mut t, &[(v, Entity::Player(P1))]);
    assert!(legal_blocks(&mut t, P1, &[]));
    // Vortex attacks another player: no requirement for P1's creature.
    let mut t = TestGame::new(3);
    let v = t.battlefield(P0, "Vortex Elemental");
    let bears = t.battlefield(P1, "Grizzly Bears");
    lure(&mut t, v, bears);
    attack_with(&mut t, &[(v, Entity::Player(P2))]);
    assert!(legal_blocks(&mut t, P1, &[]));
    // A cost to block (Archangel of Tithes attacking too): not forced to pay.
    let mut t = TestGame::new(2);
    let v = t.battlefield(P0, "Vortex Elemental");
    let angel = t.battlefield(P0, "Archangel of Tithes");
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.lands(P1, "Wastes", 1);
    lure(&mut t, v, bears);
    attack_with(
        &mut t,
        &[(v, Entity::Player(P1)), (angel, Entity::Player(P1))],
    );
    t.answer(
        P1,
        DecisionKind::Blockers,
        mtg_engine::decision::Answer::Blockers(vec![]),
    );
    crate::r_s21_common::go_to(&mut t, Step::DeclareBlockers);
    assert!(crate::r_s21_common::blocks_now(&t).is_empty());
}

// ---------------------------------------------------------------------------------
// Ride Down.
// ---------------------------------------------------------------------------------

#[test]
fn ride_down_attackers_stay_blocked_and_trample_over() {
    cr!("509.1h", "702.19e");
    ruling!(
        "Ride Down",
        "The attacking creatures that the destroyed creature was blocking remain blocked (even if no other creatures were blocking them). An attacking creature with trample that has no creature blocking it can deal its combat damage to the defending player, planeswalker, or battle."
    );
    ruling!(
        "Ride Down",
        "The attacking creatures that the destroyed creature was blocking remain blocked (even if no other creatures were blocking them). An attacking creature with trample that has no creature blocking it will deal its combat damage to the defending player or planeswalker."
    );
    supported("Ride Down");
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P0, "Hill Giant");
    let other = t.battlefield(P0, "Grizzly Bears");
    let bears = t.battlefield(P1, "Grizzly Bears");
    blocked(&mut t, &[giant, other], &[(bears, giant)]);
    cast_new(&mut t, P0, "Ride Down", &[obj(bears)]);
    t.resolve_all();
    assert!(!t.on_battlefield(bears));
    assert!(is_blocked(&t, giant));
    assert!(t.obj_now(giant).chars.has_keyword(KeywordKind::Trample));
    assert!(!t.obj_now(other).chars.has_keyword(KeywordKind::Trample));
    finish_combat(&mut t);
    assert_eq!(t.life(P1), 15, "3 trample damage plus the unblocked Bears' 2");
}

#[test]
fn ride_down_on_a_creature_that_entered_blocking_gives_no_trample() {
    cr!("509.1a", "506.3");
    ruling!(
        "Ride Down",
        "In some rare cases, the blocking creature wasn't declared as a blocking creature that combat (for example, if it entered the battlefield blocking). In that case, the attacking creatures it was blocking won't gain trample even though the blocking creature is destroyed."
    );
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P0, "Hill Giant");
    blocked(&mut t, &[giant], &[]);
    let late = crate::r_s21_common::enter_blocking(&mut t, P1, "Grizzly Bears", giant);
    assert!(is_blocked(&t, giant));
    cast_new(&mut t, P0, "Ride Down", &[obj(late)]);
    t.resolve_all();
    assert!(!t.on_battlefield(late));
    assert!(!t.obj_now(giant).chars.has_keyword(KeywordKind::Trample));
}

// ---------------------------------------------------------------------------------
// Outmaneuver.
// ---------------------------------------------------------------------------------

#[test]
fn outmaneuver_damage_goes_to_the_attacked_planeswalker() {
    cr!("510.1c", "506.2");
    ruling!(
        "Outmaneuver",
        "If a creature is attacking a planeswalker, assigning its damage as though it weren’t blocked means the damage is assigned to the planeswalker, not to the defending player."
    );
    supported("Outmaneuver");
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P0, "Hill Giant");
    let pw = t.battlefield(P1, "Ajani Goldmane");
    let loyalty = t.counters(pw, "loyalty");
    let bears = t.battlefield(P1, "Grizzly Bears");
    crate::r_s03_common::to_blockers(&mut t, &[(giant, obj(pw))], &[(bears, giant)]);
    t.lands(P0, "Mountain", 2);
    let c = t.hand(P0, "Outmaneuver");
    t.cast(P0, c).x(1).targets(&[obj(giant)]).go();
    t.resolve_all();
    finish_combat(&mut t);
    assert_eq!(t.life(P1), 20);
    assert_eq!(t.obj_now(bears).damage, 0);
    assert_eq!(loyalty, 4);
    assert_eq!(t.counters(pw, "loyalty"), 1, "the planeswalker was dealt 3 damage");
}
