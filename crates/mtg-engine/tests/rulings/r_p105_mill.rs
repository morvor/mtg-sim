//! Rulings batch P105 — milling yourself and what's done with the milled cards: choices
//! made as the ability resolves (Cauldron's Gift, Tymaret Calls the Dead, Acolyte of
//! Affliction, Sudden Reclamation, Egon), cards put into a graveyard from a library or from
//! anywhere (Sidisi, Soulcipher Board, rad counters), graveyard-counting power and
//! toughness in every zone, values of X fixed or not, piles, reflexive triggers, empty
//! libraries, and Two-Headed Giant and Commander games.

use crate::r_p104_common::{mill, obj};
use crate::r_p105_common::*;
use crate::r_s01_common::*;
use crate::r_s02_common::destroy;
use crate::r_s05_common::{enter, move_to, tokens_with_subtype};
use crate::r_s06_common::attach_new;
use mtg_engine::decision::{Answer, Decision};
use mtg_engine::facedown::can_look_at;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

fn pl(p: PlayerId) -> Entity {
    Entity::Player(p)
}

/// Whether any decision since `from` was of the given kind.
fn asked_any(t: &TestGame, from: usize, f: fn(&Decision) -> bool) -> bool {
    t.asked()[from..].iter().any(|(_, d)| f(d))
}

// ---------------------------------------------------------------------------
// Two-Headed Giant: Valakut Exploration
// ---------------------------------------------------------------------------

#[test]
fn valakut_exploration_damages_each_opposing_head_in_two_headed_giant() {
    cr!("810.9", "120.3a");
    ruling!(
        "Valakut Exploration",
        "In a Two-Headed Giant game, the end step triggered ability causes the opposing team to lose 2 life for each card put into its owner's graveyard."
    );
    supported("Valakut Exploration");
    let mut t = crate::r_s33_common::two_headed_giant();
    t.battlefield(P0, "Valakut Exploration");
    let before = t.life(P2);
    for _ in 0..2 {
        enter(&mut t, P0, "Forest");
        t.resolve_all();
    }
    t.advance_to(P0, Step::End);
    t.resolve_all();
    // Two cards: each opponent is dealt 2 damage; the team loses 4.
    assert_eq!(t.life(P2), before - 4);
    assert_eq!(t.life(P3), before - 4);
}

// ---------------------------------------------------------------------------
// Madame Web, Clairvoyant: "You may look at the top card of your library any time."
// ---------------------------------------------------------------------------

#[test]
fn madame_web_looks_at_the_top_card_any_time() {
    cr!("401.5");
    ruling!(
        "Madame Web, Clairvoyant",
        "Madame Web lets you look at the top card of your library whenever you want (with one restriction—see below), even if you don't have priority. This action doesn't use the stack. Knowing what that card is becomes part of the information you have access to, just like you can look at the cards in your hand."
    );
    supported("Madame Web, Clairvoyant");
    let mut t = TestGame::new(2);
    let top = t.library_top(P0, "Grizzly Bears");
    assert!(!can_look_at(&t.g, P0, top));
    t.battlefield(P0, "Madame Web, Clairvoyant");
    t.g.recompute();
    t.set_step(P1, Step::DeclareAttackers);
    assert_eq!(t.g.turn.priority, Some(P1));
    assert!(can_look_at(&t.g, P0, top));
    assert!(!can_look_at(&t.g, P1, top));
    assert_eq!(t.stack_len(), 0);
}

// ---------------------------------------------------------------------------
// Choices made while the ability resolves can use the cards just milled
// ---------------------------------------------------------------------------

#[test]
fn cauldrons_gift_mills_and_returns_with_no_actions_in_between() {
    cr!("608.2c", "701.17a");
    ruling!(
        "Cauldron's Gift",
        "No player may take actions between the time you put cards into your graveyard, the time you choose a creature card to return, and the time you return that card to the battlefield."
    );
    supported("Cauldron's Gift");
    let mut t = TestGame::new(2);
    let lib = stack_library(&mut t, P0, &["Island", "Hill Giant", "Forest", "Island"]);
    t.lands(P0, "Swamp", 5);
    let gift = t.hand(P0, "Cauldron's Gift");
    t.cast(P0, gift).go();
    t.answer_yes(P0, true);
    t.answer_choose_moved(P0, &[obj(lib[1])]);
    let from = t.asked().len();
    t.resolve();
    // The milled Hill Giant was chosen and returned during the resolution; no player got
    // priority in between.
    assert!(t.on_battlefield(lib[1]));
    assert_eq!(t.counters(lib[1], counters::PLUS1), 1);
    assert!(!asked_any(&t, from, |d| matches!(d, Decision::Priority { .. })));
}

