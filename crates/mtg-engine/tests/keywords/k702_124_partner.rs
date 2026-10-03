//! CR 702.124 Partner.

use crate::common_k702_111_124::*;
use crate::k702_001_010_common::custom_card;
use mtg_engine::ability::Filter;
use mtg_engine::card::{card, CardDef};
use mtg_engine::deck::DeckProblem;
use mtg_engine::eval::Ctx;
use mtg_engine::game::GameConfig;
use mtg_engine::kw::partner::{check_commander_deck, commanders_problem};
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::types::*;
use mtg_engine::game::Variant;
use mtg_engine::*;
use std::sync::Arc;

fn can_pair(a: &str, b: &str) -> bool {
    commanders_problem(&[&card(a), &card(b)]).is_none()
}

/// A Commander deck: the commanders plus `n` copies of `land`.
fn deck(commanders: &[&str], land: &str, n: usize) -> Vec<Arc<CardDef>> {
    commanders
        .iter()
        .map(|c| card(c))
        .chain((0..n).map(|_| card(land)))
        .collect()
}

fn cards(names: &[&str]) -> Vec<Arc<CardDef>> {
    names.iter().map(|n| card(n)).collect()
}

/// Puts `name` into `p`'s command zone as one of their commanders.
fn commander(t: &mut TestGame, p: PlayerId, name: &str) -> ObjectId {
    let id = t.command(p, name);
    t.g.objects[id.0 as usize].is_commander = true;
    t.g.players[p.idx()].commander_names.push(name.into());
    id
}

fn commander_game() -> TestGame {
    TestGame::with_config(
        2,
        GameConfig {
            variant: Variant::Commander,
            ..Default::default()
        },
    )
}

#[test]
fn partner_lets_two_legendary_cards_with_partner_be_commanders() {
    cr!("702.124", "702.124a", "702.124h");
    for c in ["Kraum, Ludevic's Opus", "Tymna the Weaver"] {
        assert!(card(c).faces[0].chars.has_keyword(mtg_engine::keywords::KeywordKind::Partner));
    }
    assert!(can_pair("Kraum, Ludevic's Opus", "Tymna the Weaver"));
    // A commander without partner can't be paired with one.
    assert!(!can_pair("Kraum, Ludevic's Opus", "Isamaru, Hound of Konda"));
    assert!(!can_pair("Isamaru, Hound of Konda", "Kraum, Ludevic's Opus"));
    // Either can still be a commander on its own.
    assert!(commanders_problem(&[&card("Kraum, Ludevic's Opus")]).is_none());
    // Both must be legendary.
    let plain = custom_card("Plain Partner", "Creature — Soldier", "{2}", Some((2, 2)), "Partner");
    assert!(commanders_problem(&[&card("Kraum, Ludevic's Opus"), &plain]).is_some());
}

#[test]
fn the_deck_has_100_cards_including_both_commanders() {
    cr!("702.124b");
    ruling!(
        "Kraum, Ludevic's Opus",
        "Both commanders start in the command zone, and the remaining 98 cards"
    );
    let pair = cards(&["Kraum, Ludevic's Opus", "Tymna the Weaver"]);
    let ok = deck(&["Kraum, Ludevic's Opus", "Tymna the Weaver"], "Island", 98);
    assert!(check_commander_deck(&ok, &pair, &[], false).is_empty());
    let big = deck(&["Kraum, Ludevic's Opus", "Tymna the Weaver"], "Island", 99);
    assert!(check_commander_deck(&big, &pair, &[], false)
        .contains(&DeckProblem::TooManyCards { have: 101, max: 100 }));
    // Both commanders begin the game in the command zone.
    let mut lib = deck(&["Kraum, Ludevic's Opus", "Tymna the Weaver"], "Island", 28);
    lib.rotate_left(1);
    let mut t = pregame(
        GameConfig {
            variant: Variant::Commander,
            skip_mulligans: true,
            ..Default::default()
        },
        vec![lib, deck(&[], "Island", 30)],
    );
    assert!(t.g.designate_commander(P0, "Kraum, Ludevic's Opus"));
    assert!(t.g.designate_commander(P0, "Tymna the Weaver"));
    t.g.start();
    for c in ["Kraum, Ludevic's Opus", "Tymna the Weaver"] {
        let ids = t.g.find_in_zone(Zone::Command, c);
        assert_eq!(ids.len(), 1, "{c} in the command zone");
        assert!(t.g.obj(ids[0]).is_commander);
    }
    assert_eq!(t.library_size(P0) + t.hand_size(P0), 28);
}

