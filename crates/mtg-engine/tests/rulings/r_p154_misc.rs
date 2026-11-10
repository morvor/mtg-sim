//! Rulings batch P154 — abilities that scale with a mana value, miscellany (first half):
//! Auras put onto the battlefield (CR 303.4f–g), animated artifacts keeping their types
//! (CR 205.1a), X chosen on the stack (CR 202.3e, 707.10), empty hands and libraries,
//! redirected damage whose new recipient left (CR 614.1, 608.2h), ties, forced
//! self-targets (CR 115.3), Phyrexian artifact creatures (CR 603.2c), Adventures (CR
//! 715), noted mana values (CR 607.2e), suspend with no counters (CR 702.62), players
//! leaving the game (CR 800.4a), multiplayer opponents (CR 102.2), and a copy that died
//! (CR 707.2, 400.7).

use crate::r_p154_common::*;
use mtg_engine::decision::Answer;
use mtg_engine::keywords::KeywordKind;
use mtg_engine::object::{CastMethod, Zone};
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

#[test]
fn starfield_of_nyx_returns_an_aura_choosing_what_it_enchants_without_targeting() {
    cr!("303.4f", "303.4g", "115.1");
    ruling!(
        "Starfield of Nyx",
        "If the first ability returns an Aura card to the battlefield, you choose what that Aura will enchant as it enters the battlefield. If the Aura can’t legally enchant anything, it stays in the graveyard."
    );
    ruling!(
        "Starfield of Nyx",
        "Note that if an Aura is returned to the battlefield this way, whatever the Aura enchants isn’t a target of Starfield of Nyx’s ability, nor is it a target of the Aura card itself. You could put an Aura onto the battlefield this way enchanting a creature with hexproof controlled by an opponent, for example."
    );
    supported("Starfield of Nyx");
    supported("Invisible Stalker");
    // Pacifism returns enchanting P1's hexproof Invisible Stalker.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Starfield of Nyx");
    let pac = t.graveyard(P0, "Pacifism");
    let stalker = t.battlefield(P1, "Invisible Stalker");
    t.answer_targets(P0, &[obj(pac)]);
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[obj(stalker)]);
    crate::r_s04_common::next_upkeep(&mut t, P0);
    t.resolve_all();
    assert!(t.on_battlefield(pac));
    assert_eq!(t.obj_now(pac).attached_to, Some(obj(stalker)));
    // No creature at all: Pacifism stays in the graveyard.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Starfield of Nyx");
    let pac = t.graveyard(P0, "Pacifism");
    t.answer_targets(P0, &[obj(pac)]);
    t.answer_yes(P0, true);
    crate::r_s04_common::next_upkeep(&mut t, P0);
    t.resolve_all();
    assert_eq!(t.zone(pac), Zone::Graveyard(P0));
}

#[test]
fn sydri_animates_an_artifact_keeping_its_other_types() {
    cr!("205.1a", "613.1d", "613.1g");
    ruling!(
        "Sydri, Galvanic Genius",
        "If the noncreature artifact had any other supertypes, types, or subtypes, it will keep those. In most cases, the artifact creature won’t have any creature types."
    );
    supported("Sydri, Galvanic Genius");
    let mut t = TestGame::new(2);
    let sydri = t.battlefield(P0, "Sydri, Galvanic Genius");
    let splitter = t.battlefield(P0, "Bonesplitter");
    mana(&mut t, P0, "U");
    act(&mut t, P0, sydri, "becomes an artifact creature", &[obj(splitter)]).expect("Sydri");
    t.resolve_all();
    let o = t.obj_now(splitter);
    assert!(o.is(CardType::Artifact) && o.is(CardType::Creature));
    assert!(o.chars.has_subtype("Equipment"));
    assert_eq!(t.pt(splitter), (1, 1));
    assert!(o.chars.subtypes.iter().all(|s| s.as_str() == "Equipment"), "no creature types");
}