#[test]
fn milled_cards_can_be_the_ones_exiled_or_returned() {
    cr!("608.2c", "701.17a");
    ruling!(
        "Tymaret Calls the Dead",
        "The card that you exile for the first two chapter abilities may be one of the cards that you just put into your graveyard."
    );
    ruling!(
        "Acolyte of Affliction",
        "The card you return with Acolyte of Affliction's ability may be one that you just put into your graveyard with its ability. This is because the ability doesn't target the card that you'll return."
    );
    ruling!(
        "Sudden Reclamation",
        "You choose which creature card and land card to return to your hand as Sudden Reclamation resolves. This doesn’t target either card."
    );
    supported("Tymaret Calls the Dead");
    supported("Acolyte of Affliction");
    supported("Sudden Reclamation");
    // Tymaret Calls the Dead, chapter I: "Mill three cards. Then you may exile a creature
    // or enchantment card from your graveyard. If you do, create a 2/2 black Zombie
    // creature token."
    let mut t = TestGame::new(2);
    let lib = stack_library(&mut t, P0, &["Island", "Grizzly Bears", "Forest"]);
    t.answer_yes(P0, true);
    t.answer_choose_moved(P0, &[obj(lib[1])]);
    enter(&mut t, P0, "Tymaret Calls the Dead");
    t.resolve_all();
    assert_eq!(t.zone(lib[1]), Zone::Exile);
    assert_eq!(tokens_with_subtype(&t, P0, "Zombie").len(), 1);
    // Acolyte of Affliction: "When this creature enters, mill two cards, then you may
    // return a permanent card from your graveyard to your hand."
    let mut t = TestGame::new(2);
    let lib = stack_library(&mut t, P0, &["Grizzly Bears", "Lightning Bolt"]);
    t.answer_yes(P0, true);
    t.answer_choose_moved(P0, &[obj(lib[0])]);
    let from = t.asked().len();
    enter(&mut t, P0, "Acolyte of Affliction");
    t.resolve_all();
    assert_eq!(t.zone(lib[0]), Zone::Hand(P0));
    assert!(!asked_any(&t, from, |d| matches!(d, Decision::ChooseTargets { .. })));
    // Sudden Reclamation: "Mill four cards, then return a creature card and a land card
    // from your graveyard to your hand."
    let mut t = TestGame::new(2);
    let lib = stack_library(&mut t, P0, &["Grizzly Bears", "Island", "Hill Giant", "Forest"]);
    t.graveyard(P0, "Mountain");
    t.lands(P0, "Forest", 4);
    let sr = t.hand(P0, "Sudden Reclamation");
    let from = t.asked().len();
    t.cast(P0, sr).go();
    assert!(!asked_any(&t, from, |d| matches!(d, Decision::ChooseTargets { .. })));
    t.answer_choose_moved(P0, &[obj(lib[2])]);
    t.answer_choose_moved(P0, &[obj(lib[3])]);
    t.resolve_all();
    assert_eq!(t.zone(lib[2]), Zone::Hand(P0));
    assert_eq!(t.zone(lib[3]), Zone::Hand(P0));
}

#[test]
fn tymaret_exiles_only_one_card_for_one_zombie() {
    cr!("608.2c");
    ruling!(
        "Tymaret Calls the Dead",
        "While resolving the first two chapter abilities, you can’t exile multiple cards to create multiple Zombies."
    );
    let mut t = TestGame::new(2);
    let lib = stack_library(&mut t, P0, &["Grizzly Bears", "Hill Giant", "Pacifism"]);
    t.answer_yes(P0, true);
    t.answer_choose_moved(P0, &[obj(lib[0]), obj(lib[1]), obj(lib[2])]);
    enter(&mut t, P0, "Tymaret Calls the Dead");
    t.resolve_all();
    let exiled = lib.iter().filter(|c| t.zone(**c) == Zone::Exile).count();
    assert_eq!(exiled, 1);
    assert_eq!(tokens_with_subtype(&t, P0, "Zombie").len(), 1);
}

#[test]
fn egon_must_exile_two_cards_when_it_can() {
    cr!("608.2c", "603.2");
    ruling!(
        "Egon, God of Death // Throne of Death",
        "You choose which cards you're exiling as Egon's triggered ability is resolving. If there are two or more cards in your graveyard at that time, you must exile two of them. You can't choose not to exile cards and sacrifice Egon instead to draw a card."
    );
    supported("Egon, God of Death // Throne of Death");
    // P0 chooses the two cards as the ability resolves; answering with no cards (trying not
    // to exile any) isn't a choice the ability offers: two are exiled all the same, and
    // Egon stays.
    for answer in [vec![0usize, 2], vec![]] {
        let mut t = TestGame::new(2);
        let egon = t.battlefield(P0, "Egon, God of Death // Throne of Death");
        let gy: Vec<ObjectId> = ["Island", "Forest", "Grizzly Bears"]
            .iter()
            .map(|n| t.graveyard(P0, n))
            .collect();
        t.set_step(P1, Step::End);
        t.advance_to(P0, Step::Upkeep);
        let hand = t.hand_size(P0);
        let chosen: Vec<Entity> = answer.iter().map(|i| obj(gy[*i])).collect();
        t.answer_choose(P0, &chosen);
        let from = t.asked().len();
        t.resolve_all();
        // The choice was made as the ability resolved, and it was of exactly two cards.
        assert!(asked_any(&t, from, |d| matches!(
            d,
            Decision::ChooseEntities { min: 2, max: 2, .. }
        )));
        assert!(t.on_battlefield(egon), "{answer:?}");
        let exiled: Vec<usize> = (0..3).filter(|i| t.zone(gy[*i]) == Zone::Exile).collect();
        assert_eq!(exiled.len(), 2, "{answer:?}");
        if !answer.is_empty() {
            assert_eq!(exiled, answer);
        }
        assert_eq!(t.hand_size(P0), hand, "{answer:?}");
    }
}

// ---------------------------------------------------------------------------
// Cards put into a graveyard from a library, or from anywhere
// ---------------------------------------------------------------------------

#[test]
fn sidisi_triggers_for_creature_cards_from_anywhere_in_the_library() {
    cr!("603.2", "701.23a");
    ruling!(
        "Sidisi, Brood Tyrant",
        "Sidisi's last ability triggers any time creature cards are put into your graveyard from anywhere in your library, not just from the top."
    );
    supported("Sidisi, Brood Tyrant");
    supported("Entomb");
    // Entomb ({B}): "Search your library for a card, put that card into your graveyard,
    // then shuffle." The Hill Giant is deep in the library.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Sidisi, Brood Tyrant");
    let giant = t.library_top(P0, "Hill Giant");
    t.library_top(P0, "Island");
    t.library_top(P0, "Island");
    t.lands(P0, "Swamp", 1);
    let entomb = t.hand(P0, "Entomb");
    t.cast(P0, entomb).go();
    t.answer_choose(P0, &[obj(giant)]);
    t.resolve_all();
    assert_eq!(t.zone(giant), Zone::Graveyard(P0));
    assert_eq!(tokens_with_subtype(&t, P0, "Zombie").len(), 1);
}

