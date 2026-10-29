//! Rulings batch P183 — typal anthems and counts: a permanent with several of the listed
//! types is counted or boosted once (CR 205.3, 613.4c), marked damage becoming lethal
//! when the anthem goes away (CR 120.6, 704.5g), and X read on resolution (CR 608.2h).

use crate::r_p009_common::{kill, obj, pl};
use crate::r_p185_common::lethal_when_it_leaves;
use crate::r_p189_common::triggers_of;
use crate::r_s01_common::{attack_with, supported, with_subtype};
use crate::r_s19_common::add_lore;
use crate::r_s24_common::choose_creature_type;
use mtg_engine::testing::*;
use mtg_engine::types::counters;
use mtg_engine::*;

#[test]
fn anthem_leaving_makes_marked_damage_lethal() {
    cr!("120.6", "704.5g", "611.3a");
    ruling!(
        "Sporecrown Thallid",
        "Because damage remains marked on a creature until it's removed as the turn ends, nonlethal damage dealt to a Fungus or Saproling creature you control may become lethal if Sporecrown Thallid leaves the battlefield during that turn."
    );
    ruling!(
        "Lyra Dawnbringer",
        "Because damage remains marked on a creature until it's removed as the turn ends, nonlethal damage dealt to an Angel you control may become lethal if Lyra leaves the battlefield during that turn."
    );
    ruling!(
        "King of the Pride",
        "Because damage remains marked on a creature until the damage is removed as the turn ends, nonlethal damage dealt to a Cat you control may become lethal if King of the Pride leaves the battlefield during that turn."
    );
    ruling!(
        "Vanquisher's Banner",
        "Because damage remains marked on a creature until the damage is removed as the turn ends, nonlethal damage dealt to a creature you control of the chosen type may become lethal if Vanquisher's Banner leaves the battlefield during that turn."
    );
    ruling!(
        "Morophon, the Boundless",
        "Because damage remains marked on a creature until the damage is removed as the turn ends, nonlethal damage dealt to other creatures you control of the chosen type may become lethal if Morophon leaves the battlefield during that turn."
    );
    ruling!(
        "Rally the Ranks",
        "Because damage remains marked on creatures until the damage is removed as the turn ends, nonlethal damage dealt to creatures you control may become lethal if Rally the Ranks leaves the battlefield that turn."
    );
    // (anthem, whether it chooses a creature type (Bear) as it enters, a creature it
    // boosts, that creature's size with it)
    let cases: &[(&str, bool, &str, (i32, i32))] = &[
        ("Sporecrown Thallid", false, "Woodland Changeling", (3, 3)),
        ("Lyra Dawnbringer", false, "Serra Angel", (5, 5)),
        ("King of the Pride", false, "Savannah Lions", (4, 2)),
        ("Vanquisher's Banner", true, "Grizzly Bears", (3, 3)),
        ("Morophon, the Boundless", true, "Grizzly Bears", (3, 3)),
        ("Rally the Ranks", true, "Grizzly Bears", (3, 3)),
    ];
    for (anthem, choose, member, pt) in cases {
        supported(anthem);
        let mut t = TestGame::new(2);
        let a = if *choose {
            choose_creature_type(&mut t, P0, "Bear");
            t.enter(P0, anthem)
        } else {
            t.battlefield(P0, anthem)
        };
        let m = t.battlefield(P0, member);
        assert_eq!(t.pt(m), *pt, "{anthem}");
        lethal_when_it_leaves(&mut t, m, a, anthem);
    }
}

#[test]
fn righteous_valkyries_bonus_ending_makes_marked_damage_lethal() {
    cr!("120.6", "704.5g", "611.3a");
    ruling!(
        "Righteous Valkyrie",
        "Because damage remains marked on creatures until the damage is removed as the turn ends, nonlethal damage dealt to creatures you control may become lethal if your life total drops below 7 life more than your starting life total or if Righteous Valkyrie leaves the battlefield that turn."
    );
    supported("Righteous Valkyrie");
    for life_drops in [false, true] {
        let mut t = TestGame::new(2);
        t.g.players[0].life = 27;
        let v = t.battlefield(P0, "Righteous Valkyrie");
        let bears = t.battlefield(P0, "Grizzly Bears");
        assert_eq!(t.pt(bears), (4, 4));
        crate::r_s06_common::damage(&mut t, bears, 3, bears);
        assert!(t.on_battlefield(bears));
        if life_drops {
            crate::r_p009_common::lose(&mut t, P0, 1);
            assert!(t.on_battlefield(v));
        } else {
            kill(&mut t, v);
        }
        assert!(!t.on_battlefield(bears), "life drops: {life_drops}");
    }
}

