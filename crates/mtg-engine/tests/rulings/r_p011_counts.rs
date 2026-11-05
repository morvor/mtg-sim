//! Rulings batch P011 — counting cards in graveyards: a resolving spell is still on the
//! stack and doesn't count itself (CR 608.2n), counts taken as the spell or ability
//! resolves (CR 608.2h) including a source that's already in the graveyard, card types
//! (CR 205.2a, 205.4a, 308.1) and split cards (CR 709.4c), different mana values, face-down
//! exiled cards (CR 708.2), costs locked in before mana abilities (CR 601.2f-g, 602.2b),
//! excess damage (CR 120.4a) and a "dies this turn" replacement (CR 614.1a).

use crate::r_p009_common::{dmg, kill, obj, pl};
use crate::r_p011_common::*;
use crate::r_p012_common::{cast_card, fill_graveyard};
use crate::r_p127_common::{exile_now, sacrifice};
use crate::r_s01_common::{creatures, stack_library, supported, tokens};
use crate::r_s20_common::tap_for_mana;
use mtg_engine::decision::Decision;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

// --- The resolving spell isn't in the graveyard yet -----------------------------------------

#[test]
fn resolving_spells_dont_count_themselves() {
    cr!("608.2n", "608.2h");
    ruling!(
        "Lucid Dreams",
        "Lucid Dreams is still on the stack and not in your graveyard as you calculate the value of X."
    );
    ruling!(
        "Ancestral Anger",
        "Ancestral Anger isn't put into your graveyard until after it finishes resolving, so it doesn't count itself for its own effect."
    );
    ruling!(
        "Accumulated Knowledge",
        "Because Accumulated Knowledge is still on the stack as you perform its instructions, it isn't in your graveyard and won't add to the number of cards drawn."
    );
    ruling!(
        "Galvanic Bombardment",
        "Because Galvanic Bombardment is still on the stack as you determine the value of X, it is not in your graveyard and will not add to the value of X."
    );
    ruling!(
        "Kindle",
        "Because Kindle is still on the stack as you perform its instructions, it isn't in your graveyard and won't add to the amount of damage dealt."
    );
    ruling!(
        "Take Inventory",
        "Because Take Inventory is still on the stack as it’s resolving, that spell card isn’t in your graveyard and won’t affect the number of cards you draw."
    );
    ruling!(
        "Cosmic Epiphany",
        "Cosmic Epiphany is still resolving at the time that the instant and sorcery cards in your graveyard are counted, so it isn’t in your graveyard yet and won’t be counted."
    );
    ruling!(
        "Festive Funeral",
        "Festive Funeral is still on the stack while you determine the value of X. It doesn't contribute to the value of X."
    );
    ruling!(
        "Frantic Firebolt",
        "Frantic Firebolt isn't put into your graveyard until after it finishes resolving, so it doesn't count itself for its own effect."
    );
    ruling!(
        "Gandalf's Sanction",
        "Gandalf's Sanction is not yet in your graveyard when you determine the value of X."
    );
    ruling!(
        "Goblin Gathering",
        "Goblin Gathering is still on the stack while you count the cards named Goblin Gathering in your graveyard. It doesn’t count itself."
    );

    // Lucid Dreams: an instant and a land in the graveyard — 2 cards, not 3.
    let mut t = TestGame::new(2);
    t.graveyard(P0, "Opt");
    t.graveyard(P0, "Forest");
    let hand = t.hand_size(P0);
    cast_card(&mut t, P0, "Lucid Dreams", &[]);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand + 2, "Lucid Dreams");

    // Ancestral Anger: one in the graveyard — +2/+0, not +3/+0.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.graveyard(P0, "Ancestral Anger");
    cast_card(&mut t, P0, "Ancestral Anger", &[obj(bears)]);
    t.resolve_all();
    assert_eq!(t.pt(bears), (4, 2), "Ancestral Anger");

    // Accumulated Knowledge / Take Inventory: one in the graveyard — draw 2, not 3.
    for card in ["Accumulated Knowledge", "Take Inventory"] {
        let mut t = TestGame::new(2);
        t.graveyard(P0, card);
        let hand = t.hand_size(P0);
        cast_card(&mut t, P0, card, &[]);
        t.resolve_all();
        assert_eq!(t.hand_size(P0), hand + 2, "{card}");
    }

    // Galvanic Bombardment: one in the graveyard — 3 damage, not 4 (the Wurm survives).
    let mut t = TestGame::new(2);
    let wurm = t.battlefield(P1, "Craw Wurm");
    t.graveyard(P0, "Galvanic Bombardment");
    cast_card(&mut t, P0, "Galvanic Bombardment", &[obj(wurm)]);
    t.resolve_all();
    assert_eq!(dmg(&t, wurm), 3, "Galvanic Bombardment");

    // Kindle: one in P1's graveyard — 3 damage, not 4.
    let mut t = TestGame::new(2);
    t.graveyard(P1, "Kindle");
    cast_card(&mut t, P0, "Kindle", &[pl(P1)]);
    t.resolve_all();
    assert_eq!(t.life(P1), 17, "Kindle");

    // Cosmic Epiphany: two instants — draw 2, not 3.
    let mut t = TestGame::new(2);
    fill_graveyard(&mut t, P0, "Opt", 2);
    let hand = t.hand_size(P0);
    cast_card(&mut t, P0, "Cosmic Epiphany", &[]);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand + 2, "Cosmic Epiphany");

    // Festive Funeral: two cards — the Giant gets -2/-2 and survives.
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P1, "Hill Giant");
    fill_graveyard(&mut t, P0, "Opt", 2);
    cast_card(&mut t, P0, "Festive Funeral", &[obj(giant)]);
    t.resolve_all();
    assert_eq!(t.pt(giant), (1, 1), "Festive Funeral");

    // Frantic Firebolt: one instant — 3 damage, not 4.
    let mut t = TestGame::new(2);
    let wurm = t.battlefield(P1, "Craw Wurm");
    t.graveyard(P0, "Opt");
    cast_card(&mut t, P0, "Frantic Firebolt", &[obj(wurm)]);
    t.resolve_all();
    assert_eq!(dmg(&t, wurm), 3, "Frantic Firebolt");

    // Gandalf's Sanction: two instants — 2 damage, not 3 (the Giant survives, no excess).
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P1, "Hill Giant");
    fill_graveyard(&mut t, P0, "Opt", 2);
    cast_card(&mut t, P0, "Gandalf's Sanction", &[obj(giant)]);
    t.resolve_all();
    assert_eq!(dmg(&t, giant), 2, "Gandalf's Sanction");
    assert_eq!(t.life(P1), 20);

    // Goblin Gathering: one in the graveyard — 3 Goblins, not 4.
    let mut t = TestGame::new(2);
    t.graveyard(P0, "Goblin Gathering");
    cast_card(&mut t, P0, "Goblin Gathering", &[]);
    t.resolve_all();
    assert_eq!(tokens(&t, P0).len(), 3, "Goblin Gathering");
}