/// P0's Soulcipher Board (with its three omen counters).
fn soulcipher(t: &mut TestGame) -> ObjectId {
    supported("Soulcipher Board // Cipherbound Spirit");
    let b = t.battlefield(P0, "Soulcipher Board // Cipherbound Spirit");
    set_counters(t, b, "omen", 3);
    b
}

#[test]
fn soulcipher_board_counts_creature_cards_not_creatures_dying() {
    cr!("603.2", "603.6c", "603.10", "700.4", "111.1");
    ruling!(
        "Soulcipher Board // Cipherbound Spirit",
        "Soulcipher Board's last ability triggers when a creature card is put into a graveyard from anywhere. This is different than an ability that triggers when a creature dies. Notably, a noncreature card on the battlefield that becomes a creature and dies (such as an animated land) won't cause this ability to trigger. Similarly, a creature card on the battlefield that has been made a noncreature permanent (such as one enchanted with Minimus Containment) will cause this ability to trigger when that card is put into your graveyard."
    );
    ruling!(
        "Soulcipher Board // Cipherbound Spirit",
        "Token creatures dying never cause Soulcipher Board's last ability to trigger."
    );
    supported("Minimus Containment");
    supported("Mutavault");
    // A creature card discarded from the hand: triggers.
    let mut t = TestGame::new(2);
    let b = soulcipher(&mut t);
    let bears = t.hand(P0, "Grizzly Bears");
    t.g.discard(P0, bears, None);
    t.g.flush_events();
    t.resolve_all();
    assert_eq!(t.counters(b, "omen"), 2);
    // An animated land (Mutavault, "{1}: Until end of turn, this land becomes a 2/2
    // creature with all creature types") dies: no trigger.
    let mut t = TestGame::new(2);
    let b = soulcipher(&mut t);
    let vault = t.battlefield(P0, "Mutavault");
    t.lands(P0, "Wastes", 1);
    crate::r_s06_common::activate_containing(&mut t, P0, vault, "becomes a 2/2").unwrap();
    t.resolve_all();
    assert!(t.obj_now(vault).is(CardType::Creature));
    destroy(&mut t, vault);
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Mutavault"));
    assert_eq!(t.counters(b, "omen"), 3);
    // A creature card made a noncreature Treasure by Minimus Containment: triggers.
    let mut t = TestGame::new(2);
    let b = soulcipher(&mut t);
    let giant = t.battlefield(P0, "Hill Giant");
    attach_new(&mut t, P1, "Minimus Containment", obj(giant));
    assert!(!t.obj_now(giant).is(CardType::Creature));
    destroy(&mut t, giant);
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Hill Giant"));
    assert_eq!(t.counters(b, "omen"), 2);
    // A creature token dies: no trigger.
    let mut t = TestGame::new(2);
    let b = soulcipher(&mut t);
    let token = crate::r_s02_common::create_token(&mut t, P0, "Soldier");
    destroy(&mut t, token);
    t.resolve_all();
    assert_eq!(t.counters(b, "omen"), 3);
}

#[test]
fn rad_counter_mills_are_one_batch() {
    cr!("728.1", "701.17a", "603.2c");
    ruling!(
        "Nightkin Ambusher",
        "The cards are milled all at once, which means abilities that trigger \"whenever one or more nonland cards are milled\" will trigger exactly once as long as at least one nonland card was milled."
    );
    supported("Nightkin Ambusher");
    supported("The Wise Mothman");
    // P1's Nightkin Ambusher enters: "When this creature enters, target player gets four
    // rad counters." P1 targets P0. At the beginning of P0's precombat main phase, P0 mills
    // four cards, two of them nonland. P1's The Wise Mothman ("Whenever one or more nonland
    // cards are milled, put a +1/+1 counter on each of up to X target creatures, where X is
    // the number of nonland cards milled this way") triggers once.
    let mut t = TestGame::new(2);
    t.battlefield(P1, "The Wise Mothman");
    let a = t.battlefield(P1, "Grizzly Bears");
    let b = t.battlefield(P1, "Grizzly Bears");
    stack_library(
        &mut t,
        P0,
        &["Hill Giant", "Lightning Bolt", "Island", "Forest"],
    );
    t.answer_targets(P1, &[pl(P0)]);
    enter(&mut t, P1, "Nightkin Ambusher");
    t.resolve_all();
    assert_eq!(t.g.player(P0).counters.get("rad").copied(), Some(4));
    t.answer_targets(P1, &[obj(a), obj(b)]);
    t.set_step(P1, Step::End);
    t.advance_to(P0, Step::PrecombatMain);
    t.settle();
    assert_eq!(crate::r_s01_common::triggers_on_stack(&t, "nonland cards are milled"), 0);
    t.resolve(); // the rad counters' trigger: mill four
    assert_eq!(t.graveyard_size(P0), 4);
    assert_eq!(crate::r_s01_common::triggers_on_stack(&t, "nonland cards are milled"), 1);
    t.resolve_all();
    assert_eq!(t.counters(a, counters::PLUS1), 1);
    assert_eq!(t.counters(b, counters::PLUS1), 1);
    // Two nonland cards: P0 lost 2 life and two rad counters.
    assert_eq!(t.life(P0), 18);
    assert_eq!(t.g.player(P0).counters.get("rad").copied(), Some(2));
}