#[test]
fn a_creature_with_several_of_the_listed_types_gets_the_bonus_once() {
    cr!("205.3a", "613.4c", "702.73a");
    ruling!(
        "Kaheera, the Orphanguard",
        "A creature that's more than one of the five types gets +1/+1 only once from Kaheera's last ability."
    );
    ruling!(
        "Death-Priest of Myrkul",
        "A creature you control that's more than one of the creature types listed in the first ability will get only +1/+1."
    );
    ruling!(
        "Lovisa Coldeyes",
        "Each creature with at least one of those subtypes gets +2/+2 from Lovisa Coldeyes’ ability, regardless of how many of those subtypes it has. For example, a 3/3 Mistform Ultimus would become 5/5, not 9/9."
    );
    // (anthem, a creature of several of its types, that creature's size with it)
    let cases: &[(&str, &str, (i32, i32))] = &[
        ("Kaheera, the Orphanguard", "Coeurl", (3, 3)),
        ("Death-Priest of Myrkul", "Skeletal Vampire", (4, 4)),
        ("Lovisa Coldeyes", "Mistform Ultimus", (5, 5)),
    ];
    for (anthem, member, pt) in cases {
        supported(anthem);
        supported(member);
        let mut t = TestGame::new(2);
        t.battlefield(P0, anthem);
        let m = t.battlefield(P0, member);
        assert_eq!(t.pt(m), *pt, "{anthem}");
    }
}

#[test]
fn vault_12_counts_rad_counters_of_players_in_the_game_and_counters_each_zombie_mutant_once()
{
    cr!("714.2b", "800.4", "122.1");
    ruling!(
        "Vault 12: The Necropolis",
        "A creature you control that’s both a Zombie and a Mutant will get only two +1/+1 counters from Vault 12’s last ability."
    );
    ruling!(
        "Vault 12: The Necropolis",
        "Use the number of rad counters among all players still in the game when Vault 12’s second chapter ability resolves to determine the value of X."
    );
    supported("Vault 12: The Necropolis");
    let mut t = TestGame::new(3);
    let vault = t.battlefield(P0, "Vault 12: The Necropolis");
    add_lore(&mut t, vault, 1);
    t.resolve_all();
    for p in [P0, P1, P2] {
        assert_eq!(t.g.player(p).counter(counters::RAD), 3);
    }
    // P2 leaves the game before chapter II: 6 rad counters among the players left.
    t.g.lose_game(P2);
    t.settle();
    add_lore(&mut t, vault, 1);
    t.resolve_all();
    let zombies = with_subtype(&t, P0, "Zombie");
    assert_eq!(zombies.len(), 6);
    add_lore(&mut t, vault, 1);
    t.resolve_all();
    for z in zombies {
        assert!(t.obj_now(z).chars.has_subtype("Mutant"));
        assert_eq!(t.counters(z, counters::PLUS1), 2);
    }
}

#[test]
fn silvergill_douser_counts_each_permanent_once_and_reads_x_on_resolution() {
    cr!("608.2h", "702.73a");
    ruling!(
        "Silvergill Douser",
        "A permanent that’s both a Merfolk and a Faerie is counted only once when determining X."
    );
    ruling!(
        "Silvergill Douser",
        "The value of X is determined when Silvergill Douser’s ability resolves. It won’t change later, even if the number of Merfolk and/or Faeries you control changes."
    );
    supported("Silvergill Douser");
    // The Douser and a changeling (a Merfolk and a Faerie): X is 2.
    let mut t = TestGame::new(2);
    let douser = t.battlefield(P0, "Silvergill Douser");
    t.battlefield(P0, "Woodland Changeling");
    let giant = t.battlefield(P1, "Craw Wurm");
    t.activate(P0, douser, 0, &[obj(giant)]).unwrap();
    t.resolve_all();
    assert_eq!(t.pt(giant).0, 6 - 2);
    // The Douser alone when activated; a Merfolk joins in response: X is 2. It then
    // leaves: X stays 2.
    let mut t = TestGame::new(2);
    let douser = t.battlefield(P0, "Silvergill Douser");
    let giant = t.battlefield(P1, "Craw Wurm");
    t.activate(P0, douser, 0, &[obj(giant)]).unwrap();
    let merfolk = t.battlefield(P0, "Coral Merfolk");
    t.resolve_all();
    assert_eq!(t.pt(giant).0, 6 - 2);
    kill(&mut t, merfolk);
    assert_eq!(t.pt(giant).0, 6 - 2);
}

#[test]
fn kithkin_mourncaller_triggers_once_for_a_kithkin_elf() {
    cr!("603.2", "702.73a");
    ruling!(
        "Kithkin Mourncaller",
        "If an attacking creature that’s both a Kithkin and an Elf is put into your graveyard from the battlefield, Kithkin Mourncaller’s ability triggers only once."
    );
    supported("Kithkin Mourncaller");
    let mut t = TestGame::new(2);
    let mourn = t.battlefield(P0, "Kithkin Mourncaller");
    let both = t.battlefield(P0, "Woodland Changeling");
    attack_with(&mut t, &[(both, pl(P1))]);
    let hand = t.hand_size(P0);
    t.answer_yes(P0, true);
    kill(&mut t, both);
    t.resolve_all();
    assert_eq!(triggers_of(&t, mourn), 1);
    assert_eq!(t.hand_size(P0), hand + 1);
}