#[test]
fn archangels_light_isnt_counted_or_shuffled() {
    cr!("608.2n", "608.2h");
    ruling!(
        "Archangel's Light",
        "Archangel’s Light isn’t put into the graveyard until after its instructions are followed. You won’t gain 2 life for it, and it won’t be shuffled into your library."
    );
    let mut t = TestGame::new(2);
    fill_graveyard(&mut t, P0, "Opt", 2);
    let lib = t.library_size(P0);
    cast_card(&mut t, P0, "Archangel's Light", &[]);
    t.resolve_all();
    assert_eq!(t.life(P0), 24);
    assert_eq!(t.library_size(P0), lib + 2);
    assert_eq!(t.graveyard_size(P0), 1);
    assert!(t.in_graveyard(P0, "Archangel's Light"));
}

#[test]
fn ghastly_demise_compares_toughness_as_it_resolves() {
    cr!("608.2n", "608.2h", "115.1");
    ruling!(
        "Ghastly Demise",
        "Because Ghastly Demise isn't put into your graveyard until it's finished resolving, it won't be counted when determining whether the creature is destroyed."
    );
    ruling!(
        "Ghastly Demise",
        "Ghastly Demise can target any nonblack creature. Compare the creature's toughness to the number of cards in your graveyard as Ghastly Demise resolves to see if that creature is destroyed."
    );
    // Two cards, toughness 3: legal target, not destroyed (Ghastly Demise isn't counted).
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P1, "Hill Giant");
    fill_graveyard(&mut t, P0, "Opt", 2);
    cast_card(&mut t, P0, "Ghastly Demise", &[obj(giant)]);
    t.resolve_all();
    assert!(t.on_battlefield(giant));
    // A third card arrives in response: destroyed.
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P1, "Hill Giant");
    fill_graveyard(&mut t, P0, "Opt", 2);
    cast_card(&mut t, P0, "Ghastly Demise", &[obj(giant)]);
    t.graveyard(P0, "Opt");
    t.resolve_all();
    assert!(!t.on_battlefield(giant));
    // An empty graveyard: still a legal target.
    let mut t = TestGame::new(2);
    let wurm = t.battlefield(P1, "Craw Wurm");
    cast_card(&mut t, P0, "Ghastly Demise", &[obj(wurm)]);
    t.resolve_all();
    assert!(t.on_battlefield(wurm));
}

