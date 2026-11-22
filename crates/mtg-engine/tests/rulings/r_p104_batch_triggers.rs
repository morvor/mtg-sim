//! Rulings batch P104 — triggers on cards being milled, put into or leaving a graveyard:
//! several cards moving at once trigger "one or more" abilities once (Sidisi, Devourer of
//! Memory, Quintorius, Teval, Advanced Reconstruction, Crawling Sensation), "from
//! anywhere" triggers need their source on the battlefield afterward (Skola Grovedancer,
//! Crawling Infestation), and triggers during a resolution wait for it (Countryside
//! Crusher).

use crate::r_p104_common::*;
use crate::r_p146_common::{destroy_together, exile_together};
use crate::r_s01_common::*;
use crate::r_s02_common::destroy;
use crate::r_s05_common::enter;
use crate::r_s19_common::gain_level;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

fn zombies(t: &TestGame) -> usize {
    with_subtype(t, P0, "Zombie")
        .into_iter()
        .filter(|id| t.g.obj(*id).is_token())
        .count()
}

#[test]
fn sidisi_milling_three_creature_cards_makes_one_zombie() {
    cr!("603.2c", "701.17a");
    ruling!(
        "Sidisi, Brood Tyrant",
        "Any instruction, including Sidisi's first ability, that tells you to put multiple cards from a library into a graveyard moves all the cards at the same time. For example, if Sidisi enters the battlefield and you put three creature cards into your graveyard, Sidisi's last ability will trigger once and you'll put one Zombie token onto the battlefield."
    );
    supported("Sidisi, Brood Tyrant");
    let mut t = TestGame::new(2);
    stack_library(&mut t, P0, &["Grizzly Bears", "Hill Giant", "Llanowar Elves"]);
    enter(&mut t, P0, "Sidisi, Brood Tyrant");
    t.resolve();
    assert_eq!(t.stack_len(), 1);
    t.resolve_all();
    assert_eq!(zombies(&t), 1);
}

#[test]
fn devourer_of_memory_gets_one_bonus_per_batch() {
    cr!("603.2c", "701.17a");
    ruling!(
        "Devourer of Memory",
        "Devourer of Memory’s triggered ability grants it just +1/+1 each time cards are put into your graveyard from your library, no matter how many cards are put there at once."
    );
    supported("Devourer of Memory");
    let mut t = TestGame::new(2);
    let d = t.battlefield(P0, "Devourer of Memory");
    let base = t.pt(d);
    mill(&mut t, P0, 3);
    t.resolve_all();
    assert_eq!(t.pt(d), (base.0 + 1, base.1 + 1));
    mill(&mut t, P0, 1);
    t.resolve_all();
    assert_eq!(t.pt(d), (base.0 + 2, base.1 + 2));
}

/// Three cards leave P0's graveyard at once with `name` on the battlefield (with `setup`
/// run first); returns how many abilities triggered.
fn leave_at_once(t: &mut TestGame) -> usize {
    let cards: Vec<ObjectId> = (0..3).map(|_| t.graveyard(P0, "Grizzly Bears")).collect();
    exile_together(t, &cards);
    t.stack_len()
}

#[test]
fn cards_leaving_a_graveyard_at_once_trigger_once() {
    cr!("603.2c");
    ruling!(
        "Quintorius, History Chaser",
        "If multiple cards leave your graveyard at the same time, Quintorius's first ability will trigger only once."
    );
    ruling!(
        "Teval, the Balanced Scale",
        "If multiple cards leave your graveyard at the same time, Teval’s last ability will trigger only once."
    );
    ruling!(
        "Advanced Reconstruction",
        "If multiple cards leave your graveyard at the same time, Advanced Reconstruction's level 2 ability will trigger only once."
    );
    for name in ["Quintorius, History Chaser", "Teval, the Balanced Scale"] {
        supported(name);
        let mut t = TestGame::new(2);
        t.battlefield(P0, name);
        assert_eq!(leave_at_once(&mut t), 1, "{name}");
        t.resolve_all();
        assert_eq!(tokens(&t, P0).len(), 1, "{name}");
    }
    supported("Advanced Reconstruction");
    let mut t = TestGame::new(2);
    let class = t.battlefield(P0, "Advanced Reconstruction");
    t.lands(P0, "Mountain", 2);
    gain_level(&mut t, P0, class, 2).unwrap();
    t.resolve_all();
    assert_eq!(leave_at_once(&mut t), 1);
    t.resolve_all();
    assert_eq!(t.life(P1), 18);
}