#[test]
fn spells_on_the_stack_use_the_chosen_x_for_their_mana_value() {
    cr!("202.3e", "707.10", "702.179a");
    ruling!(
        "Chrome Host Seedshark",
        "If the noncreature spell has {X} in its mana cost, use the value chosen for X when calculating that spell's mana value."
    );
    ruling!(
        "Counterpoint",
        "If the target spell has {X} in its mana cost, use the value chosen for X when determining that spell’s mana value."
    );
    ruling!(
        "Refuse // Cooperate",
        "If the spell that's copied has an X whose value was determined as it was cast (like Torment of Hailfire does), the copy will have the same value of X."
    );
    supported("Chrome Host Seedshark");
    supported("Counterpoint");
    supported("Refuse // Cooperate");
    supported("Torment of Hailfire");
    // Chrome Host Seedshark: Fireball with X = 3 (mana value 4) incubates 4.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Chrome Host Seedshark");
    let fb = t.hand(P0, "Fireball");
    mana(&mut t, P0, "3R");
    t.cast(P0, fb).x(3).target(pl(P1)).go();
    t.resolve_all();
    let inc = tokens(&t, P0);
    assert_eq!(inc.len(), 1);
    assert!(t.obj_now(inc[0]).chars.has_subtype("Incubator"));
    assert_eq!(t.counters(inc[0], counters::PLUS1), 4);
    // Counterpoint countering P1's Fireball with X = 3: Hill Giant (4) may be cast from
    // the graveyard, Air Elemental (5) may not.
    let mut t = TestGame::new(2);
    let giant = t.graveyard(P0, "Hill Giant");
    let ae = t.graveyard(P0, "Air Elemental");
    t.set_step(P1, Step::PrecombatMain);
    let fb = t.hand(P1, "Fireball");
    mana(&mut t, P1, "3R");
    let fb = t.cast(P1, fb).x(3).target(pl(P0)).go();
    let from = t.asked().len();
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[obj(giant)]);
    cast(&mut t, P0, "Counterpoint", "3UB", &[obj(fb)]);
    t.resolve_all();
    assert!(was_offered(&t, P0, from, giant));
    assert!(!was_offered(&t, P0, from, ae));
    assert_eq!(count_named(&t, "Hill Giant"), 1);
    assert_eq!(t.life(P0), 20, "Fireball was countered");
    // Cooperate copying Torment of Hailfire with X = 2: the copy has X = 2 (P1, with no
    // cards or nonland permanents, loses 3 twice for each).
    let mut t = TestGame::new(2);
    let rc = t.graveyard(P0, "Refuse // Cooperate");
    let torment = t.hand(P0, "Torment of Hailfire");
    mana(&mut t, P0, "2BB");
    let torment = t.cast(P0, torment).x(2).go();
    mana(&mut t, P0, "2U");
    t.answer_targets(P0, &[obj(torment)]);
    t.cast(P0, rc).method(CastMethod::Keyword(KeywordKind::Aftermath)).go();
    keep_copy_targets(&mut t, P0);
    t.resolve();
    let copies = spell_copies(&t);
    assert_eq!(copies.len(), 1);
    assert_eq!(x_of(&t, copies[0]), Some(2));
    assert_eq!(mana_value(&t, copies[0]), 4);
    t.resolve_all();
    assert_eq!(t.life(P1), 8);
}

#[test]
fn planeswalkers_fury_and_mirth_do_nothing_with_an_empty_hand() {
    cr!("701.17a", "608.2c");
    ruling!(
        "Planeswalker's Fury",
        "If the opponent has no cards in hand, then no damage is dealt."
    );
    ruling!(
        "Planeswalker's Mirth",
        "If the opponent has no cards in hand, then no life is gained."
    );
    supported("Planeswalker's Fury");
    supported("Planeswalker's Mirth");
    for (card, cost, with_card, life0, life1) in [
        ("Planeswalker's Fury", "3R", false, 20, 20),
        ("Planeswalker's Fury", "3R", true, 20, 16),
        ("Planeswalker's Mirth", "3W", false, 20, 20),
        ("Planeswalker's Mirth", "3W", true, 24, 20),
    ] {
        let mut t = TestGame::new(2);
        let src = t.battlefield(P0, card);
        if with_card {
            t.hand(P1, "Hill Giant");
        }
        mana(&mut t, P0, cost);
        act(&mut t, P0, src, "reveals a card at random", &[pl(P1)]).expect(card);
        t.resolve_all();
        assert_eq!((t.life(P0), t.life(P1)), (life0, life1), "{card} with card: {with_card}");
    }
}