#[test]
fn counter_unless_pay_counts_without_itself() {
    cr!("608.2n", "118.12a");
    ruling!(
        "Circular Logic",
        "Count the number of cards in your graveyard as Circular Logic resolves to determine how much the controller of the target spell has to pay. Circular Logic is still on the stack at that time. It doesn't count itself."
    );
    ruling!(
        "Rakshasa's Disdain",
        "Count the number of cards in your graveyard as Rakshasa’s Disdain resolves to determine how much mana the controller of the target spell must pay to avoid the spell being countered. Rakshasa’s Disdain is still on the stack at this time and won’t count toward this number."
    );
    for card in ["Circular Logic", "Rakshasa's Disdain"] {
        supported(card);
        // Two cards in P0's graveyard; P1 has exactly two more lands: pays {2}.
        let mut t = TestGame::new(2);
        fill_graveyard(&mut t, P0, "Opt", 2);
        t.set_step(P1, Step::PrecombatMain);
        let div = cast_card(&mut t, P1, "Divination", &[]);
        t.lands(P1, "Wastes", 2);
        cast_card(&mut t, P0, card, &[obj(div)]);
        // A card put into the graveyard after Circular Logic is cast does count.
        t.answer_yes(P1, true);
        t.resolve();
        assert_eq!(t.zone(div), Zone::Stack, "{card}: paid {{2}}");
        t.resolve_all();
        assert_eq!(t.zone(div), Zone::Graveyard(P1));
        assert_eq!(t.hand_size(P1), 2, "{card}: Divination resolved");
        // Three cards when it resolves (one added in response): two lands can't pay.
        let mut t = TestGame::new(2);
        fill_graveyard(&mut t, P0, "Opt", 2);
        t.set_step(P1, Step::PrecombatMain);
        let div = cast_card(&mut t, P1, "Divination", &[]);
        t.lands(P1, "Wastes", 2);
        cast_card(&mut t, P0, card, &[obj(div)]);
        t.graveyard(P0, "Opt");
        t.answer_yes(P1, true);
        t.resolve_all();
        assert_eq!(t.hand_size(P1), 0, "{card}: countered");
    }
}

// --- The source is already in the graveyard and counts itself -------------------------------

#[test]
fn sources_in_the_graveyard_count_themselves() {
    cr!("608.2h", "113.7a", "602.2b");
    ruling!(
        "Elvish Eulogist",
        "By the time this ability resolves, Elvish Eulogist will most likely be in your graveyard, so it will be counted by its own ability."
    );
    ruling!(
        "Undead Servant",
        "Count the number of Undead Servants in your graveyard as the ability resolves to determine how many tokens to put onto the battlefield. If Undead Servant dies in response to its own triggered ability and is in your graveyard as that ability resolves, it will count toward the number of Zombies you get."
    );
    ruling!(
        "Sunflare Shaman",
        "If Sunflare Shaman leaves the battlefield before its ability resolves, it will still deal damage to the target. If Sunflare Shaman is put into your graveyard, it even counts toward the amount of damage dealt."
    );
    supported("Elvish Eulogist");
    supported("Undead Servant");
    supported("Sunflare Shaman");
    // Elvish Eulogist (sacrificed as the cost) and one other Elf card: 2 life.
    let mut t = TestGame::new(2);
    let ee = t.battlefield(P0, "Elvish Eulogist");
    t.graveyard(P0, "Llanowar Elves");
    t.activate(P0, ee, 0, &[]).unwrap();
    assert!(t.in_graveyard(P0, "Elvish Eulogist"));
    t.resolve_all();
    assert_eq!(t.life(P0), 22);

    // Undead Servant: one in the graveyard; another enters (an ETB trigger counts at
    // resolution). Dies in response: two Zombies; stays: one.
    for dies in [false, true] {
        let mut t = TestGame::new(2);
        t.graveyard(P0, "Undead Servant");
        let us = t.enter(P0, "Undead Servant");
        t.settle();
        if dies {
            kill(&mut t, us);
        }
        t.resolve_all();
        assert_eq!(tokens(&t, P0).len(), 1 + dies as usize, "Undead Servant dies: {dies}");
    }

    // Sunflare Shaman: one Elemental card in the graveyard; the Shaman is destroyed in
    // response: 2 damage to P1 (counting itself).
    let mut t = TestGame::new(2);
    let ss = t.battlefield(P0, "Sunflare Shaman");
    t.graveyard(P0, "Sunflare Shaman");
    t.lands(P0, "Mountain", 2);
    t.activate(P0, ss, 0, &[pl(P1)]).unwrap();
    kill(&mut t, ss);
    t.resolve_all();
    assert_eq!(t.life(P1), 18);
    // ... and exiled in response instead: only the one in the graveyard counts.
    let mut t = TestGame::new(2);
    let ss = t.battlefield(P0, "Sunflare Shaman");
    t.graveyard(P0, "Sunflare Shaman");
    t.lands(P0, "Mountain", 2);
    t.activate(P0, ss, 0, &[pl(P1)]).unwrap();
    exile_now(&mut t, ss);
    t.resolve_all();
    assert_eq!(t.life(P1), 19);
}

