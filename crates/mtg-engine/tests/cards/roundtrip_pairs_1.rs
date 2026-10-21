//! roundtrip-pairs-1 (docs/agents/items/roundtrip-pairs-1.md): the last 3+-card round-trip
//! clusters and the first 45 two-card clusters.

use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

#[test]
fn deceiver_of_form_doesnt_become_a_copy_itself() {
    cr!("707.2", "608.2c");
    // "If a creature card is revealed this way, you may have creatures you control other
    // than Deceiver of Form become copies of that card until end of turn."
    let mut t = TestGame::new(2);
    let deceiver = t.battlefield(P0, "Deceiver of Form");
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.library_top(P0, "Craw Wurm");
    t.answer_yes(P0, true);
    t.answer_yes(P0, false);
    t.advance_to(P0, Step::BeginningOfCombat);
    t.resolve_all();
    t.g.recompute();
    assert_eq!(t.obj_now(bears).chars.name, "Craw Wurm", "{}", t.dump_log());
    assert_eq!(t.obj_now(deceiver).chars.name, "Deceiver of Form");
}

#[test]
fn grothama_counts_damage_a_spell_dealt_to_it() {
    cr!("120.1", "603.10a");
    // "When Grothama leaves the battlefield, each player draws cards equal to the amount
    // of damage dealt to Grothama this turn by sources they controlled." A spell is a
    // source too (CR 120.1).
    let mut t = TestGame::new(2);
    let grothama = t.battlefield(P0, "Grothama, All-Devouring");
    t.lands(P1, "Mountain", 1);
    let bolt = t.hand(P1, "Lightning Bolt");
    t.cast(P1, bolt).target(Entity::Object(grothama)).go();
    t.resolve_all();
    assert_eq!(t.obj_now(grothama).damage, 3);
    let (h0, h1) = (t.hand_size(P0), t.hand_size(P1));
    t.g.destroy(grothama, None);
    t.resolve_all();
    assert_eq!(t.hand_size(P1), h1 + 3, "{}", t.dump_log());
    assert_eq!(t.hand_size(P0), h0);
}

fn treasures(t: &TestGame) -> usize {
    t.g.permanents()
        .filter(|o| o.chars.subtypes.iter().any(|s| s.as_str() == "Treasure"))
        .count()
}

#[test]
fn smashing_success_makes_no_treasure_if_the_artifact_isnt_destroyed() {
    cr!("608.2c", "702.12b");
    // "Destroy target artifact or land. If an artifact is destroyed this way, create a
    // Treasure token." An indestructible artifact isn't destroyed.
    let mut t = TestGame::new(2);
    let citadel = t.battlefield(P1, "Darksteel Citadel");
    t.lands(P0, "Mountain", 4);
    let spell = t.hand(P0, "Smashing Success");
    t.cast(P0, spell).target(Entity::Object(citadel)).go();
    t.resolve_all();
    assert!(t.on_battlefield(citadel));
    assert_eq!(treasures(&t), 0, "{}", t.dump_log());
    // An artifact that is destroyed does make one.
    let bauble = t.battlefield(P1, "Mishra's Bauble");
    t.lands(P0, "Mountain", 4);
    let spell = t.hand(P0, "Smashing Success");
    t.cast(P0, spell).target(Entity::Object(bauble)).go();
    t.resolve_all();
    assert_eq!(treasures(&t), 1, "{}", t.dump_log());
}

#[test]
fn elven_farsight_draws_only_if_the_card_was_revealed() {
    cr!("608.2c", "701.20a");
    // "Scry 3, then you may reveal the top card of your library. If a creature card is
    // revealed this way, draw a card."
    for reveal in [false, true] {
        let mut t = TestGame::new(2);
        t.library_top(P0, "Grizzly Bears");
        t.lands(P0, "Forest", 1);
        let spell = t.hand(P0, "Elven Farsight");
        t.cast(P0, spell).go();
        let hand = t.hand_size(P0);
        t.answer_yes(P0, reveal);
        t.resolve_all();
        let drew = if reveal { 1 } else { 0 };
        assert_eq!(t.hand_size(P0), hand + drew, "reveal {reveal}: {}", t.dump_log());
    }
}

#[test]
fn ravenous_slime_gets_counters_equal_to_the_dying_creatures_power() {
    cr!("614.1a", "608.2h");
    // "If a creature an opponent controls would die, instead exile it and put a number of
    // +1/+1 counters equal to that creature's power on Ravenous Slime."
    let mut t = TestGame::new(2);
    let slime = t.battlefield(P0, "Ravenous Slime");
    let wurm = t.battlefield(P1, "Craw Wurm");
    t.g.destroy(wurm, None);
    t.resolve_all();
    assert_eq!(t.zone(wurm), Zone::Exile);
    assert_eq!(t.counters(slime, "+1/+1"), 6, "{}", t.dump_log());
}

#[test]
fn kindle_the_carnage_can_repeat_more_than_once() {
    cr!("608.2c", "701.9a");
    // "Discard a card at random. If you do, Kindle the Carnage deals damage equal to that
    // card's mana value to each creature. You may repeat this process any number of
    // times."
    let mut t = TestGame::new(2);
    let dreadmaw = t.battlefield(P1, "Colossal Dreadmaw");
    t.hand(P0, "Grizzly Bears");
    t.hand(P0, "Hill Giant");
    t.lands(P0, "Mountain", 3);
    let spell = t.hand(P0, "Kindle the Carnage");
    t.cast(P0, spell).go();
    t.answer_yes(P0, true);
    t.answer_yes(P0, true);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), 0);
    // 2 + 4 damage: the 6/6 dies.
    assert!(!t.on_battlefield(dreadmaw), "{}", t.dump_log());
}