#[test]
fn shining_shoal_doesnt_redirect_damage_to_a_creature_that_left() {
    cr!("614.1", "614.9", "608.2h");
    ruling!(
        "Shining Shoal",
        "If the player or creature that was targeted is no longer playing or on the battlefield when the damage would occur, the damage isn’t redirected."
    );
    supported("Shining Shoal");
    for bounced in [false, true] {
        let mut t = TestGame::new(2);
        let bears = t.battlefield(P1, "Grizzly Bears");
        t.set_step(P1, Step::PrecombatMain);
        let shock = cast(&mut t, P1, "Shock", "R", &[pl(P0)]);
        let shoal = t.hand(P0, "Shining Shoal");
        mana(&mut t, P0, "3WW");
        t.answer_choose(P0, &[obj(shock)]);
        t.cast(P0, shoal).x(3).target(obj(bears)).go();
        t.resolve();
        if bounced {
            bounce(&mut t, bears);
        }
        t.resolve_all();
        if bounced {
            assert_eq!(t.life(P0), 18, "dealt to P0 as normal");
        } else {
            assert_eq!(t.life(P0), 20);
            assert!(t.in_graveyard(P1, "Grizzly Bears"), "2 damage redirected");
        }
    }
}

#[test]
fn radagast_finds_nothing_without_a_creature_of_a_new_type() {
    cr!("701.20a", "608.2c");
    ruling!(
        "Radagast the Brown",
        "If the top X cards of your library don't contain a creature card that doesn't share a creature type with a creature you control, you won't get to put a card into your hand. Apologies to Mistform Ultimus fans everywhere."
    );
    supported("Radagast the Brown");
    // Radagast (mana value 4) enters: the top four are lands and a Bear (P0 controls
    // Grizzly Bears): nothing goes to hand.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Grizzly Bears");
    stack_library(&mut t, P0, &["Forest", "Grizzly Bears", "Island", "Mountain", "Swamp"]);
    let hand = t.hand_size(P0);
    t.enter(P0, "Radagast the Brown");
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand);
    assert_eq!(t.library_size(P0), 35, "the four went to the bottom");
    // A Hill Giant (Giant) among them is found.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Grizzly Bears");
    let giant = t.library_top(P0, "Hill Giant");
    for n in ["Forest", "Island", "Mountain"] {
        t.library_top(P0, n);
    }
    t.answer_choose(P0, &[obj(giant)]);
    t.enter(P0, "Radagast the Brown");
    t.resolve_all();
    assert!(t.in_hand(P0, "Hill Giant"));
}

#[test]
fn scythe_specter_ties_make_each_tied_player_lose_life() {
    cr!("608.2c", "102.2");
    ruling!(
        "Scythe Specter",
        "If there are multiple cards tied for the highest mana value, each player who discarded one of those cards will lose life equal to that mana value."
    );
    supported("Scythe Specter");
    // Three players: P1 and P2 each discard a Hill Giant (4): each loses 4.
    let mut t = TestGame::new(3);
    let specter = t.battlefield(P0, "Scythe Specter");
    t.hand(P1, "Hill Giant");
    t.hand(P2, "Hill Giant");
    t.hand(P2, "Forest");
    let giant2 = t.g.find_in_zone(Zone::Hand(P2), "Hill Giant")[0];
    t.answer_choose(P2, &[obj(giant2)]);
    t.attack(&[(specter, pl(P1))], &[]);
    t.resolve_all();
    assert_eq!(t.life(P1), 20 - 4 - 4, "combat damage and life loss");
    assert_eq!(t.life(P2), 16);
}

#[test]
fn scythe_specter_in_two_headed_giant_drains_the_team_for_each_member() {
    cr!("810.7", "810.9", "102.3");
    ruling!(
        "Scythe Specter",
        "In a Two-Headed Giant game, each member of the opposing team will be able to see the other's hand before deciding what to discard. If each member discards a card with the same mana value, each player will lose that much life, resulting in the team losing life equal to twice that mana value."
    );
    supported("Scythe Specter");
    let mut t = two_headed_giant();
    let specter = t.battlefield(P0, "Scythe Specter");
    t.hand(P2, "Hill Giant");
    t.hand(P3, "Hill Giant");
    let team = t.life(P2);
    t.attack(&[(specter, pl(P2))], &[]);
    t.resolve_all();
    assert_eq!(t.life(P2), team - 4 - 8, "4 combat damage, then 4 twice");
    assert_eq!(t.life(P3), t.life(P2), "shared life total");
}

