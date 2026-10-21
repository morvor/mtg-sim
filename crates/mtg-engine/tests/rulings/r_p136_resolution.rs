//! Rulings batch P136 (removal) — removal spells and abilities as they resolve: illegal
//! targets (CR 608.2b), a legal target that isn't destroyed (CR 608.2c), random choices
//! (CR 608.2d), searches and shuffles (CR 701.23, 701.24), Chaos Warp's reveal (CR 701.20)
//! and tokens leaving the battlefield (CR 111.7, 111.8).

use crate::r_p136_common::*;
use crate::r_s01_common::*;
use mtg_engine::decision::{Answer, Decision};
use mtg_engine::game::GameConfig;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::types::*;
use mtg_engine::*;

fn library(t: &TestGame, p: PlayerId) -> Vec<ObjectId> {
    t.g.player(p).library.clone()
}

// ---------------------------------------------------------------------------
// Wild Swing
// ---------------------------------------------------------------------------

#[test]
fn wild_swing_chooses_at_random_among_the_still_legal_targets() {
    cr!("608.2b", "608.2d");
    ruling!(
        "Wild Swing",
        "If one of those permanents has become an illegal target by then, you randomly choose between the other two. If two of those permanents have become illegal targets by then, there is no random choice"
    );
    supported("Wild Swing");
    // Two targets become illegal: the third is destroyed.
    for seed in 0..4 {
        let mut t = TestGame::with_config(
            2,
            GameConfig {
                seed,
                ..Default::default()
            },
        );
        let a = t.battlefield(P1, "Grizzly Bears");
        let b = t.battlefield(P1, "Hill Giant");
        let c = t.battlefield(P1, "Gray Ogre");
        cast(&mut t, P0, "Wild Swing", &[&[o(a), o(b), o(c)]]);
        bounce(&mut t, a);
        bounce(&mut t, b);
        t.resolve_all();
        assert!(t.in_graveyard(P1, "Gray Ogre"), "seed {seed}");
    }
    // One target becomes illegal: one of the other two is destroyed, never the
    // illegal one — over several seeds, each of the two is sometimes chosen.
    let mut destroyed = std::collections::HashSet::new();
    for seed in 0..16 {
        let mut t = TestGame::with_config(
            2,
            GameConfig {
                seed,
                ..Default::default()
            },
        );
        let a = t.battlefield(P1, "Grizzly Bears");
        let b = t.battlefield(P1, "Hill Giant");
        let c = t.battlefield(P1, "Gray Ogre");
        cast(&mut t, P0, "Wild Swing", &[&[o(a), o(b), o(c)]]);
        bounce(&mut t, a);
        t.resolve_all();
        assert!(t.in_hand(P1, "Grizzly Bears"));
        let gone: Vec<&str> = ["Hill Giant", "Gray Ogre"]
            .into_iter()
            .filter(|n| t.in_graveyard(P1, n))
            .collect();
        assert_eq!(gone.len(), 1, "seed {seed}");
        destroyed.insert(gone[0]);
    }
    assert_eq!(destroyed.len(), 2);
}

#[test]
fn wild_swing_regeneration_must_be_set_up_before_it_resolves() {
    cr!("608.2d", "701.19a");
    ruling!(
        "Wild Swing",
        "If you want to put a regeneration shield on one of those permanents, or sacrifice it for some effect, or anything else, you must do so before Wild Swing resolves"
    );
    let mut t = TestGame::new(2);
    let boas: Vec<ObjectId> = (0..3).map(|_| t.battlefield(P1, "River Boa")).collect();
    t.lands(P1, "Forest", 3);
    let swing = cast(&mut t, P0, "Wild Swing", &[&[o(boas[0]), o(boas[1]), o(boas[2])]]);
    // P1 doesn't know which one will be chosen: they shield all three in response.
    for b in &boas {
        t.activate(P1, *b, 0, &[]).unwrap();
        t.resolve();
    }
    assert_eq!(t.g.stack, vec![swing]);
    let from = t.asked().len();
    t.resolve();
    // No player got to act while Wild Swing resolved.
    assert!(asked_since(&t, from)
        .iter()
        .all(|(_, d)| !matches!(d, Decision::Priority { .. })));
    // The chosen one regenerated (tapped) and every Boa is still there.
    for b in &boas {
        assert!(t.on_battlefield(*b));
    }
    let tapped = boas.iter().filter(|b| t.obj_now(**b).tapped).count();
    assert_eq!(tapped, 1);
}