#[test]
fn the_commanders_combined_color_identity() {
    cr!("702.124c");
    ruling!(
        "Kraum, Ludevic's Opus",
        "If your Commander deck has two commanders, you can only include cards whose own color identities are also found in your commanders' combined color identities."
    );
    // Kraum is blue-red, Tymna white-black.
    let pair = cards(&["Kraum, Ludevic's Opus", "Tymna the Weaver"]);
    let mut d = deck(&["Kraum, Ludevic's Opus", "Tymna the Weaver"], "Island", 95);
    d.extend(cards(&["Plains", "Swamp", "Mountain"]));
    assert!(check_commander_deck(&d, &pair, &[], false).is_empty());
    d.pop();
    d.push(card("Forest"));
    assert!(check_commander_deck(&d, &pair, &[], false)
        .contains(&DeckProblem::OutsideColorIdentity { name: "Forest".into() }));
}

#[test]
fn each_commander_has_its_own_commander_tax() {
    cr!("702.124d");
    ruling!(
        "Kraum, Ludevic's Opus",
        "If you cast one, you won't have to pay an additional {2} the first time you cast the other."
    );
    let mut t = commander_game();
    let kraum = commander(&mut t, P0, "Kraum, Ludevic's Opus");
    let tymna = commander(&mut t, P0, "Tymna the Weaver");
    // Kraum: {3}{U}{R}.
    t.lands(P0, "Island", 4);
    t.lands(P0, "Mountain", 1);
    t.cast(P0, kraum).go();
    t.resolve_all();
    assert!(t.on_battlefield(kraum));
    // Kraum dies and returns to the command zone.
    t.answer_yes(P0, true);
    t.g.destroy(t.g.current(kraum), None);
    t.settle();
    assert_eq!(t.zone(kraum), Zone::Command);
    // Tymna ({1}{W}{B}) costs no more for Kraum having been cast.
    t.lands(P0, "Plains", 1);
    t.lands(P0, "Swamp", 1);
    t.lands(P0, "Wastes", 1);
    let tymna_spell = t.cast(P0, tymna).try_go();
    assert!(tymna_spell.is_ok());
    assert_eq!(untapped_lands(&t, P0), 0);
    t.resolve_all();
    // Kraum again: {3}{U}{R} plus {2}.
    let kraum = t.g.current(kraum);
    t.lands(P0, "Island", 4);
    t.lands(P0, "Mountain", 1);
    assert!(!castable(&mut t, P0, kraum, mtg_engine::object::CastMethod::Normal));
    t.lands(P0, "Wastes", 2);
    t.cast(P0, kraum).go();
    assert_eq!(untapped_lands(&t, P0), 0);
}

#[test]
fn a_commander_is_taxed_for_casts_of_either_of_its_faces() {
    // The commander tax counts casts of the commander (the card), whichever face of a
    // modal double-faced card was cast.
    cr!("903.8");
    let mut t = commander_game();
    // Halvar, God of Battle ({2}{W}{W}) // Sword of the Realms ({1}{W}).
    let halvar = commander(&mut t, P0, "Halvar, God of Battle");
    t.lands(P0, "Plains", 2);
    t.cast(P0, halvar)
        .method(mtg_engine::object::CastMethod::Half(1))
        .go();
    assert_eq!(untapped_lands(&t, P0), 0);
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Sword of the Realms").len(), 1);
    // The Sword is destroyed and returns to the command zone (CR 903.9a).
    t.answer_yes(P0, true);
    t.g.destroy(t.g.current(halvar), None);
    t.settle();
    assert_eq!(t.zone(halvar), Zone::Command);
    // Halvar now costs {2}{W}{W} plus {2}.
    let halvar = t.g.current(halvar);
    t.lands(P0, "Plains", 4);
    assert!(!castable(&mut t, P0, halvar, mtg_engine::object::CastMethod::Normal));
    t.lands(P0, "Plains", 2);
    t.cast(P0, halvar).go();
    assert_eq!(untapped_lands(&t, P0), 0);
}