#[test]
fn enchanters_bane_must_target_itself_when_it_is_the_only_enchantment() {
    cr!("115.3", "603.3d");
    ruling!(
        "Enchanter's Bane",
        "If there are no other enchantments on the battlefield, the triggered ability of Enchanter's Bane must target Enchanter's Bane."
    );
    supported("Enchanter's Bane");
    let mut t = TestGame::new(2);
    let bane = t.battlefield(P0, "Enchanter's Bane");
    let from = t.asked().len();
    t.answer_yes(P0, false);
    end_step(&mut t, P0);
    let cands = target_candidates(&t, P0, from);
    assert_eq!(cands, vec![vec![obj(bane)]]);
    assert_eq!(t.life(P0), 18, "2 damage to its controller rather than sacrificing it");
    assert!(t.on_battlefield(bane));
}

#[test]
fn brimaz_triggers_once_for_a_phyrexian_artifact_creature_spell() {
    cr!("603.2c", "702.179a");
    ruling!(
        "Brimaz, Blight of Oreskos",
        "If you cast a Phyrexian artifact creature spell, Brimaz's first ability will trigger only once."
    );
    supported("Brimaz, Blight of Oreskos");
    supported("Vault Skirge");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Brimaz, Blight of Oreskos");
    cast(&mut t, P0, "Vault Skirge", "1B", &[]);
    t.settle();
    assert_eq!(t.stack_len(), 2, "one trigger");
    t.resolve_all();
    let inc: Vec<ObjectId> = tokens(&t, P0)
        .into_iter()
        .filter(|o| t.obj_now(*o).chars.has_subtype("Incubator"))
        .collect();
    assert_eq!(inc.len(), 1);
    assert_eq!(t.counters(inc[0], counters::PLUS1), 2);
}

#[test]
fn viviens_search_cant_find_the_target_of_a_mutating_spell() {
    cr!("702.140a", "601.2c", "603.7a");
    ruling!(
        "Vivien, Monsters' Advocate",
        "If you cast a mutating creature spell, you must choose a target for it before the triggered ability from Vivien's last ability puts a creature onto the battlefield. The creature you find can't be the target of the mutating creature spell."
    );
    supported("Vivien, Monsters' Advocate");
    supported("Dreamtail Heron");
    let mut t = TestGame::new(2);
    let vivien = t.battlefield(P0, "Vivien, Monsters' Advocate");
    t.g.objects[vivien.0 as usize].counters.insert("loyalty".into(), 5);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let giant = t.library_top(P0, "Hill Giant");
    act(&mut t, P0, vivien, "When you next cast", &[]).expect("-2");
    t.resolve_all();
    // Dreamtail Heron (5) mutating onto the Bears, chosen as it's cast; the delayed
    // trigger then finds the Hill Giant (4 < 5), which the Heron can't be mutating onto.
    let heron = t.hand(P0, "Dreamtail Heron");
    mana(&mut t, P0, "3U");
    t.answer(P0, DecisionKind::Option, Answer::Index(0));
    t.cast(P0, heron)
        .method(CastMethod::Keyword(KeywordKind::Mutate))
        .target(obj(bears))
        .go();
    t.answer_choose(P0, &[obj(giant)]);
    t.resolve();
    assert_eq!(count_named(&t, "Hill Giant"), 1);
    t.resolve_all();
    assert_eq!(t.obj_now(bears).chars.name.as_str(), "Dreamtail Heron", "merged onto the Bears");
    assert_eq!(count_named(&t, "Hill Giant"), 1);
    assert_eq!(t.obj_now(giant).chars.name.as_str(), "Hill Giant");
}

