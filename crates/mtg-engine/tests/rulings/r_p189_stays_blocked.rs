//! Rulings batch P189 — "can't be blocked" abilities that start to apply after the
//! creature has become blocked don't make it unblocked (CR 509.1h, 506.4), and
//! conditional evasion is checked only as blockers are declared (CR 509.1b).

use crate::r_p076_common::*;
use crate::r_p189_common::*;
use crate::r_s01_common::{attack_with, supported};
use crate::r_s11_common::turn_face_up;
use crate::r_s21_common::legal_blocks;
use mtg_engine::mana::ManaType;
use mtg_engine::object::CastMethod;
use mtg_engine::testing::*;
use mtg_engine::types::*;
use mtg_engine::*;

/// `p` casts `name` face down with morph (paying {3} with Wastes) and it resolves.
fn cast_face_down(t: &mut TestGame, p: PlayerId, name: &str) -> ObjectId {
    use mtg_engine::keywords::KeywordKind;
    t.lands(p, "Wastes", 3);
    let card = t.hand(p, name);
    let spell = t
        .cast(p, card)
        .method(CastMethod::FaceDown(KeywordKind::Morph))
        .go();
    t.resolve_all();
    let id = t.g.current(spell);
    assert!(t.obj(id).face_down && t.on_battlefield(id));
    // It has been under its controller's control since the turn began.
    t.g.objects[id.0 as usize].summoning_sick = false;
    id
}

fn face_down_blocked_then_turned_up(name: &str, cost: &[(ManaType, u32)]) {
    supported(name);
    let mut t = TestGame::new(2);
    let fd = cast_face_down(&mut t, P0, name);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let cost = cost.to_vec();
    unblockable_after_blocked(&mut t, fd, bears, move |t| {
        for (ty, n) in &cost {
            mana(t, P0, *ty, *n);
        }
        assert!(turn_face_up(t, P0, fd));
        assert_eq!(t.obj(fd).chars.name.as_str(), name);
    });
}

#[test]
fn gudul_lurker_turned_face_up_after_blocked() {
    cr!("509.1h", "506.4", "702.37e");
    ruling!(
        "Gudul Lurker",
        "If a face-down Gudul Lurker attacks and is blocked, turning it face up won’t cause it to become unblocked."
    );
    face_down_blocked_then_turned_up("Gudul Lurker", &[(ManaType::U, 1)]);
}

#[test]
fn mystic_of_the_hidden_way_turned_face_up_after_blocked() {
    cr!("509.1h", "506.4", "702.37e");
    ruling!(
        "Mystic of the Hidden Way",
        "If a face-down Mystic of the Hidden Way is blocked and then turned face up, it stays blocked."
    );
    face_down_blocked_then_turned_up("Mystic of the Hidden Way", &[(ManaType::U, 3)]);
}

/// `source`'s `idx`th activated ability makes itself unblockable after it's blocked.
fn self_activation_after_blocked(name: &str, idx: usize, pool: &[(ManaType, u32)]) {
    supported(name);
    let mut t = TestGame::new(2);
    let me = t.battlefield(P0, name);
    let blocker = t.battlefield(P1, "Grizzly Bears");
    let pool = pool.to_vec();
    unblockable_after_blocked(&mut t, me, blocker, move |t| {
        for (ty, n) in &pool {
            mana(t, P0, *ty, *n);
        }
        t.activate(P0, me, idx, &[])
            .unwrap_or_else(|e| panic!("{name}: {e:?}"));
    });
}

#[test]
fn frilled_sea_serpent_activated_after_blocked() {
    cr!("509.1h", "506.4");
    ruling!(
        "Frilled Sea Serpent",
        "Once Frilled Sea Serpent has been blocked, activating its ability won't change or undo that block."
    );
    self_activation_after_blocked("Frilled Sea Serpent", 0, &[(ManaType::U, 7)]);
}

#[test]
fn ashioks_skulker_activated_after_blocked() {
    cr!("509.1h", "506.4");
    ruling!(
        "Ashiok's Skulker",
        "Once this creature has been blocked, activating its ability won't change or undo that block."
    );
    self_activation_after_blocked("Ashiok's Skulker", 0, &[(ManaType::U, 4)]);
}

