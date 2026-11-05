//! Rulings batch P012 — counting cards in graveyards: when the count is taken (continuously
//! for a static ability, CR 611.3a; once as a spell or ability resolves, CR 608.2h), what
//! it counts (a resolving spell isn't in the graveyard yet, CR 608.2n), intervening "if"
//! conditions (CR 603.4) and conditions checked only as attackers or blockers are declared
//! (CR 508.1c, 509.1c).

use crate::r_p012_common::*;
use crate::r_p057_common::into_upkeep;
use crate::r_p127_common::{exile_now, move_to};
use crate::r_s01_common::{attack_with, block_and_finish, supported};
use mtg_engine::events::Event;
use mtg_engine::object::{FaceState, Zone};
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

// --- Static abilities: the count is continuous ----------------------------------------------

#[test]
fn syndicate_infiltrator_loses_its_bonus_and_may_die() {
    cr!("611.3a", "704.5g");
    ruling!(
        "Syndicate Infiltrator",
        "If you have five or more mana values among cards in your graveyard and then some of them are removed such that you have fewer than five mana values among those cards, Syndicate Infiltrator loses its bonus to power and toughness. Notably, this may cause it to die if it now has damage marked on it greater than its toughness."
    );
    supported("Syndicate Infiltrator");
    let mut t = TestGame::new(2);
    let si = t.battlefield(P0, "Syndicate Infiltrator");
    // Mana values 0, 1, 2, 3 and 4.
    let cards: Vec<ObjectId> = ["Ornithopter", "Llanowar Elves", "Grizzly Bears", "Divination", "Hill Giant"]
        .iter()
        .map(|n| t.graveyard(P0, n))
        .collect();
    t.g.recompute();
    assert_eq!(t.pt(si), (5, 5));
    t.g.objects[si.0 as usize].damage = 4;
    t.settle();
    assert!(t.on_battlefield(si));
    exile_now(&mut t, cards[4]);
    t.settle();
    assert!(!t.on_battlefield(si), "3/3 with 4 damage");
}

#[test]
fn jaces_phantasm_watches_each_opponent_separately() {
    cr!("611.3a");
    ruling!(
        "Jace's Phantasm",
        "In a multiplayer game, a single opponent must have ten or more cards in their graveyard for the bonus to apply, although this opponent may change over the course of the game. You don't have to pick a single opponent; the ability will monitor each opponent's graveyard."
    );
    ruling!(
        "Jace's Phantasm",
        "The last ability of Jace's Phantasm constantly monitors each opponent's graveyard to see if the bonus applies. If the stated condition becomes no longer true, the bonus immediately stops applying."
    );
    supported("Jace's Phantasm");
    let mut t = TestGame::new(3);
    let jp = t.battlefield(P0, "Jace's Phantasm");
    fill_graveyard(&mut t, P1, "Grizzly Bears", 6);
    let p2 = fill_graveyard(&mut t, P2, "Grizzly Bears", 9);
    t.g.recompute();
    assert_eq!(t.pt(jp), (1, 1), "15 cards, but no single opponent has 10");
    t.graveyard(P2, "Grizzly Bears");
    t.g.recompute();
    assert_eq!(t.pt(jp), (5, 5));
    exile_now(&mut t, p2[0]);
    t.g.recompute();
    assert_eq!(t.pt(jp), (1, 1));
    fill_graveyard(&mut t, P1, "Grizzly Bears", 4);
    t.g.recompute();
    assert_eq!(t.pt(jp), (5, 5), "now P1 has 10");
}

#[test]
fn tenured_oilcaster_needs_only_one_opponent_with_eight() {
    cr!("611.3a");
    ruling!(
        "Tenured Oilcaster",
        "Tenured Oilcaster's second ability will apply if any opponent has eight or more cards in their graveyard, not necessarily all of them."
    );
    supported("Tenured Oilcaster");
    let mut t = TestGame::new(3);
    let to = t.battlefield(P0, "Tenured Oilcaster");
    fill_graveyard(&mut t, P2, "Grizzly Bears", 8);
    t.g.recompute();
    assert_eq!(t.pt(to), (5, 4));
}

