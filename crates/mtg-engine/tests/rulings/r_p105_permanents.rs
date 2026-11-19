//! Rulings batch P105 — permanents and copies: gained activated abilities (Grell
//! Philosopher), animated artifacts that are animated again (Izzet Keyrune, the Monuments),
//! effects that track an object (Adverse Conditions), copies entering the battlefield (Copy
//! Land, City of Death, Romana II), entering as a noncreature (The Warring Triad), exile
//! that outlasts a copy or a turn (Wondrous Crucible, Harnfel), Auras returned without
//! targeting (Estrid, the Masked) and spells cast during resolution (Jetsam).

use crate::r_p104_common::obj;
use crate::r_p105_common::*;
use crate::r_s01_common::*;
use crate::r_s02_common::destroy;
use crate::r_s04_common::add_mana;
use crate::r_s05_common::enter;
use crate::r_s06_common::{activate_containing, attached_to, give_control};
use crate::r_s17_common::token_copy;
use crate::r_s26_common::modify_until_eot;
use mtg_engine::ability::{Modification, Value};
use mtg_engine::decision::Decision;
use mtg_engine::mana::ManaType;
use mtg_engine::object::{CastMethod, Zone};
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

// ---------------------------------------------------------------------------
// Grell Philosopher: "... each Horror you control gains all activated abilities of target
// artifact an opponent controls until end of turn. ..."
// ---------------------------------------------------------------------------

#[test]
fn grell_philosopher_gained_abilities_refer_to_the_grell() {
    cr!("201.5b", "613.1f");
    ruling!(
        "Grell Philosopher",
        "The granted abilities effectively use “this permanent” rather than “[that card's name],” so you treat the abilities as if they were printed on the creature that gained the ability."
    );
    supported("Grell Philosopher");
    supported("Universal Solvent");
    let mut t = TestGame::new(2);
    let grell = t.battlefield(P0, "Grell Philosopher");
    let solvent = t.battlefield(P1, "Universal Solvent");
    let giant = t.battlefield(P1, "Hill Giant");
    t.lands(P0, "Island", 7);
    // At P0's upkeep, Grell's trigger targets Universal Solvent.
    t.answer_targets(P0, &[obj(solvent)]);
    t.set_step(P1, Step::End);
    t.advance_to(P0, Step::Upkeep);
    t.resolve_all();
    t.advance_to(P0, Step::PrecombatMain);
    // "{7}, {T}, Sacrifice Universal Solvent: Destroy target permanent." on the Grell:
    // sacrifice the Grell.
    t.answer_targets(P0, &[obj(giant)]);
    activate_containing(&mut t, P0, grell, "Destroy target permanent").unwrap();
    assert!(!t.on_battlefield(grell));
    assert!(t.on_battlefield(solvent));
    t.resolve_all();
    assert!(!t.on_battlefield(giant));
    assert!(t.on_battlefield(solvent));
}

// ---------------------------------------------------------------------------
// Animating an artifact that's already a creature
// ---------------------------------------------------------------------------

#[test]
fn animating_again_overrides_set_power_but_keeps_modifications() {
    cr!("613.4b", "613.4c", "613.7");
    ruling!(
        "Izzet Keyrune",
        "Activating the ability that turns the Keyrune into a creature while it’s already a creature will override any effects that set its power and/or toughness to another number, but effects that modify power and/or toughness without directly setting them will still apply."
    );
    ruling!(
        "Atarka Monument",
        "Activating the last ability of a Monument while it's already a creature will override any effects that set its power or toughness to a specific value. Effects that modify power or toughness without directly setting them to a specific value will continue to apply."
    );
    supported("Izzet Keyrune");
    supported("Atarka Monument");
    // (artifact, lands, animated power and toughness)
    let cases: [(&str, &[&str], (i32, i32)); 2] = [
        ("Izzet Keyrune", &["Island", "Mountain"], (2, 1)),
        (
            "Atarka Monument",
            &["Mountain", "Forest", "Wastes", "Wastes", "Wastes", "Wastes"],
            (4, 4),
        ),
    ];
    for (name, lands, (p, x)) in cases {
        let mut t = TestGame::new(2);
        let a = t.battlefield(P0, name);
        let animate = |t: &mut TestGame| {
            for l in lands {
                t.lands(P0, l, 1);
            }
            activate_containing(t, P0, a, "becomes").unwrap();
            t.resolve_all();
        };
        animate(&mut t);
        assert_eq!(t.pt(a), (p, x), "{name}");
        // Set to 0/1, then +3/+3: 3/4.
        modify_until_eot(
            &mut t,
            a,
            vec![Modification::SetPT(Some(Value::c(0)), Some(Value::c(1)))],
        );
        modify_until_eot(&mut t, a, vec![Modification::ModifyPT(Value::c(3), Value::c(3))]);
        assert_eq!(t.pt(a), (3, 4), "{name}");
        // Animated again: its own values are back, plus 3/3.
        animate(&mut t);
        assert_eq!(t.pt(a), (p + 3, x + 3), "{name}");
    }
}

