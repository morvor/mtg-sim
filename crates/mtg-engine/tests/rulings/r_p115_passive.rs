//! Rulings batch P115 — planeswalkers' static ("passive") abilities: Arlinn, Voice of the
//! Pack's "enters with an additional +1/+1 counter" replacement, Huatli, the Sun's Heart's
//! damage-by-toughness and Tibalt, Rakish Instigator's "can't gain life".

use crate::r_p115_common::*;
use mtg_engine::testing::*;
use mtg_engine::*;

/// Puts `name` (a planeswalker) onto P0's battlefield with `loyalty` loyalty counters.
fn walker(t: &mut TestGame, name: &str, loyalty: u32) -> ObjectId {
    let id = t.battlefield(P0, name);
    t.g.objects[id.0 as usize]
        .counters
        .insert("loyalty".into(), loyalty);
    t.g.dirty = true;
    t.g.recompute();
    id
}

#[test]
fn arlinn_gives_one_counter_to_wolves_and_werewolves() {
    cr!("614.1c", "122.6");
    ruling!(
        "Arlinn, Voice of the Pack",
        "A creature that’s a Wolf or Werewolf enters the battlefield with one +1/+1 counter if it would otherwise enter with no +1/+1 counters."
    );
    ruling!(
        "Arlinn, Voice of the Pack",
        "A creature that’s both a Wolf and a Werewolf receives only one additional +1/+1 counter from Arlinn’s ability."
    );
    supported("Arlinn, Voice of the Pack");
    let mut t = TestGame::new(2);
    walker(&mut t, "Arlinn, Voice of the Pack", 7);
    // A Wolf, a changeling (both a Wolf and a Werewolf) and a Bear.
    let wolf = t.enter(P0, "Timberpack Wolf");
    let both = t.enter(P0, "Changeling Outcast");
    let bear = t.enter(P0, "Grizzly Bears");
    // The opponent's Wolf isn't "a creature you control".
    let theirs = t.enter(P1, "Timberpack Wolf");
    t.settle();
    assert_eq!(t.counters(wolf, "+1/+1"), 1);
    assert_eq!(t.counters(both, "+1/+1"), 1);
    assert_eq!(t.counters(bear, "+1/+1"), 0);
    assert_eq!(t.counters(theirs, "+1/+1"), 0);
}

#[test]
fn arlinns_wolf_has_no_counter_if_she_left_first() {
    cr!("614.1c", "606.4", "704.5i");
    ruling!(
        "Arlinn, Voice of the Pack",
        "If Arlinn leaves the battlefield before her last ability resolves, most likely because she only had 2 loyalty when you activated the ability, the Wolf token won’t enter the battlefield with a +1/+1 counter."
    );
    for (loyalty, counter) in [(2u32, 0u32), (7, 1)] {
        let mut t = TestGame::new(2);
        let arlinn = walker(&mut t, "Arlinn, Voice of the Pack", loyalty);
        t.activate(P0, arlinn, 0, &[]).expect("activate -2");
        t.resolve_all();
        assert_eq!(t.on_battlefield(arlinn), loyalty > 2);
        let wolves = with_subtype(&t, P0, "Wolf");
        assert_eq!(wolves.len(), 1, "loyalty {loyalty}");
        assert_eq!(t.counters(wolves[0], "+1/+1"), counter, "loyalty {loyalty}");
    }
}

