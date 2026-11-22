//! cards4-referents-statics, family `combat-history`.

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

/// Faramir's delayed ability at P1's next end step, after P1 attacked P0 (`attack`) or not.
fn faramir(attack: bool) -> (TestGame, usize) {
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Faramir, Prince of Ithilien");
    let bears = t.battlefield(P1, "Grizzly Bears");
    for _ in 0..5 {
        t.library_top(P0, "Island");
        t.library_top(P1, "Island");
    }
    // P0's end step: choose P1.
    t.advance_to(P0, Step::End);
    t.resolve_all();
    let hand = t.hand_size(P0);
    if attack {
        t.advance_to(P1, Step::BeginningOfCombat);
        t.attack(&[(bears, Entity::Player(P0))], &[]);
    }
    t.advance_to(P1, Step::End);
    t.resolve_all();
    (t, hand)
}

#[test]
fn faramir_draws_if_the_chosen_opponent_didnt_attack_you() {
    cr!("603.7", "508.1");
    supported("Faramir, Prince of Ithilien");
    let (t, hand) = faramir(false);
    assert_eq!(t.hand_size(P0), hand + 1, "{}", t.dump_log());
}

#[test]
fn faramir_creates_soldiers_if_the_chosen_opponent_attacked_you() {
    cr!("603.7", "508.1");
    let (t, hand) = faramir(true);
    assert_eq!(t.hand_size(P0), hand, "{}", t.dump_log());
    let soldiers = t
        .g
        .battlefield
        .iter()
        .filter(|o| t.g.obj(**o).controller == P0 && t.g.obj(**o).chars.has_subtype("Soldier"))
        .count();
    assert_eq!(soldiers, 3);
}

#[test]
fn godsend_exiles_one_of_the_creatures_blocking_it() {
    cr!("509.1h", "506.4");
    ruling!("Godsend", "You choose which creature to exile when the ability resolves.");
    supported("Godsend");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let godsend = t.battlefield(P0, "Godsend");
    t.g.attach(godsend, Entity::Object(bears));
    let a = t.battlefield(P1, "Llanowar Elves");
    let b = t.battlefield(P1, "Llanowar Elves");
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[Entity::Object(b)]);
    t.attack(&[(bears, Entity::Player(P1))], &[(a, bears), (b, bears)]);
    t.resolve_all();
    assert_eq!(t.zone(b), mtg_engine::object::Zone::Exile, "{}", t.dump_log());
    // The other blocker was dealt lethal damage.
    assert!(!t.on_battlefield(a));
}
