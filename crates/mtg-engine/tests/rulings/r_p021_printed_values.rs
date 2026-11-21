//! Rulings batch P021 — copies of a creature copy exactly what was printed on it (its
//! copiable values, plus any copy effects and the copy's own exceptions) and nothing
//! else: not whether it's tapped, its counters, the Auras and Equipment attached to it,
//! or non-copy effects that changed its power, toughness, types or color (CR 707.2,
//! 707.9). The copied creature here is a dressed-up Hill Giant (see
//! [`dressed_giant`]).

use crate::r_p021_common::*;
use crate::r_s01_common::supported;
use crate::r_s02_common::destroy;
use crate::r_s06_common::{activate_containing, has_kw};
use crate::r_s18_common::lands_for;
use crate::r_s24_common::choose_creature_type;
use crate::r_s26_common::new_tokens;
use mtg_engine::keywords::KeywordKind;
use mtg_engine::object::CastMethod;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

/// `p`'s `copier` enters as a copy of a dressed-up Hill Giant controlled by `giant_p`.
fn enters_copying_giant(copier: &str, p: PlayerId, giant_p: PlayerId) -> (TestGame, ObjectId) {
    supported(copier);
    let mut t = TestGame::new(2);
    let giant = dressed_giant(&mut t, giant_p);
    let c = crate::r_p023_common::enter_copying(&mut t, p, copier, giant);
    assert!(!t.obj_now(c).is_token());
    (t, c)
}

/// Casts `p`'s `spell` (paying `cost` with basic lands) with one target per entry of
/// `targets`, resolves everything, and returns the tokens `p` got.
fn cast_for_tokens(
    t: &mut TestGame,
    p: PlayerId,
    spell: &str,
    cost: &str,
    targets: &[Entity],
) -> Vec<ObjectId> {
    supported(spell);
    lands_for(t, p, cost);
    let card = t.hand(p, spell);
    let before = t.g.battlefield.clone();
    let mut b = t.cast(p, card);
    for e in targets {
        b = b.target(*e);
    }
    b.go();
    t.resolve_all();
    new_tokens(t, p, &before)
}

// --- Entering as a copy ------------------------------------------------------------------

#[test]
fn deceptive_frostkite_copies_only_the_printed_creature() {
    cr!("707.2", "707.9b", "614.1c");
    ruling!(
        "Deceptive Frostkite",
        "Deceptive Frostkite copies exactly what was printed on the original creature and nothing else"
    );
    // The Hill Giant's power is 10 now (it qualifies), but the copy is a printed 3/3.
    let (t, c) = enters_copying_giant("Deceptive Frostkite", P0, P0);
    assert_printed_giant(&t, c, Some((3, 3)), true);
    assert!(has_subtype(&t, c, "Dragon"));
    assert!(has_kw(&t, c, KeywordKind::Flying));
}

#[test]
fn evil_twin_copies_only_the_printed_creature() {
    cr!("707.2", "707.9a");
    ruling!(
        "Evil Twin",
        "Evil Twin copies exactly what was printed on the original creature (unless that creature is copying something else or is a token; see below) and it gains the activated ability."
    );
    let (t, c) = enters_copying_giant("Evil Twin", P0, P1);
    assert_printed_giant(&t, c, Some((3, 3)), true);
    assert_eq!(
        abilities_with(&t, c, "Destroy target creature with the same name"),
        1
    );
}

#[test]
fn synth_infiltrator_copies_only_the_printed_creature() {
    cr!("707.2", "707.9b");
    ruling!(
        "Synth Infiltrator",
        "Except for being a Synth artifact creature in addition to its other types, Synth Infiltrator copies exactly what was printed on the original creature and nothing else"
    );
    let (t, c) = enters_copying_giant("Synth Infiltrator", P0, P1);
    assert_printed_giant(&t, c, Some((3, 3)), true);
    assert!(t.obj_now(c).is(CardType::Artifact) && has_subtype(&t, c, "Synth"));
}