#[test]
fn graveyard_count_bonuses_are_fixed_amounts() {
    cr!("611.3a");
    ruling!(
        "Magmatic Channeler",
        "Magmatic Channeler gets just +3/+1 from its first ability, no matter how many instant and sorcery cards are in your graveyard beyond the fourth."
    );
    ruling!(
        "Ramunap Hydra",
        "Unlike other cards in this set that care only whether you control a Desert or there is a Desert card in your graveyard, Ramunap Hydra's abilities reward you for meeting both conditions. If you control one or more Deserts and have one or more Desert cards in your graveyard, it'll get +2/+2."
    );
    supported("Magmatic Channeler");
    let mut t = TestGame::new(2);
    let mc = t.battlefield(P0, "Magmatic Channeler");
    fill_graveyard(&mut t, P0, "Opt", 3);
    t.g.recompute();
    assert_eq!(t.pt(mc), (1, 3));
    fill_graveyard(&mut t, P0, "Opt", 1);
    t.g.recompute();
    assert_eq!(t.pt(mc), (4, 4));
    fill_graveyard(&mut t, P0, "Opt", 4);
    t.g.recompute();
    assert_eq!(t.pt(mc), (4, 4));

    supported("Ramunap Hydra");
    supported("Desert");
    let mut t = TestGame::new(2);
    let rh = t.battlefield(P0, "Ramunap Hydra");
    t.battlefield(P0, "Desert");
    t.g.recompute();
    assert_eq!(t.pt(rh), (4, 4));
    t.graveyard(P0, "Desert");
    t.g.recompute();
    assert_eq!(t.pt(rh), (5, 5));
}

#[test]
fn spontaneous_mutations_x_updates_constantly() {
    cr!("611.3a");
    ruling!(
        "Spontaneous Mutation",
        "The value of X will constantly update as the number of cards in your graveyard changes."
    );
    supported("Spontaneous Mutation");
    let mut t = TestGame::new(2);
    let wurm = t.battlefield(P1, "Craw Wurm");
    cast_card(&mut t, P0, "Spontaneous Mutation", &[obj(wurm)]);
    t.resolve_all();
    assert_eq!(t.pt(wurm), (6, 4));
    let opts = fill_graveyard(&mut t, P0, "Opt", 2);
    t.g.recompute();
    assert_eq!(t.pt(wurm), (4, 4));
    exile_now(&mut t, opts[0]);
    t.g.recompute();
    assert_eq!(t.pt(wurm), (5, 4));
}

// --- Spells and abilities: the count is taken as they resolve --------------------------------

#[test]
fn x_is_counted_once_as_the_spell_or_ability_resolves() {
    cr!("608.2h");
    ruling!(
        "Festive Funeral",
        "The value of X is determined only as Festive Funeral begins to resolve. The target creature's power and toughness won't change if other cards are put into your graveyard later in the turn."
    );
    ruling!(
        "Gloom Ripper",
        "The value of X is calculated only once, as Gloom Ripper's ability resolves."
    );
    supported("Festive Funeral");
    let mut t = TestGame::new(2);
    let wurm = t.battlefield(P1, "Craw Wurm");
    fill_graveyard(&mut t, P0, "Opt", 2);
    cast_card(&mut t, P0, "Festive Funeral", &[obj(wurm)]);
    t.resolve_all();
    assert_eq!(t.pt(wurm), (4, 2), "X = 2 (Festive Funeral isn't counted)");
    fill_graveyard(&mut t, P0, "Opt", 3);
    t.g.recompute();
    assert_eq!(t.pt(wurm), (4, 2));

    supported("Gloom Ripper");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let wurm = t.battlefield(P1, "Craw Wurm");
    t.graveyard(P0, "Llanowar Elves");
    // X = Gloom Ripper (an Elf) + one Elf card = 2.
    t.answer_targets(P0, &[obj(bears)]);
    t.answer_targets(P0, &[obj(wurm)]);
    cast_card(&mut t, P0, "Gloom Ripper", &[]);
    t.resolve_all();
    assert_eq!(t.pt(bears), (4, 2));
    assert_eq!(t.pt(wurm), (6, 2));
    fill_graveyard(&mut t, P0, "Llanowar Elves", 2);
    t.g.recompute();
    assert_eq!(t.pt(bears), (4, 2));
    assert_eq!(t.pt(wurm), (6, 2));
}