// ---------------------------------------------------------------------------
// Power and toughness defined in every zone
// ---------------------------------------------------------------------------

#[test]
fn graveyard_counting_power_and_toughness_works_in_every_zone() {
    cr!("604.3", "611.3a", "208.2a");
    ruling!(
        "Lumra, Bellow of the Woods",
        "The ability that defines Lumra's power and toughness functions in all zones, not just the battlefield."
    );
    ruling!(
        "Splinterfright",
        "The ability that defines Splinterfright’s power and toughness works in all zones, not just the battlefield. If Splinterfright is in your graveyard, it will count itself."
    );
    ruling!(
        "Urborg Lhurgoyf",
        "The ability that defines Urborg Lhurgoyf's power and toughness works in all zones, not just on the battlefield."
    );
    ruling!(
        "Old Stickfingers",
        "The ability that defines the power and toughness of Old Stickfingers functions in all zones, not only while Old Stickfingers is on the battlefield. While Old Stickfingers is in a graveyard, the ability will count Old Stickfingers itself, presumably using a single, withered stickfinger."
    );
    for n in [
        "Lumra, Bellow of the Woods",
        "Splinterfright",
        "Urborg Lhurgoyf",
        "Old Stickfingers",
    ] {
        supported(n);
    }
    // P0 controls three lands and has two creature cards in the graveyard.
    // (card, zone, expected power and toughness)
    let cases: [(&str, Zone, (i32, i32)); 7] = [
        ("Lumra, Bellow of the Woods", Zone::Hand(P0), (3, 3)),
        ("Lumra, Bellow of the Woods", Zone::Graveyard(P0), (3, 3)),
        ("Splinterfright", Zone::Hand(P0), (2, 2)),
        // In the graveyard it counts itself.
        ("Splinterfright", Zone::Graveyard(P0), (3, 3)),
        ("Urborg Lhurgoyf", Zone::Library(P0), (2, 3)),
        ("Old Stickfingers", Zone::Hand(P0), (2, 2)),
        ("Old Stickfingers", Zone::Graveyard(P0), (3, 3)),
    ];
    for (name, zone, pt) in cases {
        let mut t = TestGame::new(2);
        t.lands(P0, "Forest", 3);
        t.graveyard(P0, "Grizzly Bears");
        t.graveyard(P0, "Hill Giant");
        t.graveyard(P0, "Island");
        let id = place(&mut t, P0, name, zone);
        t.g.recompute();
        let o = t.obj_now(id);
        assert_eq!((o.power(), o.toughness()), pt, "{name} in {zone:?}");
    }
}

// ---------------------------------------------------------------------------
// Values fixed as an ability resolves, or continuously updated
// ---------------------------------------------------------------------------

#[test]
fn x_is_locked_in_as_the_ability_resolves() {
    cr!("608.2h", "611.2c");
    ruling!(
        "Summon: Titan",
        "The value of X in Summon: Titan's third chapter ability is calculated only once, as the ability resolves."
    );
    ruling!(
        "Siegfried, Famed Swordsman",
        "The value of X is calculated only once, as Siegfried's last ability resolves."
    );
    supported("Summon: Titan");
    supported("Siegfried, Famed Swordsman");
    // Summon: Titan, chapter III: "Until end of turn, another target creature you control
    // gains trample and gets +X/+X, where X is the number of lands you control."
    let mut t = TestGame::new(2);
    let titan = t.battlefield(P0, "Summon: Titan");
    set_counters(&mut t, titan, counters::LORE, 2);
    t.lands(P0, "Forest", 3);
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.answer_targets(P0, &[obj(bears)]);
    crate::r_s19_common::add_lore(&mut t, titan, 1);
    t.resolve_all();
    assert_eq!(t.pt(bears), (5, 5));
    t.lands(P0, "Forest", 2);
    t.g.recompute();
    assert_eq!(t.pt(bears), (5, 5));
    // Siegfried: "When Siegfried enters, mill three cards. Then put X +1/+1 counters on
    // Siegfried, where X is twice the number of creature cards in your graveyard."
    let mut t = TestGame::new(2);
    stack_library(&mut t, P0, &["Grizzly Bears", "Island", "Island"]);
    let s = enter(&mut t, P0, "Siegfried, Famed Swordsman");
    t.resolve_all();
    assert_eq!(t.counters(s, counters::PLUS1), 2);
    t.graveyard(P0, "Hill Giant");
    t.g.recompute();
    assert_eq!(t.counters(s, counters::PLUS1), 2);
}

#[test]
fn song_of_stupefaction_x_changes_with_the_graveyard() {
    cr!("611.3a", "613.4c");
    ruling!(
        "Song of Stupefaction",
        "The value of X will change as the number of permanent cards in your graveyard changes."
    );
    supported("Song of Stupefaction");
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P1, "Hill Giant");
    attach_new(&mut t, P0, "Song of Stupefaction", obj(giant));
    t.graveyard(P0, "Grizzly Bears");
    t.g.recompute();
    assert_eq!(t.pt(giant), (2, 3));
    t.graveyard(P0, "Island");
    t.graveyard(P0, "Lightning Bolt");
    t.g.recompute();
    assert_eq!(t.pt(giant), (1, 3));
    let gy = t.g.player(P0).graveyard.clone();
    for c in gy {
        move_to(&mut t, c, Zone::Exile);
    }
    t.g.recompute();
    assert_eq!(t.pt(giant), (3, 3));
}

