//! CR 402–404: the hand, the battlefield, and the graveyard.

use crate::r100_common::pregame;
use crate::r703_common::supported;
use mtg_engine::ability::*;
use mtg_engine::card::CardDef;
use mtg_engine::decision::Decision;
use mtg_engine::eval::Ctx;
use mtg_engine::facedown::can_look_at;
use mtg_engine::game::GameConfig;
use mtg_engine::object::{Characteristics, Zone};
use mtg_engine::testing::*;
use mtg_engine::types::*;
use mtg_engine::*;
use smol_str::SmolStr;
use std::sync::Arc;

fn names_of(t: &TestGame, ids: &[ObjectId]) -> Vec<String> {
    ids.iter()
        .map(|id| t.obj(*id).chars.name.to_string())
        .collect()
}

// ---------------------------------------------------------------------------
// 402: hand
// ---------------------------------------------------------------------------

#[test]
fn a_player_may_look_at_their_own_hand_and_count_but_not_look_at_anothers() {
    cr!("402.3");
    supported("Sudden Impact");
    let mut t = TestGame::new(2);
    let mine = t.hand(P0, "Grizzly Bears");
    let theirs: Vec<ObjectId> = ["Hill Giant", "Shock", "Opt"]
        .iter()
        .map(|n| t.hand(P1, n))
        .collect();
    assert!(can_look_at(&t.g, P0, mine));
    assert!(!can_look_at(&t.g, P1, mine));
    for id in &theirs {
        assert!(can_look_at(&t.g, P1, *id));
        assert!(!can_look_at(&t.g, P0, *id));
    }
    // "Sudden Impact deals damage to target player equal to the number of cards in that
    // player's hand": another player's hand can be counted.
    t.lands(P0, "Mountain", 4);
    let impact = t.hand(P0, "Sudden Impact");
    t.cast(P0, impact).target(P1).go();
    t.resolve();
    assert_eq!(t.life(P1), 17);
}

// ---------------------------------------------------------------------------
// 403: battlefield
// ---------------------------------------------------------------------------

fn distinct(prefix: &str, n: usize) -> Vec<Arc<CardDef>> {
    (0..n)
        .map(|i| {
            Arc::new(CardDef::custom(Characteristics {
                name: SmolStr::new(format!("{prefix} {i}")),
                rules_text: Arc::from(""),
                ..Default::default()
            }))
        })
        .collect()
}

#[test]
fn the_battlefield_starts_out_empty_and_permanents_are_controlled_by_players() {
    cr!("403.1");
    let mut t = pregame(
        GameConfig::default(),
        vec![distinct("A", 20), distinct("B", 20)],
    );
    assert!(t.g.battlefield.is_empty());
    t.g.start();
    assert!(t.g.battlefield.is_empty());
    // Permanents are each controlled by a player.
    let mut t = TestGame::new(2);
    let a = t.battlefield(P0, "Grizzly Bears");
    let b = t.battlefield(P1, "Grizzly Bears");
    assert_eq!(t.obj(a).controller, P0);
    assert_eq!(t.obj(b).controller, P1);
}

#[test]
fn a_spell_affects_and_checks_only_the_battlefield_unless_it_says_otherwise() {
    cr!("403.2");
    supported("Glorious Anthem");
    supported("Wrath of God");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Glorious Anthem");
    let on_bf = t.battlefield(P0, "Grizzly Bears");
    let in_hand = t.hand(P0, "Grizzly Bears");
    let in_gy = t.graveyard(P0, "Grizzly Bears");
    t.g.recompute();
    // "Creatures you control get +1/+1": only the creature on the battlefield.
    assert_eq!(t.pt(on_bf), (3, 3));
    assert_eq!(t.obj(in_hand).chars.power, Some(2));
    assert_eq!(t.obj(in_gy).chars.power, Some(2));
    // "Destroy all creatures": the cards in the hand and graveyard aren't affected.
    t.lands(P0, "Plains", 4);
    let wrath = t.hand(P0, "Wrath of God");
    t.cast(P0, wrath).go();
    t.resolve();
    assert!(!t.on_battlefield(on_bf));
    assert!(t.is_live(in_hand) && t.zone(in_hand) == Zone::Hand(P0));
    assert!(t.is_live(in_gy) && t.zone(in_gy) == Zone::Graveyard(P0));
    // Checking "creatures" counts only those on the battlefield.
    t.battlefield(P1, "Grizzly Bears");
    let n = t
        .g
        .eval_value(&Value::Count(Filter::creature()), &Ctx::new(None, P0));
    assert_eq!(n, 1);
}

