//! Rulings batch P186 — typal Zombie cards: mass reanimation and simultaneous entry (CR
//! 603.6a, 614.12), "enters with" counts made from the graveyard, prevention replacement
//! effects (CR 615), simultaneous deaths (CR 603.10a), random choices among cards, costs
//! paid before activation restrictions are checked (CR 602.5b, 601.2h), counts on
//! resolution (CR 608.2h), blocking restrictions checked as blockers are declared (CR
//! 509.1b), effects locked in on resolution (CR 611.2c), searches for cards with a stated
//! quality (CR 701.23b), and text-changing effects on face-down permanents (CR 612, 708).

use crate::r_p185_common::*;
use crate::r_p186_common::*;
use crate::r_s01_common::*;
use crate::r_s02_common::{can_activate, destroy};
use crate::r_s06_common::damage;
use crate::r_s25_common::cast_new;
use mtg_engine::ability::Modification;
use mtg_engine::decision::Answer;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::types::counters;
use mtg_engine::*;

/// A custom Zombie Human creature card.
fn zombie_human() -> mtg_engine::card::CardDef {
    custom_card("Ghoulish Villager", "Creature — Zombie Human", "{1}{B}", Some((2, 2)), "")
}

#[test]
fn zombie_apocalypse_destroys_every_human_including_returned_ones() {
    cr!("608.2c", "403.4");
    ruling!(
        "Zombie Apocalypse",
        "All Humans are destroyed, not just ones you control."
    );
    ruling!(
        "Zombie Apocalypse",
        "If a creature card in your graveyard is both a Zombie and a Human, it will be returned to the battlefield tapped, then be destroyed."
    );
    supported("Zombie Apocalypse");
    let mut t = TestGame::new(2);
    let mine = vanilla(&mut t, P0, "Creature — Human Soldier", (2, 2));
    let theirs = vanilla(&mut t, P1, "Creature — Human Soldier", (2, 2));
    let zh = t.custom(P0, zombie_human(), Zone::Graveyard(P0));
    t.graveyard(P0, "Scathe Zombies");
    cast_new(&mut t, P0, "Zombie Apocalypse", &[]);
    t.resolve_all();
    assert!(!t.on_battlefield(mine) && !t.on_battlefield(theirs));
    let z = t.named_on_battlefield("Scathe Zombies");
    assert_eq!(z.len(), 1);
    assert!(t.g.obj(z[0]).tapped);
    assert_eq!(entries_of(&t, "Ghoulish Villager"), 1, "it was returned");
    assert!(entered_tapped(&t, "Ghoulish Villager"));
    assert_eq!(t.zone(zh), Zone::Graveyard(P0), "then destroyed");
}

#[test]
fn diregraf_colossus_cast_trigger_and_graveyard_count() {
    cr!("603.2", "614.1c", "603.6a");
    ruling!(
        "Diregraf Colossus",
        "Diregraf Colossus's last ability won't trigger when you cast it because it's not on the battlefield yet."
    );
    ruling!(
        "Diregraf Colossus",
        "If Diregraf Colossus enters the battlefield from your graveyard, its first ability will count itself among the Zombie cards in your graveyard. Similarly, if it enters the battlefield at the same time as another Zombie card enters the battlefield from your graveyard, the ability will count that other Zombie."
    );
    supported("Diregraf Colossus");
    let mut t = TestGame::new(2);
    t.graveyard(P0, "Scathe Zombies");
    let c = cast_new(&mut t, P0, "Diregraf Colossus", &[]);
    t.resolve_all();
    assert_eq!(tokens(&t, P0).len(), 0, "no token for itself");
    assert_eq!(t.counters(c, counters::PLUS1), 1);
    cast_new(&mut t, P0, "Scathe Zombies", &[]);
    t.resolve_all();
    assert_eq!(tokens(&t, P0).len(), 1, "a later Zombie spell makes a token");
    // From the graveyard, with another Zombie card returned at the same time.
    let mut t = TestGame::new(2);
    let c = t.graveyard(P0, "Diregraf Colossus");
    t.graveyard(P0, "Scathe Zombies");
    cast_new(&mut t, P0, "Zombie Apocalypse", &[]);
    t.resolve_all();
    assert_eq!(t.counters(c, counters::PLUS1), 2, "itself and the other Zombie");
}