#[test]
fn gearseeker_serpent_activated_after_blocked() {
    cr!("509.1h", "506.4");
    ruling!(
        "Gearseeker Serpent",
        "Once this creature has been blocked, activating its last ability won’t cause it to become unblocked."
    );
    self_activation_after_blocked("Gearseeker Serpent", 0, &[(ManaType::U, 6)]);
}

#[test]
fn june_draws_two_after_blocked() {
    cr!("509.1h", "506.4");
    ruling!(
        "June, Bounty Hunter",
        "Once June, Bounty Hunter has been blocked, drawing two or more cards will not cause June to become unblocked."
    );
    supported("June, Bounty Hunter");
    let mut t = TestGame::new(2);
    let june = t.battlefield(P0, "June, Bounty Hunter");
    let bears = t.battlefield(P1, "Grizzly Bears");
    unblockable_after_blocked(&mut t, june, bears, |t| {
        t.g.draw_cards(P0, 2);
        t.g.flush_events();
    });
}

#[test]
fn tome_anima_draws_two_after_blocked() {
    cr!("509.1h", "506.4");
    ruling!(
        "Tome Anima",
        "Once Tome Anima has become blocked, it won't become unblocked if you draw a second card."
    );
    supported("Tome Anima");
    let mut t = TestGame::new(2);
    let anima = t.battlefield(P0, "Tome Anima");
    let bears = t.battlefield(P1, "Grizzly Bears");
    unblockable_after_blocked(&mut t, anima, bears, |t| {
        t.g.draw_cards(P0, 2);
        t.g.flush_events();
    });
}

#[test]
fn otter_penguin_second_draw_after_blocked() {
    cr!("509.1h", "506.4");
    ruling!(
        "Otter-Penguin",
        "Once Otter-Penguin has been blocked, its triggered ability will not cause it to become unblocked."
    );
    supported("Otter-Penguin");
    let mut t = TestGame::new(2);
    let otter = t.battlefield(P0, "Otter-Penguin");
    let bears = t.battlefield(P1, "Grizzly Bears");
    unblockable_after_blocked(&mut t, otter, bears, |t| {
        t.g.draw_cards(P0, 2);
        t.g.flush_events();
        t.resolve_all();
        // The ability resolved: +1/+2.
        assert_eq!(t.pt(otter), (3, 3));
    });
}

#[test]
fn nightwhorl_hermit_threshold_after_blocked() {
    cr!("509.1h", "506.4");
    ruling!(
        "Nightwhorl Hermit",
        "Once Nightwhorl Hermit has been blocked, putting enough cards in your graveyard to reach a total of seven or more cards won’t cause it to become unblocked."
    );
    supported("Nightwhorl Hermit");
    let mut t = TestGame::new(2);
    let hermit = t.battlefield(P0, "Nightwhorl Hermit");
    let bears = t.battlefield(P1, "Grizzly Bears");
    unblockable_after_blocked(&mut t, hermit, bears, |t| {
        t.g.mill(P0, 7);
        t.g.recompute();
        t.g.flush_events();
        assert_eq!(t.pt(hermit), (2, 4), "threshold applies");
    });
}

#[test]
fn nimrodel_watcher_scry_after_blocked() {
    cr!("509.1h", "506.4");
    ruling!(
        "Nimrodel Watcher",
        "Once Nimrodel Watcher has been blocked, resolving its triggered ability won't change or undo that block."
    );
    supported("Nimrodel Watcher");
    let mut t = TestGame::new(2);
    let watcher = t.battlefield(P0, "Nimrodel Watcher");
    let bears = t.battlefield(P1, "Grizzly Bears");
    unblockable_after_blocked(&mut t, watcher, bears, |t| {
        mtg_engine::library::scry(&mut t.g, P0, 1);
        t.g.flush_events();
        t.resolve_all();
        assert_eq!(t.pt(watcher), (3, 1), "the ability resolved");
    });
}

