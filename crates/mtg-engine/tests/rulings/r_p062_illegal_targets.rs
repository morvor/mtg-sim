//! Rulings batch P062 — a spell or ability whose targets are all illegal as it tries to
//! resolve doesn't resolve: none of its effects happen, including the untargeted ones
//! ("Draw a card", "put a +1/+1 counter on Grimgrin", "The Ring tempts you") (CR 608.2b).
//! One with some illegal targets resolves without affecting them. A triggered ability
//! with no legal targets is removed from the stack (CR 603.3d).

use crate::r_p062_common::*;
use crate::r_s01_common::{attack_with, supported, triggers_on_stack};
use crate::r_s02_common::destroy;
use crate::r_s05_common::enter;
use crate::r_s25_common::cast_new;
use mtg_engine::testing::*;
use mtg_engine::*;

/// Everything about P0's hand and library, to check that nothing was drawn, discarded or
/// exiled.
fn cards(t: &TestGame) -> (usize, usize, usize) {
    (t.hand_size(P0), t.library_size(P0), t.graveyard_size(P0))
}

/// P0 casts the real spell `name` targeting P1's Grizzly Bears, the Bears are destroyed
/// in response, and the spell tries to resolve: it does nothing.
fn spell_fizzles(t: &mut TestGame, name: &str) {
    supported(name);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let spell = cast_new(t, P0, name, &[obj(bears)]);
    destroy(t, bears);
    let before = cards(t);
    t.resolve_all();
    let (hand, library, graveyard) = cards(t);
    // Only the spell itself went to the graveyard.
    assert_eq!(
        (hand, library, graveyard),
        (before.0, before.1, before.2 + 1)
    );
    assert!(!t.g.is_live(spell));
}

#[test]
fn rapier_wit_with_an_illegal_target_doesnt_draw() {
    cr!("608.2b");
    ruling!(
        "Rapier Wit",
        "If the target creature is an illegal target as Rapier Wit tries to resolve, it won't resolve and none of its effects will happen. You won't draw a card."
    );
    let mut t = TestGame::new(2);
    spell_fizzles(&mut t, "Rapier Wit");
}

#[test]
fn code_of_constraint_with_an_illegal_target_doesnt_draw() {
    cr!("608.2b");
    ruling!(
        "Code of Constraint",
        "If the target creature is an illegal target by the time Code of Constraint tries to resolve, the spell doesn't resolve. You don't draw a card."
    );
    let mut t = TestGame::new(2);
    spell_fizzles(&mut t, "Code of Constraint");
}

#[test]
fn riverwheel_sweep_with_an_illegal_target_doesnt_exile_cards() {
    cr!("608.2b");
    ruling!(
        "Riverwheel Sweep",
        "If the target creature is an illegal target as Riverwheel Sweep tries to resolve, it won’t resolve and none of its effects will happen. You won’t exile any cards."
    );
    supported("Riverwheel Sweep");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    // {2/U}{2/R}{2/W}, paid with one mana of each color.
    for land in ["Island", "Mountain", "Plains"] {
        t.lands(P0, land, 1);
    }
    let card = t.hand(P0, "Riverwheel Sweep");
    t.cast_with(P0, card, &[obj(bears)]).expect("cast");
    destroy(&mut t, bears);
    let exiled = t.g.exile.len();
    let library = t.library_size(P0);
    t.resolve_all();
    assert_eq!(t.g.exile.len(), exiled);
    assert_eq!(t.library_size(P0), library);
    assert!(t.in_graveyard(P0, "Riverwheel Sweep"));
}

#[test]
fn snow_day_with_only_illegal_targets_doesnt_draw_or_discard() {
    cr!("608.2b");
    ruling!(
        "Snow Day",
        "If you chose targets for Snow Day as you cast it, and none of those targets are legal as it tries to resolve, it will be removed from the stack and do nothing. You will not draw or discard."
    );
    supported("Snow Day");
    let mut t = TestGame::new(2);
    let a = t.battlefield(P1, "Grizzly Bears");
    let b = t.battlefield(P1, "Hill Giant");
    crate::r_s25_common::lands_for_cost(&mut t, P0, "Snow Day");
    t.hand(P0, "Grizzly Bears");
    let card = t.hand(P0, "Snow Day");
    t.answer_targets(P0, &[obj(a), obj(b)]);
    t.cast_with(P0, card, &[]).expect("cast");
    destroy(&mut t, a);
    destroy(&mut t, b);
    let before = cards(&t);
    t.resolve_all();
    assert_eq!(cards(&t), (before.0, before.1, before.2 + 1));
    assert!(t.in_hand(P0, "Grizzly Bears"));
}

