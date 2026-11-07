//! "As ~ enters, exile [cards] from your graveyard." (CR 614.1c, 614.12): cards exiled
//! by the replacement effect are linked to the permanent's abilities that refer to them
//! ("the exiled card", CR 607.2b); and an optional cost paid as it enters ("you may put
//! two cards your opponents own from exile into their owners' graveyards. If you do, ~
//! enters with four +1/+1 counters on it.").

use crate::basic_effects_common::*;
use mtg_engine::events::MoveCause;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::*;

#[test]
fn living_lore_has_the_exiled_cards_mana_value() {
    cr!("614.12", "607.2b");
    assert_supported("Living Lore");
    let mut t = TestGame::new(2);
    t.graveyard(P0, "Lightning Bolt");
    let div = t.graveyard(P0, "Divination");
    t.lands(P0, "Island", 4);
    let ll = t.hand(P0, "Living Lore");
    t.answer_choose(P0, &[Entity::Object(div)]);
    t.cast(P0, ll).go();
    t.resolve();
    let ll = t.named_on_battlefield("Living Lore")[0];
    assert!(t.in_exile("Divination"));
    assert!(t.in_graveyard(P0, "Lightning Bolt"));
    assert_eq!(t.pt(ll), (3, 3));
}

#[test]
fn living_lore_with_nothing_to_exile_is_zero_zero() {
    cr!("614.12", "704.5f");
    ruling!("Living Lore", "it will enter the battlefield as a 0/0 creature");
    let mut t = TestGame::new(2);
    t.lands(P0, "Island", 4);
    let ll = t.hand(P0, "Living Lore");
    t.cast(P0, ll).go();
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Living Lore"));
}

#[test]
fn sutured_ghoul_adds_up_the_exiled_creature_cards() {
    cr!("614.12", "607.2b");
    ruling!("Sutured Ghoul", "You can't have Sutured Ghoul exile itself");
    assert_supported("Sutured Ghoul");
    let mut t = TestGame::new(2);
    let giant = t.graveyard(P0, "Hill Giant");
    let bears = t.graveyard(P0, "Grizzly Bears");
    t.graveyard(P0, "Craw Wurm");
    let ghoul = t.graveyard(P0, "Sutured Ghoul");
    // It enters from the graveyard: it can't exile itself.
    t.answer_choose(P0, &[Entity::Object(giant), Entity::Object(bears)]);
    let g = t
        .g
        .move_object(ghoul, Zone::Battlefield, MoveCause::Effect, Some(P0))
        .unwrap();
    assert!(t.on_battlefield(g));
    assert!(t.in_exile("Hill Giant") && t.in_exile("Grizzly Bears"));
    assert!(t.in_graveyard(P0, "Craw Wurm"));
    assert_eq!(t.pt(g), (5, 5));
    let offered = t.asked().iter().any(|(_, d)| {
        matches!(d, decision::Decision::ChooseEntities { candidates, .. }
            if candidates.contains(&Entity::Object(ghoul)))
    });
    assert!(!offered);
}

#[test]
fn dermotaxi_imprints_a_creature_card_from_any_graveyard() {
    cr!("614.12", "607.2b");
    ruling!("Dermotaxi", "a replacement effect, not a triggered ability");
    assert_supported("Dermotaxi");
    let mut t = TestGame::new(2);
    let giant = t.graveyard(P1, "Hill Giant");
    t.answer_choose(P0, &[Entity::Object(giant)]);
    t.lands(P0, "Plains", 2);
    let taxi = t.hand(P0, "Dermotaxi");
    t.cast(P0, taxi).go();
    t.resolve();
    let taxi = t.named_on_battlefield("Dermotaxi")[0];
    // Exiled as it entered: nothing went on the stack.
    assert!(t.in_exile("Hill Giant"));
    assert_eq!(t.stack_len(), 0);
    t.battlefield(P0, "Grizzly Bears");
    t.battlefield(P0, "Grizzly Bears");
    t.activate(P0, taxi, 0, &[]).unwrap();
    t.resolve();
    assert_eq!(t.pt(taxi), (3, 3));
    assert!(t.g.obj(taxi).chars.has_subtype("Vehicle"));
}

#[test]
fn mimeoplasm_enters_with_counters_for_each_card_exiled() {
    cr!("614.12", "107.3m");
    assert_supported("Mimeoplasm, Revered One");
    let mut t = TestGame::new(2);
    let bears: Vec<ObjectId> = (0..3).map(|_| t.graveyard(P0, "Grizzly Bears")).collect();
    t.answer_choose(P0, &[Entity::Object(bears[0]), Entity::Object(bears[1])]);
    t.lands(P0, "Forest", 3);
    t.lands(P0, "Island", 1);
    t.lands(P0, "Swamp", 1);
    let m = t.hand(P0, "Mimeoplasm, Revered One");
    // X = 2: up to two of the three Bears.
    t.answer(P0, DecisionKind::X, decision::Answer::Number(2));
    t.cast(P0, m).go();
    t.resolve();
    let m = t.named_on_battlefield("Mimeoplasm, Revered One")[0];
    assert_eq!(t.graveyard_size(P0), 1);
    assert_eq!(t.counters(m, "+1/+1"), 6);
}

#[test]
fn ulamogs_despoiler_processes_two_cards_for_counters() {
    cr!("614.12", "118.12");
    ruling!("Ulamog's Despoiler", "you may choose cards owned by different opponents");
    assert_supported("Ulamog's Despoiler");
    let mut t = TestGame::new(3);
    t.exile(P1, "Grizzly Bears");
    t.exile(P2, "Hill Giant");
    t.answer_yes(P0, true);
    let d = t.enter(P0, "Ulamog's Despoiler");
    assert_eq!(t.counters(d, "+1/+1"), 4);
    assert!(t.in_graveyard(P1, "Grizzly Bears"));
    assert!(t.in_graveyard(P2, "Hill Giant"));
    // Only one card: it can't be paid.
    let mut t = TestGame::new(2);
    t.exile(P1, "Grizzly Bears");
    t.answer_yes(P0, true);
    let d = t.enter(P0, "Ulamog's Despoiler");
    assert_eq!(t.counters(d, "+1/+1"), 0);
    assert!(t.in_exile("Grizzly Bears"));
}