#[test]
fn razzle_dazzler_second_spell_after_blocked() {
    cr!("509.1h", "506.4");
    ruling!(
        "Razzle-Dazzler",
        "Once Razzle-Dazzler has been blocked, casting your second spell for the turn won’t remove the blocking creature from combat or cause Razzle-Dazzler to become unblocked."
    );
    supported("Razzle-Dazzler");
    let mut t = TestGame::new(2);
    let rd = t.battlefield(P0, "Razzle-Dazzler");
    let bears = t.battlefield(P1, "Grizzly Bears");
    unblockable_after_blocked(&mut t, rd, bears, |t| {
        for _ in 0..2 {
            mana(t, P0, ManaType::U, 1);
            let opt = t.hand(P0, "Opt");
            t.cast(P0, opt).go();
            t.resolve_all();
        }
        assert_eq!(t.counters(rd, counters::PLUS1), 1, "the ability resolved");
    });
}

#[test]
fn slippery_scoundrel_citys_blessing_after_blocked() {
    cr!("509.1h", "506.4", "702.131c");
    ruling!(
        "Slippery Scoundrel",
        "Once Slippery Scoundrel has become blocked, getting the city’s blessing won’t cause it to become unblocked."
    );
    supported("Slippery Scoundrel");
    let mut t = TestGame::new(2);
    let sc = t.battlefield(P0, "Slippery Scoundrel");
    let bears = t.battlefield(P1, "Grizzly Bears");
    unblockable_after_blocked(&mut t, sc, bears, |t| {
        t.lands(P0, "Wastes", 9);
        t.settle();
        assert!(t.g.player(P0).has_citys_blessing, "got the city's blessing");
    });
}

#[test]
fn glassdust_hulk_artifact_enters_after_blocked() {
    cr!("509.1h", "506.4");
    ruling!(
        "Glassdust Hulk",
        "Once a creature has blocked Glassdust Hulk, resolving its triggered ability won't cause it to become unblocked."
    );
    supported("Glassdust Hulk");
    let mut t = TestGame::new(2);
    let hulk = t.battlefield(P0, "Glassdust Hulk");
    let bears = t.battlefield(P1, "Grizzly Bears");
    unblockable_after_blocked(&mut t, hulk, bears, |t| {
        t.enter(P0, "Ornithopter");
        t.g.flush_events();
        t.resolve_all();
        assert_eq!(t.pt(hulk), (4, 5), "the ability resolved");
    });
}

#[test]
fn tetsuko_power_lowered_after_blocked() {
    cr!("509.1h", "506.4", "704.5f");
    ruling!(
        "Tetsuko Umezawa, Fugitive",
        "Once a creature you control has been blocked, changing its power to 1 or less won't cause it to become unblocked."
    );
    supported("Tetsuko Umezawa, Fugitive");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Tetsuko Umezawa, Fugitive");
    let giant = t.battlefield(P0, "Hill Giant");
    let bears = t.battlefield(P1, "Grizzly Bears");
    // Toughness lowered to 1 (and power to 1): it stays blocked.
    unblockable_after_blocked(&mut t, giant, bears, |t| {
        mana(t, P0, ManaType::B, 1);
        let d = t.hand(P0, "Disfigure");
        t.cast(P0, d).target(giant).go();
        t.resolve_all();
        assert_eq!(t.pt(giant), (1, 1));
    });
    // Lowered below 1, it dies.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Tetsuko Umezawa, Fugitive");
    let giant = t.battlefield(P0, "Hill Giant");
    let bears = t.battlefield(P1, "Grizzly Bears");
    crate::r_s03_common::to_blockers(&mut t, &[(giant, Entity::Player(P1))], &[(bears, giant)]);
    for _ in 0..2 {
        mana(&mut t, P0, ManaType::B, 1);
        let d = t.hand(P0, "Disfigure");
        t.cast(P0, d).target(giant).go();
        t.resolve_all();
    }
    assert!(t.in_graveyard(P0, "Hill Giant"));
}

