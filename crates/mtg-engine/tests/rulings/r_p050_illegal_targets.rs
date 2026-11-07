//! Rulings batch P050 (second round) — drain spells and abilities whose targets become
//! illegal before they resolve: with every target illegal they don't resolve at all
//! (CR 608.2b), with some targets illegal the rest of the effect still happens but the
//! illegal target isn't affected (CR 608.2c), and a target's legality can depend on an
//! object's last known information (CR 608.2h).

use crate::r_p057_common::into_upkeep;
use crate::r_s01_common::supported;
use crate::r_s02_common::destroy;
use crate::r_s05_common::enter;
use crate::r_s25_common::cast_new;
use mtg_engine::decision::Decision;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

fn obj(id: ObjectId) -> Entity {
    Entity::Object(id)
}

/// P1 casts Veil of Summer ("You and permanents you control gain hexproof from blue and
/// from black until end of turn.") in response.
fn veil(t: &mut TestGame) {
    supported("Veil of Summer");
    cast_new(t, P1, "Veil of Summer", &[]);
    t.resolve();
}

/// P1 activates Tormod's Crypt ("{T}, Sacrifice this artifact: Exile target player's
/// graveyard.") targeting itself, in response, and it resolves.
fn crypt(t: &mut TestGame) {
    supported("Tormod's Crypt");
    let c = t.battlefield(P1, "Tormod's Crypt");
    t.activate(P1, c, 0, &[Entity::Player(P1)]).unwrap();
    t.resolve();
}

/// The total mana in `p`'s pool.
fn pool(t: &TestGame, p: PlayerId) -> usize {
    t.g.player(p).mana_pool.total()
}

/// Klothys on P0's battlefield; its first-main-phase trigger targets P1's graveyard card
/// `name`; `respond` runs with the trigger on the stack. Returns the game with the
/// trigger resolved (and the mana it may have added still in the pool).
fn klothys(name: &str, respond: fn(&mut TestGame)) -> TestGame {
    supported("Klothys, God of Destiny");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Klothys, God of Destiny");
    let card = t.graveyard(P1, name);
    t.answer_targets(P0, &[obj(card)]);
    t.set_step(P0, Step::Draw);
    t.advance_to(P0, Step::PrecombatMain);
    t.settle();
    assert_eq!(t.stack_len(), 1);
    respond(&mut t);
    assert_eq!(pool(&t, P0), 0);
    t.g.resolve_top();
    t
}

#[test]
fn klothys_illegal_target_no_mana_life_or_damage() {
    cr!("608.2b");
    ruling!(
        "Klothys, God of Destiny",
        "If the target card in a graveyard is an illegal target by the time Klothys’s last ability tries to resolve, the ability won’t resolve. You won’t add mana, gain life, or deal damage."
    );
    // A nonland card, exiled in response: no life, no damage.
    let t = klothys("Grizzly Bears", crypt);
    assert_eq!((t.life(P0), t.life(P1)), (20, 20));
    assert_eq!(pool(&t, P0), 0);
    // A land card, exiled in response: no mana.
    let t = klothys("Forest", crypt);
    assert_eq!(pool(&t, P0), 0);
    assert_eq!((t.life(P0), t.life(P1)), (20, 20));
    // Controls: the same targets left alone.
    let t = klothys("Grizzly Bears", |_| {});
    assert!(t.in_exile("Grizzly Bears"));
    assert_eq!((t.life(P0), t.life(P1)), (22, 18));
    let t = klothys("Forest", |_| {});
    assert!(t.in_exile("Forest"));
    assert_eq!(pool(&t, P0), 1);
}

#[test]
fn klothys_an_artifact_land_card_adds_mana_only() {
    cr!("205.2a");
    ruling!(
        "Klothys, God of Destiny",
        "If you exile a land card that has another card type (such as an artifact land), you add one mana, but you won’t gain life or deal damage."
    );
    let t = klothys("Seat of the Synod", |_| {});
    assert!(t.in_exile("Seat of the Synod"));
    let p = &t.g.player(P0).mana_pool;
    assert_eq!(p.total(), 1);
    assert_eq!(
        p.count(mana::ManaType::R) + p.count(mana::ManaType::G),
        1,
        "{{R}} or {{G}}"
    );
    assert_eq!((t.life(P0), t.life(P1)), (20, 20));
    // An artifact card that isn't a land: life and damage, no mana.
    let t = klothys("Ornithopter", |_| {});
    assert_eq!(pool(&t, P0), 0);
    assert_eq!((t.life(P0), t.life(P1)), (22, 18));
}

