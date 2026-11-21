//! Rulings batch P221 — prowess and cast triggers (Monastery Mentor), prowl (Auntie's
//! Snitch), rally (Lantern Scout), rampage (Varchild's War-Riders) and reconfigure
//! (Cloudsteel Kirin).

use crate::r_s01_common::*;
use crate::r_s02_common::destroy;
use crate::r_s04_common::{add_mana, stack_items};
use crate::r_s06_common::damage;
use crate::r_s14_common::*;
use crate::r_s30_common::set_life;
use mtg_engine::decision::Answer;
use mtg_engine::keywords::KeywordKind;
use mtg_engine::mana::ManaType;
use mtg_engine::object::{CastMethod, Zone};
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

#[test]
fn monastery_mentor_prowess_and_token_trigger_in_either_order() {
    cr!("603.3b", "702.108a", "601.2i");
    ruling!(
        "Monastery Mentor",
        "Casting a noncreature spell will cause both prowess and Monastery Mentor's other ability to trigger. You can put these abilities on the stack in either order. Whichever ability is put on the stack last will resolve first."
    );
    ruling!(
        "Monastery Mentor",
        "The spell that causes Monastery Mentor's second ability to trigger will not cause the prowess ability of the Monk token that's created to trigger."
    );
    supported("Monastery Mentor");
    let mut tops = Vec::new();
    for order in [vec![0, 1], vec![1, 0]] {
        let mut t = TestGame::new(2);
        let mentor = t.battlefield(P0, "Monastery Mentor");
        t.answer(P0, DecisionKind::Order, Answer::Indices(order));
        cast_from_hand(&mut t, P0, "Shock", &[Entity::Player(P1)]);
        t.settle();
        assert_eq!(triggers_from(&t, mentor), 2);
        let token_on_top = stack_items(&t).last().unwrap().contains("Monk");
        tops.push(token_on_top);
        // The one put on the stack last resolves first.
        t.resolve();
        if token_on_top {
            assert_eq!(t.named_on_battlefield("Monk Token").len(), 1);
            assert_eq!(t.pt(mentor), (2, 2));
        } else {
            assert!(t.named_on_battlefield("Monk Token").is_empty());
            assert_eq!(t.pt(mentor), (3, 3));
        }
        t.resolve_all();
        assert_eq!(t.pt(mentor), (3, 3));
        // The Monk token's prowess didn't trigger for the Shock that made it.
        let monk = t.named_on_battlefield("Monk Token")[0];
        assert_eq!(t.pt(monk), (1, 1));
        assert_eq!(t.life(P1), 18);
    }
    assert_ne!(tops[0], tops[1]);
}

#[test]
fn auntie_s_snitch_returns_then_can_be_cast_for_its_prowl_cost() {
    cr!("702.76a", "603.2", "118.9a");
    ruling!(
        "Auntie's Snitch",
        "If a Goblin or Rogue you control deals combat damage to a player while Auntie's Snitch is in your graveyard, you can return Auntie's Snitch to your hand, then you can cast it for its prowl cost that turn."
    );
    supported("Auntie's Snitch");
    let mut t = TestGame::new(2);
    let snitch = place(&mut t, P0, "Auntie's Snitch", Zone::Graveyard(P0));
    let goblin = t.battlefield(P0, "Raging Goblin");
    t.answer_yes(P0, true);
    attack_with(&mut t, &[(goblin, Entity::Player(P1))]);
    block_and_finish(&mut t, P1, &[]);
    assert_eq!(t.life(P1), 19);
    let snitch = t.g.current(snitch);
    assert_eq!(t.zone(snitch), Zone::Hand(P0));
    // Postcombat main: cast it for {1}{B}.
    t.advance_to(P0, Step::PostcombatMain);
    add_mana(&mut t, P0, ManaType::B, 2);
    let spell = t
        .cast(P0, snitch)
        .method(CastMethod::Keyword(KeywordKind::Prowl))
        .go();
    assert_eq!(t.zone(spell), Zone::Stack);
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Auntie's Snitch").len(), 1);
}