#[test]
fn lozhan_triggers_on_the_adventure_not_on_the_creature_half() {
    cr!("715.3", "715.3a", "715.3b");
    ruling!(
        "Lozhan, Dragons' Legacy",
        "If you cast a non-Dragon permanent spell that has an Adventure (rather than the Adventure itself) Lozhan's triggered ability will not trigger."
    );
    supported("Lozhan, Dragons' Legacy");
    supported("Bonecrusher Giant // Stomp");
    // The Giant (creature): no trigger.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Lozhan, Dragons' Legacy");
    let bg = t.hand(P0, "Bonecrusher Giant // Stomp");
    mana(&mut t, P0, "2R");
    t.cast(P0, bg).go();
    t.settle();
    assert_eq!(t.stack_len(), 1);
    t.resolve_all();
    assert_eq!(t.life(P1), 20);
    // Stomp (the Adventure, mana value 2): 2 damage from Stomp and 2 from Lozhan.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Lozhan, Dragons' Legacy");
    let bg = t.hand(P0, "Bonecrusher Giant // Stomp");
    mana(&mut t, P0, "1R");
    t.answer_targets(P0, &[pl(P1)]);
    t.cast(P0, bg).method(CastMethod::Half(1)).target(pl(P1)).go();
    t.settle();
    assert_eq!(t.stack_len(), 2);
    t.resolve_all();
    assert_eq!(t.life(P1), 16);
}

#[test]
fn severance_priest_makes_no_token_if_no_card_was_exiled() {
    cr!("608.2c", "607.2a");
    ruling!(
        "Severance Priest",
        "If you chose not to exile a card, the effect of the last ability won’t do anything. No one creates a token."
    );
    supported("Severance Priest");
    let mut t = TestGame::new(2);
    t.hand(P1, "Hill Giant");
    t.answer_targets(P0, &[pl(P1)]);
    t.answer_choose(P0, &[]);
    let priest = t.enter(P0, "Severance Priest");
    t.resolve_all();
    assert!(t.in_hand(P1, "Hill Giant"));
    kill(&mut t, priest);
    t.resolve_all();
    assert!(tokens(&t, P0).is_empty() && tokens(&t, P1).is_empty());
}

#[test]
fn bell_borca_notes_the_exiled_cards_mana_value_and_leaves_it_exiled() {
    cr!("607.2e", "400.7", "707.2");
    ruling!(
        "Bell Borca, Spectral Sergeant",
        "If you don't play the exiled card, it remains exiled."
    );
    ruling!(
        "Bell Borca, Spectral Sergeant",
        "Look at the card in exile, not its previous zone, to determine its mana value."
    );
    supported("Bell Borca, Spectral Sergeant");
    supported("Phantasmal Image");
    // Upkeep: the top card is exiled and not played; it's still in exile next turn.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Bell Borca, Spectral Sergeant");
    t.library_top(P0, "Grizzly Bears");
    t.advance_to(P0, Step::Upkeep);
    t.resolve_all();
    assert!(t.in_exile("Grizzly Bears"));
    t.advance_to(P1, Step::PrecombatMain);
    assert!(t.in_exile("Grizzly Bears"));
    // A Phantasmal Image (2) copying Hill Giant (4) is exiled: the card's 2 is noted.
    let mut t = TestGame::new(2);
    let bell = t.battlefield(P0, "Bell Borca, Spectral Sergeant");
    let giant = t.battlefield(P1, "Hill Giant");
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[obj(giant)]);
    let image = t.enter(P0, "Phantasmal Image");
    t.resolve_all();
    assert_eq!(t.obj_now(image).chars.name.as_str(), "Hill Giant");
    assert_eq!(mana_value(&t, image), 4);
    exile(&mut t, image);
    t.g.recompute();
    assert_eq!(t.pt(bell).0, 2);
}

#[test]
fn the_wedding_of_river_song_gives_suspend_without_counters_to_a_mana_value_0_card() {
    cr!("702.62a", "702.62b", "702.62c");
    ruling!(
        "The Wedding of River Song",
        "If you exile a card with a mana value of 0, it will gain suspend, but no time counters will be placed on it. It can't be cast unless a time counter is later placed on it (and then, subsequently removed), and it isn't considered to be a \"suspended card.\""
    );
    supported("The Wedding of River Song");
    let mut t = TestGame::new(2);
    let thopter = t.hand(P0, "Ornithopter");
    t.answer_choose(P0, &[obj(thopter)]);
    cast_resolve(&mut t, P0, "The Wedding of River Song", "2W", &[pl(P1)]);
    assert!(t.in_exile("Ornithopter"));
    let o = t.obj_now(thopter);
    assert!(o.chars.has_keyword(KeywordKind::Suspend));
    assert_eq!(o.counter("time"), 0);
    assert!(!can_cast(&mut t, P0, thopter, CastMethod::Normal));
    // Not suspended: nothing happens at the upkeep, and it stays in exile.
    t.advance_to(P0, Step::Upkeep);
    t.resolve_all();
    assert!(t.in_exile("Ornithopter"));
    assert_eq!(count_named(&t, "Ornithopter"), 0);
}