// ---------------------------------------------------------------------------
// Adverse Conditions: "Tap up to two target creatures. Those creatures don't untap during
// their controller's next untap step. ..."
// ---------------------------------------------------------------------------

#[test]
fn adverse_conditions_tracks_the_creatures_not_their_controller() {
    cr!("611.2c", "502.3", "613.1b");
    ruling!(
        "Adverse Conditions",
        "Adverse Conditions tracks the creatures, but not their controller. If either of the creatures changes controllers before its first controller's next untap step has come around, then it won't untap during its new controller's next untap step."
    );
    supported("Adverse Conditions");
    let mut t = TestGame::new(2);
    let stolen = t.battlefield(P1, "Grizzly Bears");
    let kept = t.battlefield(P1, "Hill Giant");
    t.lands(P0, "Island", 4);
    let c = t.hand(P0, "Adverse Conditions");
    t.cast(P0, c).targets(&[obj(stolen), obj(kept)]).go();
    t.resolve_all();
    assert!(t.obj_now(stolen).tapped && t.obj_now(kept).tapped);
    // P0 gains control of one of them before P1's untap step.
    give_control(&mut t, stolen, P0);
    t.advance_to(P1, Step::Upkeep);
    assert!(t.obj_now(kept).tapped);
    t.advance_to(P0, Step::Upkeep);
    // It didn't untap during its new controller's next untap step.
    assert_eq!(t.obj_now(stolen).controller, P0);
    assert!(t.obj_now(stolen).tapped);
    t.advance_to(P1, Step::Upkeep);
    assert!(!t.obj_now(kept).tapped);
    t.advance_to(P0, Step::Upkeep);
    assert!(!t.obj_now(stolen).tapped);
}

// ---------------------------------------------------------------------------
// Copies entering the battlefield
// ---------------------------------------------------------------------------

#[test]
fn copy_land_gets_the_copied_lands_enters_abilities() {
    cr!("707.5", "614.1c", "603.6a");
    ruling!(
        "Copy Land",
        "Any enters-the-battlefield abilities of the copied permanent will trigger when Copy Land enters the battlefield. Any \"as [this land] enters the battlefield\" or \"[this land] enters the battlefield with\" abilities of the chosen permanent will also work."
    );
    supported("Copy Land");
    supported("Khalni Garden");
    // Khalni Garden: "This land enters tapped. When this land enters, create a 0/1 green
    // Plant creature token."
    let mut t = TestGame::new(2);
    let garden = t.battlefield(P1, "Khalni Garden");
    t.lands(P0, "Island", 3);
    let c = t.hand(P0, "Copy Land");
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[obj(garden)]);
    let spell = t.cast(P0, c).go();
    t.resolve_all();
    let copy = t.obj_now(spell);
    assert_eq!(copy.chars.name, "Khalni Garden");
    assert!(copy.is(CardType::Enchantment) && copy.is(CardType::Land));
    assert!(copy.tapped);
    assert_eq!(crate::r_s05_common::tokens_with_subtype(&t, P0, "Plant").len(), 1);
}

/// A token copy of `name` for P0 that entered this turn.
fn token_of(t: &mut TestGame, name: &str) -> ObjectId {
    let card = t.graveyard(P1, name);
    let tokens = token_copy(t, P0, card);
    t.resolve_all();
    tokens[0]
}