#[test]
fn will_of_the_naga_doesnt_affect_a_target_that_became_illegal() {
    cr!("608.2b", "702.11b");
    ruling!(
        "Will of the Naga",
        "If you chose two targets and one is an illegal target as Will of the Naga resolves, that creature won't become tapped and it won't be stopped from untapping during its controller's next untap step. It won't be affected by Will of the Naga in any way."
    );
    supported("Will of the Naga");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let giant = t.battlefield(P1, "Hill Giant");
    crate::r_s25_common::lands_for_cost(&mut t, P0, "Will of the Naga");
    let card = t.hand(P0, "Will of the Naga");
    t.answer_targets(P0, &[obj(bears), obj(giant)]);
    t.cast_with(P0, card, &[]).expect("cast");
    // In response, P1 gives the Giant hexproof (Blossoming Defense).
    t.lands(P1, "Forest", 1);
    let defense = t.hand(P1, "Blossoming Defense");
    t.cast_with(P1, defense, &[obj(giant)]).expect("cast");
    t.resolve();
    t.resolve_all();
    assert!(is_tapped(&t, bears));
    assert!(!is_tapped(&t, giant));
    // Tapped (by attacking, say) before P1's next untap step, the Giant untaps then;
    // the Bears don't.
    tap(&mut t, giant);
    through_untap_step(&mut t, P1);
    assert!(!is_tapped(&t, giant));
    assert!(is_tapped(&t, bears));
}

#[test]
fn dreamdew_entrancer_with_an_illegal_target_doesnt_draw() {
    cr!("608.2b");
    ruling!(
        "Dreamdew Entrancer",
        "If the target creature is an illegal target as Dreamdew Entrancer’s last ability tries to resolve, it won’t resolve and none of its effects will happen. You won’t put stun counters on the creature, and you won’t draw cards no matter who controls it."
    );
    supported("Dreamdew Entrancer");
    // Legal: P0 targets their own Bears: three stun counters, and P0 draws two.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.answer_targets(P0, &[obj(bears)]);
    enter(&mut t, P0, "Dreamdew Entrancer");
    let hand = t.hand_size(P0);
    t.resolve_all();
    assert_eq!(t.counters(bears, "stun"), 3);
    assert_eq!(t.hand_size(P0), hand + 2);
    // Illegal: the Bears leave before it resolves; no cards are drawn.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.answer_targets(P0, &[obj(bears)]);
    enter(&mut t, P0, "Dreamdew Entrancer");
    destroy(&mut t, bears);
    let hand = t.hand_size(P0);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand);
}

#[test]
fn grimgrins_attack_trigger_with_an_illegal_target_doesnt_add_a_counter() {
    cr!("608.2b");
    ruling!(
        "Grimgrin, Corpse-Born",
        "If the targeted creature is an illegal target by the time Grimgrin's last ability resolves, the entire ability doesn't resolve and none of its effects will occur. You won't put a +1/+1 counter on Grimgrin."
    );
    supported("Grimgrin, Corpse-Born");
    let mut t = TestGame::new(2);
    let grimgrin = t.battlefield(P0, "Grimgrin, Corpse-Born");
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.answer_targets(P0, &[obj(bears)]);
    attack_with(&mut t, &[(grimgrin, Entity::Player(P1))]);
    assert_eq!(triggers_on_stack(&t, "destroy target creature"), 1);
    destroy(&mut t, bears);
    t.resolve_all();
    assert_eq!(t.counters(grimgrin, "+1/+1"), 0);
    // With a legal target, it gets the counter.
    let mut t = TestGame::new(2);
    let grimgrin = t.battlefield(P0, "Grimgrin, Corpse-Born");
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.answer_targets(P0, &[obj(bears)]);
    attack_with(&mut t, &[(grimgrin, Entity::Player(P1))]);
    t.resolve_all();
    assert!(t.in_graveyard(P1, "Grizzly Bears"));
    assert_eq!(t.counters(grimgrin, "+1/+1"), 1);
}

#[test]
fn grimgrins_attack_trigger_with_no_creatures_to_target_is_removed() {
    cr!("603.3d");
    ruling!(
        "Grimgrin, Corpse-Born",
        "If the defending player controls no creatures when Grimgrin attacks, the last ability will be removed from the stack and have no effect."
    );
    supported("Grimgrin, Corpse-Born");
    let mut t = TestGame::new(2);
    let grimgrin = t.battlefield(P0, "Grimgrin, Corpse-Born");
    // (P0's own creature isn't a legal target.)
    t.battlefield(P0, "Grizzly Bears");
    attack_with(&mut t, &[(grimgrin, Entity::Player(P1))]);
    assert_eq!(triggers_on_stack(&t, "destroy target creature"), 0);
    t.resolve_all();
    assert_eq!(t.counters(grimgrin, "+1/+1"), 0);
    assert_eq!(t.named_on_battlefield("Grizzly Bears").len(), 1);
}