// --- Card types -----------------------------------------------------------------------------

#[test]
fn card_types_among_graveyard_cards() {
    cr!("205.2a", "205.4a", "308.1");
    ruling!(
        "Lucid Dreams",
        "A card can have more than one type. For example, an artifact creature in your graveyard counts as both types for Lucid Dreams' effect."
    );
    ruling!(
        "Lucid Dreams",
        "Card types that can appear on cards in a graveyard include artifact, battle, creature, enchantment, instant, kindred, land, planeswalker, and sorcery. Legendary, basic, and snow are supertypes, not card types; Vedalken and Wizard are subtypes, not card types."
    );
    ruling!(
        "Scuttletide",
        "The card types that can appear in a graveyard are artifact, battle, creature, enchantment, instant, kindred, land, planeswalker, and sorcery. Legendary, basic, and snow are supertypes, not card types; Vedalken and Wizard are subtypes, not card types."
    );
    supported("Nameless Inversion");
    supported("Invasion of Zendikar // Awakened Skyclave");
    supported("Lucid Dreams");
    // An artifact creature: two types, two cards.
    let mut t = TestGame::new(2);
    t.graveyard(P0, "Ornithopter");
    let hand = t.hand_size(P0);
    cast_card(&mut t, P0, "Lucid Dreams", &[]);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand + 2);
    // A basic snow land (land), a legendary creature (creature), a Vedalken Wizard
    // (creature), a kindred instant (kindred, instant), a battle: five types.
    let mut t = TestGame::new(2);
    for n in [
        "Snow-Covered Forest",
        "Isamaru, Hound of Konda",
        "Vedalken Aethermage",
        "Nameless Inversion",
        "Invasion of Zendikar // Awakened Skyclave",
    ] {
        t.graveyard(P0, n);
    }
    assert_eq!(graveyard_types(&t, P0), 5);
    let hand = t.hand_size(P0);
    cast_card(&mut t, P0, "Lucid Dreams", &[]);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand + 5);

    // Scuttletide's delirium: land, creature (legendary / Vedalken Wizard) — no bonus;
    // a kindred instant adds two types: four.
    supported("Scuttletide");
    let mut t = TestGame::new(2);
    let st = t.battlefield(P0, "Scuttletide");
    for n in ["Snow-Covered Forest", "Isamaru, Hound of Konda", "Vedalken Aethermage"] {
        t.graveyard(P0, n);
    }
    t.lands(P0, "Island", 1);
    t.hand(P0, "Opt");
    t.answer_choose(P0, &[]);
    t.activate(P0, st, 0, &[]).unwrap();
    t.resolve_all();
    let crab = tokens(&t, P0)[0];
    // The discarded Opt made three types (land, creature, instant).
    assert_eq!(t.pt(crab), (0, 3));
    t.graveyard(P0, "Nameless Inversion");
    t.g.recompute();
    assert_eq!(t.pt(crab), (1, 4));
}

