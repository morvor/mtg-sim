//! Rulings batch P223 — "target creature or planeswalker that's black or red" (Devout
//! Decree and the other cards with a color-list suffix, CR 115.1a, 105.2).

use crate::r_p223_common::*;
use crate::r_s01_common::*;
use crate::r_s02_common::destroy;
use crate::r_s03_common::in_hand_with_mana;
use crate::r_s04_common::spell_targets;
use mtg_engine::testing::*;
use mtg_engine::types::*;
use mtg_engine::*;

/// P1's permanents: Grizzly Bears (green), Serra Angel (white), Vampire Nighthawk
/// (black), Wind Drake (blue), Samut, Tyrant Smasher (red and green), Goblin Piker (red).
fn rainbow(t: &mut TestGame) -> [ObjectId; 6] {
    [
        t.battlefield(P1, "Grizzly Bears"),
        t.battlefield(P1, "Serra Angel"),
        t.battlefield(P1, "Vampire Nighthawk"),
        t.battlefield(P1, "Wind Drake"),
        t.battlefield(P1, "Samut, Tyrant Smasher"),
        t.battlefield(P1, "Goblin Piker"),
    ]
}

#[test]
fn a_color_list_suffix_restricts_the_targets() {
    cr!("115.1a", "105.2");
    // Index into `rainbow` of the permanents each spell can target.
    for (name, legal) in [
        ("Devout Decree", vec![2, 4, 5]),        // black or red
        ("Terminal Criticism", vec![3, 4, 5]),   // blue or red
        ("Noxious Grasp", vec![0, 1, 4]),        // green or white
        ("Refute Destiny", vec![0, 3, 4]),       // green or blue
        ("Fry", vec![1, 3]),                     // white or blue
        ("Flourishing Grapple", vec![1, 4, 5]),  // red or white, an opponent controls
    ] {
        supported(name);
        let mut t = TestGame::new(2);
        let objs = rainbow(&mut t);
        // A red creature of P0's: never a legal target for Flourishing Grapple's first.
        let mine = t.battlefield(P0, "Goblin Piker");
        let got = spell_targets(&mut t, P0, name);
        let want: Vec<Entity> = legal.iter().map(|i| Entity::Object(objs[*i])).collect();
        // P0's red Goblin Piker is legal for the spells that care only about color.
        let red_ok = legal.contains(&5) && name != "Flourishing Grapple";
        assert_eq!(got.contains(&Entity::Object(mine)), red_ok, "{name}");
        let mut got_objs: Vec<Entity> = got
            .into_iter()
            .filter(|e| *e != Entity::Object(mine))
            .collect();
        got_objs.sort_by_key(|e| e.object().map(|o| o.0));
        assert_eq!(got_objs, want, "{name}");
    }
}

#[test]
fn devout_decree_exiles_and_scries_but_not_with_an_illegal_target() {
    cr!("608.2b", "701.22a");
    ruling!("Devout Decree", "If the target creature or planeswalker is an illegal target by the time Devout Decree tries to resolve, the spell doesn't resolve. You won't scry 1.");
    supported("Devout Decree");
    for illegal in [false, true] {
        let mut t = TestGame::new(2);
        for _ in 0..3 {
            t.library_top(P0, "Hill Giant");
        }
        let hawk = t.battlefield(P1, "Vampire Nighthawk");
        let dd = in_hand_with_mana(&mut t, P0, "Devout Decree");
        t.cast(P0, dd).target(hawk).go();
        if illegal {
            destroy(&mut t, hawk);
        }
        let from = t.asked().len();
        t.resolve_all();
        if illegal {
            assert!(scries_since(&t, from).is_empty());
            assert!(t.in_graveyard(P1, "Vampire Nighthawk"));
        } else {
            assert!(t.in_exile("Vampire Nighthawk"));
            assert_eq!(scry_sizes(&t, P0, from), vec![1]);
        }
    }
}

#[test]
fn quirion_dryad_counts_spells_of_the_listed_colors() {
    cr!("603.2", "105.2");
    supported("Quirion Dryad");
    // "Whenever you cast a spell that's white, blue, black, or red, put a +1/+1 counter
    // on this creature."
    for (spell, yes) in [
        ("Lightning Bolt", true),
        ("Giant Growth", false),
        ("Ornithopter", false),
    ] {
        let mut t = TestGame::new(2);
        let dryad = t.battlefield(P0, "Quirion Dryad");
        let card = in_hand_with_mana(&mut t, P0, spell);
        let mut c = t.cast(P0, card);
        if spell == "Lightning Bolt" {
            c = c.target(P1);
        } else if spell == "Giant Growth" {
            c = c.target(dryad);
        }
        c.go();
        t.resolve_all();
        assert_eq!(t.counters(dryad, counters::PLUS1), u32::from(yes), "{spell}");
    }
}