// ---------------------------------------------------------------------------
// Illegal targets vs. legal targets that aren't destroyed
// ---------------------------------------------------------------------------

/// "Destroy target permanent. Its controller creates a token": the spells whose other
/// effects depend only on the target being legal.
#[test]
fn destroy_and_compensate_needs_a_legal_target_not_a_destroyed_one() {
    cr!("608.2b", "608.2c", "702.12b");
    ruling!(
        "Beast Within",
        "If the target permanent is an illegal target by the time Beast Within tries to resolve, the spell won't resolve. No player creates a Beast token. If the target is legal but not destroyed"
    );
    ruling!(
        "Generous Gift",
        "If the target permanent is an illegal target by the time Generous Gift tries to resolve, the spell doesn't resolve. No player creates an Elephant. If the target is legal but not destroyed"
    );
    for (spell, token) in [("Beast Within", "Beast"), ("Generous Gift", "Elephant")] {
        supported(spell);
        // Illegal target: no token.
        let mut t = TestGame::new(2);
        let bear = t.battlefield(P1, "Grizzly Bears");
        cast(&mut t, P0, spell, &[&[o(bear)]]);
        bounce(&mut t, bear);
        t.resolve_all();
        assert!(with_subtype(&t, P1, token).is_empty(), "{spell}");
        // Indestructible target: not destroyed, but its controller gets the token.
        let mut t = TestGame::new(2);
        let myr = t.battlefield(P1, "Darksteel Myr");
        cast(&mut t, P0, spell, &[&[o(myr)]]);
        t.resolve_all();
        assert!(t.on_battlefield(myr), "{spell}");
        assert_eq!(with_subtype(&t, P1, token).len(), 1, "{spell}");
    }
}

#[test]
fn noxious_grasp_and_vraskas_contempt_gain_life_only_with_a_legal_target() {
    cr!("608.2b", "608.2c", "702.12b");
    ruling!(
        "Noxious Grasp",
        "If the target creature or planeswalker is an illegal target by the time Noxious Grasp tries to resolve, the spell doesn’t resolve. You won’t gain life. If the target is legal but not destroyed"
    );
    ruling!(
        "Vraska's Contempt",
        "If the target creature or planeswalker is an illegal target by the time Vraska’s Contempt resolves, the entire spell doesn’t resolve. You won’t gain life."
    );
    for (spell, gain) in [("Noxious Grasp", 1), ("Vraska's Contempt", 2)] {
        supported(spell);
        let mut t = TestGame::new(2);
        let bear = t.battlefield(P1, "Grizzly Bears");
        cast(&mut t, P0, spell, &[&[o(bear)]]);
        bounce(&mut t, bear);
        t.resolve_all();
        assert_eq!(t.life(P0), 20, "{spell}");
        // A legal target that survives (here an indestructible green creature) still
        // means life gained.
        let mut t = TestGame::new(2);
        let bear = t.battlefield(P1, "Grizzly Bears");
        t.g.objects[bear.0 as usize]
            .counters
            .insert("indestructible".into(), 1);
        t.g.recompute();
        cast(&mut t, P0, spell, &[&[o(bear)]]);
        t.resolve_all();
        assert_eq!(t.life(P0), 20 + gain, "{spell}");
    }
}

#[test]
fn assassins_trophy_search_depends_on_a_legal_target() {
    cr!("608.2b", "608.2c", "701.23a");
    ruling!(
        "Assassin's Trophy",
        "If the target permanent is an illegal target by the time Assassin's Trophy tries to resolve, the spell doesn't resolve. No player searches their library."
    );
    ruling!(
        "Assassin's Trophy",
        "If the target permanent is a legal target but isn't destroyed, most likely because it has indestructible, its controller may search their library."
    );
    supported("Assassin's Trophy");
    // Illegal target: no search.
    let mut t = TestGame::new(2);
    let bear = t.battlefield(P1, "Grizzly Bears");
    t.library_top(P1, "Forest");
    cast(&mut t, P0, "Assassin's Trophy", &[&[o(bear)]]);
    bounce(&mut t, bear);
    let from = t.asked().len();
    t.resolve_all();
    assert!(t.named_on_battlefield("Forest").is_empty());
    assert!(asked_since(&t, from).iter().all(|(p, _)| *p != P1));
    // Indestructible target: its controller still searches.
    let mut t = TestGame::new(2);
    let myr = t.battlefield(P1, "Darksteel Myr");
    let forest = t.library_top(P1, "Forest");
    t.answer_yes(P1, true);
    t.answer_choose(P1, &[o(forest)]);
    cast(&mut t, P0, "Assassin's Trophy", &[&[o(myr)]]);
    t.resolve_all();
    assert!(t.on_battlefield(myr));
    assert_eq!(t.named_on_battlefield("Forest").len(), 1);
}