#[test]
fn keening_stone_counts_the_graveyard_as_it_resolves() {
    cr!("608.2h", "701.17a");
    ruling!(
        "Keening Stone",
        "The value of X is determined as the ability starts to resolve. If the targeted player has five cards in their graveyard at that time, for example, this ability will put five more cards into that graveyard from the top of that player's library."
    );
    supported("Keening Stone");
    let mut t = TestGame::new(2);
    let ks = t.battlefield(P0, "Keening Stone");
    t.lands(P0, "Wastes", 5);
    fill_graveyard(&mut t, P1, "Grizzly Bears", 4);
    t.activate(P0, ks, 0, &[pl(P1)]).unwrap();
    // In response, a fifth card is put into P1's graveyard.
    t.graveyard(P1, "Grizzly Bears");
    t.resolve_all();
    assert_eq!(t.graveyard_size(P1), 10);
    assert_eq!(t.library_size(P1), 25);
}

#[test]
fn a_resolving_spell_doesnt_count_itself() {
    cr!("608.2n", "608.2h");
    ruling!(
        "Rise from the Tides",
        "Rise from the Tides is still on the stack while you count the number of instant and sorcery cards in your graveyard. It doesn’t count itself."
    );
    ruling!(
        "Slime Against Humanity",
        "Slime Against Humanity isn't put into your graveyard until after it finishes resolving, so it doesn't count itself for its own effect."
    );
    ruling!(
        "Tribute to Urborg",
        "Tribute to Urborg is not in your graveyard yet as it is resolving, so it doesn't count itself."
    );
    // Rise from the Tides with two instants in the graveyard: two Zombies.
    supported("Rise from the Tides");
    let mut t = TestGame::new(2);
    fill_graveyard(&mut t, P0, "Opt", 2);
    cast_card(&mut t, P0, "Rise from the Tides", &[]);
    t.resolve_all();
    assert_eq!(crate::r_s25_common::creature_tokens(&t, P0), 2);

    // Slime Against Humanity with one other in the graveyard: 2 + 1 counters.
    supported("Slime Against Humanity");
    let mut t = TestGame::new(2);
    t.graveyard(P0, "Slime Against Humanity");
    cast_card(&mut t, P0, "Slime Against Humanity", &[]);
    t.resolve_all();
    let ooze = crate::r_s01_common::tokens(&t, P0)[0];
    assert_eq!(t.counters(ooze, "+1/+1"), 3);

    // Kicked Tribute to Urborg with two instants in the graveyard: -4/-4 in all.
    supported("Tribute to Urborg");
    let mut t = TestGame::new(2);
    let dreadmaw = t.battlefield(P1, "Colossal Dreadmaw");
    fill_graveyard(&mut t, P0, "Opt", 2);
    crate::r_s25_common::lands_for_cost(&mut t, P0, "Tribute to Urborg");
    t.lands(P0, "Island", 2);
    let tu = t.hand(P0, "Tribute to Urborg");
    t.cast(P0, tu).kicked(true).target(dreadmaw).go();
    t.resolve_all();
    assert_eq!(t.pt(dreadmaw), (2, 2));
}