#[test]
fn rally_resolving_again_affects_new_creatures_and_lifelink_is_redundant() {
    cr!("702.15f", "603.2", "611.2c", "207.2c");
    ruling!(
        "Lantern Scout",
        "Multiple instances of lifelink on the same creature are redundant. There usually isn't much benefit in having the rally ability resolve more than once in a turn, other than that new creatures will be affected by the ability resolving an additional time."
    );
    supported("Lantern Scout");
    supported("Kor Bladewhirl");
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let scout = t.enter(P0, "Lantern Scout");
    t.resolve_all();
    assert!(t.obj_now(bears).has_keyword(KeywordKind::Lifelink));
    // A creature that enters later doesn't have lifelink...
    let ape = t.enter(P0, "Grizzly Bears");
    t.resolve_all();
    assert!(!t.obj_now(ape).has_keyword(KeywordKind::Lifelink));
    // ...until rally resolves again (another Ally, Kor Bladewhirl, enters).
    t.enter(P0, "Kor Bladewhirl");
    t.resolve_all();
    assert!(t.obj_now(ape).has_keyword(KeywordKind::Lifelink));
    assert!(t.obj_now(scout).has_keyword(KeywordKind::Lifelink));
    // The Bears have lifelink twice: 2 damage still gains 2 life.
    let target = t.battlefield(P1, "Hill Giant");
    let life = t.life(P0);
    damage(&mut t, bears, 2, target);
    assert_eq!(t.life(P0), life + 2);
}

#[test]
fn the_rampage_bonus_is_locked_in_when_the_trigger_resolves() {
    cr!("702.23a", "702.23b", "509.1h", "611.2c");
    ruling!(
        "Varchild's War-Riders",
        "The rampage bonus is calculated only once per combat, when the triggered ability resolves. Adding or removing blockers later in combat won't change the bonus."
    );
    supported("Varchild's War-Riders");
    let mut t = TestGame::new(2);
    let riders = t.battlefield(P0, "Varchild's War-Riders");
    let walls: Vec<ObjectId> = (0..3).map(|_| t.battlefield(P1, "Ornithopter")).collect();
    attack_with(&mut t, &[(riders, Entity::Player(P1))]);
    t.answer(
        P1,
        DecisionKind::Blockers,
        Answer::Blockers(walls.iter().map(|w| (*w, riders)).collect()),
    );
    t.advance_to(P0, Step::DeclareBlockers);
    t.settle();
    t.resolve_all();
    // Three blockers: +2/+2.
    assert_eq!(t.pt(riders), (5, 6));
    // Removing a blocker doesn't change the bonus.
    destroy(&mut t, walls[0]);
    assert_eq!(t.pt(riders), (5, 6));
}

#[test]
fn cloudsteel_kirin_moving_between_your_creatures_doesnt_lose_the_game() {
    cr!("702.151a", "104.3b", "704.5a", "301.5c");
    ruling!(
        "Cloudsteel Kirin",
        "If you have 0 or less life while Cloudsteel Kirin is equipped to a creature, activating reconfigure to attach Cloudsteel Kirin to another creature you control doesn't cause you to lose the game. You would lose the game if Cloudsteel Kirin becomes unattached or becomes attached to a creature you don't control."
    );
    supported("Cloudsteel Kirin");
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let kirin = t.battlefield(P0, "Cloudsteel Kirin");
    let a = t.battlefield(P0, "Grizzly Bears");
    let b = t.battlefield(P0, "Hill Giant");
    add_mana(&mut t, P0, ManaType::C, 5);
    t.activate(P0, kirin, 0, &[Entity::Object(a)]).unwrap();
    t.resolve_all();
    assert_eq!(t.obj(kirin).attached_to, Some(Entity::Object(a)));
    set_life(&mut t, P0, 0);
    t.settle();
    assert!(!t.g.player(P0).has_lost);
    // Reconfigure onto the Hill Giant: still can't lose.
    add_mana(&mut t, P0, ManaType::C, 5);
    t.activate(P0, kirin, 0, &[Entity::Object(b)]).unwrap();
    t.resolve_all();
    assert_eq!(t.obj(kirin).attached_to, Some(Entity::Object(b)));
    assert!(!t.g.player(P0).has_lost);
    // Unattaching it: P0 loses.
    add_mana(&mut t, P0, ManaType::C, 5);
    t.activate(P0, kirin, 1, &[]).unwrap();
    t.resolve_all();
    assert!(t.g.player(P0).has_lost);
}