#[test]
fn unbreathing_horde_from_the_graveyard_counts_itself() {
    cr!("614.1c", "614.12");
    ruling!(
        "Unbreathing Horde",
        "If Unbreathing Horde enters from a graveyard, it will count itself when determining how many +1/+1 counters it enters with."
    );
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Scathe Zombies");
    t.graveyard(P0, "Scathe Zombies");
    let h = t.graveyard(P0, "Unbreathing Horde");
    crate::r_s05_common::move_to(&mut t, h, Zone::Battlefield);
    assert!(t.on_battlefield(h));
    assert_eq!(t.counters(h, counters::PLUS1), 3);
}

#[test]
fn unbreathing_horde_without_counters_still_prevents_damage() {
    cr!("615.1", "614.1a");
    ruling!(
        "Unbreathing Horde",
        "If Unbreathing Horde has no +1/+1 counters on it (but its toughness is raised above 0 by another effect), any damage dealt to it will still be prevented, even though no counter will be removed."
    );
    let mut t = TestGame::new(2);
    let h = t.battlefield(P0, "Unbreathing Horde");
    pump(&mut t, h, 2, 2);
    let src = t.battlefield(P1, "Grizzly Bears");
    damage(&mut t, src, 3, h);
    assert!(t.on_battlefield(h));
    assert_eq!(marked(&t, h), 0);
    assert_eq!(t.counters(h, counters::PLUS1), 0);
}

#[test]
fn plague_belcher_triggers_for_a_zombie_dying_with_it() {
    cr!("603.10a", "704.3");
    ruling!(
        "Plague Belcher",
        "If another Zombie you control dies at the same time as Plague Belcher, Plague Belcher’s last ability will trigger for that Zombie."
    );
    supported("Plague Belcher");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Plague Belcher");
    t.battlefield(P0, "Scathe Zombies");
    crate::r_p044_common::wrath(&mut t, P0);
    assert_eq!(t.life(P1), 19);
}

#[test]
fn plague_belcher_two_headed_giant_team_loses_two() {
    cr!("810.8a", "810.9");
    ruling!(
        "Plague Belcher",
        "In a Two-Headed Giant game, the triggered ability of Plague Belcher causes the opposing team to lose 2 life."
    );
    let mut t = crate::r_s33_common::two_headed_giant();
    let team = t.life(P2);
    t.battlefield(P0, "Plague Belcher");
    let z = t.battlefield(P0, "Scathe Zombies");
    destroy(&mut t, z);
    t.resolve_all();
    assert_eq!(t.life(P2), team - 2);
}

#[test]
fn tomb_tyrant_needs_three_zombie_cards_before_activating() {
    cr!("602.5b", "602.2");
    ruling!(
        "Tomb Tyrant",
        "You need at least three Zombie creature cards in your graveyard before you begin to activate Tomb Tyrant's activated ability. You can't begin to activate it with two and plan to sacrifice a Zombie for the third."
    );
    supported("Tomb Tyrant");
    let mut t = TestGame::new(2);
    let tyrant = t.battlefield(P0, "Tomb Tyrant");
    t.battlefield(P0, "Scathe Zombies");
    t.lands(P0, "Swamp", 3);
    t.graveyard(P0, "Scathe Zombies");
    t.graveyard(P0, "Scathe Zombies");
    assert!(!can_activate(&mut t, P0, tyrant));
    t.graveyard(P0, "Scathe Zombies");
    assert!(can_activate(&mut t, P0, tyrant));
}

#[test]
fn tomb_tyrant_can_return_the_zombie_sacrificed_for_its_cost() {
    cr!("601.2h", "608.2h");
    ruling!(
        "Tomb Tyrant",
        "If the creature you sacrifice to pay the cost is a Zombie card, it will be among the cards that may be chosen."
    );
    let mut returned_sacrificed = false;
    for seed in 0..24 {
        let mut t = TestGame::with_config(
            2,
            mtg_engine::game::GameConfig {
                seed,
                ..Default::default()
            },
        );
        let tyrant = t.battlefield(P0, "Tomb Tyrant");
        let fodder = t.battlefield(P0, "Gravedigger");
        t.lands(P0, "Swamp", 3);
        for _ in 0..3 {
            t.graveyard(P0, "Scathe Zombies");
        }
        t.answer_choose(P0, &[Entity::Object(fodder)]);
        t.activate(P0, tyrant, 0, &[]).unwrap();
        assert!(!t.on_battlefield(fodder));
        t.answer_yes(P0, false);
        t.resolve_all();
        let back = t.named_on_battlefield("Gravedigger").len()
            + t.named_on_battlefield("Scathe Zombies").len();
        assert_eq!(back, 1, "one card returns");
        if t.named_on_battlefield("Gravedigger").len() == 1 {
            returned_sacrificed = true;
            break;
        }
    }
    assert!(returned_sacrificed, "the sacrificed Zombie was never chosen");
}

