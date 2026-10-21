//! Player phrases compared across players: "each player with fewer cards in hand than the
//! player with the most cards in hand ... the difference", "each player who controls
//! fewer lands than the player who controls the most lands ...", "the player with the
//! lowest life total" with ties broken by a choice.

use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

fn supported(name: &str) {
    let def = card(name);
    assert!(
        def.unsupported_text().is_empty(),
        "{name} has unsupported text: {:?}",
        def.unsupported_text()
    );
}

#[test]
fn tales_of_the_ancestors_each_player_draws_up_to_the_most_cards_in_hand() {
    cr!("608.2h");
    supported("Tales of the Ancestors");
    let mut t = TestGame::new(3);
    t.lands(P0, "Island", 4);
    let spell = t.hand(P0, "Tales of the Ancestors");
    // After the spell leaves P0's hand: P0 has 1 card, P1 has 4, P2 has 2.
    t.hand(P0, "Grizzly Bears");
    for _ in 0..4 {
        t.hand(P1, "Grizzly Bears");
    }
    for _ in 0..2 {
        t.hand(P2, "Grizzly Bears");
    }
    t.cast(P0, spell).go();
    t.resolve();
    assert_eq!(
        (t.hand_size(P0), t.hand_size(P1), t.hand_size(P2)),
        (4, 4, 4)
    );
}

#[test]
fn scholarship_sponsor_players_search_for_the_difference_in_lands() {
    cr!("608.2h");
    ruling!(
        "Scholarship Sponsor",
        "the first opponent will search their library for one basic land card"
    );
    supported("Scholarship Sponsor");
    let mut t = TestGame::new(3);
    // P0 controls four lands, P1 six, P2 seven.
    t.lands(P0, "Plains", 4);
    t.lands(P1, "Forest", 6);
    t.lands(P2, "Mountain", 7);
    let p0_lands: Vec<_> = (0..3).map(|_| t.library_top(P0, "Plains")).collect();
    let p1_land = t.library_top(P1, "Forest");
    t.library_top(P2, "Mountain");
    let sponsor = t.hand(P0, "Scholarship Sponsor");
    t.answer_choose(
        P0,
        &p0_lands
            .iter()
            .map(|o| Entity::Object(*o))
            .collect::<Vec<_>>(),
    );
    t.answer_choose(P1, &[Entity::Object(p1_land)]);
    t.cast(P0, sponsor).go();
    t.resolve_all();
    let lands = |t: &TestGame, p: PlayerId| {
        t.g.battlefield
            .iter()
            .filter(|o| {
                let o = t.obj_now(**o);
                o.controller == p && o.chars.card_types.contains(types::CardType::Land)
            })
            .count()
    };
    assert_eq!((lands(&t, P0), lands(&t, P1), lands(&t, P2)), (7, 7, 7));
}

#[test]
fn loxodon_peacekeeper_goes_to_the_player_with_the_lowest_life_total() {
    cr!("101.4");
    supported("Loxodon Peacekeeper");
    let mut t = TestGame::new(3);
    let ele = t.battlefield(P0, "Loxodon Peacekeeper");
    t.g.player_mut(P2).life = 12;
    t.advance_to(P0, Step::Upkeep);
    t.resolve_all();
    assert_eq!(t.obj_now(ele).controller, P2);
}

#[test]
fn loxodon_peacekeeper_tie_is_broken_by_its_controller() {
    cr!("101.4");
    supported("Loxodon Peacekeeper");
    let mut t = TestGame::new(3);
    let ele = t.battlefield(P0, "Loxodon Peacekeeper");
    t.g.player_mut(P1).life = 12;
    t.g.player_mut(P2).life = 12;
    t.answer_choose(P0, &[Entity::Player(P2)]);
    t.advance_to(P0, Step::Upkeep);
    t.resolve_all();
    assert_eq!(t.obj_now(ele).controller, P2);
    assert!(t
        .asked()
        .iter()
        .any(|(p, d)| *p == P0 && matches!(d, decision::Decision::ChooseEntities { candidates, .. } if candidates.len() == 2)));
}

#[test]
fn curse_of_surveillance_other_target_players_draw_per_curse() {
    cr!("115.1", "303.4");
    supported("Curse of Surveillance");
    let mut t = TestGame::new(3);
    let c1 = t.battlefield(P0, "Curse of Surveillance");
    let c2 = t.battlefield(P0, "Curse of Death's Hold");
    t.g.attach(c1, Entity::Player(P1));
    t.g.attach(c2, Entity::Player(P1));
    // P0 targets P0 and P2 (not P1, the enchanted player).
    t.answer_targets(P0, &[Entity::Player(P0), Entity::Player(P2)]);
    t.advance_to(P1, Step::Upkeep);
    let hands = (t.hand_size(P0), t.hand_size(P1), t.hand_size(P2));
    t.resolve_all();
    assert_eq!(
        (t.hand_size(P0), t.hand_size(P1), t.hand_size(P2)),
        (hands.0 + 2, hands.1, hands.2 + 2)
    );
    // P1 wasn't a legal target.
    let cands: Vec<Vec<Entity>> = t
        .asked()
        .into_iter()
        .filter_map(|(_, d)| match d {
            decision::Decision::ChooseTargets { candidates, .. } => Some(candidates),
            _ => None,
        })
        .collect();
    assert!(!cands.is_empty());
    assert!(cands.iter().all(|c| !c.contains(&Entity::Player(P1))));
}