#[test]
fn april_oneil_counts_card_types_not_supertypes_or_subtypes() {
    cr!("205.2a", "205.4a", "308.1");
    ruling!(
        "April O'Neil, Hacktivist",
        "The card types in Magic include artifact, battle, creature, enchantment, instant, kindred, land, planeswalker, and sorcery. Legendary, basic, and snow are supertypes, not card types; Human and Scientist are subtypes, not card types."
    );
    supported("April O'Neil, Hacktivist");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "April O'Neil, Hacktivist");
    let bears = t.battlefield(P1, "Grizzly Bears");
    // A legendary creature and a kindred instant: creature, kindred, instant.
    cast_card(&mut t, P0, "Isamaru, Hound of Konda", &[]);
    t.resolve_all();
    cast_card(&mut t, P0, "Nameless Inversion", &[obj(bears)]);
    t.resolve_all();
    let hand = t.hand_size(P0);
    t.advance_to(P0, Step::End);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand + 3);
}

#[test]
fn warfire_javelineer_counts_a_split_card_once() {
    cr!("709.4c", "608.2h");
    ruling!(
        "Warfire Javelineer",
        "A split card only counts once for Warfire Javelineer’s ability, even if it’s both an instant and a sorcery."
    );
    supported("Commit // Memory");
    let mut t = TestGame::new(2);
    let wurm = t.battlefield(P1, "Craw Wurm");
    t.graveyard(P0, "Commit // Memory");
    t.graveyard(P0, "Opt");
    t.answer_targets(P0, &[obj(wurm)]);
    t.enter(P0, "Warfire Javelineer");
    t.resolve_all();
    assert_eq!(dmg(&t, wurm), 2);
}

#[test]
fn ral_doesnt_count_face_down_exiled_cards() {
    cr!("708.2", "406.3");
    ruling!(
        "Ral, Izzet Viceroy",
        "If any exiled cards you own are face down, they have no characteristics. If they're normally instants or sorceries, they won't be counted for Ral's second ability."
    );
    let mut t = TestGame::new(2);
    let wurm = t.battlefield(P1, "Craw Wurm");
    let ral = t.battlefield(P0, "Ral, Izzet Viceroy");
    t.exile(P0, "Lightning Bolt");
    let down = t.exile(P0, "Lightning Bolt");
    t.g.objects[down.0 as usize].face_down = true;
    t.graveyard(P0, "Opt");
    t.g.recompute();
    t.activate(P0, ral, 1, &[obj(wurm)]).unwrap();
    t.resolve_all();
    assert_eq!(dmg(&t, wurm), 2);
}

// --- Mana values ----------------------------------------------------------------------------

#[test]
fn sewer_crocodile_needs_five_different_mana_values() {
    cr!("202.3", "601.2f");
    ruling!(
        "Sewer Crocodile",
        "\"Five or more mana values among cards in your graveyard\" means there are at least five different mana values among those cards."
    );
    supported("Sewer Crocodile");
    // Six cards, four values: {3}{U}; a fifth value: {U}.
    for (fifth, lands) in [(false, 4), (true, 1)] {
        let mut t = TestGame::new(2);
        let sc = t.battlefield(P0, "Sewer Crocodile");
        for n in ["Ornithopter", "Memnite", "Llanowar Elves", "Grizzly Bears", "Divination", "Opt"] {
            t.graveyard(P0, n);
        }
        if fifth {
            t.graveyard(P0, "Hill Giant");
        }
        t.lands(P0, "Island", lands);
        assert!(t.activate(P0, sc, 0, &[]).is_ok(), "fifth {fifth}");
        assert_eq!(crate::r_s01_common::tapped_lands(&t, P0), lands, "fifth {fifth}");
        if !fifth {
            // With four values one Island isn't enough.
            let mut t = TestGame::new(2);
            let sc = t.battlefield(P0, "Sewer Crocodile");
            for n in ["Ornithopter", "Llanowar Elves", "Grizzly Bears", "Divination"] {
                t.graveyard(P0, n);
            }
            t.lands(P0, "Island", 1);
            assert!(t.activate(P0, sc, 0, &[]).is_err());
        }
    }
}