#[test]
fn copied_tokens_enters_abilities_work() {
    cr!("707.5", "614.1c", "111.10");
    ruling!(
        "City of Death",
        "Any enters-the-battlefield abilities of the copied token will trigger when the token enters the battlefield. Any \"as [this permanent] enters the battlefield\" or \"[this permanent] enters the battlefield with\" abilities of the chosen token will also work."
    );
    ruling!(
        "Romana II",
        "Any enters-the-battlefield abilities of the copied token will trigger when the token enters the battlefield. Any \"as [this permanent] enters the battlefield\" or \"[this permanent] enters the battlefield with\" abilities of the chosen token will also work."
    );
    supported("City of Death");
    supported("Romana II");
    supported("Corsair Captain");
    supported("Bloatfly Swarm");
    let treasures = |t: &TestGame| crate::r_s05_common::tokens_with_subtype(t, P0, "Treasure").len();
    // City of Death, chapter II: "Create a token that's a copy of target non-Saga token
    // you control."
    let mut t = TestGame::new(2);
    let captain = token_of(&mut t, "Corsair Captain");
    let swarm = token_of(&mut t, "Bloatfly Swarm");
    let before = treasures(&t);
    let city = t.battlefield(P0, "City of Death");
    set_counters(&mut t, city, counters::LORE, 1);
    t.answer_targets(P0, &[obj(captain)]);
    crate::r_s19_common::add_lore(&mut t, city, 1);
    t.resolve_all();
    assert_eq!(treasures(&t), before + 1);
    t.answer_targets(P0, &[obj(swarm)]);
    crate::r_s19_common::add_lore(&mut t, city, 1);
    t.resolve_all();
    let swarms = t.named_on_battlefield("Bloatfly Swarm");
    assert_eq!(swarms.len(), 2);
    assert!(swarms.iter().all(|s| t.counters(*s, counters::PLUS1) == 5));
    // Romana II: "{1}, {T}: Create a tapped token that's a copy of target token that
    // entered this turn."
    let mut t = TestGame::new(2);
    let romana = t.battlefield(P0, "Romana II");
    t.lands(P0, "Wastes", 2);
    let captain = token_of(&mut t, "Corsair Captain");
    let before = treasures(&t);
    t.activate(P0, romana, 0, &[obj(captain)]).unwrap();
    t.resolve_all();
    assert_eq!(treasures(&t), before + 1);
    let mut t = TestGame::new(2);
    let romana = t.battlefield(P0, "Romana II");
    t.lands(P0, "Wastes", 2);
    let swarm = token_of(&mut t, "Bloatfly Swarm");
    t.activate(P0, romana, 0, &[obj(swarm)]).unwrap();
    t.resolve_all();
    let swarms = t.named_on_battlefield("Bloatfly Swarm");
    assert_eq!(swarms.len(), 2);
    assert!(swarms.iter().all(|s| t.counters(*s, counters::PLUS1) == 5));
}

#[test]
fn the_warring_triad_enters_as_a_creature_only_with_eight_cards_in_the_graveyard() {
    cr!("614.12", "614.1c");
    ruling!(
        "The Warring Triad",
        "As The Warring Triad enters the battlefield, the number of cards in your graveyard will determine whether any replacement effects that affect creatures entering the battlefield apply to The Warring Triad."
    );
    supported("The Warring Triad");
    supported("Tayam, Luminous Enigma");
    // Tayam: "Each other creature you control enters with an additional vigilance counter
    // on it."
    for (gy, counter) in [(7usize, 0u32), (8, 1)] {
        let mut t = TestGame::new(2);
        t.battlefield(P0, "Tayam, Luminous Enigma");
        for _ in 0..gy {
            t.graveyard(P0, "Island");
        }
        let triad = enter(&mut t, P0, "The Warring Triad");
        t.settle();
        assert_eq!(t.counters(triad, "vigilance"), counter, "{gy} cards");
    }
}

// ---------------------------------------------------------------------------
// Exiled cards stay exiled
// ---------------------------------------------------------------------------

#[test]
fn wondrous_crucible_card_stays_exiled_whatever_happens_to_the_copy() {
    cr!("707.10", "707.12", "111.1");
    ruling!(
        "Wondrous Crucible",
        "The card remains exiled no matter what happens to the copy."
    );
    supported("Wondrous Crucible");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Wondrous Crucible");
    let bears = t.graveyard(P0, "Grizzly Bears");
    stack_library(&mut t, P0, &["Island", "Forest"]);
    t.advance_to(P0, Step::End);
    t.answer_yes(P0, true);
    t.resolve_all();
    assert_eq!(t.zone(bears), Zone::Exile);
    // The copy became a token Grizzly Bears; it dies, and the card is still in exile.
    let token = t.named_on_battlefield("Grizzly Bears");
    assert_eq!(token.len(), 1);
    assert!(t.obj(token[0]).is_token());
    destroy(&mut t, token[0]);
    t.resolve_all();
    assert_eq!(t.zone(bears), Zone::Exile);
}