#[test]
fn quicksilver_gargantuan_copies_only_the_printed_creature() {
    cr!("707.2", "707.9b");
    ruling!(
        "Quicksilver Gargantuan",
        "Except for its power and toughness, Quicksilver Gargantuan copies exactly what was printed on the original creature and nothing more"
    );
    let (t, c) = enters_copying_giant("Quicksilver Gargantuan", P0, P1);
    assert_printed_giant(&t, c, Some((7, 7)), true);
}

#[test]
fn machine_gods_effigy_copies_only_the_printed_creature() {
    cr!("707.2", "707.9b");
    ruling!(
        "Machine God's Effigy",
        "Except for its types and non-artifact subtypes, Machine God’s Effigy copies exactly what was printed on the original creature and nothing more"
    );
    let (mut t, c) = enters_copying_giant("Machine God's Effigy", P0, P1);
    assert_printed(&t, c, "Hill Giant", red(), None, true);
    assert!(t.obj_now(c).is(CardType::Artifact) && !t.obj_now(c).is(CardType::Creature));
    assert!(t.obj_now(c).chars.subtypes.is_empty());
    activate_containing(&mut t, P0, c, "Add {U}").expect("mana ability");
    assert_eq!(
        t.g.player(P0)
            .mana_pool
            .count(mtg_engine::mana::ManaType::U),
        1
    );
}

#[test]
fn mocking_doppelganger_copies_only_the_printed_creature() {
    cr!("707.2", "707.9a");
    ruling!(
        "Mocking Doppelganger",
        "Except for the added ability, Mocking Doppelganger copies exactly what was printed on the original creature"
    );
    let (t, c) = enters_copying_giant("Mocking Doppelganger", P0, P1);
    assert_printed_giant(&t, c, Some((3, 3)), true);
    assert_eq!(abilities_with(&t, c, "goaded"), 1);
}

#[test]
fn copycrook_copies_only_the_printed_creature() {
    cr!("707.2", "707.9a");
    ruling!(
        "Copycrook",
        "Except for the listed exception, Copycrook copies exactly what was printed on the original creature and nothing more"
    );
    let (t, c) = enters_copying_giant("Copycrook", P0, P1);
    assert_printed_giant(&t, c, Some((3, 3)), true);
    assert_eq!(abilities_with(&t, c, "connives"), 1);
}

#[test]
fn malleable_impostor_copies_only_the_printed_creature() {
    cr!("707.2", "707.9b");
    ruling!(
        "Malleable Impostor",
        "Except for the listed exceptions, Malleable Impostor copies exactly what was printed on the original permanent and nothing more"
    );
    let (t, c) = enters_copying_giant("Malleable Impostor", P0, P1);
    assert_printed_giant(&t, c, Some((3, 3)), true);
    assert!(has_subtype(&t, c, "Faerie") && has_subtype(&t, c, "Shapeshifter"));
    assert!(has_kw(&t, c, KeywordKind::Flying));
}

#[test]
fn visage_bandit_copies_only_the_printed_creature() {
    cr!("707.2", "707.9b");
    ruling!(
        "Visage Bandit",
        "Except for the listed exceptions, Visage Bandit copies exactly what was printed on the original creature and nothing else"
    );
    let (t, c) = enters_copying_giant("Visage Bandit", P0, P0);
    assert_printed_giant(&t, c, Some((3, 3)), true);
    assert!(has_subtype(&t, c, "Shapeshifter") && has_subtype(&t, c, "Rogue"));
}

#[test]
fn gigantoplasm_copies_only_the_printed_creature() {
    cr!("707.2", "707.9a");
    ruling!(
        "Gigantoplasm",
        "Gigantoplasm copies exactly what was printed on the original creature and nothing more"
    );
    let (t, c) = enters_copying_giant("Gigantoplasm", P0, P1);
    assert_printed_giant(&t, c, Some((3, 3)), true);
    assert_eq!(abilities_with(&t, c, "base power and toughness X/X"), 1);
}