#[test]
fn drown_in_filth_targets_on_cast_and_counts_on_resolution() {
    cr!("601.2c", "608.2h");
    ruling!(
        "Drown in Filth",
        "You choose the target of Drown in Filth as you cast the spell. You won’t necessarily know what will happen to that creature’s power and toughness at that time."
    );
    supported("Drown in Filth");
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P1, "Hill Giant");
    stack_library(&mut t, P0, &["Island", "Forest", "Grizzly Bears", "Swamp"]);
    t.lands(P0, "Swamp", 1);
    t.lands(P0, "Forest", 1);
    let c = t.hand(P0, "Drown in Filth");
    let from = t.asked().len();
    t.cast(P0, c).target(giant).go();
    // The target was chosen while casting, before any card was milled.
    assert!(asked_any(&t, from, |d| matches!(d, Decision::ChooseTargets { .. })));
    assert_eq!(t.graveyard_size(P0), 0);
    t.resolve_all();
    // Three lands milled: -3/-3.
    assert!(!t.on_battlefield(giant));
}

// ---------------------------------------------------------------------------
// Countryside Crusher, Crow of Dark Tidings, Curator Beastie, Candles of Leng
// ---------------------------------------------------------------------------

#[test]
fn countryside_crusher_stops_at_the_nonland_card_which_stays_on_top() {
    cr!("701.20a", "608.2c");
    ruling!(
        "Countryside Crusher",
        "The first ability will keep putting cards from the top of your library into your graveyard until you reveal a nonland card. That nonland card remains on top of your library."
    );
    supported("Countryside Crusher");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Countryside Crusher");
    let lib = stack_library(&mut t, P0, &["Forest", "Island", "Grizzly Bears", "Swamp"]);
    t.set_step(P1, Step::End);
    t.advance_to(P0, Step::Upkeep);
    t.resolve_all();
    assert_eq!(t.zone(lib[0]), Zone::Graveyard(P0));
    assert_eq!(t.zone(lib[1]), Zone::Graveyard(P0));
    assert_eq!(t.g.library_top(P0), Some(lib[2]));
    assert_eq!(t.zone(lib[3]), Zone::Library(P0));
}

#[test]
fn crow_of_dark_tidings_triggers_on_entering_and_on_dying() {
    cr!("603.6a", "603.10a", "700.4");
    ruling!(
        "Crow of Dark Tidings",
        "The triggered ability triggers both when Crow of Dark Tidings enters the battlefield and when it dies. You don't have to choose only one."
    );
    supported("Crow of Dark Tidings");
    let mut t = TestGame::new(2);
    let crow = enter(&mut t, P0, "Crow of Dark Tidings");
    t.resolve_all();
    assert_eq!(t.graveyard_size(P0), 2);
    destroy(&mut t, crow);
    t.resolve_all();
    // The Crow and four milled cards.
    assert_eq!(t.graveyard_size(P0), 5);
}

#[test]
fn curator_beastie_gives_counters_to_its_manifested_dread() {
    cr!("701.62a", "708.2a", "614.1c");
    ruling!(
        "Curator Beastie",
        "The face-down card put onto the battlefield by Curator Beastie's last ability is a colorless creature you control entering the battlefield (unless an effect gives it a color) so it will enter with +1/+1 counters if Curator Beastie is still on the battlefield."
    );
    supported("Curator Beastie");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Curator Beastie");
    let lib = stack_library(&mut t, P0, &["Grizzly Bears", "Island"]);
    t.answer_choose(P0, &[obj(lib[0])]);
    t.set_step(P0, Step::BeginningOfCombat);
    let beastie = t.named_on_battlefield("Curator Beastie")[0];
    crate::r_s01_common::attack_with(&mut t, &[(beastie, pl(P1))]);
    t.resolve_all();
    let fd: Vec<ObjectId> = t.g.permanents().filter(|o| o.face_down).map(|o| o.id).collect();
    assert_eq!(fd.len(), 1);
    assert_eq!(t.counters(fd[0], counters::PLUS1), 2);
    assert_eq!(t.pt(fd[0]), (4, 4));
}

#[test]
fn candles_of_leng_draws_the_revealed_card() {
    cr!("701.20a", "121.1");
    ruling!(
        "Candles of Leng",
        "Unless something weird happens, the card that’s drawn will be the card that was revealed."
    );
    supported("Candles of Leng");
    let mut t = TestGame::new(2);
    let candles = t.battlefield(P0, "Candles of Leng");
    t.lands(P0, "Wastes", 4);
    let top = t.library_top(P0, "Grizzly Bears");
    t.activate(P0, candles, 0, &[]).unwrap();
    t.resolve_all();
    assert_eq!(t.zone(top), Zone::Hand(P0));
}

#[test]
fn morality_shift_lets_its_owner_order_the_new_graveyard() {
    cr!("404.3", "701.12a");
    ruling!(
        "Morality Shift",
        "You can choose the order of the cards being placed into the graveyard."
    );
    supported("Morality Shift");
    let mut results = vec![];
    for order in [vec![0usize, 1, 2], vec![2, 1, 0]] {
        let mut t = TestGame::new(2);
        crate::r_s11_common::empty_library(&mut t, P0);
        stack_library(&mut t, P0, &["Grizzly Bears", "Hill Giant", "Island"]);
        t.graveyard(P0, "Forest");
        t.lands(P0, "Swamp", 7);
        let c = t.hand(P0, "Morality Shift");
        t.cast(P0, c).go();
        t.answer(P0, DecisionKind::Order, Answer::Indices(order.clone()));
        let from = t.asked().len();
        t.resolve_all();
        assert!(t.asked()[from..]
            .iter()
            .any(|(p, d)| *p == P0 && matches!(d, Decision::Order { .. })));
        let gy = gy_names(&t, P0);
        assert_eq!(gy.len(), 4, "{gy:?}");
        assert_eq!(gy[3], "Morality Shift");
        assert_eq!(t.library_size(P0), 1);
        results.push(gy[..3].to_vec());
    }
    // P0's choice of order decided the graveyard's order.
    assert_ne!(results[0], results[1]);
    let mut a = results[0].clone();
    let mut b = results[1].clone();
    a.sort();
    b.sort();
    assert_eq!(a, b);
}