#[test]
fn harnfel_cards_not_played_stay_exiled() {
    cr!("611.2a", "514.2");
    ruling!(
        "Birgi, God of Storytelling // Harnfel, Horn of Bounty",
        "Any cards exiled with Harnfel that you don’t play remain exiled."
    );
    supported("Birgi, God of Storytelling // Harnfel, Horn of Bounty");
    let mut t = TestGame::new(2);
    add_mana(&mut t, P0, ManaType::R, 1);
    add_mana(&mut t, P0, ManaType::C, 4);
    let b = t.hand(P0, "Birgi, God of Storytelling // Harnfel, Horn of Bounty");
    let spell = t.cast(P0, b).method(CastMethod::Half(1)).go();
    t.resolve_all();
    let harnfel = t.g.current(spell);
    let lib = stack_library(&mut t, P0, &["Lightning Bolt", "Shock"]);
    t.hand(P0, "Plains");
    activate_containing(&mut t, P0, harnfel, "Exile the top two").unwrap();
    t.resolve_all();
    assert!(lib.iter().all(|c| t.zone(*c) == Zone::Exile));
    t.advance_to(P1, Step::Upkeep);
    assert!(lib.iter().all(|c| t.zone(*c) == Zone::Exile));
    // They can't be cast any more.
    t.advance_to(P0, Step::PrecombatMain);
    t.lands(P0, "Mountain", 2);
    let bolt = t.g.current(lib[0]);
    assert!(!crate::r_s02_common::can_cast(&mut t, P0, bolt, CastMethod::Normal));
}

// ---------------------------------------------------------------------------
// Estrid, the Masked: "−7: Mill seven cards. Return all non-Aura enchantment cards from
// your graveyard to the battlefield, then do the same for Aura cards."
// ---------------------------------------------------------------------------

#[test]
fn estrid_returning_auras_may_enchant_hexproof_permanents() {
    cr!("303.4f", "702.11b", "115.1");
    ruling!(
        "Estrid, the Masked",
        "You choose what each returning Aura will enchant just before that Aura enters the battlefield. The chosen recipient must be able to legally be enchanted by the Aura. This doesn't target the player or permanent it will enchant, so an opponent's permanent with hexproof may be chosen this way."
    );
    supported("Estrid, the Masked");
    supported("Gladecover Scout");
    let mut t = TestGame::new(2);
    let e = t.battlefield(P0, "Estrid, the Masked");
    set_loyalty(&mut t, e, 7);
    let scout = t.battlefield(P1, "Gladecover Scout");
    t.battlefield(P0, "Grizzly Bears");
    t.battlefield(P1, "Hill Giant");
    let pacifism = t.graveyard(P0, "Pacifism");
    t.answer_choose(P0, &[obj(scout)]);
    let from = t.asked().len();
    t.activate(P0, e, 2, &[]).unwrap();
    t.resolve_all();
    assert_eq!(attached_to(&t, pacifism), Some(obj(scout)));
    assert!(!t.asked()[from..]
        .iter()
        .any(|(_, d)| matches!(d, Decision::ChooseTargets { .. })));
}

// ---------------------------------------------------------------------------
// Jetsam: "Each opponent mills three cards, then you may cast a spell from each
// opponent's graveyard without paying its mana cost. ..."
// ---------------------------------------------------------------------------

#[test]
fn jetsam_casts_during_resolution_ignoring_timing() {
    cr!("608.2g", "601.2", "118.9");
    ruling!(
        "Flotsam // Jetsam",
        "You choose whether to cast spells from opponents' graveyards as Jetsam resolves. If you do, you do so as part of the resolution of Jetsam. You can't wait to cast them later in the turn. Timing restrictions based on the cards' types are ignored."
    );
    supported("Flotsam // Jetsam");
    // Cast: the creature spell goes on the stack while Jetsam is resolving (a sorcery,
    // with a spell on the stack).
    let mut t = TestGame::new(2);
    add_mana(&mut t, P0, ManaType::U, 6);
    let bears = t.graveyard(P1, "Grizzly Bears");
    let j = t.hand(P0, "Flotsam // Jetsam");
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[obj(bears)]);
    t.cast(P0, j).method(CastMethod::Half(1)).go();
    t.resolve();
    let spell = t.g.current(bears);
    assert_eq!(t.zone(spell), Zone::Stack);
    t.resolve_all();
    assert!(t.on_battlefield(spell));
    assert_eq!(t.obj_now(spell).controller, P0);
    // Declined: the card can't be cast later.
    let mut t = TestGame::new(2);
    add_mana(&mut t, P0, ManaType::U, 6);
    let bears = t.graveyard(P1, "Grizzly Bears");
    let j = t.hand(P0, "Flotsam // Jetsam");
    t.answer_yes(P0, false);
    t.answer_choose(P0, &[]);
    t.cast(P0, j).method(CastMethod::Half(1)).go();
    t.resolve_all();
    assert_eq!(t.zone(bears), Zone::Graveyard(P1));
    add_mana(&mut t, P0, ManaType::G, 2);
    assert!(!crate::r_s02_common::can_cast(&mut t, P0, bears, CastMethod::Normal));
    assert!(!crate::r_s02_common::can_cast(&mut t, P0, bears, CastMethod::Free));
}