#[test]
fn sewer_crocodile_cost_is_locked_before_mana_abilities() {
    cr!("601.2f", "601.2g", "602.2b");
    ruling!(
        "Sewer Crocodile",
        "Costs to activate an ability are determined only once, before mana abilities are activated. If cards are removed from your graveyard after that point, such as by exiling Jack-o'-Lantern from your graveyard to pay for its mana ability, this won't affect the cost of activating Sewer Crocodile, even if that reduces the number of mana values in your graveyard to below five."
    );
    // The ruling's example (Jack-o'-Lantern's mana ability from the graveyard, which costs
    // {1}) isn't planned by the automatic payment; Molt Tender's "{T}, Exile a card from your
    // graveyard: Add one mana of any color." removes a card the same way.
    supported("Molt Tender");
    // Values 0, 1, 2, 3, 4: the cost is {U}, paid by Molt Tender exiling one of them.
    let mut t = TestGame::new(2);
    let sc = t.battlefield(P0, "Sewer Crocodile");
    for n in ["Ornithopter", "Llanowar Elves", "Grizzly Bears", "Divination", "Hill Giant"] {
        t.graveyard(P0, n);
    }
    let mt = t.battlefield(P0, "Molt Tender");
    let r = t.activate(P0, sc, 0, &[]);
    assert!(r.is_ok(), "{r:?}");
    assert!(t.obj_now(mt).tapped);
    assert_eq!(t.graveyard_size(P0), 4, "a card was exiled for mana");
    t.resolve_all();
    let bears = t.battlefield(P1, "Grizzly Bears");
    crate::r_s01_common::attack_with(&mut t, &[(sc, pl(P1))]);
    assert!(!t.g.can_block(bears, sc), "can't be blocked this turn");
    // Getting the mana first (outside the activation) leaves four values: {3}{U}.
    let mut t = TestGame::new(2);
    let sc = t.battlefield(P0, "Sewer Crocodile");
    for n in ["Ornithopter", "Llanowar Elves", "Grizzly Bears", "Divination", "Hill Giant"] {
        t.graveyard(P0, n);
    }
    let mt = t.battlefield(P0, "Molt Tender");
    assert!(tap_for_mana(&mut t, P0, mt, "Add"));
    assert_eq!(t.graveyard_size(P0), 4);
    assert!(t.activate(P0, sc, 0, &[]).is_err());
}

// --- Costs and colors -----------------------------------------------------------------------

#[test]
fn locket_of_yesterdays_reduces_only_generic_mana() {
    cr!("601.2f", "118.7a");
    ruling!(
        "Locket of Yesterdays",
        "Colored costs can’t be reduced by this effect. For example, a spell that costs {2}{R} can’t be reduced by more than {2}."
    );
    supported("Locket of Yesterdays");
    // Three Divinations in the graveyard: Divination costs {U}, not nothing.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Locket of Yesterdays");
    fill_graveyard(&mut t, P0, "Divination", 3);
    let div = t.hand(P0, "Divination");
    assert!(t.cast(P0, div).try_go().is_err(), "{{U}} can't be reduced");
    t.lands(P0, "Island", 1);
    t.cast(P0, div).go();
    assert_eq!(crate::r_s01_common::tapped_lands(&t, P0), 1);
}