#[test]
fn wicked_wolf_doesnt_fight_an_illegal_target_or_after_leaving() {
    cr!("608.2b", "701.14b");
    ruling!(
        "Wicked Wolf",
        "If the target creature is an illegal target when Wicked Wolf's first ability tries to resolve, the ability doesn't resolve. If Wicked Wolf is no longer on the battlefield, the target creature won't deal or be dealt damage."
    );
    supported("Wicked Wolf");
    // The target gains hexproof in response: no fight, the Wolf isn't dealt damage.
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P1, "Hill Giant");
    t.answer_targets(P0, &[obj(giant)]);
    let wolf = enter(&mut t, P0, "Wicked Wolf");
    t.lands(P1, "Forest", 1);
    let defense = t.hand(P1, "Blossoming Defense");
    t.cast_with(P1, defense, &[obj(giant)]).expect("cast");
    t.resolve();
    t.resolve_all();
    assert_eq!(t.obj_now(wolf).damage, 0);
    assert_eq!(t.obj_now(giant).damage, 0);
    // The Wolf leaves in response: the Giant isn't dealt damage.
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P1, "Hill Giant");
    t.answer_targets(P0, &[obj(giant)]);
    let wolf = enter(&mut t, P0, "Wicked Wolf");
    destroy(&mut t, wolf);
    t.resolve_all();
    assert_eq!(t.obj_now(giant).damage, 0);
    // Otherwise they fight.
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P1, "Hill Giant");
    t.answer_targets(P0, &[obj(giant)]);
    let wolf = enter(&mut t, P0, "Wicked Wolf");
    t.resolve_all();
    assert!(!t.on_battlefield(wolf) || t.obj_now(wolf).damage == 3);
    assert!(!t.on_battlefield(giant));
}

#[test]
fn scroll_of_isildur_with_an_illegal_target_doesnt_tempt_you() {
    cr!("608.2b", "701.54a");
    ruling!(
        "Scroll of Isildur",
        "If the target is not legal as Scroll of Isildur's first chapter ability tries to resolve, the ability is removed from the stack. The Ring won't tempt you."
    );
    supported("Scroll of Isildur");
    let mut t = TestGame::new(2);
    let thopter = t.battlefield(P1, "Ornithopter");
    t.answer_targets(P0, &[obj(thopter)]);
    enter(&mut t, P0, "Scroll of Isildur");
    destroy(&mut t, thopter);
    t.resolve_all();
    assert_eq!(t.g.player(P0).ring_level, 0);
    // With a legal target, P0 gains control of it and the Ring tempts P0.
    let mut t = TestGame::new(2);
    let thopter = t.battlefield(P1, "Ornithopter");
    t.battlefield(P0, "Grizzly Bears");
    t.answer_targets(P0, &[obj(thopter)]);
    enter(&mut t, P0, "Scroll of Isildur");
    t.resolve_all();
    assert_eq!(t.obj_now(thopter).controller, P0);
    assert_eq!(t.g.player(P0).ring_level, 1);
}

#[test]
fn scroll_of_isildur_leaving_before_chapter_one_resolves_changes_no_control() {
    cr!("611.2b");
    ruling!(
        "Scroll of Isildur",
        "If Scroll of Isildur leaves the battlefield or you lose control of it before its first chapter ability resolves, control of the target artifact doesn't change at all."
    );
    supported("Scroll of Isildur");
    let mut t = TestGame::new(2);
    let thopter = t.battlefield(P1, "Ornithopter");
    t.answer_targets(P0, &[obj(thopter)]);
    let scroll = enter(&mut t, P0, "Scroll of Isildur");
    destroy(&mut t, scroll);
    t.resolve_all();
    assert_eq!(t.obj_now(thopter).controller, P1);
    // P0 losing control of the Saga first has the same result.
    let mut t = TestGame::new(2);
    let thopter = t.battlefield(P1, "Ornithopter");
    t.answer_targets(P0, &[obj(thopter)]);
    let scroll = enter(&mut t, P0, "Scroll of Isildur");
    crate::r_s06_common::give_control(&mut t, scroll, P1);
    t.resolve_all();
    assert_eq!(t.obj_now(thopter).controller, P1);
}