#[test]
fn oath_of_kaya_illegal_target_no_life() {
    cr!("608.2b");
    ruling!(
        "Oath of Kaya",
        "If the target is an illegal target by the time the first ability of Oath of Kaya tries to resolve, the ability doesn’t resolve. You won’t gain 3 life."
    );
    supported("Oath of Kaya");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Hill Giant");
    t.answer_targets(P0, &[obj(bears)]);
    enter(&mut t, P0, "Oath of Kaya");
    assert_eq!(t.stack_len(), 1);
    destroy(&mut t, bears);
    t.resolve_all();
    assert_eq!((t.life(P0), t.life(P1)), (20, 20));
    // Control: the target still there.
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P1, "Hill Giant");
    t.answer_targets(P0, &[obj(giant)]);
    enter(&mut t, P0, "Oath of Kaya");
    t.resolve_all();
    assert!(t.in_graveyard(P1, "Hill Giant"));
    assert_eq!(t.life(P0), 23);
}

/// Cruel Ultimatum cast by P0 at P1; `respond` runs with it on the stack. P1 has Grizzly
/// Bears and three cards in hand; P0 has Hill Giant in its graveyard.
fn cruel_ultimatum(respond: fn(&mut TestGame)) -> (TestGame, usize, usize) {
    supported("Cruel Ultimatum");
    let mut t = TestGame::new(2);
    t.battlefield(P1, "Grizzly Bears");
    for _ in 0..3 {
        t.hand(P1, "Island");
    }
    t.graveyard(P0, "Hill Giant");
    cast_new(&mut t, P0, "Cruel Ultimatum", &[Entity::Player(P1)]);
    let hands = (t.hand_size(P0), t.hand_size(P1));
    respond(&mut t);
    t.resolve_all();
    (t, hands.0, hands.1)
}

#[test]
fn cruel_ultimatum_illegal_target_does_nothing() {
    cr!("608.2b", "702.11c", "702.11d");
    ruling!(
        "Cruel Ultimatum",
        "If the target opponent is an illegal target as Cruel Ultimatum tries to resolve, it won't resolve and none of its effects will happen. (I'm not going to list them all here because it would just be disappointing to hear all of the things you won't get to do.)"
    );
    let (t, hand0, _) = cruel_ultimatum(veil);
    assert_eq!(t.named_on_battlefield("Grizzly Bears").len(), 1);
    // 3, plus the card Veil of Summer drew (an opponent cast a blue or black spell).
    assert_eq!(t.hand_size(P1), 4, "no discard");
    assert_eq!((t.life(P0), t.life(P1)), (20, 20));
    assert!(t.in_graveyard(P0, "Hill Giant"));
    assert_eq!(t.hand_size(P0), hand0);
    assert!(t.in_graveyard(P0, "Cruel Ultimatum"));
    // Control: everything happens.
    let (t, hand0, _) = cruel_ultimatum(|_| {});
    assert!(t.named_on_battlefield("Grizzly Bears").is_empty());
    assert_eq!(t.hand_size(P1), 0);
    assert_eq!((t.life(P0), t.life(P1)), (25, 15));
    assert!(t.in_hand(P0, "Hill Giant"));
    assert_eq!(t.hand_size(P0), hand0 + 4);
}

#[test]
fn cruel_ultimatum_finishes_resolving_before_both_players_lose() {
    cr!("608.2", "704.3", "704.5a", "704.5b", "104.4a");
    ruling!(
        "Cruel Ultimatum",
        "If, as Cruel Ultimatum begins to resolve, your opponent's life total is 5 or less and you have two or fewer cards in your library, the game will result in a draw. Your opponent's life total will drop to 0 or less, but Cruel Ultimatum must finish resolving completely before state-based actions are performed. You'll then be forced to draw three cards and fail to draw one. When state-based actions are finally performed, you and your opponent will both lose the game at the same time, which means the game is a draw."
    );
    supported("Cruel Ultimatum");
    let mut t = TestGame::new(2);
    t.g.players[0].library.truncate(2);
    crate::r_s33_common::set_life(&mut t, P1, 5);
    cast_new(&mut t, P0, "Cruel Ultimatum", &[Entity::Player(P1)]);
    t.settle();
    t.g.resolve_top();
    // The spell finished: P0 gained 5 life after P1 hit 0 and the draws.
    assert_eq!((t.life(P0), t.life(P1)), (25, 0));
    assert_eq!(t.library_size(P0), 0);
    assert!(!t.has_lost(P0) && !t.has_lost(P1), "no SBAs yet");
    t.settle();
    assert!(t.has_lost(P0) && t.has_lost(P1));
    assert_eq!(t.g.result, Some(mtg_engine::game::GameResult::Draw));
}