#[test]
fn endless_ranks_counts_zombies_on_resolution() {
    cr!("608.2h", "107.1a");
    ruling!(
        "Endless Ranks of the Dead",
        "If you control fewer than two Zombies, you won't get any tokens."
    );
    ruling!(
        "Endless Ranks of the Dead",
        "The number of Zombies you control is counted when the ability resolves. If you control multiple Endless Ranks of the Dead, the tokens you get when the first ability resolves will count for the subsequent abilities (if the tokens are still under your control at that time)."
    );
    supported("Endless Ranks of the Dead");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Endless Ranks of the Dead");
    t.battlefield(P0, "Scathe Zombies");
    crate::r_s04_common::next_upkeep(&mut t, P0);
    t.resolve_all();
    assert_eq!(with_subtype(&t, P0, "Zombie").len(), 1);
    // Two of them with four Zombies: 2 tokens, then 3.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Endless Ranks of the Dead");
    t.battlefield(P0, "Endless Ranks of the Dead");
    for _ in 0..4 {
        t.battlefield(P0, "Scathe Zombies");
    }
    crate::r_s04_common::next_upkeep(&mut t, P0);
    t.resolve_all();
    assert_eq!(with_subtype(&t, P0, "Zombie").len(), 9);
}

#[test]
fn empty_the_laboratory_zombies_enter_together_and_see_each_other() {
    cr!("603.6a", "603.2");
    ruling!(
        "Empty the Laboratory",
        "The Zombies will enter the battlefield simultaneously. They will see each other enter for the purposes of triggered abilities."
    );
    supported("Empty the Laboratory");
    supported("Champion of the Perished");
    supported("Wayward Servant");
    let mut t = TestGame::new(2);
    let a = t.battlefield(P0, "Scathe Zombies");
    let b = t.battlefield(P0, "Scathe Zombies");
    crate::r_p189_common::set_library(&mut t, P0, &["Champion of the Perished", "Wayward Servant"]);
    t.answer_choose(P0, &[Entity::Object(a), Entity::Object(b)]);
    let card = t.hand(P0, "Empty the Laboratory");
    t.lands(P0, "Island", 2);
    t.lands(P0, "Wastes", 2);
    t.cast(P0, card).x(2).go();
    t.resolve_all();
    assert!(!t.on_battlefield(a) && !t.on_battlefield(b));
    let champ = t.named_on_battlefield("Champion of the Perished");
    assert_eq!(champ.len(), 1);
    assert_eq!(t.named_on_battlefield("Wayward Servant").len(), 1);
    assert_eq!(t.counters(champ[0], counters::PLUS1), 1, "saw the Servant enter");
    assert_eq!(t.life(P1), 19, "the Servant saw the Champion enter");
}

#[test]
fn aphetto_vulture_can_put_itself_on_top_of_the_library() {
    cr!("603.10a", "115.1");
    ruling!(
        "Aphetto Vulture",
        "The ability can be used to return itself to the top of your library if it is still in your graveyard when the ability resolves."
    );
    supported("Aphetto Vulture");
    let mut t = TestGame::new(2);
    let v = t.battlefield(P0, "Aphetto Vulture");
    t.answer_yes(P0, true);
    destroy(&mut t, v);
    t.resolve_all();
    let top = *t.g.player(P0).library.last().unwrap();
    assert_eq!(t.g.obj(top).chars.name.as_str(), "Aphetto Vulture");
}