#[test]
fn glasspool_mimic_copies_only_the_printed_creature() {
    cr!("707.2", "707.9b");
    ruling!(
        "Glasspool Mimic // Glasspool Shore",
        "Glasspool Mimic copies exactly what was printed on the original creature (unless that creature is copying something else or is a token; see below), except that it's also a Shapeshifter Rogue."
    );
    let (t, c) = enters_copying_giant("Glasspool Mimic // Glasspool Shore", P0, P0);
    assert_printed_giant(&t, c, Some((3, 3)), true);
    assert!(has_subtype(&t, c, "Shapeshifter") && has_subtype(&t, c, "Rogue"));
}

// --- Becoming a copy -------------------------------------------------------------------------

#[test]
fn muddle_copies_only_the_printed_creature() {
    cr!("707.2", "707.9a");
    ruling!(
        "Muddle, the Ever-Changing",
        "Except for the listed exception, Muddle copies exactly what was printed on the target creature and nothing else"
    );
    supported("Muddle, the Ever-Changing");
    let mut t = TestGame::new(2);
    let muddle = t.battlefield(P0, "Muddle, the Ever-Changing");
    let giant = dressed_giant(&mut t, P0);
    // Casting an instant (Giant Growth on the Giant) makes Muddle copy the Giant.
    lands_for(&mut t, P0, "{G}");
    let growth = t.hand(P0, "Giant Growth");
    t.cast(P0, growth).target(obj(giant)).go();
    t.answer_targets(P0, &[obj(giant)]);
    t.resolve_all();
    assert_eq!(t.pt(giant), (13, 11));
    assert_printed_giant(&t, muddle, Some((3, 3)), true);
    assert!(has_kw(&t, muddle, KeywordKind::Myriad));
}

#[test]
fn nanogene_conversion_copies_the_printed_values_and_creatures_keep_their_counters() {
    cr!("707.2", "707.9b", "613.1a");
    ruling!(
        "Nanogene Conversion",
        "Each other creature copies the printed values of the targeted creature, except that they aren't legendary."
    );
    supported("Nanogene Conversion");
    let mut t = TestGame::new(2);
    let isamaru = t.battlefield(P0, "Isamaru, Hound of Konda");
    dress(&mut t, P0, isamaru);
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.g.add_counters(obj(bears), counters::PLUS1, 1, None);
    let giant = t.battlefield(P0, "Hill Giant");
    let toks = cast_for_tokens(&mut t, P0, "Nanogene Conversion", "{3}{U}", &[obj(isamaru)]);
    assert!(toks.is_empty());
    for c in [bears, giant] {
        let o = t.obj_now(c);
        assert_eq!(o.chars.name, "Isamaru, Hound of Konda");
        assert_eq!(o.chars.colors, ColorSet::single(Color::White));
        assert!(!legendary(&t, c));
    }
    // A printed 2/2; the Bears keeps its own +1/+1 counter, the Giant has none.
    assert_eq!(t.pt(bears), (3, 3));
    assert_eq!(t.counters(bears, counters::PLUS1), 1);
    assert_eq!(t.pt(giant), (2, 2));
    // Isamaru itself is unchanged: still legendary and dressed up.
    assert!(legendary(&t, isamaru));
    assert_eq!(t.pt(isamaru), (9, 7));
}

#[test]
fn mirrorweave_copies_the_printed_values_plus_copy_effects() {
    cr!("707.2", "707.3", "613.1a");
    ruling!(
        "Mirrorweave",
        "Each other creature copies the printed values of the targeted creature, plus any copy effects that have been applied to that creature."
    );
    supported("Mirrorweave");
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P1, "Hill Giant");
    let clone = crate::r_p023_common::clone_of(&mut t, P1, giant);
    dress(&mut t, P1, clone);
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.g.add_counters(obj(bears), counters::PLUS1, 1, None);
    cast_for_tokens(&mut t, P0, "Mirrorweave", "{2}{U}{U}", &[obj(clone)]);
    // The Bears is a Hill Giant (what the Clone is copying), with its own counter only.
    assert_eq!(t.obj_now(bears).chars.name, "Hill Giant");
    assert_eq!(t.obj_now(bears).chars.colors, red());
    assert_eq!(t.pt(bears), (4, 4));
    // The original Hill Giant too, with no counters.
    assert_printed_giant(&t, giant, Some((3, 3)), true);
}