#[test]
fn assassins_trophy_no_search_no_shuffle() {
    cr!("701.24a");
    ruling!(
        "Assassin's Trophy",
        "If the permanent's controller doesn't search their library, they don't shuffle their library."
    );
    let mut t = TestGame::new(2);
    let bear = t.battlefield(P1, "Grizzly Bears");
    t.library_top(P1, "Forest");
    let before = library(&t, P1);
    t.answer_yes(P1, false);
    t.answer_choose(P1, &[]);
    cast(&mut t, P0, "Assassin's Trophy", &[&[o(bear)]]);
    t.resolve_all();
    assert!(t.in_graveyard(P1, "Grizzly Bears"));
    assert!(t.named_on_battlefield("Forest").is_empty());
    assert_eq!(library(&t, P1), before);
    // Searching (finding the land) shuffles.
    let mut t = TestGame::new(2);
    let bear = t.battlefield(P1, "Grizzly Bears");
    let forest = t.library_top(P1, "Forest");
    let before = library(&t, P1);
    t.answer_yes(P1, true);
    t.answer_choose(P1, &[o(forest)]);
    cast(&mut t, P0, "Assassin's Trophy", &[&[o(bear)]]);
    t.resolve_all();
    let mut after = library(&t, P1);
    let mut rest: Vec<ObjectId> = before.into_iter().filter(|c| *c != forest).collect();
    assert_ne!(after, rest);
    after.sort();
    rest.sort();
    assert_eq!(after, rest);
}

// ---------------------------------------------------------------------------
// Mangara of Corondor
// ---------------------------------------------------------------------------

#[test]
fn mangara_exiles_the_target_even_if_mangara_left() {
    cr!("608.2b", "400.7");
    ruling!(
        "Mangara of Corondor",
        "If Mangara leaves the battlefield before its ability resolves, the target permanent is still exiled. Mangara won't be exiled from the zone it's in."
    );
    supported("Mangara of Corondor");
    let mut t = TestGame::new(2);
    let mangara = t.battlefield(P0, "Mangara of Corondor");
    let bear = t.battlefield(P1, "Grizzly Bears");
    t.activate(P0, mangara, 0, &[o(bear)]).unwrap();
    crate::r_s02_common::destroy(&mut t, mangara);
    t.resolve_all();
    assert!(t.in_exile("Grizzly Bears"));
    assert!(t.in_graveyard(P0, "Mangara of Corondor"));
}

#[test]
fn mangara_isnt_exiled_if_the_target_is_illegal() {
    cr!("608.2b");
    ruling!(
        "Mangara of Corondor",
        "If the target permanent is an illegal target by the time Mangara's ability tries to resolve, the ability doesn't resolve. You don't exile Mangara."
    );
    let mut t = TestGame::new(2);
    let mangara = t.battlefield(P0, "Mangara of Corondor");
    let bear = t.battlefield(P1, "Grizzly Bears");
    t.activate(P0, mangara, 0, &[o(bear)]).unwrap();
    bounce(&mut t, bear);
    t.resolve_all();
    assert!(t.on_battlefield(mangara));
}

// ---------------------------------------------------------------------------
// Chaos Warp
// ---------------------------------------------------------------------------

#[test]
fn chaos_warp_illegal_target_no_shuffle_no_reveal() {
    cr!("608.2b");
    ruling!(
        "Chaos Warp",
        "If the permanent is an illegal target by the time Chaos Warp tries to resolve, it won't resolve and none of its effects will occur."
    );
    supported("Chaos Warp");
    let mut t = TestGame::new(2);
    let bear = t.battlefield(P1, "Grizzly Bears");
    t.library_top(P1, "Hill Giant");
    let before = library(&t, P1);
    cast(&mut t, P0, "Chaos Warp", &[&[o(bear)]]);
    bounce(&mut t, bear);
    t.resolve_all();
    assert_eq!(library(&t, P1), before);
    assert!(t.named_on_battlefield("Hill Giant").is_empty());
}