#[test]
fn commander_damage_is_counted_for_each_commander_separately() {
    cr!("702.124d");
    ruling!(
        "Kraum, Ludevic's Opus",
        "A player loses the game after having been dealt 21 damage from any one of them, not from both of them combined."
    );
    let mut t = commander_game();
    t.g.players[1].life = 40;
    t.g.players[1]
        .commander_damage
        .insert("Kraum, Ludevic's Opus".into(), 20);
    let tymna = t.battlefield(P0, "Tymna the Weaver");
    t.g.objects[tymna.0 as usize].is_commander = true;
    let kraum = t.battlefield(P0, "Kraum, Ludevic's Opus");
    t.g.objects[kraum.0 as usize].is_commander = true;
    declare_attack(&mut t, &[(tymna, Entity::Player(P1))]);
    finish_combat(&mut t, P1, &[]);
    t.settle();
    // 22 combat damage from commanders in all, but at most 20 from either.
    assert!(!t.has_lost(P1));
    t.advance_to(P1, mtg_engine::turn::Step::Upkeep);
    t.advance_to(P0, mtg_engine::turn::Step::PrecombatMain);
    declare_attack(&mut t, &[(kraum, Entity::Player(P1))]);
    t.answer(P1, DecisionKind::Blockers, decision::Answer::Blockers(vec![]));
    // 24 from Kraum: P1 loses (CR 903.10a) and the game ends.
    assert!(t.g.run_until(10_000, |g| g.result.is_some()));
    assert!(t.has_lost(P1));
    assert!(t.life(P1) > 0);
}

#[test]
fn an_effect_on_your_commander_affects_the_one_you_choose() {
    cr!("702.124e");
    ruling!(
        "Kraum, Ludevic's Opus",
        "If you are instructed to perform an action on your commander (e.g. put it from the command zone into your hand due to Command Beacon), you choose one of your commanders at the time the effect happens."
    );
    assert_supported_card("Command Beacon");
    let mut t = commander_game();
    let kraum = commander(&mut t, P0, "Kraum, Ludevic's Opus");
    let tymna = commander(&mut t, P0, "Tymna the Weaver");
    // Command Beacon: "{T}, Sacrifice this land: Put your commander into your hand from
    // the command zone."
    let beacon = t.battlefield(P0, "Command Beacon");
    t.answer_choose(P0, &[Entity::Object(tymna)]);
    // Its owner doesn't put it back into the command zone instead (CR 903.9b).
    t.answer_yes(P0, false);
    t.activate(P0, beacon, 1, &[]).unwrap();
    t.resolve_all();
    assert_eq!(t.zone(tymna), Zone::Hand(P0));
    assert_eq!(t.zone(kraum), Zone::Command);
}

#[test]
fn an_effect_on_commanders_you_own_affects_both() {
    cr!("702.124e");
    ruling!(
        "Wilson, Refined Grizzly",
        "If you control a Background that grants an ability to commander creatures you own, and you own more than one commander creature, each of them will have that ability."
    );
    let mut t = commander_game();
    // Raised by Giants: "Commander creatures you own have base power and toughness 10/10
    // and are Giants in addition to their other types."
    t.battlefield(P0, "Raised by Giants");
    let a = t.battlefield(P0, "Kraum, Ludevic's Opus");
    let b = t.battlefield(P0, "Tymna the Weaver");
    t.g.objects[a.0 as usize].is_commander = true;
    t.g.objects[b.0 as usize].is_commander = true;
    t.g.recompute();
    assert_eq!(t.pt(a), (10, 10));
    assert_eq!(t.pt(b), (10, 10));
}