#[test]
fn identity_thief_copies_the_printed_values_plus_copy_effects_but_not_counters() {
    cr!("707.2", "707.3");
    ruling!(
        "Identity Thief",
        "Identity Thief copies the printed values of the targeted creature, plus any copy effects that have been applied to that creature."
    );
    supported("Identity Thief");
    let mut t = TestGame::new(2);
    let thief = t.battlefield(P0, "Identity Thief");
    let giant = t.battlefield(P1, "Hill Giant");
    let clone = crate::r_p023_common::clone_of(&mut t, P1, giant);
    dress(&mut t, P1, clone);
    t.set_step(P0, Step::BeginningOfCombat);
    t.answer_yes(P0, true);
    t.answer_targets(P0, &[obj(clone)]);
    t.attack(&[(thief, Entity::Player(P1))], &[]);
    assert!(!t.on_battlefield(clone));
    // The Thief is a Hill Giant (tapped from attacking), with no counters.
    assert_printed_giant(&t, thief, Some((3, 3)), false);
}

#[test]
fn hall_of_mirrors_copies_the_printed_values_and_creatures_keep_their_counters() {
    cr!("707.2", "613.1a", "702.159a");
    ruling!(
        "Hall of Mirrors",
        "Hall of Mirrors copies the printed values of the target creature. It won’t copy any counters or stickers on that creature"
    );
    supported("Hall of Mirrors");
    let mut t = TestGame::new(2);
    let hall = t.battlefield(P0, "Hall of Mirrors");
    let giant = dressed_giant(&mut t, P0);
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.g.add_counters(obj(bears), counters::PLUS1, 1, None);
    let other_bears = t.battlefield(P1, "Grizzly Bears");
    let lit = mtg_engine::card::card("Hall of Mirrors").attraction_lights[0];
    let _ = hall;
    t.g.dice.loaded.push_back(lit);
    t.answer_targets(P0, &[obj(giant)]);
    mtg_engine::variants::roll_to_visit(&mut t.g, P0);
    t.g.flush_events();
    t.resolve_all();
    // The Bears is a printed Hill Giant that keeps its own counter.
    assert_eq!(t.obj_now(bears).chars.name, "Hill Giant");
    assert_eq!(t.obj_now(bears).chars.colors, red());
    assert_eq!(t.pt(bears), (4, 4));
    // Only creatures you control.
    assert_eq!(t.obj_now(other_bears).chars.name, "Grizzly Bears");
}

// --- Token copies ----------------------------------------------------------------------------

#[test]
fn hate_mirage_tokens_copy_only_the_printed_creature() {
    cr!("707.2");
    ruling!(
        "Hate Mirage",
        "Each token copies exactly what was printed on the original creature and nothing else"
    );
    let mut t = TestGame::new(2);
    let giant = dressed_giant(&mut t, P1);
    let toks = cast_for_tokens(&mut t, P0, "Hate Mirage", "{3}{R}", &[obj(giant)]);
    assert_eq!(toks.len(), 1);
    assert_printed_giant(&t, toks[0], Some((3, 3)), true);
    assert!(has_kw(&t, toks[0], KeywordKind::Haste));
}

#[test]
fn kindred_charge_tokens_copy_only_the_printed_creature() {
    cr!("707.2");
    ruling!(
        "Kindred Charge",
        "Each token copies exactly what was printed on the original creature and nothing else (unless that creature is copying something else; see below)."
    );
    let mut t = TestGame::new(2);
    dressed_giant(&mut t, P0);
    choose_creature_type(&mut t, P0, "Giant");
    let toks = cast_for_tokens(&mut t, P0, "Kindred Charge", "{4}{R}{R}", &[]);
    assert_eq!(toks.len(), 1);
    assert_printed_giant(&t, toks[0], Some((3, 3)), true);
}