/// Chaos Warp on a token P1 owns, with P1's library holding only `top`: the token isn't
/// a card, so `top` is the card revealed.
fn chaos_warp_a_token_onto(top: &str) -> (TestGame, ObjectId) {
    let mut t = TestGame::new(2);
    empty_library(&mut t, P1);
    let card = t.library_top(P1, top);
    let token = crate::r_s02_common::create_token(&mut t, P1, "Soldier");
    cast(&mut t, P0, "Chaos Warp", &[&[o(token)]]);
    t.resolve_all();
    assert!(!t.g.is_live(token) || t.zone(token) != Zone::Battlefield);
    (t, card)
}

#[test]
fn chaos_warp_reveals_only_cards_and_puts_only_permanent_cards_onto_the_battlefield() {
    cr!("111.7", "111.8", "110.4a", "303.4i");
    ruling!(
        "Chaos Warp",
        "The owner of a token is the player under whose control the token was put onto the battlefield. If a token is shuffled into a player's library this way, that player shuffles before revealing the top card of that library."
    );
    ruling!(
        "Chaos Warp",
        "A permanent card is a card with one or more of the following card types: artifact, creature, enchantment, land, or planeswalker."
    );
    ruling!(
        "Chaos Warp",
        "If the revealed card is not a permanent card, it remains on top of that library."
    );
    ruling!(
        "Chaos Warp",
        "If the revealed card is a permanent card but can't enter (perhaps because it's an Aura with nothing to enchant), it remains on top of that library."
    );
    // A creature, a land, an artifact and an enchantment card enter.
    for top in ["Grizzly Bears", "Forest", "Ornithopter", "Glorious Anthem"] {
        let (t, card) = chaos_warp_a_token_onto(top);
        assert!(t.on_battlefield(card), "{top}");
        assert_eq!(t.obj_now(card).controller, P1);
        assert!(library(&t, P1).is_empty(), "{top}");
    }
    // An instant stays on top; so does an Aura with nothing it could enchant (P1
    // controls no basic land for Ossification).
    for top in ["Lightning Bolt", "Ossification"] {
        let (t, card) = chaos_warp_a_token_onto(top);
        assert_eq!(library(&t, P1), vec![t.g.current(card)], "{top}");
    }
}

// ---------------------------------------------------------------------------
// Crumble to Dust
// ---------------------------------------------------------------------------

#[test]
fn crumble_to_dust_may_leave_cards_with_that_name() {
    cr!("701.23b", "701.23d");
    ruling!(
        "Crumble to Dust",
        "You may leave any cards with that name in the player’s graveyard, hand, or library. You don’t have to exile them."
    );
    supported("Crumble to Dust");
    let mut t = TestGame::new(2);
    let vents = t.battlefield(P1, "Steam Vents");
    let in_gy = t.graveyard(P1, "Steam Vents");
    t.hand(P1, "Steam Vents");
    t.library_top(P1, "Steam Vents");
    // P0 finds only the one in the graveyard.
    t.answer_choose(P0, &[o(in_gy)]);
    cast(&mut t, P0, "Crumble to Dust", &[&[o(vents)]]);
    t.resolve_all();
    assert_eq!(t.g.find_in_zone(Zone::Exile, "Steam Vents").len(), 2);
    assert!(t.in_hand(P1, "Steam Vents"));
    assert_eq!(t.g.find_in_zone(Zone::Library(P1), "Steam Vents").len(), 1);
    assert!(!t.in_graveyard(P1, "Steam Vents"));
}

// ---------------------------------------------------------------------------
// Ward and "can't be countered"
// ---------------------------------------------------------------------------

#[test]
fn long_goodbye_isnt_countered_by_ward() {
    cr!("702.21a", "101.2");
    ruling!(
        "Long Goodbye",
        "If you target a creature or planeswalker with ward, you may still pay the ward cost, but Long Goodbye won't be countered even if you don't."
    );
    supported("Long Goodbye");
    let mut t = TestGame::new(2);
    let guard = t.battlefield(P1, "Tomakul Honor Guard");
    let spell = cast(&mut t, P0, "Long Goodbye", &[&[o(guard)]]);
    t.g.players[0].mana_pool = Default::default();
    t.settle();
    assert_eq!(triggers_on_stack(&t, "Ward"), 1);
    t.resolve_all();
    assert!(t.in_graveyard(P1, "Tomakul Honor Guard"));
    assert!(t.in_graveyard(P0, "Long Goodbye"));
    let _ = spell;
    // Control: a spell that can be countered is countered by ward.
    let mut t = TestGame::new(2);
    let guard = t.battlefield(P1, "Tomakul Honor Guard");
    cast(&mut t, P0, "Eliminate", &[&[o(guard)]]);
    t.g.players[0].mana_pool = Default::default();
    t.resolve_all();
    assert!(t.on_battlefield(guard));
}

