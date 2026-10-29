//! roundtrip-gaps-5 (docs/agents/items/roundtrip-gaps-5.md): the remaining gap clusters
//! (custom nodes) and the next 40 two-card clusters.

use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

#[test]
fn spikeshell_harrier_slows_the_opponent_who_controlled_the_creature() {
    cr!("702.179b", "608.2h");
    // "return target creature or Vehicle an opponent controls to its owner's hand. If that
    // opponent's speed is greater than each other player's speed, reduce that opponent's
    // speed by 1": the opponent who controlled it, even when its owner is another player.
    let mut t = TestGame::new(3);
    t.set_step(P0, Step::PrecombatMain);
    let bears = t.battlefield(P2, "Grizzly Bears");
    t.g.objects[bears.0 as usize].controller = P1;
    t.g.objects[bears.0 as usize].base_controller = P1;
    t.g.players[0].speed = Some(1);
    t.g.players[1].speed = Some(3);
    t.g.players[2].speed = Some(2);
    t.answer_targets(P0, &[Entity::Object(bears)]);
    t.enter(P0, "Spikeshell Harrier");
    t.resolve_all();
    assert!(t.in_hand(P2, "Grizzly Bears"));
    assert_eq!(t.g.players[1].speed, Some(2), "{}", t.dump_log());
    assert_eq!(t.g.players[2].speed, Some(2));
}

#[test]
fn bloodthirsty_adversary_casts_the_copies_its_reflexive_ability_made() {
    cr!("603.12", "707.12");
    // "When you pay this cost one or more times, put that many +1/+1 counters on ~, then
    // exile up to that many target instant and/or sorcery cards ... and copy them. You may
    // cast any number of the copies without paying their mana costs.": the copies are
    // made as the reflexive ability resolves, and cast then.
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    t.lands(P0, "Mountain", 3);
    let bolt = t.graveyard(P0, "Lightning Bolt");
    // Pay {2}{R} once (there's no mana for more).
    t.answer_yes(P0, true);
    t.answer_yes(P0, true);
    t.answer_targets(P0, &[Entity::Object(bolt)]);
    // Cast the copy, at the opponent.
    t.answer_yes(P0, true);
    t.answer_yes(P0, true);
    t.answer_targets(P0, &[Entity::Player(P1)]);
    let a = t.enter(P0, "Bloodthirsty Adversary");
    t.resolve_all();
    assert_eq!(t.counters(a, "+1/+1"), 1);
    assert!(t.in_exile("Lightning Bolt"));
    assert_eq!(t.life(P1), 17, "{}", t.dump_log());
}