#[test]
fn different_partner_abilities_cant_be_combined() {
    cr!("702.124f");
    ruling!(
        "Rose Tyler",
        "Notably, Time Lord Doctors and cards with Doctor's companion do not interact with cards which have another partner ability."
    );
    // Partner and partner with [name].
    assert!(!can_pair("Kraum, Ludevic's Opus", "Pir, Imaginative Rascal"));
    // Partner and partner—[text].
    assert!(!can_pair("Kraum, Ludevic's Opus", "Sophina, Spearsage Deserter"));
    // Partner and choose a Background.
    assert!(!can_pair("Kraum, Ludevic's Opus", "Wilson, Refined Grizzly"));
    assert!(!can_pair("Kraum, Ludevic's Opus", "Raised by Giants"));
    // Partner and Doctor's companion or a Doctor.
    assert!(!can_pair("Kraum, Ludevic's Opus", "Rose Tyler"));
    assert!(!can_pair("Kraum, Ludevic's Opus", "The Tenth Doctor"));
    assert!(!can_pair("Rose Tyler", "Raised by Giants"));
}

#[test]
fn a_card_with_several_partner_abilities_uses_one_of_them() {
    cr!("702.124g");
    let both = custom_card(
        "Two Partners",
        "Legendary Creature — Human",
        "{2}",
        Some((2, 2)),
        "Partner\nPartner—Friends forever",
    );
    let kraum = card("Kraum, Ludevic's Opus");
    let sophina = card("Sophina, Spearsage Deserter");
    assert!(commanders_problem(&[&both, &kraum]).is_none());
    assert!(commanders_problem(&[&both, &sophina]).is_none());
    // Never more than two commanders.
    assert!(commanders_problem(&[&both, &kraum, &sophina]).is_some());
    let pair = [Arc::new(both), kraum];
    let d: Vec<Arc<CardDef>> = pair
        .iter()
        .cloned()
        .chain((0..98).map(|_| card("Island")))
        .collect();
    assert!(check_commander_deck(&d, &pair, &[], false).is_empty());
}

#[test]
fn partner_with_text_needs_the_same_text() {
    cr!("702.124i");
    // Sophina and Othelm: partner—Friends forever; Ellie, Brick Master: partner—Survivors.
    assert!(can_pair(
        "Sophina, Spearsage Deserter",
        "Othelm, Sigardian Outcast"
    ));
    assert!(can_pair("Ellie, Brick Master", "Joel, Resolute Survivor"));
    assert!(!can_pair("Sophina, Spearsage Deserter", "Ellie, Brick Master"));
}

#[test]
fn partner_with_a_name_pairs_only_those_two() {
    cr!("702.124j");
    ruling!(
        "Pir, Imaginative Rascal",
        "A creature with a \"partner with\" ability can't partner with any creature other than its designated partner."
    );
    assert!(can_pair(
        "Pir, Imaginative Rascal",
        "Toothy, Imaginary Friend"
    ));
    assert!(can_pair(
        "Toothy, Imaginary Friend",
        "Pir, Imaginative Rascal"
    ));
    assert!(!can_pair("Pir, Imaginative Rascal", "Tymna the Weaver"));
    assert!(!can_pair("Pir, Imaginative Rascal", "Pir, Imaginative Rascal"));
}