#[test]
fn voracious_reader_reductions_apply_after_increases_and_x() {
    cr!("601.2f", "107.3a");
    ruling!(
        "Curious Homunculus // Voracious Reader",
        "Abilities that reduce the cost of a spell apply after abilities that raise them. For example, Borrowed Malevolence with both modes chosen costs {1}{B} to cast if you control Voracious Reader."
    );
    ruling!(
        "Curious Homunculus // Voracious Reader",
        "If an instant or sorcery spell has {X} in its mana cost, choose the value for X first, and then reduce the cost by {1}. For example, Dark Salvation with X chosen as 2 costs {3}{B} to cast if you control Voracious Reader."
    );
    supported("Borrowed Malevolence");
    supported("Dark Salvation");
    let reader = |t: &mut TestGame| {
        let ch = t.battlefield(P0, "Curious Homunculus // Voracious Reader");
        assert!(mtg_engine::dfc::transform(&mut t.g, ch));
        t.g.recompute();
    };
    // Borrowed Malevolence ({B}, escalate {1}) with both modes: {1}{B} — two Swamps.
    let mut t = TestGame::new(2);
    reader(&mut t);
    t.lands(P0, "Swamp", 2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let giant = t.battlefield(P1, "Hill Giant");
    let bm = t.hand(P0, "Borrowed Malevolence");
    t.cast(P0, bm).modes(&[0, 1]).target(bears).target(giant).go();
    assert_eq!(crate::r_s01_common::tapped_lands(&t, P0), 2);
    t.resolve_all();
    assert_eq!(t.pt(giant), (2, 2));
    // Dark Salvation, X = 2 ({X}{X}{B} = {4}{B}): {3}{B} — four Swamps, not three.
    for (lands, ok) in [(3, false), (4, true)] {
        let mut t = TestGame::new(2);
        reader(&mut t);
        t.lands(P0, "Swamp", lands);
        let ds = t.hand(P0, "Dark Salvation");
        t.answer_targets(P0, &[]);
        let r = t.cast(P0, ds).x(2).target(pl(P0)).try_go();
        assert_eq!(r.is_ok(), ok, "{lands} lands");
        if ok {
            assert_eq!(crate::r_s01_common::tapped_lands(&t, P0), 4);
            t.resolve_all();
            assert_eq!(tokens(&t, P0).len(), 2);
        }
    }
}

#[test]
fn corrupted_grafstone_cant_add_colorless() {
    cr!("105.2c", "106.1b");
    ruling!(
        "Corrupted Grafstone",
        "Colorless cards have no color, so you can't add {C} this way."
    );
    // Only colorless cards in the graveyard: no mana.
    let mut t = TestGame::new(2);
    let cg = t.battlefield(P0, "Corrupted Grafstone");
    t.graveyard(P0, "Ornithopter");
    t.graveyard(P0, "Wastes");
    assert!(!tap_for_mana(&mut t, P0, cg, "Add"));
    assert_eq!(t.g.player(P0).mana_pool.total(), 0);
    // A red card and a colorless one: only red is offered.
    let mut t = TestGame::new(2);
    let cg = t.battlefield(P0, "Corrupted Grafstone");
    t.graveyard(P0, "Ornithopter");
    t.graveyard(P0, "Lightning Bolt");
    let from = t.asked().len();
    assert!(tap_for_mana(&mut t, P0, cg, "Add"));
    assert_eq!(
        t.g.player(P0).mana_pool.count(mtg_engine::mana::ManaType::R),
        1
    );
    assert_eq!(
        t.g.player(P0).mana_pool.count(mtg_engine::mana::ManaType::C),
        0
    );
    for (_, d) in &t.asked()[from..] {
        if let Decision::ChooseOption { options, .. } = d {
            assert!(options.iter().all(|o| !o.contains('C')), "{options:?}");
        }
    }
}

// --- Damage -----------------------------------------------------------------------------------

#[test]
fn gandalfs_sanction_excess_ignores_prevention_and_indestructible() {
    cr!("120.4a", "120.6");
    ruling!(
        "Gandalf's Sanction",
        "Excess damage caused by a spell or ability is similar to how combat damage from a creature with trample is handled. Start with the amount of damage being dealt to the creature and determine what is \"lethal.\" This is the creature's toughness minus the amount of damage that it already has marked on it, but ignoring any replacement or prevention effects that will modify this damage. Also ignore whether the creature has an ability such as indestructible that will result in it not being destroyed by this damage."
    );
    // Four instants: 4 damage to a Hill Giant with 1 damage marked: lethal is 2, 2 excess.
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P1, "Hill Giant");
    t.g.objects[giant.0 as usize].damage = 1;
    fill_graveyard(&mut t, P0, "Opt", 4);
    cast_card(&mut t, P0, "Gandalf's Sanction", &[obj(giant)]);
    t.resolve_all();
    assert!(!t.on_battlefield(giant));
    assert_eq!(t.life(P1), 18);
    // An indestructible 0/1 (Darksteel Myr): lethal is 1, 3 excess; it survives.
    let mut t = TestGame::new(2);
    let myr = t.battlefield(P1, "Darksteel Myr");
    fill_graveyard(&mut t, P0, "Opt", 4);
    cast_card(&mut t, P0, "Gandalf's Sanction", &[obj(myr)]);
    t.resolve_all();
    assert!(t.on_battlefield(myr));
    assert_eq!(t.life(P1), 17);
}

#[test]
fn combustion_technique_exiles_on_any_death_this_turn() {
    cr!("614.1a", "614.6");
    ruling!(
        "Combustion Technique",
        "Combustion Technique's replacement effect will exile the target creature if it would die this turn for any reason, not just due to lethal damage."
    );
    supported("Combustion Technique");
    let mut t = TestGame::new(2);
    let wurm = t.battlefield(P1, "Craw Wurm");
    cast_card(&mut t, P0, "Combustion Technique", &[obj(wurm)]);
    t.resolve_all();
    assert_eq!(dmg(&t, wurm), 2);
    sacrifice(&mut t, wurm);
    assert!(t.in_exile("Craw Wurm"));
    assert!(!t.in_graveyard(P1, "Craw Wurm"));
}