#[test]
fn huatli_changes_combat_damage_not_power() {
    cr!("510.1a");
    ruling!(
        "Huatli, the Sun's Heart",
        "Huatli’s first ability doesn’t actually change any creature’s power. It changes only the amount of combat damage it assigns. All other rules and effects that check power or toughness use the real values. For example, Domri’s Ambush won’t cause a creature to deal damage equal to its toughness."
    );
    supported("Huatli, the Sun's Heart");
    // Domri's Ambush: Horned Turtle (1/4) gets a counter (2/5) and deals damage equal to
    // its power, 2, to Hill Giant (3/3), which survives.
    let mut t = TestGame::new(2);
    walker(&mut t, "Huatli, the Sun's Heart", 7);
    let turtle = t.battlefield(P0, "Horned Turtle");
    let giant = t.battlefield(P1, "Hill Giant");
    assert_eq!(t.pt(turtle), (1, 4));
    cast_at(&mut t, P0, "Domri's Ambush", &[obj(turtle), obj(giant)]);
    t.resolve_all();
    assert_eq!(t.pt(turtle), (2, 5));
    assert!(t.on_battlefield(giant));
    assert_eq!(t.obj_now(giant).damage, 2);
    // In combat, the Turtle assigns damage equal to its toughness: 5.
    t.attack(&[(turtle, pl(P1))], &[]);
    assert_eq!(t.life(P1), 15);
}

#[test]
fn huatlis_toughness_is_read_as_her_ability_resolves() {
    cr!("608.2h", "606.3");
    ruling!(
        "Huatli, the Sun's Heart",
        "The greatest toughness among creatures you control is determined only as Huatli’s loyalty ability begins to resolve."
    );
    let mut t = TestGame::new(2);
    let huatli = walker(&mut t, "Huatli, the Sun's Heart", 7);
    let turtle = t.battlefield(P0, "Horned Turtle");
    t.activate(P0, huatli, 0, &[]).expect("activate -3");
    // In response, the Turtle grows to 4/7; the ability then gains 7 life.
    cast_at(&mut t, P0, "Giant Growth", &[obj(turtle)]);
    t.resolve();
    assert_eq!(t.pt(turtle), (4, 7));
    t.resolve_all();
    assert_eq!(t.life(P0), 27);
}

#[test]
fn tibalt_stops_set_life_totals_from_raising_an_opponents_life() {
    cr!("119.5", "119.7");
    ruling!(
        "Tibalt, Rakish Instigator",
        "If an effect says to set an opponent's life total to a number that's higher than their current life total while Tibalt is on the battlefield, the player's life total doesn't change."
    );
    supported("Tibalt, Rakish Instigator");
    supported("Blessed Wind");
    let mut t = TestGame::new(2);
    walker(&mut t, "Tibalt, Rakish Instigator", 5);
    t.g.players[P1.idx()].life = 5;
    t.g.players[P0.idx()].life = 5;
    // "Target player's life total becomes 20."
    cast_at(&mut t, P0, "Blessed Wind", &[pl(P1)]);
    t.resolve_all();
    assert_eq!(t.life(P1), 5);
    // Tibalt's controller isn't affected.
    cast_at(&mut t, P0, "Blessed Wind", &[pl(P0)]);
    t.resolve_all();
    assert_eq!(t.life(P0), 20);
    // Lowering the opponent's life total still works.
    t.g.players[P1.idx()].life = 25;
    cast_at(&mut t, P0, "Blessed Wind", &[pl(P1)]);
    t.resolve_all();
    assert_eq!(t.life(P1), 20);
}

#[test]
fn tibalt_lets_life_gaining_spells_do_everything_else() {
    cr!("119.7", "608.2c");
    ruling!(
        "Tibalt, Rakish Instigator",
        "Spells and abilities that cause opponents to gain life still resolve while Tibalt is on the battlefield. No opponent will gain life, but any other effects of that spell or ability will happen."
    );
    let mut t = TestGame::new(2);
    walker(&mut t, "Tibalt, Rakish Instigator", 5);
    let giant = t.battlefield(P1, "Hill Giant");
    // Swords to Plowshares: the Giant is exiled; its controller gains no life.
    cast_at(&mut t, P0, "Swords to Plowshares", &[obj(giant)]);
    t.resolve_all();
    assert!(t.in_exile("Hill Giant"));
    assert_eq!(t.life(P1), 20);
}