// ---------------------------------------------------------------------------
// Piles
// ---------------------------------------------------------------------------

#[test]
fn piles_can_be_split_all_and_nothing() {
    cr!("700.3d", "700.3a");
    ruling!(
        "Atris, Oracle of Half-Truths",
        "The opponent may split the cards into one pile of three and one pile of zero. The pile of three cards could be the face-up pile or the face-down pile."
    );
    ruling!(
        "Curator of Destinies",
        "You may split the cards into one pile of five and one pile of zero. The pile of five cards could be the face-up pile or the face-down pile. The opponent may choose the empty pile to be put into your hand."
    );
    supported("Atris, Oracle of Half-Truths");
    supported("Curator of Destinies");
    // (card, cards, separator, chooser): all face down or all face up, and the chooser
    // takes either pile.
    let cases = [
        ("Atris, Oracle of Half-Truths", 3usize, P1, P0),
        ("Curator of Destinies", 5, P0, P1),
    ];
    for (name, n, separator, chooser) in cases {
        let mut outcomes = vec![];
        for all_face_down in [true, false] {
            for pick in [0usize, 1] {
                let mut t = TestGame::new(2);
                let names = ["Grizzly Bears", "Island", "Hill Giant", "Shock", "Forest"];
                let lib = stack_library(&mut t, P0, &names[..n]);
                let down: Vec<Entity> = if all_face_down {
                    lib.iter().map(|c| obj(*c)).collect()
                } else {
                    vec![]
                };
                t.answer_choose(separator, &down);
                t.answer(chooser, DecisionKind::Option, Answer::Index(pick));
                if name == "Atris, Oracle of Half-Truths" {
                    t.answer_targets(P0, &[pl(P1)]);
                }
                let hand = t.hand_size(P0);
                enter(&mut t, P0, name);
                t.resolve_all();
                let to_hand = t.hand_size(P0) - hand;
                let to_gy = lib
                    .iter()
                    .filter(|c| t.zone(**c) == Zone::Graveyard(P0))
                    .count();
                assert_eq!(to_hand + to_gy, n, "{name}");
                outcomes.push(to_hand);
            }
        }
        // Every split and pick happened: all the cards to the hand, or none of them.
        outcomes.sort();
        outcomes.dedup();
        assert_eq!(outcomes, vec![0, n], "{name}");
    }
}

#[test]
fn only_the_target_opponent_looks_at_and_separates_the_cards() {
    cr!("700.3a", "115.1");
    ruling!(
        "Atris, Oracle of Half-Truths",
        "You choose just one target opponent to look at the top three cards of your library and put them into piles. Other players can't look, but they may attempt to offer advice without knowing what cards are there."
    );
    // P0 targets P2 of three players: P2 separates the cards, looking at them; neither P0
    // nor P1 can see them meanwhile.
    let mut t = TestGame::new(3);
    let lib = stack_library(&mut t, P0, &["Grizzly Bears", "Island", "Hill Giant"]);
    let seen = crate::r_s01_common::watch(
        &mut t,
        P2,
        |d| matches!(d, Decision::ChooseEntities { .. }),
        |g| {
            let top: Vec<ObjectId> = g.players[0].library.iter().rev().take(3).copied().collect();
            let looks = |p| top.iter().all(|c| can_look_at(g, p, *c));
            (looks(P0), looks(P1), looks(P2))
        },
    );
    t.answer_targets(P0, &[pl(P2)]);
    t.answer_choose(P2, &[obj(lib[0])]);
    let from = t.asked().len();
    enter(&mut t, P0, "Atris, Oracle of Half-Truths");
    t.resolve_all();
    // Only P2 separates the cards, and its decision shows it the three cards.
    let separations: Vec<(PlayerId, Vec<Entity>)> = t.asked()[from..]
        .iter()
        .filter_map(|(p, d)| match d {
            Decision::ChooseEntities { candidates, .. } => Some((*p, candidates.clone())),
            _ => None,
        })
        .collect();
    assert_eq!(separations.len(), 1);
    assert_eq!(separations[0].0, P2);
    let mut shown = separations[0].1.clone();
    shown.sort_by_key(|e| format!("{e:?}"));
    let mut top: Vec<Entity> = lib.iter().map(|c| obj(*c)).collect();
    top.sort_by_key(|e| format!("{e:?}"));
    assert_eq!(shown, top);
    let seen = seen.lock().unwrap().clone();
    assert!(!seen.is_empty());
    assert!(seen.iter().all(|(p0, p1, _)| !p0 && !p1), "{seen:?}");
}

