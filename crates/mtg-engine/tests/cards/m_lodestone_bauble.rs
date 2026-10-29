//! Lodestone Bauble (hand-written, `src/cards/lodestone_bauble.rs`): "{1}, {T}, Sacrifice
//! this artifact: Put up to four target basic land cards from a player's graveyard on top
//! of their library in any order. That player draws a card at the beginning of the next
//! turn's upkeep."

use mtg_engine::decision::Decision;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

const BAUBLE: &str = "Lodestone Bauble";

/// The players P0 was offered for the choice, and the target candidates offered.
fn offered(t: &TestGame) -> (Vec<Entity>, Vec<Entity>) {
    let mut players = vec![];
    let mut targets = vec![];
    for (p, d) in t.asked() {
        match d {
            Decision::ChooseEntities {
                prompt, candidates, ..
            } if p == P0 && prompt.starts_with("Choose a player") => players.extend(candidates),
            Decision::ChooseTargets { candidates, .. } if p == P0 => targets.extend(candidates),
            _ => {}
        }
    }
    (players, targets)
}

/// P0 in their main phase with Lodestone Bauble and a land for its cost.
fn setup() -> (TestGame, ObjectId) {
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    t.lands(P0, "Plains", 1);
    let b = t.battlefield(P0, BAUBLE);
    (t, b)
}

#[test]
fn lodestone_bauble_compiles() {
    let def = mtg_engine::card::card(BAUBLE);
    assert!(
        def.unsupported_text().is_empty(),
        "{:?}",
        def.unsupported_text()
    );
}

#[test]
fn the_chosen_players_basic_lands_go_on_top_and_they_draw_next_upkeep() {
    interp!("player-chosen-before-targets");
    cr!("601.2c", "602.2b", "115.10");
    let (mut t, b) = setup();
    let f1 = t.graveyard(P1, "Forest");
    let f2 = t.graveyard(P1, "Island");
    let nonbasic = t.graveyard(P1, "Mutavault");
    let mine = t.graveyard(P0, "Plains");
    // P0 chooses P1, then targets P1's two basic land cards.
    t.answer_choose(P0, &[Entity::Player(P1)]);
    t.answer_targets(P0, &[Entity::Object(f1), Entity::Object(f2)]);
    t.activate(P0, b, 0, &[]).unwrap();
    let (players, targets) = offered(&t);
    assert_eq!(players.len(), 2, "any player: {players:?}");
    assert!(targets.contains(&Entity::Object(f1)) && targets.contains(&Entity::Object(f2)));
    assert!(
        !targets.contains(&Entity::Object(nonbasic)) && !targets.contains(&Entity::Object(mine)),
        "only basic land cards in the chosen player's graveyard: {targets:?}"
    );
    t.resolve_all();
    assert!(t.in_graveyard(P0, BAUBLE));
    let lib = &t.g.player(P1).library;
    let top2: Vec<&str> = lib[lib.len() - 2..]
        .iter()
        .map(|c| t.obj_now(*c).chars.name.as_str())
        .collect();
    assert!(
        top2.contains(&"Forest") && top2.contains(&"Island"),
        "{top2:?}"
    );
    assert_eq!(t.zone(nonbasic), Zone::Graveyard(P1));
    // "That player draws a card at the beginning of the next turn's upkeep."
    let hand = t.hand_size(P1);
    t.advance_to(P1, Step::Upkeep);
    t.resolve_all();
    assert_eq!(t.hand_size(P1), hand + 1);
    assert!(t.in_hand(P1, "Forest") || t.in_hand(P1, "Island"));
    assert_eq!(t.hand_size(P0), 0, "not the controller");
}

#[test]
fn with_zero_targets_a_player_is_still_chosen_and_draws() {
    cr!("601.2c");
    ruling!(
        "Lodestone Bauble",
        "If you choose zero targets, you still choose a player. That player will draw a card at the beginning of the next turn’s upkeep."
    );
    let (mut t, b) = setup();
    t.library_top(P1, "Grizzly Bears");
    t.answer_choose(P0, &[Entity::Player(P1)]);
    t.answer_targets(P0, &[]);
    t.activate(P0, b, 0, &[]).unwrap();
    t.resolve_all();
    t.advance_to(P1, Step::Upkeep);
    t.resolve_all();
    assert!(t.in_hand(P1, "Grizzly Bears"));
}

#[test]
fn the_player_is_chosen_not_targeted() {
    interp!("player-chosen-before-targets");
    cr!("115.10", "702.11c");
    // P1 has hexproof (Leyline of Sanctity): still a legal choice, and the cards in
    // their graveyard can be targeted.
    let (mut t, b) = setup();
    t.battlefield(P1, "Leyline of Sanctity");
    let f = t.graveyard(P1, "Swamp");
    t.answer_choose(P0, &[Entity::Player(P1)]);
    t.answer_targets(P0, &[Entity::Object(f)]);
    t.activate(P0, b, 0, &[]).unwrap();
    let (players, _) = offered(&t);
    assert!(players.contains(&Entity::Player(P1)));
    t.resolve_all();
    let top = *t.g.player(P1).library.last().unwrap();
    assert_eq!(t.obj_now(top).chars.name, "Swamp");
}

#[test]
fn choosing_yourself_targets_your_own_graveyard() {
    cr!("601.2c");
    let (mut t, b) = setup();
    let mine = t.graveyard(P0, "Mountain");
    let theirs = t.graveyard(P1, "Mountain");
    t.answer_choose(P0, &[Entity::Player(P0)]);
    t.answer_targets(P0, &[Entity::Object(mine)]);
    t.activate(P0, b, 0, &[]).unwrap();
    let (_, targets) = offered(&t);
    assert!(targets.contains(&Entity::Object(mine)));
    assert!(!targets.contains(&Entity::Object(theirs)));
    t.resolve_all();
    let top = *t.g.player(P0).library.last().unwrap();
    assert_eq!(t.obj_now(top).chars.name, "Mountain");
    // The next turn is P1's: P0 draws at the beginning of that upkeep.
    t.advance_to(P1, Step::Upkeep);
    t.resolve_all();
    assert!(t.in_hand(P0, "Mountain"));
}