#[test]
fn clone_legion_tokens_copy_only_the_printed_creature() {
    cr!("707.2");
    ruling!(
        "Clone Legion",
        "Each token copies exactly what was printed on the original creature and nothing else (unless that permanent is copying something else or is a token; see below)."
    );
    let mut t = TestGame::new(2);
    dressed_giant(&mut t, P1);
    let toks = cast_for_tokens(
        &mut t,
        P0,
        "Clone Legion",
        "{7}{U}{U}",
        &[Entity::Player(P1)],
    );
    assert_eq!(toks.len(), 1);
    assert_printed_giant(&t, toks[0], Some((3, 3)), true);
}

#[test]
fn molten_duplication_token_copies_only_the_printed_creature() {
    cr!("707.2", "707.9b");
    ruling!(
        "Molten Duplication",
        "Except for the listed exceptions, the token copies exactly what was printed on the original permanent and nothing else"
    );
    let mut t = TestGame::new(2);
    let giant = dressed_giant(&mut t, P0);
    let toks = cast_for_tokens(&mut t, P0, "Molten Duplication", "{1}{R}", &[obj(giant)]);
    assert_eq!(toks.len(), 1);
    assert_printed_giant(&t, toks[0], Some((3, 3)), true);
    assert!(t.obj_now(toks[0]).is(CardType::Artifact));
}

#[test]
fn mirror_room_token_copies_only_the_printed_creature() {
    cr!("707.2", "707.9b", "709.5");
    ruling!(
        "Mirror Room // Fractured Realm",
        "Except for the stated exception, the token created by Mirror Room's ability copies exactly what was printed on the original creature and nothing else"
    );
    supported("Mirror Room // Fractured Realm");
    let mut t = TestGame::new(2);
    let giant = dressed_giant(&mut t, P0);
    lands_for(&mut t, P0, "{2}{U}");
    let card = t.hand(P0, "Mirror Room // Fractured Realm");
    let before = t.g.battlefield.clone();
    t.cast(P0, card).method(CastMethod::Half(0)).go();
    t.answer_targets(P0, &[obj(giant)]);
    t.resolve_all();
    let toks = new_tokens(&t, P0, &before);
    assert_eq!(toks.len(), 1);
    assert_printed_giant(&t, toks[0], Some((3, 3)), true);
    assert!(has_subtype(&t, toks[0], "Reflection"));
}

#[test]
fn the_jolly_balloon_man_token_copies_only_the_printed_creature() {
    cr!("707.2", "707.9b");
    ruling!(
        "The Jolly Balloon Man",
        "Except for the listed exceptions, the token created by The Jolly Balloon Man's ability copies exactly what was printed on the original creature and nothing else"
    );
    supported("The Jolly Balloon Man");
    let mut t = TestGame::new(2);
    let man = t.battlefield(P0, "The Jolly Balloon Man");
    let giant = dressed_giant(&mut t, P0);
    lands_for(&mut t, P0, "{1}");
    let before = t.g.battlefield.clone();
    t.activate(P0, man, 0, &[obj(giant)]).expect("activate");
    t.resolve_all();
    let toks = new_tokens(&t, P0, &before);
    assert_eq!(toks.len(), 1);
    // Red (it already was) Balloon Giant 1/1 with flying and haste; not green or blue.
    assert_printed_giant(&t, toks[0], Some((1, 1)), true);
    assert!(has_subtype(&t, toks[0], "Balloon"));
    assert!(has_kw(&t, toks[0], KeywordKind::Flying) && has_kw(&t, toks[0], KeywordKind::Haste));
}