#[test]
fn slime_against_humanity_ignores_face_down_exiled_cards() {
    cr!("406.3");
    ruling!(
        "Slime Against Humanity",
        "If you own face-down cards in exile, they won't count toward the value of X, even if you're allowed to look at them and you know they are Ooze cards or cards named Slime Against Humanity."
    );
    let mut t = TestGame::new(2);
    t.exile(P0, "Slime Against Humanity");
    let down = t.exile(P0, "Slime Against Humanity");
    t.g.objects[down.0 as usize].face_down = true;
    t.g.recompute();
    cast_card(&mut t, P0, "Slime Against Humanity", &[]);
    t.resolve_all();
    let ooze = crate::r_s01_common::tokens(&t, P0)[0];
    assert_eq!(t.counters(ooze, "+1/+1"), 3, "2 + the face-up one");
}

#[test]
fn spoils_of_evil_doesnt_count_artifact_creatures_twice() {
    cr!("608.2h");
    ruling!(
        "Spoils of Evil",
        "The text “for each artifact or creature” means the sum of cards which are either creature and/or artifact. Artifact creatures are not double counted."
    );
    supported("Spoils of Evil");
    let mut t = TestGame::new(2);
    for n in ["Ornithopter", "Grizzly Bears", "Mind Stone", "Opt"] {
        t.graveyard(P1, n);
    }
    cast_card(&mut t, P0, "Spoils of Evil", &[pl(P1)]);
    t.resolve_all();
    assert_eq!(t.life(P0), 23);
    assert_eq!(t.g.player(P0).mana_pool.total(), 3);
}

#[test]
fn archangels_light_shuffles_even_with_an_empty_graveyard() {
    cr!("608.2c", "701.24a");
    ruling!(
        "Archangel's Light",
        "If you have no cards in your graveyard, you won’t gain any life, but you’ll still shuffle your library."
    );
    supported("Archangel's Light");
    let mut t = TestGame::new(2);
    cast_card(&mut t, P0, "Archangel's Light", &[]);
    let before = t.g.turn_events.len();
    t.resolve_all();
    assert_eq!(t.life(P0), 20);
    let shuffled = t.g.turn_events[before..]
        .iter()
        .any(|e| matches!(e, Event::Shuffled { player } if *player == P0));
    assert!(shuffled);
}

// --- Intervening "if" conditions -------------------------------------------------------------

#[test]
fn intervening_if_graveyard_counts_are_checked_twice() {
    cr!("603.4");
    ruling!(
        "Curious Homunculus // Voracious Reader",
        "If you don't have enough instant and/or sorcery cards in your graveyard as your upkeep begins, Curious Homunculus's last ability doesn't trigger. If you don't have enough as the ability resolves, the ability has no effect."
    );
    ruling!(
        "Sidequest: Card Collection // Magicked Card",
        "Sidequest: Card Collection's last ability checks at the moment it would trigger to see if you have eight or more cards in your graveyard. If you don't, the ability won't trigger at all. If it does trigger, the ability will check again as it tries to resolve. If there aren't eight or more cards in your graveyard at that time, the ability won't resolve and none of its effects will happen."
    );
    ruling!(
        "Walltop Sentries",
        "Walltop Sentries's triggered ability checks if there is a Lesson card in your graveyard when it is put on the stack and again when it goes to resolve. If there is no Lesson card in your graveyard when Walltop Sentries dies, the ability will not be put on the stack at all."
    );
    // (cards in the graveyard, removed in response, triggers, transformed)
    for (n, remove, triggers, done) in [
        (2, false, false, false),
        (3, true, true, false),
        (3, false, true, true),
    ] {
        let mut t = TestGame::new(2);
        let ch = t.battlefield(P0, "Curious Homunculus // Voracious Reader");
        let opts = fill_graveyard(&mut t, P0, "Opt", n);
        into_upkeep(&mut t, P0);
        assert_eq!(trigger_on_top(&t), triggers, "Homunculus {n} {remove}");
        if remove {
            exile_now(&mut t, opts[0]);
        }
        t.resolve_all();
        assert_eq!((t.obj_now(ch).face == FaceState::Back), done, "Homunculus {n} {remove}");
    }
    for (n, remove, triggers, done) in [
        (7, false, false, false),
        (8, true, true, false),
        (8, false, true, true),
    ] {
        let mut t = TestGame::new(2);
        let sq = t.battlefield(P0, "Sidequest: Card Collection // Magicked Card");
        let cards = fill_graveyard(&mut t, P0, "Opt", n);
        t.advance_to(P0, Step::End);
        t.settle();
        assert_eq!(trigger_on_top(&t), triggers, "Sidequest {n} {remove}");
        if remove {
            exile_now(&mut t, cards[0]);
        }
        t.resolve_all();
        assert_eq!((t.obj_now(sq).face == FaceState::Back), done, "Sidequest {n} {remove}");
    }
    supported("Walltop Sentries");
    for (lesson, remove, triggers, life) in [
        (false, false, false, 20),
        (true, true, true, 20),
        (true, false, true, 22),
    ] {
        let mut t = TestGame::new(2);
        let ws = t.battlefield(P0, "Walltop Sentries");
        let l = lesson.then(|| t.graveyard(P0, "Environmental Sciences"));
        crate::r_p030_common::kill(&mut t, ws);
        assert_eq!(trigger_on_top(&t), triggers, "Sentries {lesson} {remove}");
        if remove {
            exile_now(&mut t, l.unwrap());
        }
        t.resolve_all();
        assert_eq!(t.life(P0), life, "Sentries {lesson} {remove}");
    }
}