#[test]
fn vizkopa_confessor_illegal_target_no_life_paid_no_reveal() {
    cr!("608.2b", "702.11c", "702.11d");
    ruling!(
        "Vizkopa Confessor",
        "If the target opponent is an illegal target when Vizkopa Confessor's last ability tries to resolve, it won't resolve and none of its effects will happen. You won't pay life and no player will reveal any cards."
    );
    supported("Vizkopa Confessor");
    for hexproof in [true, false] {
        let mut t = TestGame::new(2);
        for _ in 0..3 {
            t.hand(P1, "Island");
        }
        t.answer_targets(P0, &[Entity::Player(P1)]);
        t.answer(P0, DecisionKind::X, Answer::Number(2));
        enter(&mut t, P0, "Vizkopa Confessor");
        assert_eq!(t.stack_len(), 1);
        if hexproof {
            veil(&mut t);
        }
        let from = t.asked().len();
        t.resolve_all();
        let numbers = t.asked()[from..]
            .iter()
            .filter(|(_, d)| matches!(d, Decision::ChooseX { .. }))
            .count();
        if hexproof {
            assert_eq!(numbers, 0, "not asked to pay life");
            assert_eq!(t.life(P0), 20);
            assert_eq!(t.hand_size(P1), 3);
            assert!(!t.in_exile("Island"));
        } else {
            assert_eq!(numbers, 1);
            assert_eq!(t.life(P0), 18);
            assert_eq!(t.hand_size(P1), 2);
            assert!(t.in_exile("Island"));
        }
    }
}

#[test]
fn stolen_grain_illegal_target_no_life() {
    cr!("608.2b", "702.11c", "702.11d");
    ruling!(
        "Stolen Grain",
        "If the targeted opponent is an illegal target by the time Stolen Grain resolves, the ability doesn’t resolve. You won’t gain life."
    );
    supported("Stolen Grain");
    let mut t = TestGame::new(2);
    cast_new(&mut t, P0, "Stolen Grain", &[Entity::Player(P1)]);
    veil(&mut t);
    t.resolve_all();
    assert_eq!((t.life(P0), t.life(P1)), (20, 20));
    assert!(t.in_graveyard(P0, "Stolen Grain"));
    let mut t = TestGame::new(2);
    cast_new(&mut t, P0, "Stolen Grain", &[Entity::Player(P1)]);
    t.resolve_all();
    assert_eq!((t.life(P0), t.life(P1)), (25, 15));
}

/// Suffer the Past (X = 2) at P1 and two cards in P1's graveyard; `respond` runs with it
/// on the stack.
fn suffer_the_past(respond: fn(&mut TestGame)) -> TestGame {
    supported("Suffer the Past");
    let mut t = TestGame::new(2);
    let a = t.graveyard(P1, "Grizzly Bears");
    let b = t.graveyard(P1, "Hill Giant");
    t.graveyard(P1, "Ornithopter");
    crate::r_s25_common::lands_for_cost(&mut t, P0, "Suffer the Past");
    t.lands(P0, "Swamp", 2);
    let c = t.hand(P0, "Suffer the Past");
    t.answer_targets(P0, &[Entity::Player(P1)]);
    t.answer_targets(P0, &[obj(a), obj(b)]);
    t.cast(P0, c).x(2).go();
    assert_eq!(t.stack_len(), 1);
    respond(&mut t);
    t.resolve_all();
    t
}

#[test]
fn suffer_the_past_player_illegal_cards_legal() {
    cr!("608.2b", "608.2c", "702.11c", "702.11d");
    interp!("dependent-target-checked-against-an-illegal-target");
    ruling!(
        "Suffer the Past",
        "If the targeted player becomes an illegal target by the time Suffer the Past resolves but the targeted cards don’t, you still exile the cards and gain life. The player won’t be lose any life, though."
    );
    let t = suffer_the_past(veil);
    assert!(t.in_exile("Grizzly Bears") && t.in_exile("Hill Giant"));
    assert!(t.in_graveyard(P1, "Ornithopter"));
    assert_eq!((t.life(P0), t.life(P1)), (22, 20));
    // Control: the player legal too.
    let t = suffer_the_past(|_| {});
    assert!(t.in_exile("Grizzly Bears") && t.in_exile("Hill Giant"));
    assert_eq!((t.life(P0), t.life(P1)), (22, 18));
}