#[test]
fn partner_with_a_name_searches_for_the_partner_as_it_enters() {
    cr!("702.124j");
    ruling!(
        "Pir, Imaginative Rascal",
        "The triggered ability of the \"partner with\" keyword still triggers in a Commander game."
    );
    assert_supported_card("Toothy, Imaginary Friend");
    let mut t = TestGame::new(2);
    t.library_top(P0, "Pir, Imaginative Rascal");
    for _ in 0..3 {
        t.library_top(P0, "Island");
    }
    // Toothy: "Partner with Pir, Imaginative Rascal (When this creature enters, target
    // player may put Pir into their hand from their library, then shuffle.)"
    t.lands(P0, "Island", 4);
    let toothy = t.hand(P0, "Toothy, Imaginary Friend");
    t.cast(P0, toothy).go();
    t.resolve();
    assert_eq!(on_stack(&t, "Partner with"), 1);
    t.resolve_all();
    assert!(t.in_hand(P0, "Pir, Imaginative Rascal"));
    // Another player can be the target.
    let mut t = TestGame::new(2);
    t.library_top(P1, "Toothy, Imaginary Friend");
    t.lands(P0, "Forest", 3);
    let pir = t.hand(P0, "Pir, Imaginative Rascal");
    t.cast(P0, pir).go();
    t.answer_targets(P0, &[Entity::Player(P1)]);
    t.resolve_all();
    assert!(t.in_hand(P1, "Toothy, Imaginary Friend"));
}

#[test]
fn choose_a_background_pairs_with_a_legendary_background() {
    cr!("702.124k");
    ruling!(
        "Wilson, Refined Grizzly",
        "You may have two commanders if one of them is a legendary creature with the choose a background ability and the other is a legendary Background enchantment."
    );
    assert!(can_pair("Wilson, Refined Grizzly", "Raised by Giants"));
    assert!(can_pair("Raised by Giants", "Wilson, Refined Grizzly"));
    // A Background can't be a commander on its own, or with another partner.
    assert!(commanders_problem(&[&card("Raised by Giants")]).is_some());
    assert!(!can_pair("Tymna the Weaver", "Raised by Giants"));
    // Nor can a creature with choose a Background pair with anything but a Background.
    assert!(!can_pair("Wilson, Refined Grizzly", "Isamaru, Hound of Konda"));
    assert!(!can_pair("Wilson, Refined Grizzly", "Wilson, Refined Grizzly"));
}

#[test]
fn doctors_companion_pairs_with_a_time_lord_doctor() {
    cr!("702.124m");
    ruling!(
        "Rose Tyler",
        "The Doctor's companion ability allows you to have two commanders if one has the ability and the other is a legendary creature that is a Time Lord Doctor and has no other creature types."
    );
    assert!(card("The Tenth Doctor").faces[0]
        .chars
        .subtypes
        .iter()
        .any(|s| s == "Time Lord"));
    assert!(can_pair("Rose Tyler", "The Tenth Doctor"));
    assert!(can_pair("The Tenth Doctor", "Rose Tyler"));
    // A Time Lord who isn't a Doctor, or a Doctor with another creature type.
    assert!(!can_pair("Rose Tyler", "Susan Foreman"));
    let doctor_rogue = custom_card(
        "Doctor Rogue",
        "Legendary Creature — Time Lord Doctor Rogue",
        "{2}",
        Some((2, 2)),
        "",
    );
    assert!(commanders_problem(&[&card("Rose Tyler"), &doctor_rogue]).is_some());
    // Two Doctors, or a Doctor with another commander, aren't allowed.
    assert!(!can_pair("The Tenth Doctor", "The Eleventh Doctor"));
    assert!(!can_pair("The Tenth Doctor", "Isamaru, Hound of Konda"));
}

#[test]
fn partner_refers_only_to_partner_partner_text_and_partner_with() {
    cr!("702.124n");
    let mut t = TestGame::new(2);
    let has_partner = Filter::HasKeyword(mtg_engine::keywords::KeywordKind::Partner);
    let ctx = Ctx::new(None, P0);
    for (name, yes) in [
        ("Kraum, Ludevic's Opus", true),
        ("Sophina, Spearsage Deserter", true),
        ("Pir, Imaginative Rascal", true),
        ("Wilson, Refined Grizzly", false),
        ("Rose Tyler", false),
    ] {
        let id = t.battlefield(P0, name);
        assert_eq!(t.g.matches(id, &has_partner, &ctx), yes, "{name}");
    }
}
