//! Rulings batch S26 — who owns a token: the player who created it, or for a resolving
//! copy of a permanent spell the player who controlled that spell (CR 111.2, 108.3),
//! with Staff of Eden, Vault's Key ("each permanent you control but don't own").

use crate::r_s01_common::supported;
use crate::r_s02_common::create_token;
use crate::r_s06_common::give_control;
use mtg_engine::decision::Decision;
use mtg_engine::testing::*;
use mtg_engine::*;

#[test]
fn staff_of_eden_counts_tokens_created_by_other_players() {
    cr!("111.2", "108.3");
    ruling!(
        "Staff of Eden, Vault's Key",
        "The owner of a token is the player who created that token or, in the case of a resolving copy of a permanent spell that became a token, the player who controlled that spell as it resolved."
    );
    supported("Staff of Eden, Vault's Key");
    // "{T}: Draw a card for each permanent you control but don't own."
    let mut t = TestGame::new(2);
    let staff = t.battlefield(P0, "Staff of Eden, Vault's Key");
    // P0's own token and card: P0 owns them.
    create_token(&mut t, P0, "Rat");
    t.battlefield(P0, "Grizzly Bears");
    // A token P1 created and a card P1 owns, both controlled by P0.
    let theirs = create_token(&mut t, P1, "Goblin");
    give_control(&mut t, theirs, P0);
    let giant = t.battlefield(P1, "Hill Giant");
    give_control(&mut t, giant, P0);
    assert_eq!(t.obj_now(theirs).owner, P1);
    let hand = t.hand_size(P0);
    t.activate(P0, staff, 0, &[]).expect("activate");
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand + 2);
}

#[test]
fn staff_of_eden_cant_return_a_card_named_staff_of_eden() {
    cr!("201.2", "115.1");
    supported("Staff of Eden, Vault's Key");
    // "When ~ enters, put target legendary permanent card not named Staff of Eden, Vault's
    // Key from a graveyard onto the battlefield under your control."
    let mut t = TestGame::new(2);
    let kiki = t.graveyard(P1, "Kiki-Jiki, Mirror Breaker");
    let other = t.graveyard(P1, "Staff of Eden, Vault's Key");
    t.answer_targets(P0, &[Entity::Object(kiki)]);
    t.enter(P0, "Staff of Eden, Vault's Key");
    t.resolve_all();
    let cands: Vec<Vec<Entity>> = t
        .asked()
        .into_iter()
        .filter_map(|(_, d)| match d {
            Decision::ChooseTargets { candidates, .. } => Some(candidates),
            _ => None,
        })
        .collect();
    assert_eq!(cands.len(), 1);
    assert!(cands[0].contains(&Entity::Object(kiki)));
    assert!(!cands[0].contains(&Entity::Object(other)));
    // Kiki-Jiki is on the battlefield under P0's control, still owned by P1.
    let kiki = t.g.current(kiki);
    assert!(t.on_battlefield(kiki));
    assert_eq!(t.obj(kiki).controller, P0);
    assert_eq!(t.obj(kiki).owner, P1);
}