// --- Looking at X cards; greatest power ---------------------------------------------------

#[test]
fn sinuous_benthisaur_with_x_1_and_0() {
    cr!("609.3", "608.2h");
    ruling!(
        "Sinuous Benthisaur",
        "If X is 1, you'll look at the top card of your library and put it into your hand. If X is 0, you won't look at any cards from the top of your library."
    );
    supported("Sinuous Benthisaur");
    supported("Hidden Cataract");
    for caves in [0usize, 1] {
        let mut t = TestGame::new(2);
        stack_library(&mut t, P0, &["Opt", "Divination"]);
        if caves == 1 {
            t.graveyard(P0, "Hidden Cataract");
        }
        let hand = t.hand_size(P0);
        let lib = t.library_size(P0);
        let from = t.asked().len();
        t.enter(P0, "Sinuous Benthisaur");
        t.resolve_all();
        assert_eq!(t.hand_size(P0), hand + caves, "caves {caves}");
        assert_eq!(t.in_hand(P0, "Opt"), caves == 1);
        assert_eq!(t.library_size(P0), lib - caves);
        // Nothing to choose among.
        let looked = t.asked()[from..]
            .iter()
            .any(|(_, d)| matches!(d, Decision::ChooseEntities { candidates, .. } if !candidates.is_empty()));
        assert!(!looked || caves == 1, "caves {caves}");
    }
}

#[test]
fn ambitious_dragonborn_isnt_on_the_battlefield_yet() {
    cr!("614.1c", "614.12");
    ruling!(
        "Ambitious Dragonborn",
        "Ambitious Dragonborn is not yet on the battlefield when its effect looks for the creature or creature card with the greatest power, so any creatures that have a bonus to their power based on the number of creatures on the battlefield won't count it until after it enters with counters."
    );
    supported("Ambitious Dragonborn");
    supported("Crusader of Odric");
    let mut t = TestGame::new(2);
    let crusader = t.battlefield(P0, "Crusader of Odric");
    t.battlefield(P0, "Grizzly Bears");
    assert_eq!(t.pt(crusader), (2, 2));
    let ad = t.enter(P0, "Ambitious Dragonborn");
    t.resolve_all();
    assert_eq!(t.counters(ad, counters::PLUS1), 2);
    assert_eq!(t.pt(crusader), (3, 3));
    assert_eq!(creatures(&t, P0).len(), 3);
}

// --- Spell mastery / delirium resolving ------------------------------------------------------

#[test]
fn descend_upon_the_sinful_doesnt_count_itself() {
    cr!("608.2n", "608.2h");
    ruling!(
        "Descend upon the Sinful",
        "If you have three non-sorcery card types among cards in your graveyard at the time Descend upon the Sinful resolves, you won't get an Angel token. Descend upon the Sinful isn't put into your graveyard until after it's finished resolving."
    );
    // Land, instant, enchantment: three types — no Angel; a sorcery added: an Angel.
    for sorcery in [false, true] {
        let mut t = TestGame::new(2);
        for n in ["Forest", "Opt", "Pacifism"] {
            t.graveyard(P0, n);
        }
        if sorcery {
            t.graveyard(P0, "Divination");
        }
        t.battlefield(P1, "Grizzly Bears");
        cast_card(&mut t, P0, "Descend upon the Sinful", &[]);
        t.resolve_all();
        assert_eq!(tokens(&t, P0).len(), sorcery as usize, "sorcery {sorcery}");
        assert!(t.in_exile("Grizzly Bears"));
    }
}

#[test]
fn mana_abilities_that_exile_graveyard_cards_pay_costs() {
    cr!("601.2g", "118.3");
    // Regression (found with Sewer Crocodile's ruling): the automatic payment plans mana
    // abilities that cost "Exile a card from your graveyard", one card per cost.
    supported("Molt Tender");
    for (cards, ok) in [(1, false), (2, true)] {
        let mut t = TestGame::new(2);
        t.lands(P0, "Island", 1);
        let a = t.battlefield(P0, "Molt Tender");
        let b = t.battlefield(P0, "Molt Tender");
        fill_graveyard(&mut t, P0, "Opt", cards);
        let div = t.hand(P0, "Divination");
        assert_eq!(t.cast(P0, div).try_go().is_ok(), ok, "{cards} cards");
        if ok {
            assert!(t.obj_now(a).tapped && t.obj_now(b).tapped);
            assert_eq!(t.graveyard_size(P0), 0);
        }
    }
}