#[test]
fn relic_runner_historic_spell_after_blocked() {
    cr!("509.1h", "506.4");
    ruling!(
        "Relic Runner",
        "Once Relic Runner has been blocked, casting a historic spell won't remove the blocking creature from combat or cause Relic Runner to become unblocked."
    );
    supported("Relic Runner");
    let mut t = TestGame::new(2);
    let rr = t.battlefield(P0, "Relic Runner");
    let bears = t.battlefield(P1, "Grizzly Bears");
    unblockable_after_blocked(&mut t, rr, bears, |t| {
        // Springjaw Trap is an artifact with flash: a historic spell.
        mana(t, P0, ManaType::C, 1);
        let trap = t.hand(P0, "Springjaw Trap");
        t.cast(P0, trap).go();
        t.resolve_all();
    });
}

#[test]
fn relic_runner_cant_be_blocked_after_a_historic_spell() {
    cr!("509.1b");
    supported("Relic Runner");
    let mut t = TestGame::new(2);
    let rr = t.battlefield(P0, "Relic Runner");
    let bears = t.battlefield(P1, "Grizzly Bears");
    attack_with(&mut t, &[(rr, Entity::Player(P1))]);
    assert!(legal_blocks(&mut t, P1, &[(bears, rr)]));
    let mut t = TestGame::new(2);
    let rr = t.battlefield(P0, "Relic Runner");
    let bears = t.battlefield(P1, "Grizzly Bears");
    let o = t.hand(P0, "Ornithopter");
    t.cast(P0, o).go();
    t.resolve_all();
    attack_with(&mut t, &[(rr, Entity::Player(P1))]);
    assert!(!legal_blocks(&mut t, P1, &[(bears, rr)]));
}

#[test]
fn expedition_lookout_any_opponent_with_eight_cards() {
    cr!("509.1b", "508.1c");
    ruling!(
        "Expedition Lookout",
        "Expedition Lookout’s last ability will apply if any opponent has eight or more cards in their graveyard, not necessarily all of them or even the one Expedition Lookout is attacking."
    );
    supported("Expedition Lookout");
    let mut t = TestGame::new(3);
    let lookout = t.battlefield(P0, "Expedition Lookout");
    let bears = t.battlefield(P2, "Grizzly Bears");
    // Only P1 (not the player it attacks) has eight cards in their graveyard.
    fill_graveyard(&mut t, P1, "Grizzly Bears", 8);
    attack_with(&mut t, &[(lookout, Entity::Player(P2))]);
    assert!(
        t.g.combat
            .as_ref()
            .unwrap()
            .attackers
            .iter()
            .any(|a| a.id == lookout),
        "it attacked despite defender"
    );
    assert!(!legal_blocks(&mut t, P2, &[(bears, lookout)]));
}

#[test]
fn expedition_lookout_graveyard_emptied_after_attacking() {
    cr!("509.1b", "506.4");
    ruling!(
        "Expedition Lookout",
        "Once Expedition Lookout has legally attacked, causing its last ability to not apply by removing cards from graveyards won’t cause it to stop attacking."
    );
    supported("Expedition Lookout");
    let mut t = TestGame::new(2);
    let lookout = t.battlefield(P0, "Expedition Lookout");
    let bears = t.battlefield(P1, "Grizzly Bears");
    fill_graveyard(&mut t, P1, "Grizzly Bears", 8);
    attack_with(&mut t, &[(lookout, Entity::Player(P1))]);
    assert!(!legal_blocks(&mut t, P1, &[(bears, lookout)]));
    exile_graveyard(&mut t, P1);
    // Still attacking, and now it can be blocked.
    assert!(t
        .g
        .combat
        .as_ref()
        .unwrap()
        .attackers
        .iter()
        .any(|a| a.id == lookout));
    assert!(legal_blocks(&mut t, P1, &[(bears, lookout)]));
    crate::r_s01_common::block_and_finish(&mut t, P1, &[(bears, lookout)]);
    assert!(t.in_graveyard(P1, "Grizzly Bears"));
    assert_eq!(t.life(P1), 20);
}