// --- Checked only as attackers or blockers are declared --------------------------------------

#[test]
fn attack_and_block_restrictions_are_checked_only_as_they_are_declared() {
    cr!("508.1c", "509.1c", "506.4");
    ruling!(
        "River Serpent",
        "Once River Serpent has attacked, it will remain in combat even if the number of cards in your graveyard becomes four or fewer."
    );
    ruling!(
        "Vantress Gargoyle",
        "Vantress Gargoyle's attacking and blocking restrictions are checked only at the moment that you declare it as an attacking or blocking creature. If the defending player loses cards from their graveyard after Vantress Gargoyle attacks, or if you lose cards from your hand after it blocks, it remains in combat."
    );
    supported("River Serpent");
    supported("Vantress Gargoyle");
    // River Serpent attacks with five cards in P0's graveyard, which are then exiled.
    let mut t = TestGame::new(2);
    let rs = t.battlefield(P0, "River Serpent");
    let cards = fill_graveyard(&mut t, P0, "Opt", 5);
    attack_with(&mut t, &[(rs, pl(P1))]);
    assert!(t.g.is_attacking(rs));
    for c in cards {
        exile_now(&mut t, c);
    }
    block_and_finish(&mut t, P1, &[]);
    assert_eq!(t.life(P1), 15);

    // Vantress Gargoyle attacks with seven cards in P1's graveyard, which are then exiled.
    let mut t = TestGame::new(2);
    let vg = t.battlefield(P0, "Vantress Gargoyle");
    let cards = fill_graveyard(&mut t, P1, "Opt", 7);
    attack_with(&mut t, &[(vg, pl(P1))]);
    assert!(t.g.is_attacking(vg));
    for c in cards {
        exile_now(&mut t, c);
    }
    block_and_finish(&mut t, P1, &[]);
    assert_eq!(t.life(P1), 15);

    // P1's Vantress Gargoyle blocks with four cards in P1's hand, which are then discarded.
    let mut t = TestGame::new(2);
    let vg = t.battlefield(P1, "Vantress Gargoyle");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let hand: Vec<ObjectId> = (0..4).map(|_| t.hand(P1, "Opt")).collect();
    attack_with(&mut t, &[(bears, pl(P1))]);
    t.answer(
        P1,
        DecisionKind::Blockers,
        mtg_engine::decision::Answer::Blockers(vec![(vg, bears)]),
    );
    t.advance_to(P0, Step::DeclareBlockers);
    assert!(t.g.is_blocking(vg));
    for c in hand {
        move_to(&mut t, c, Zone::Graveyard(P1));
    }
    t.advance_to(P0, Step::EndOfCombat);
    assert!(!t.on_battlefield(bears), "blocked by a 5/4");
    assert_eq!(t.life(P1), 20);
}