/// Nihiloor enters for P0, who taps `tapped` (its creature); the reflexive trigger
/// targets P1's `target`; `respond` runs with that trigger on the stack.
fn nihiloor(
    tapped_name: &str,
    target_name: &str,
    respond: fn(&mut TestGame, ObjectId),
) -> (TestGame, ObjectId) {
    supported("Nihiloor");
    let mut t = TestGame::new(2);
    let tapped = t.battlefield(P0, tapped_name);
    let target = t.battlefield(P1, target_name);
    t.battlefield(P1, "Craw Wurm");
    t.answer_choose(P0, &[obj(tapped)]);
    t.answer_targets(P0, &[obj(target)]);
    enter(&mut t, P0, "Nihiloor");
    t.resolve(); // the enters trigger: tap, then the reflexive trigger
    assert!(t.obj_now(tapped).tapped);
    assert_eq!(t.stack_len(), 1, "the reflexive trigger");
    respond(&mut t, tapped);
    t.resolve_all();
    (t, target)
}

#[test]
fn nihiloor_uses_the_tapped_creatures_last_known_power() {
    cr!("608.2b", "608.2h", "603.12");
    ruling!(
        "Nihiloor",
        "If the tapped creature is no longer on the battlefield as the trigger resolves, use its last existence on the battlefield to determine whether the target is still legal as the ability resolves."
    );
    // Hill Giant (3/3) tapped, targeting P1's Hill Giant; P0's Giant is destroyed in
    // response: its last known power (3) keeps the target legal.
    let (t, target) = nihiloor("Hill Giant", "Hill Giant", |t, g| destroy(t, g));
    assert_eq!(t.obj_now(target).controller, P0);
    // P0's Giant gets -2/-2 (Disfigure) and is then destroyed: its last known power is 1,
    // so P1's Hill Giant (power 3) is an illegal target.
    let (t, target) = nihiloor("Hill Giant", "Hill Giant", |t, g| {
        supported("Disfigure");
        cast_new(t, P1, "Disfigure", &[obj(g)]);
        t.resolve();
        assert_eq!(t.pt(g), (1, 1));
        destroy(t, g);
    });
    assert_eq!(t.obj_now(target).controller, P1);
    // Shrunk but still on the battlefield: also illegal.
    let (t, target) = nihiloor("Hill Giant", "Hill Giant", |t, g| {
        cast_new(t, P1, "Disfigure", &[obj(g)]);
        t.resolve();
    });
    assert_eq!(t.obj_now(target).controller, P1);
    // Control: nothing happens in response.
    let (t, target) = nihiloor("Hill Giant", "Hill Giant", |_, _| {});
    assert_eq!(t.obj_now(target).controller, P0);
}

#[test]
fn kaya_ghost_assassin_without_kaya_you_just_lose_2_life() {
    cr!("400.7");
    ruling!(
        "Kaya, Ghost Assassin",
        "If you didn't choose a target creature for the first ability, and Kaya isn't on the battlefield as the ability resolves, you'll just lose 2 life. No card will be exiled or returned to the battlefield."
    );
    supported("Kaya, Ghost Assassin");
    for kaya_destroyed in [true, false] {
        let mut t = TestGame::new(2);
        let kaya = t.battlefield(P0, "Kaya, Ghost Assassin");
        t.battlefield(P1, "Grizzly Bears");
        t.answer(P0, DecisionKind::Targets, Answer::Entities(vec![]));
        t.activate(P0, kaya, 0, &[]).unwrap();
        assert_eq!(t.stack_len(), 1);
        if kaya_destroyed {
            destroy(&mut t, kaya);
        }
        t.resolve_all();
        assert_eq!(t.life(P0), 18);
        assert_eq!(t.named_on_battlefield("Grizzly Bears").len(), 1);
        if kaya_destroyed {
            assert!(t.in_graveyard(P0, "Kaya, Ghost Assassin"));
            assert!(!t.in_exile("Kaya, Ghost Assassin"));
        } else {
            assert!(t.in_exile("Kaya, Ghost Assassin"));
        }
        // P0's next upkeep: Kaya returns only if she was exiled.
        into_upkeep(&mut t, P0);
        t.resolve_all();
        assert_eq!(
            t.named_on_battlefield("Kaya, Ghost Assassin").len(),
            usize::from(!kaya_destroyed)
        );
        if kaya_destroyed {
            assert!(t.in_graveyard(P0, "Kaya, Ghost Assassin"));
        }
    }
}