#[test]
fn every_object_on_the_battlefield_is_a_permanent_and_only_those_are() {
    cr!("403.3");
    let mut t = TestGame::new(2);
    let land = t.battlefield(P0, "Forest");
    let anthem = t.battlefield(P0, "Glorious Anthem");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let in_hand = t.hand(P0, "Grizzly Bears");
    let in_gy = t.graveyard(P0, "Forest");
    let ctx = Ctx::new(None, P0);
    for id in [land, anthem, bears] {
        assert!(t.g.matches(id, &Filter::Permanent, &ctx));
    }
    for id in [in_hand, in_gy] {
        assert!(!t.g.matches(id, &Filter::Permanent, &ctx));
    }
    // A spell on the stack isn't a permanent until it resolves.
    t.lands(P0, "Forest", 2);
    let spell = t.cast(P0, in_hand).go();
    assert!(!t.g.matches(spell, &Filter::Permanent, &ctx));
    t.resolve();
    let perm = t.g.current(spell);
    assert!(t.g.matches(perm, &Filter::Permanent, &ctx));
}

// ---------------------------------------------------------------------------
// 404: graveyard
// ---------------------------------------------------------------------------

#[test]
fn a_graveyard_is_a_face_up_pile_anyone_can_examine_in_a_fixed_order() {
    cr!("404.2");
    ruling!(
        "Bone Dancer",
        "Players may not rearrange the cards in their graveyards."
    );
    ruling!(
        "Bone Dancer",
        "The “top” card of your graveyard is the card that was put there most recently."
    );
    let mut t = TestGame::new(2);
    let a = t.graveyard(P0, "Grizzly Bears");
    let b = t.graveyard(P0, "Hill Giant");
    for id in [a, b] {
        assert!(can_look_at(&t.g, P0, id) && can_look_at(&t.g, P1, id));
        assert!(!t.obj(id).face_down);
    }
    let bolt = t.hand(P0, "Lightning Bolt");
    t.lands(P0, "Mountain", 1);
    t.cast(P0, bolt).target(P1).go();
    t.resolve();
    // The most recent card is on top; the rest keep their order.
    assert_eq!(
        names_of(&t, &t.g.player(P0).graveyard.clone()),
        ["Grizzly Bears", "Hill Giant", "Lightning Bolt"]
    );
    let ctx = Ctx::new(None, P0);
    let top = t.g.eval_sel(&Sel::TopOfGraveyard(PlayerRef::You), &ctx);
    assert_eq!(top, vec![Entity::Object(t.g.current(bolt))]);
}

#[test]
fn the_owner_arranges_cards_put_into_their_graveyard_at_the_same_time() {
    cr!("404.3", "404.1");
    ruling!(
        "Bone Dancer",
        "If an effect or rule puts two or more cards into the same graveyard at the same time, the owner of those cards may arrange them in any order."
    );
    ruling!(
        "Bone Dancer",
        "Then you put Wrath of God into your graveyard, on top of the other cards."
    );
    for order in [vec![0, 1], vec![1, 0]] {
        let mut t = TestGame::new(2);
        t.battlefield(P0, "Grizzly Bears");
        t.battlefield(P0, "Hill Giant");
        t.battlefield(P1, "Craw Wurm");
        t.lands(P0, "Plains", 4);
        let wrath = t.hand(P0, "Wrath of God");
        t.cast(P0, wrath).go();
        t.answer(P0, DecisionKind::Order, Answer::Indices(order.clone()));
        t.resolve();
        let asked: Vec<PlayerId> = t
            .asked()
            .iter()
            .filter(|(_, d)| matches!(d, Decision::Order { .. }))
            .map(|(p, _)| *p)
            .collect();
        // Only P0 had two cards going to one graveyard.
        assert_eq!(asked, vec![P0]);
        let gy = names_of(&t, &t.g.player(P0).graveyard.clone());
        let expected = if order == [0, 1] {
            ["Grizzly Bears", "Hill Giant", "Wrath of God"]
        } else {
            ["Hill Giant", "Grizzly Bears", "Wrath of God"]
        };
        assert_eq!(gy, expected);
        assert_eq!(names_of(&t, &t.g.player(P1).graveyard.clone()), ["Craw Wurm"]);
    }
}

#[test]
fn an_aura_put_into_the_graveyard_by_a_state_based_action_goes_on_top() {
    cr!("404.1", "404.3", "704.5m");
    ruling!(
        "Bone Dancer",
        "If just the enchanted permanent is destroyed, it’s put into your graveyard first."
    );
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.lands(P0, "Forest", 1);
    let rancor = t.hand(P0, "Rancor");
    t.cast(P0, rancor).target(bears).go();
    t.resolve();
    let asked = t.asked().len();
    t.g.destroy(bears, None);
    t.settle();
    // The destroyed creature went to the graveyard first; the Aura followed later, as a
    // state-based action, on top of it. They didn't go there at the same time, so their
    // owner wasn't asked to arrange them. (Rancor's own trigger, which would return it to
    // its owner's hand, is still on the stack.)
    assert_eq!(t.stack_len(), 1);
    assert_eq!(
        names_of(&t, &t.g.player(P0).graveyard.clone()),
        ["Grizzly Bears", "Rancor"]
    );
    assert!(!t.asked()[asked..]
        .iter()
        .any(|(_, d)| matches!(d, Decision::Order { .. })));
}