#[test]
fn calamity_tokens_copy_only_the_printed_creature() {
    cr!("707.2", "702.171a");
    ruling!(
        "Calamity, Galloping Inferno",
        "Each of the tokens copies exactly what was printed on the original creature and nothing else"
    );
    supported("Calamity, Galloping Inferno");
    let mut t = TestGame::new(2);
    let calamity = t.battlefield(P0, "Calamity, Galloping Inferno");
    let giant = dressed_giant(&mut t, P0);
    untap(&mut t, giant);
    t.answer_choose(P0, &[obj(giant)]);
    activate_containing(&mut t, P0, calamity, "Saddle").expect("saddle");
    t.resolve_all();
    assert!(t.obj_now(giant).tapped);
    t.answer_choose(P0, &[obj(giant)]);
    t.answer_choose(P0, &[obj(giant)]);
    let before = t.g.battlefield.clone();
    t.set_step(P0, Step::BeginningOfCombat);
    t.attack(&[(calamity, Entity::Player(P1))], &[]);
    let toks = new_tokens(&t, P0, &before);
    assert_eq!(toks.len(), 2);
    for tok in toks {
        // Tapped and attacking, but a printed Hill Giant with no counters or Equipment.
        assert_printed_giant(&t, tok, Some((3, 3)), false);
    }
    // Calamity 4 + two 3/3 tokens.
    assert_eq!(t.life(P1), 20 - 4 - 3 - 3);
}

#[test]
fn impostor_syndrome_token_copies_only_the_printed_creature() {
    cr!("707.2", "707.9b", "510.3a");
    ruling!(
        "Impostor Syndrome",
        "Except for not being legendary, the token copies exactly what was printed on the original creature and nothing else"
    );
    supported("Impostor Syndrome");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Impostor Syndrome");
    let isamaru = t.battlefield(P0, "Isamaru, Hound of Konda");
    dress(&mut t, P0, isamaru);
    untap(&mut t, isamaru);
    let before = t.g.battlefield.clone();
    t.set_step(P0, Step::BeginningOfCombat);
    t.attack(&[(isamaru, Entity::Player(P1))], &[]);
    t.resolve_all();
    let toks = new_tokens(&t, P0, &before);
    assert_eq!(toks.len(), 1);
    assert_printed(
        &t,
        toks[0],
        "Isamaru, Hound of Konda",
        ColorSet::single(Color::White),
        Some((2, 2)),
        true,
    );
    assert!(!legendary(&t, toks[0]));
}

#[test]
fn ratadrabik_token_copies_only_the_printed_creature() {
    cr!("707.2", "707.9b", "603.10a");
    ruling!(
        "Ratadrabik of Urborg",
        "Except for power and toughness, the token copies only what was printed on the original creature"
    );
    supported("Ratadrabik of Urborg");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Ratadrabik of Urborg");
    let isamaru = t.battlefield(P0, "Isamaru, Hound of Konda");
    dress(&mut t, P0, isamaru);
    let before = t.g.battlefield.clone();
    destroy(&mut t, isamaru);
    t.resolve_all();
    let toks = new_tokens(&t, P0, &before);
    assert_eq!(toks.len(), 1);
    // White (printed) and black; not green or blue; a 2/2 Dog Zombie.
    assert_printed(
        &t,
        toks[0],
        "Isamaru, Hound of Konda",
        ColorSet::single(Color::White).union(ColorSet::single(Color::Black)),
        Some((2, 2)),
        true,
    );
    assert!(has_subtype(&t, toks[0], "Dog") && has_subtype(&t, toks[0], "Zombie"));
    assert!(!legendary(&t, toks[0]));
}

#[test]
fn brenard_token_copies_only_the_printed_creature() {
    cr!("707.2", "707.9b", "603.10a");
    ruling!(
        "Brenard, Ginger Sculptor",
        "Except for the listed exceptions, the token copies exactly what was printed on the original creature and nothing else"
    );
    supported("Brenard, Ginger Sculptor");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Brenard, Ginger Sculptor");
    let giant = dressed_giant(&mut t, P0);
    let before = t.g.battlefield.clone();
    t.answer_yes(P0, true);
    destroy(&mut t, giant);
    t.resolve_all();
    let toks = new_tokens(&t, P0, &before);
    assert_eq!(toks.len(), 1);
    // A 1/1 Food Golem artifact creature (+2/+2 from Brenard), red, no counters.
    assert_printed_giant(&t, toks[0], Some((3, 3)), true);
    assert!(has_subtype(&t, toks[0], "Food") && has_subtype(&t, toks[0], "Golem"));
    assert!(t.obj_now(toks[0]).is(CardType::Artifact));
}

