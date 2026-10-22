//! In-game tests for compiler misreads found by the Oracle round trip (follow-up item
//! `roundtrip-tail-1`, cards A–D): each shows the behavior the corrected compilation has
//! and the old one didn't.

use mtg_engine::card::card;
use mtg_engine::testing::*;
use mtg_engine::object::Zone;
use mtg_engine::*;

fn supported(name: &str) {
    let c = card(name);
    assert!(
        c.unsupported_text().is_empty(),
        "{name}: {:?}",
        c.unsupported_text()
    );
}

#[test]
fn two_life_conditions_joined_by_and_are_both_checked() {
    cr!("611.3a");
    // Blood Baron of Vizkopa: "As long as you have 30 or more life and an opponent has 10
    // or less life, this creature gets +6/+6 and has flying." It was read as one condition
    // "you have 30 or less life", so the bonus applied at 20 life.
    supported("Blood Baron of Vizkopa");
    let mut t = TestGame::new(2);
    let baron = t.battlefield(P0, "Blood Baron of Vizkopa");
    t.settle();
    assert_eq!(t.pt(baron), (4, 4));
    t.g.player_mut(P0).life = 30;
    t.g.player_mut(P1).life = 10;
    t.g.dirty = true;
    t.settle();
    assert_eq!(t.pt(baron), (10, 10));
    t.g.player_mut(P0).life = 29;
    t.g.dirty = true;
    t.settle();
    assert_eq!(t.pt(baron), (4, 4));
}

#[test]
fn it_after_choosing_a_target_is_the_target() {
    cr!("701.60a", "608.2c");
    // Agrus Kos, Spirit of Justice: "choose up to one target creature. If it's suspected,
    // exile it. Otherwise, suspect it." "It" is the target (it was read as Agrus Kos
    // itself, so a suspected target was never exiled).
    supported("Agrus Kos, Spirit of Justice");
    let mut t = TestGame::new(2);
    let suspect_bears = t.battlefield(P1, "Grizzly Bears");
    assert!(mtg_engine::kwa::suspect_detain::suspect(&mut t.g, suspect_bears));
    t.settle();
    t.answer_targets(P0, &[Entity::Object(suspect_bears)]);
    t.enter(P0, "Agrus Kos, Spirit of Justice");
    t.settle();
    t.resolve_all();
    assert_eq!(t.zone(suspect_bears), Zone::Exile);
    // An unsuspected target is suspected, not exiled.
    let other = t.battlefield(P1, "Grizzly Bears");
    t.answer_targets(P0, &[Entity::Object(other)]);
    t.enter(P0, "Agrus Kos, Spirit of Justice");
    t.settle();
    t.resolve_all();
    assert!(t.on_battlefield(other));
    assert!(mtg_engine::kwa::suspect_detain::is_suspected(&t.g, other));
}

#[test]
fn mana_or_mana_for_each_counts() {
    cr!("106.1", "608.2d");
    // Culling Ritual: "Add {B} or {G} for each permanent destroyed this way." The count
    // was dropped, so it added one mana however many permanents were destroyed.
    supported("Culling Ritual");
    let mut t = TestGame::new(2);
    t.set_step(P0, mtg_engine::turn::Step::PrecombatMain);
    t.battlefield(P1, "Grizzly Bears");
    t.battlefield(P1, "Grizzly Bears");
    t.battlefield(P0, "Grizzly Bears");
    t.lands(P0, "Swamp", 2);
    t.lands(P0, "Forest", 2);
    let spell = t.hand(P0, "Culling Ritual");
    t.cast(P0, spell).go();
    t.resolve();
    assert!(t.named_on_battlefield("Grizzly Bears").is_empty());
    assert_eq!(t.g.player(P0).mana_pool.total(), 3);
}

#[test]
fn x_mana_of_one_type_or_x_of_another() {
    cr!("106.1", "107.3");
    // Brigid, Doun's Mind: "{T}: Add X {G} or X {W}, where X is the number of other
    // creatures you control." It added one mana.
    supported("Brigid, Clachan's Heart // Brigid, Doun's Mind");
    let mut t = TestGame::new(2);
    t.set_step(P0, mtg_engine::turn::Step::PrecombatMain);
    let brigid = t.battlefield(P0, "Brigid, Clachan's Heart // Brigid, Doun's Mind");
    assert!(mtg_engine::dfc::transform(&mut t.g, brigid));
    let brigid = t.g.current(brigid);
    t.battlefield(P0, "Grizzly Bears");
    t.battlefield(P0, "Grizzly Bears");
    t.battlefield(P0, "Grizzly Bears");
    t.activate(P0, brigid, 0, &[]).unwrap();
    t.resolve_all();
    assert_eq!(t.g.player(P0).mana_pool.total(), 3);
}

#[test]
fn what_a_destroyed_creature_had_is_its_last_known_information() {
    cr!("608.2h");
    // Rite of the Serpent: "Destroy target creature. If that creature had a +1/+1 counter
    // on it, create a 1/1 green Snake creature token." The round trip reads "had" as "has"
    // of the destroyed creature's last known information; this shows the engine does.
    supported("Rite of the Serpent");
    for (counters, snakes) in [(1, 1), (0, 0)] {
        let mut t = TestGame::new(2);
        t.set_step(P0, mtg_engine::turn::Step::PrecombatMain);
        let target = t.battlefield(P1, "Hill Giant");
        if counters > 0 {
            t.g.add_counters(
                Entity::Object(target),
                mtg_engine::types::counters::PLUS1,
                counters,
                None,
            );
            t.g.flush_events();
            t.settle();
        }
        t.lands(P0, "Swamp", 6);
        let spell = t.hand(P0, "Rite of the Serpent");
        t.answer_targets(P0, &[Entity::Object(target)]);
        t.cast(P0, spell).go();
        t.resolve_all();
        assert_eq!(t.zone(target), Zone::Graveyard(P1));
        let made = t
            .g
            .permanents()
            .filter(|o| o.is_token() && o.chars.has_subtype("Snake"))
            .count();
        assert_eq!(made, snakes, "{counters} counter(s)");
    }
}