#[test]
fn marauding_boneslasher_block_restriction() {
    cr!("509.1b", "509.1c");
    ruling!(
        "Marauding Boneslasher",
        "The other Zombie doesn’t have to block."
    );
    ruling!(
        "Marauding Boneslasher",
        "Whether you control another Zombie is checked only as you declare blockers. Marauding Boneslasher won’t stop blocking if you don’t control one later."
    );
    supported("Marauding Boneslasher");
    let mut t = TestGame::new(2);
    let bone = t.battlefield(P0, "Marauding Boneslasher");
    let attacker = t.battlefield(P1, "Grizzly Bears");
    crate::r_s09_common::to_combat(&mut t, P1);
    crate::r_s01_common::attack_with(&mut t, &[(attacker, Entity::Player(P0))]);
    assert!(!crate::r_s21_common::legal_blocks(&mut t, P0, &[(bone, attacker)]));
    let other = t.battlefield(P0, "Scathe Zombies");
    assert!(crate::r_s21_common::legal_blocks(&mut t, P0, &[(bone, attacker)]));
    t.answer(P0, DecisionKind::Blockers, Answer::Blockers(vec![(bone, attacker)]));
    t.advance_to(P1, mtg_engine::turn::Step::DeclareBlockers);
    assert!(crate::r_s10_common::blocking(&t, bone));
    destroy(&mut t, other);
    assert!(crate::r_s10_common::blocking(&t, bone));
}

#[test]
fn in_oketras_name_is_locked_in_as_it_resolves() {
    cr!("611.2c", "608.2h");
    ruling!(
        "In Oketra's Name",
        "The set of creatures affected by In Oketra’s Name and how those creatures are affected is locked in as In Oketra’s Name resolves."
    );
    supported("In Oketra's Name");
    let mut t = TestGame::new(2);
    let z = t.battlefield(P0, "Scathe Zombies");
    let n = t.battlefield(P0, "Grizzly Bears");
    cast_new(&mut t, P0, "In Oketra's Name", &[]);
    t.resolve_all();
    // (Its Zombies don't also get the +1/+1 of "other creatures".)
    assert_eq!(t.pt(z), (4, 3));
    assert_eq!(t.pt(n), (3, 3));
    add_types(&mut t, n, &["Zombie"]);
    assert_eq!(t.pt(n), (3, 3));
    crate::r_s26_common::modify_until_eot(
        &mut t,
        z,
        vec![Modification::RemoveSubtypes(vec!["Zombie".into()])],
    );
    assert_eq!(t.pt(z), (4, 3));
    let late = t.battlefield(P0, "Grizzly Bears");
    assert_eq!(t.pt(late), (2, 2));
}

#[test]
fn corpse_harvester_may_find_nothing() {
    cr!("701.23b");
    ruling!(
        "Corpse Harvester",
        "You do not have to find a Zombie card or swamp card if you do not want to, even if you have them in your library."
    );
    supported("Corpse Harvester");
    let mut t = TestGame::new(2);
    let h = t.battlefield(P0, "Corpse Harvester");
    let fodder = t.battlefield(P0, "Grizzly Bears");
    t.library_top(P0, "Scathe Zombies");
    t.library_top(P0, "Swamp");
    t.lands(P0, "Swamp", 2);
    t.answer_choose(P0, &[Entity::Object(fodder)]);
    t.activate(P0, h, 0, &[]).unwrap();
    let hand = t.hand_size(P0);
    t.answer_choose(P0, &[]);
    t.answer_choose(P0, &[]);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand);
    assert!(!t.in_hand(P0, "Scathe Zombies") && !t.in_hand(P0, "Swamp"));
    assert!(t
        .g
        .player(P0)
        .library
        .iter()
        .any(|c| t.g.obj(*c).chars.name.as_str() == "Scathe Zombies"));
    // Positive control: finding them puts them into the hand.
    let mut t = TestGame::new(2);
    let h = t.battlefield(P0, "Corpse Harvester");
    let fodder = t.battlefield(P0, "Grizzly Bears");
    let z = t.library_top(P0, "Scathe Zombies");
    let s = t.library_top(P0, "Swamp");
    t.lands(P0, "Swamp", 2);
    t.answer_choose(P0, &[Entity::Object(fodder)]);
    t.activate(P0, h, 0, &[]).unwrap();
    t.answer_choose(P0, &[Entity::Object(z)]);
    t.answer_choose(P0, &[Entity::Object(s)]);
    t.resolve_all();
    assert!(t.in_hand(P0, "Scathe Zombies") && t.in_hand(P0, "Swamp"));
}