#[test]
fn crawling_sensation_triggers_once_for_lands_milled_or_destroyed_together() {
    cr!("603.2c");
    ruling!(
        "Crawling Sensation",
        "If multiple land cards are put into your graveyard at once, Crawling Sensation's last ability triggers only once. This could happen because an effect (such as that of Crawling Sensation's first ability) put them there from your library at once, or because they were destroyed at the same time (such as two land creatures that were dealt lethal combat damage)."
    );
    supported("Crawling Sensation");
    // Milled together.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Crawling Sensation");
    stack_library(&mut t, P0, &["Forest", "Island"]);
    mill(&mut t, P0, 2);
    assert_eq!(t.stack_len(), 1);
    t.resolve_all();
    assert_eq!(tokens(&t, P0).len(), 1);
    // Destroyed together.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Crawling Sensation");
    let a = t.battlefield(P0, "Forest");
    let b = t.battlefield(P0, "Island");
    destroy_together(&mut t, &[a, b]);
    assert_eq!(t.stack_len(), 1);
    t.resolve_all();
    assert_eq!(tokens(&t, P0).len(), 1);
}

#[test]
fn skola_grovedancer_dying_with_a_land_doesnt_trigger() {
    cr!("603.2", "603.10");
    ruling!(
        "Skola Grovedancer",
        "Because Skola Grovedancer's ability triggers when land cards are put into your graveyard \"from anywhere,\" Skola Grovedancer must be on the battlefield immediately after a land card is put there in order to trigger. This means that if Skola Grovedancer dies at the same time as a land you control, its ability won't trigger."
    );
    supported("Skola Grovedancer");
    let mut t = TestGame::new(2);
    let s = t.battlefield(P0, "Skola Grovedancer");
    let land = t.battlefield(P0, "Forest");
    destroy_together(&mut t, &[s, land]);
    assert_eq!(t.stack_len(), 0);
    // Alive, it triggers.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Skola Grovedancer");
    let land = t.battlefield(P0, "Forest");
    destroy(&mut t, land);
    t.resolve_all();
    assert_eq!(t.life(P0), 21);
}

#[test]
fn crawling_infestation_destroyed_with_creatures_doesnt_trigger() {
    cr!("603.2", "603.10");
    ruling!(
        "Crawling Infestation",
        "Crawling Infestation must be on the battlefield for its ability to trigger. If it's destroyed at the same time as one or more creatures you own, its ability won't trigger."
    );
    supported("Crawling Infestation");
    let mut t = TestGame::new(2);
    let ci = t.battlefield(P0, "Crawling Infestation");
    let bears = t.battlefield(P0, "Grizzly Bears");
    destroy_together(&mut t, &[ci, bears]);
    assert_eq!(t.stack_len(), 0);
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Crawling Infestation");
    let bears = t.battlefield(P0, "Grizzly Bears");
    destroy(&mut t, bears);
    t.resolve_all();
    assert_eq!(tokens(&t, P0).len(), 1);
}

#[test]
fn countryside_crusher_triggers_wait_for_the_reveal_loop() {
    cr!("603.3", "608.2");
    ruling!(
        "Countryside Crusher",
        "Each time you put a land card into your graveyard with the first ability, the second ability will trigger. Those triggers wait to be put onto the stack until the first ability has completely finished resolving."
    );
    supported("Countryside Crusher");
    let mut t = TestGame::new(2);
    let c = t.battlefield(P0, "Countryside Crusher");
    t.advance_to(P1, Step::Upkeep);
    stack_library(&mut t, P0, &["Forest", "Island", "Grizzly Bears"]);
    t.advance_to(P0, Step::Upkeep);
    t.settle();
    assert_eq!(t.stack_len(), 1);
    t.resolve();
    // Both lands were milled, then both triggers were put on the stack.
    assert!(t.in_graveyard(P0, "Forest") && t.in_graveyard(P0, "Island"));
    assert_eq!(t.stack_len(), 2);
    assert_eq!(t.counters(c, "+1/+1"), 0);
    t.resolve_all();
    assert_eq!(t.counters(c, "+1/+1"), 2);
}