#[test]
fn fortunes_favor_only_the_target_opponent_sees_the_cards_in_multiplayer() {
    cr!("700.3a", "115.1", "401.2");
    ruling!(
        "Fortune's Favor",
        "In a multiplayer game, the targeted opponent may choose what information to share with others if that player wants advice on how to split the cards, but that player doesn’t have to share any information at all."
    );
    supported("Fortune's Favor");
    // P0 targets P2 of three players: P2 is shown the four cards to separate them; P1 (the
    // other opponent) and P0 can't see them meanwhile, so what they learn is up to P2.
    let mut t = TestGame::new(3);
    let lib = stack_library(&mut t, P0, &["Grizzly Bears", "Island", "Hill Giant", "Shock"]);
    t.lands(P0, "Island", 1);
    t.lands(P0, "Wastes", 3);
    let seen = crate::r_s01_common::watch(
        &mut t,
        P2,
        |d| matches!(d, Decision::ChooseEntities { .. }),
        |g| {
            let top: Vec<ObjectId> = g.players[0].library.iter().rev().take(4).copied().collect();
            let looks = |p| top.iter().any(|c| can_look_at(g, p, *c));
            (looks(P0), looks(P1))
        },
    );
    t.answer_choose(P2, &[obj(lib[0])]);
    let s = t.hand(P0, "Fortune's Favor");
    let from = t.asked().len();
    t.cast(P0, s).target(pl(P2)).go();
    t.resolve_all();
    let separations: Vec<(PlayerId, Vec<Entity>)> = t.asked()[from..]
        .iter()
        .filter_map(|(p, d)| match d {
            Decision::ChooseEntities { candidates, .. } => Some((*p, candidates.clone())),
            _ => None,
        })
        .collect();
    assert_eq!(separations.len(), 1);
    assert_eq!(separations[0].0, P2);
    assert_eq!(separations[0].1.len(), 4);
    assert!(lib.iter().all(|c| separations[0].1.contains(&obj(*c))));
    assert_eq!(*seen.lock().unwrap(), vec![(false, false)]);
    // The Grizzly Bears (the face-down pile) and the rest went to P0's hand and graveyard.
    assert!(lib
        .iter()
        .all(|c| matches!(t.zone(*c), Zone::Hand(P0) | Zone::Graveyard(P0))));
}

#[test]
fn fortunes_favor_caster_chooses_the_pile_without_seeing_the_face_down_one() {
    cr!("700.3a", "401.2");
    ruling!(
        "Fortune's Favor",
        "You choose which pile to put into your hand and which to put into your graveyard. You can’t look at the face-down pile until after it’s moving to its new zone."
    );
    supported("Fortune's Favor");
    let mut t = TestGame::new(2);
    let lib = stack_library(&mut t, P0, &["Grizzly Bears", "Island", "Hill Giant", "Shock"]);
    t.lands(P0, "Island", 1);
    t.lands(P0, "Wastes", 3);
    let seen = crate::r_s01_common::watch(
        &mut t,
        P0,
        |d| matches!(d, Decision::ChooseOption { .. }),
        |g| {
            // The face-down pile (the Grizzly Bears) and the face-up pile.
            let look = |name: &str| {
                g.objects
                    .iter()
                    .filter(|o| g.is_live(o.id) && o.owner == P0 && o.chars.name == name)
                    .any(|o| can_look_at(g, P0, o.id))
            };
            (look("Grizzly Bears"), look("Island"))
        },
    );
    // P1 puts Grizzly Bears alone in the face-down pile; P0 picks the face-down pile.
    t.answer_choose(P1, &[obj(lib[0])]);
    let s = t.hand(P0, "Fortune's Favor");
    t.cast(P0, s).target(pl(P1)).go();
    let from = t.asked().len();
    t.resolve_all();
    let choosers: Vec<PlayerId> = t.asked()[from..]
        .iter()
        .filter(|(_, d)| matches!(d, Decision::ChooseOption { .. }))
        .map(|(p, _)| *p)
        .collect();
    assert_eq!(choosers, vec![P0]);
    assert_eq!(*seen.lock().unwrap(), vec![(false, true)]);
}

#[test]
fn curator_of_destinies_controller_picks_the_choosing_opponent() {
    cr!("700.3a", "806.1");
    interp!("controller-picks-the-choosing-opponent");
    ruling!(
        "Curator of Destinies",
        "You decide which opponent chooses the pile while resolving Curator of Destinies's last ability."
    );
    for opp in [P1, P2] {
        let mut t = TestGame::new(3);
        let lib = stack_library(
            &mut t,
            P0,
            &["Grizzly Bears", "Island", "Hill Giant", "Shock", "Forest"],
        );
        // P0 separates the cards (Grizzly Bears face down), then picks the opponent.
        t.answer_choose(P0, &[obj(lib[0])]);
        t.answer_choose(P0, &[pl(opp)]);
        let from = t.asked().len();
        enter(&mut t, P0, "Curator of Destinies");
        t.resolve_all();
        let pickers: Vec<PlayerId> = t.asked()[from..]
            .iter()
            .filter(|(_, d)| matches!(d, Decision::ChooseOption { .. }))
            .map(|(p, _)| *p)
            .collect();
        assert_eq!(pickers, vec![opp]);
    }
}

// ---------------------------------------------------------------------------
// Reflexive triggered abilities
// ---------------------------------------------------------------------------

#[test]
fn reflexive_triggers_choose_targets_when_put_on_the_stack() {
    cr!("603.12", "603.3d");
    ruling!(
        "Paramecia Coloniex",
        "You don't choose a target for Paramecia Coloniex's last ability at the time it triggers. Rather, a second \"reflexive\" ability triggers when you exile Paramecia Coloniex this way. You choose a target for that ability as it goes on the stack. Each player may respond to this triggered ability as normal."
    );
    supported("Paramecia Coloniex");
    // "When this creature dies, you may exile it. When you do, put target creature card
    // from your graveyard on top of your library."
    let mut t = TestGame::new(2);
    let pc = t.battlefield(P0, "Paramecia Coloniex");
    let giant = t.graveyard(P0, "Hill Giant");
    let from = t.asked().len();
    destroy(&mut t, pc);
    assert_eq!(t.stack_len(), 1);
    assert!(!asked_any(&t, from, |d| matches!(d, Decision::ChooseTargets { .. })));
    t.answer_yes(P0, true);
    t.answer_targets(P0, &[obj(giant)]);
    t.resolve();
    // The reflexive trigger is on the stack, targeting the Hill Giant; P1 may respond.
    assert_eq!(t.stack_len(), 1);
    assert!(t.in_exile("Paramecia Coloniex"));
    assert!(asked_any(&t, from, |d| matches!(d, Decision::ChooseTargets { .. })));
    move_to(&mut t, giant, Zone::Exile);
    t.resolve_all();
    assert_ne!(t.g.library_top(P0), Some(t.g.current(giant)));
}