// ---------------------------------------------------------------------------
// Costs paid while casting
// ---------------------------------------------------------------------------

#[test]
fn bone_shards_no_player_acts_while_it_is_cast() {
    cr!("601.2", "601.2h");
    ruling!(
        "Bone Shards",
        "Once you begin to cast Bone Shards, no player may take actions until you're done."
    );
    supported("Bone Shards");
    let mut t = TestGame::new(2);
    let mine = t.battlefield(P0, "Grizzly Bears");
    let theirs = t.battlefield(P1, "Hill Giant");
    let from = t.asked().len();
    cast(&mut t, P0, "Bone Shards", &[&[o(theirs)]]);
    // With no other card in hand, the Bears were sacrificed as the cost; P1 wasn't
    // asked anything in the meantime.
    assert!(asked_since(&t, from).iter().all(|(p, _)| *p == P0));
    assert!(!t.on_battlefield(mine));
    assert!(t.in_graveyard(P0, "Grizzly Bears"));
    t.resolve_all();
    assert!(t.in_graveyard(P1, "Hill Giant"));
}

// Other effects that put a token into a library and then use that library's cards in the
// same resolution: the token isn't one of those cards.

#[test]
fn oblation_on_a_token_draws_cards_not_the_token() {
    cr!("111.6", "111.7", "121.1");
    supported("Oblation");
    let mut t = TestGame::new(2);
    empty_library(&mut t, P1);
    t.library_top(P1, "Lightning Bolt");
    t.library_top(P1, "Shock");
    let token = crate::r_s02_common::create_token(&mut t, P1, "Soldier");
    cast(&mut t, P0, "Oblation", &[&[o(token)]]);
    t.resolve_all();
    let mut hand = crate::r_s04_common::hand_names(&t, P1);
    hand.sort();
    assert_eq!(hand, vec!["Lightning Bolt", "Shock"]);
    assert!(t.named_on_battlefield("Soldier").is_empty());
}

#[test]
fn proteus_staff_on_a_token_reveals_only_cards() {
    cr!("111.6", "111.8", "701.20a");
    supported("Proteus Staff");
    let mut t = TestGame::new(2);
    empty_library(&mut t, P1);
    let bolt = t.library_top(P1, "Lightning Bolt");
    let staff = t.battlefield(P0, "Proteus Staff");
    t.lands(P0, "Island", 3);
    let token = crate::r_s02_common::create_token(&mut t, P1, "Soldier");
    t.activate(P0, staff, 0, &[o(token)]).unwrap();
    t.resolve_all();
    // No creature card was revealed: nothing entered, and the token is gone.
    assert!(t.g.permanents().all(|p| p.controller != P1));
    assert_eq!(library(&t, P1), vec![t.g.current(bolt)]);
}

#[test]
fn spin_into_myth_on_a_token_fateseals_the_top_cards() {
    cr!("111.6", "701.29a");
    supported("Spin into Myth");
    let mut t = TestGame::new(2);
    empty_library(&mut t, P1);
    let a = t.library_top(P1, "Lightning Bolt");
    let b = t.library_top(P1, "Shock");
    let token = crate::r_s02_common::create_token(&mut t, P1, "Soldier");
    let from = t.asked().len();
    cast(&mut t, P0, "Spin into Myth", &[&[o(token)]]);
    t.resolve_all();
    // P0 looked at the two cards (not the token).
    let seen: Vec<Vec<ObjectId>> = asked_since(&t, from)
        .into_iter()
        .filter_map(|(p, d)| match d {
            Decision::Scry { cards } if p == P0 => Some(cards),
            _ => None,
        })
        .collect();
    assert_eq!(seen.len(), 1);
    let mut got = seen[0].clone();
    got.sort();
    let mut want = vec![a, b];
    want.sort();
    assert_eq!(got, want);
}