#[test]
fn shaun_token_copies_only_the_printed_creature() {
    cr!("707.2", "707.9b");
    ruling!(
        "Shaun, Father of Synths",
        "Except for the noted exceptions, the token copies exactly what was printed on the original creature and nothing else"
    );
    supported("Shaun, Father of Synths");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Shaun, Father of Synths");
    let isamaru = t.battlefield(P0, "Isamaru, Hound of Konda");
    dress(&mut t, P0, isamaru);
    untap(&mut t, isamaru);
    t.answer_yes(P0, true);
    t.answer_targets(P0, &[obj(isamaru)]);
    let before = t.g.battlefield.clone();
    t.set_step(P0, Step::BeginningOfCombat);
    t.attack(&[(isamaru, Entity::Player(P1))], &[]);
    let toks = new_tokens(&t, P0, &before);
    assert_eq!(toks.len(), 1);
    assert_printed(
        &t,
        toks[0],
        "Isamaru, Hound of Konda",
        ColorSet::single(Color::White),
        Some((2, 2)),
        false,
    );
    assert!(!legendary(&t, toks[0]));
    assert!(t.obj_now(toks[0]).is(CardType::Artifact) && has_subtype(&t, toks[0], "Synth"));
}

#[test]
fn nexus_of_becoming_token_copies_only_the_printed_card() {
    cr!("707.2", "707.9b");
    ruling!(
        "Nexus of Becoming",
        "Except for the listed exceptions, the token copies exactly what’s printed on the exiled card and nothing else."
    );
    supported("Nexus of Becoming");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Nexus of Becoming");
    let angel = t.hand(P0, "Serra Angel");
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[obj(angel)]);
    let before = t.g.battlefield.clone();
    t.advance_to(P0, Step::BeginningOfCombat);
    t.resolve_all();
    let toks = new_tokens(&t, P0, &before);
    assert_eq!(toks.len(), 1);
    assert_printed(
        &t,
        toks[0],
        "Serra Angel",
        ColorSet::single(Color::White),
        Some((3, 3)),
        true,
    );
    assert!(t.obj_now(toks[0]).is(CardType::Artifact) && has_subtype(&t, toks[0], "Golem"));
    assert!(has_subtype(&t, toks[0], "Angel") && has_kw(&t, toks[0], KeywordKind::Flying));
}

#[test]
fn cursecloth_wrappings_embalm_token_copies_only_the_printed_card() {
    cr!("707.2", "702.128a");
    ruling!(
        "Cursecloth Wrappings",
        "Except for the listed exceptions, the token copies exactly what was printed on the original card and nothing else."
    );
    supported("Cursecloth Wrappings");
    let mut t = TestGame::new(2);
    let wrappings = t.battlefield(P0, "Cursecloth Wrappings");
    let giant = dressed_giant(&mut t, P0);
    destroy(&mut t, giant);
    let card = t.g.current(giant);
    assert!(t.in_graveyard(P0, "Hill Giant"));
    t.activate(P0, wrappings, 0, &[obj(card)])
        .expect("grant embalm");
    t.resolve_all();
    lands_for(&mut t, P0, "{3}{R}");
    let before = t.g.battlefield.clone();
    activate_containing(&mut t, P0, card, "Embalm").expect("embalm");
    t.resolve_all();
    let toks = new_tokens(&t, P0, &before);
    assert_eq!(toks.len(), 1);
    // A white Zombie Giant 3/3 (+1/+1 from the Wrappings), with no mana cost.
    assert_printed(
        &t,
        toks[0],
        "Hill Giant",
        ColorSet::single(Color::White),
        Some((4, 4)),
        true,
    );
    assert!(has_subtype(&t, toks[0], "Zombie") && has_subtype(&t, toks[0], "Giant"));
    assert!(t.obj_now(toks[0]).chars.mana_cost.is_none());
}