#[test]
fn terra_reflexive_trigger_targets_after_paying() {
    cr!("603.12", "603.3d");
    ruling!(
        "Terra, Herald of Hope",
        "You don't choose a target for Terra's last ability at the time it triggers. Rather, a second \"reflexive\" ability triggers when you pay {2} this way. You choose a target for that ability as it goes on the stack. Each player may respond to this triggered ability as normal."
    );
    supported("Terra, Herald of Hope");
    let mut t = TestGame::new(2);
    let terra = t.battlefield(P0, "Terra, Herald of Hope");
    t.lands(P0, "Wastes", 2);
    t.set_step(P0, Step::BeginningOfCombat);
    t.resolve_all();
    crate::r_s01_common::attack_with(&mut t, &[(terra, pl(P1))]);
    t.advance_to(P0, Step::CombatDamage);
    // The Grizzly Bears is put into the graveyard after Terra's trigger is on the stack:
    // it's still a legal target for the reflexive trigger.
    t.settle();
    assert_eq!(t.stack_len(), 1);
    let bears = t.graveyard(P0, "Grizzly Bears");
    t.answer_yes(P0, true);
    t.answer(P0, DecisionKind::OptionalCost, Answer::Bool(true));
    t.answer_targets(P0, &[obj(bears)]);
    t.resolve();
    assert_eq!(t.stack_len(), 1);
    t.resolve_all();
    assert!(t.on_battlefield(bears));
    assert!(t.obj_now(bears).tapped);
}

#[test]
fn awaken_the_honored_dead_may_return_the_card_it_just_discarded() {
    cr!("603.12", "603.3d", "714.2b");
    ruling!(
        "Awaken the Honored Dead",
        "You don’t choose a target for Awaken the Honored Dead’s final chapter ability at the time it triggers. Rather, a second “reflexive” ability triggers when you discard a card this way. You choose a target for that ability as it goes on the stack. You may choose the card you just discarded as the target or a card that was already in your graveyard. Each player may respond to this triggered ability as normal."
    );
    supported("Awaken the Honored Dead");
    let mut t = TestGame::new(2);
    let saga = t.battlefield(P0, "Awaken the Honored Dead");
    set_counters(&mut t, saga, counters::LORE, 2);
    let giant = t.hand(P0, "Hill Giant");
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[obj(giant)]);
    crate::r_s19_common::add_lore(&mut t, saga, 1);
    t.settle();
    let discarded = t.g.current(giant);
    t.answer_targets(P0, &[obj(discarded)]);
    t.resolve();
    assert_eq!(t.zone(giant), Zone::Graveyard(P0));
    t.resolve_all();
    assert_eq!(t.zone(giant), Zone::Hand(P0));
}

// ---------------------------------------------------------------------------
// Empty libraries
// ---------------------------------------------------------------------------

#[test]
fn an_empty_library_loses_only_on_drawing() {
    cr!("704.5b", "121.4", "701.17b");
    ruling!(
        "Liliana's Indignation",
        "You don’t lose the game for having zero cards in your library until you’re instructed to draw a card. X can be greater than or equal to the number of cards in your library without causing you to lose the game."
    );
    ruling!(
        "Leveler",
        "You won't lose the game until you try to draw from the empty library."
    );
    supported("Liliana's Indignation");
    supported("Leveler");
    // Liliana's Indignation with X = 5 and a two-card library.
    let mut t = TestGame::new(2);
    crate::r_s11_common::empty_library(&mut t, P0);
    stack_library(&mut t, P0, &["Grizzly Bears", "Island"]);
    t.lands(P0, "Swamp", 6);
    let c = t.hand(P0, "Liliana's Indignation");
    t.cast(P0, c).x(5).target(pl(P1)).go();
    t.resolve_all();
    assert_eq!(t.library_size(P0), 0);
    assert_eq!(t.life(P1), 18);
    assert!(!t.has_lost(P0));
    // Leveler: "When this creature enters, exile all cards from your library."
    let mut t = TestGame::new(2);
    enter(&mut t, P0, "Leveler");
    t.resolve_all();
    assert_eq!(t.library_size(P0), 0);
    assert!(!t.has_lost(P0));
    t.advance_to(P1, Step::Upkeep);
    assert!(!t.has_lost(P0));
    t.g.run_until(10_000, |g| g.players[0].has_lost || g.turn.number > 4);
    assert!(t.has_lost(P0));
}

// ---------------------------------------------------------------------------
// Commander: Sauron, Lord of the Rings
// ---------------------------------------------------------------------------

#[test]
fn sauron_triggers_when_a_commander_dies_and_goes_to_the_command_zone() {
    cr!("903.9a", "700.4", "603.10a");
    ruling!(
        "Sauron, Lord of the Rings",
        "Sauron, Lord of the Rings's last ability will trigger even if the owner of the commander that died chooses to return it to the command zone after it dies."
    );
    supported("Sauron, Lord of the Rings");
    let mut t = crate::r_s13_common::commander_game();
    t.battlefield(P0, "Sauron, Lord of the Rings");
    let cmd = crate::r_s13_common::commander(&mut t, P1, "Grizzly Bears");
    let bears = move_to(&mut t, cmd, Zone::Battlefield).unwrap();
    t.answer_yes(P1, true);
    destroy(&mut t, bears);
    t.resolve_all();
    assert_eq!(t.zone(bears), Zone::Command);
    assert_eq!(t.g.player(P0).ring_level, 1);
}