#[test]
fn combustible_gearhulks_draw_from_a_small_library_loses_the_game() {
    cr!("121.4", "704.5b", "121.3");
    ruling!(
        "Combustible Gearhulk",
        "If you have fewer than three cards in your library, the opponent may choose to have you draw three cards. If so, you'll lose the game as state-based actions are performed."
    );
    supported("Combustible Gearhulk");
    let mut t = TestGame::new(2);
    t.g.players[0].library.truncate(2);
    t.answer_targets(P0, &[pl(P1)]);
    t.answer_yes(P1, true);
    t.enter(P0, "Combustible Gearhulk");
    t.resolve_all();
    assert!(t.has_lost(P0));
    assert_eq!(t.life(P1), 20);
}

#[test]
fn undying_flames_does_nothing_with_an_empty_library() {
    cr!("608.2c", "702.50a");
    ruling!(
        "Undying Flames",
        "If you have no cards in your library, Undying Flames does nothing."
    );
    supported("Undying Flames");
    let mut t = TestGame::new(2);
    t.g.players[0].library.clear();
    cast_resolve(&mut t, P0, "Undying Flames", "4RR", &[pl(P1)]);
    assert_eq!(t.life(P1), 20);
    assert!(!t.has_lost(P0), "nothing was drawn");
}

#[test]
fn reanimated_permanents_are_exiled_when_their_controller_leaves_the_game() {
    cr!("800.4a", "800.4c");
    ruling!(
        "Visions of Dread",
        "If you leave the game while controlling permanents owned by other players that entered the battlefield under your control, those permanents are exiled. If the other players leave the game, they take their cards with them. (By contrast, if you leave the game while controlling permanents owned by another player you gained control of, those control-changing effects end and the permanents return to their previous controllers.)"
    );
    ruling!(
        "Command the Dreadhorde",
        "In a multiplayer game, if a player leaves the game, all cards that player owns leave as well. If you leave the game, the creatures and planeswalkers you control from Command the Dreadhorde’s effect are exiled."
    );
    supported("Visions of Dread");
    supported("Command the Dreadhorde");
    // Visions of Dread: P1's Hill Giant enters under P0's control; P0 leaves: exiled.
    let mut t = TestGame::new(3);
    let giant = t.graveyard(P1, "Hill Giant");
    t.answer_choose(P1, &[obj(giant)]);
    cast_resolve(&mut t, P0, "Visions of Dread", "2B", &[pl(P1)]);
    crate::r_p069_common::p0_leaves_and_its_permanent_is_exiled(&mut t, giant);
    // The owner leaves instead: the card leaves with them.
    let mut t = TestGame::new(3);
    let giant = t.graveyard(P1, "Hill Giant");
    t.answer_choose(P1, &[obj(giant)]);
    cast_resolve(&mut t, P0, "Visions of Dread", "2B", &[pl(P1)]);
    assert_eq!(t.obj_now(giant).controller, P0);
    crate::r_p069_common::p1_leaves_and_takes_the_card(&mut t, giant);
    // A stolen permanent (Act of Treason) goes back to its owner instead.
    let mut t = TestGame::new(3);
    let bears = t.battlefield(P1, "Grizzly Bears");
    gain_control(&mut t, P0, bears);
    assert_eq!(t.obj_now(bears).controller, P0);
    crate::r_p069_common::concede(&mut t, P0);
    assert!(t.on_battlefield(bears));
    assert_eq!(t.obj_now(bears).controller, P1);
    // Command the Dreadhorde: P1's Hill Giant and P2's Grizzly Bears under P0's control;
    // P0 leaves: both exiled.
    let mut t = TestGame::new(3);
    let giant = t.graveyard(P1, "Hill Giant");
    let bears = t.graveyard(P2, "Grizzly Bears");
    let cmd = t.hand(P0, "Command the Dreadhorde");
    mana(&mut t, P0, "4BB");
    t.cast(P0, cmd).targets(&[obj(giant), obj(bears)]).go();
    t.resolve_all();
    assert_eq!(t.life(P0), 14, "4 + 2 damage");
    assert!(t.on_battlefield(giant) && t.on_battlefield(bears));
    crate::r_p069_common::concede(&mut t, P0);
    assert_eq!(t.zone(giant), Zone::Exile);
    assert_eq!(t.zone(bears), Zone::Exile);
}

#[test]
fn cait_gets_no_counters_if_she_cant_discard() {
    cr!("121.4", "701.8a", "608.2c");
    ruling!(
        "Cait, Cage Brawler",
        "If you somehow can't discard a card (probably because you had no cards in hand and another effect prevented you from drawing cards), you won't put any counters on Cait no matter whether or not the defending player discarded a card or what that card's mana value is."
    );
    supported("Cait, Cage Brawler");
    // P0 has no cards in hand and none in the library: nothing is drawn or discarded,
    // while P1 draws and discards a card; Cait gets no counters (and P0 loses the game
    // for the draw from an empty library when state-based actions are next checked).
    let mut t = TestGame::new(2);
    let cait = t.battlefield(P0, "Cait, Cage Brawler");
    t.g.players[0].library.clear();
    t.hand(P1, "Forest");
    attack_with(&mut t, &[(cait, pl(P1))]);
    t.g.resolve_top();
    assert_eq!(t.counters(cait, counters::PLUS1), 0);
    assert_eq!(t.graveyard_size(P1), 1, "P1 discarded");
    assert_eq!(t.graveyard_size(P0), 0);
}

#[test]
fn invoke_prejudice_affects_every_opponent_in_a_multiplayer_game() {
    cr!("102.2", "603.2", "701.6a");
    ruling!("Invoke Prejudice", "In a multi-player game, it affects all opposing players.");
    supported("Invoke Prejudice");
    let mut t = TestGame::new(3);
    t.battlefield(P0, "Invoke Prejudice");
    t.battlefield(P0, "Grizzly Bears");
    // P1's and P2's Hill Giants (red, no green creature of P0's shares it) are countered
    // (nobody pays {4}); P0's own isn't affected.
    for p in [P1, P2] {
        t.set_step(p, Step::PrecombatMain);
        let g = t.hand(p, "Hill Giant");
        mana(&mut t, p, "3R");
        t.cast(p, g).go();
        t.resolve_all();
        assert!(t.in_graveyard(p, "Hill Giant"), "{p}");
    }
    t.set_step(P0, Step::PrecombatMain);
    cast_resolve(&mut t, P0, "Hill Giant", "3R", &[]);
    assert_eq!(count_named(&t, "Hill Giant"), 1);
}

#[test]
fn havi_can_return_a_copy_that_died_as_its_own_smaller_card() {
    cr!("707.2", "400.7", "608.2h");
    ruling!(
        "Havi, the All-Father",
        "In the rare case where a legendary creature becomes a copy of another legendary creature with greater mana value (for example, if Sakashima the Impostor is a copy of a legendary creature with mana value 5 or greater), then when that creature dies, if it's still a legendary creature card in your graveyard, Havi's last ability will be able to target it."
    );
    supported("Havi, the All-Father");
    supported("Sakashima the Impostor");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Havi, the All-Father");
    let wurm = t.battlefield(P1, "Craw Wurm");
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[obj(wurm)]);
    let sak = t.enter(P0, "Sakashima the Impostor");
    t.resolve_all();
    assert_eq!(mana_value(&t, sak), 6, "a copy of the Wurm");
    assert!(t.obj_now(sak).chars.supertypes.iter().any(|s| s == Supertype::Legendary));
    // It dies: a legendary creature (mana value 6) died, and its card (Sakashima, 4) is a
    // legendary creature card with lesser mana value.
    let from = t.asked().len();
    t.answer_targets(P0, &[obj(sak)]);
    kill(&mut t, sak);
    t.resolve_all();
    assert_eq!(target_candidate_names(&t, P0, from), vec!["Sakashima the Impostor".to_string()]);
    let back = t.named_on_battlefield("Sakashima the Impostor");
    assert_eq!(back.len(), 1);
    assert!(t.obj_now(back[0]).tapped);
}
