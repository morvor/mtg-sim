//! The Oracle round trip (`oracle/render/`): compiled abilities rendered back into Oracle
//! wording and compared with the card's Oracle text, so that a card the compiler accepts
//! but misreads ("nontoken", "another", "you control", "up to", ...) is caught.
//!
//! * Renderer tests over representative real cards of each area of the ability language.
//! * The comparison: what it treats as the same, and that a changed meaning mismatches.
//! * Regression: cards listed in `docs/roundtrip-passing.txt` keep passing.
//! * In-game tests for the parser bugs the round trip found.

use mtg_engine::ability::*;
use mtg_engine::card::{card, CardDef};
use mtg_engine::oracle::render::compare::{check_card, normalize_unit, tokens_match};
use mtg_engine::oracle::render::{render_ability, render_card, FaceInfo};
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;
use std::sync::Arc;

fn assert_round_trips(name: &str) {
    let c = card(name);
    assert!(
        c.unsupported_text().is_empty(),
        "{name} has unsupported text: {:?}",
        c.unsupported_text()
    );
    let r = check_card(&c);
    assert!(
        r.pass,
        "{name} doesn't round-trip:\n  oracle: {:?}\n  rendered: {:?}\n  gaps: {:?}",
        r.unmatched_oracle, r.unmatched_rendered, r.gaps
    );
}

/// The rendered lines of a card's first face.
fn rendered(name: &str) -> Vec<String> {
    render_card(&card(name))[0].lines.clone()
}

// --- The renderer, one area of the ability language at a time ----------------------

#[test]
fn keyword_abilities_render_with_their_parameters() {
    for name in [
        "Serra Angel",          // Flying, vigilance
        "Bonesplitter",         // Equip {1}
        "Black Knight",         // First strike, protection from white
        "Kor Skyfisher",        // a trigger after a keyword
        "Baneslayer Angel",     // protection from Demons and from Dragons
    ] {
        assert_round_trips(name);
    }
    assert_eq!(rendered("Bonesplitter")[1], "Equip {1}");
}

#[test]
fn spell_effects_targets_and_values_render() {
    for name in [
        "Lightning Bolt",
        "Swords to Plowshares",
        "Counterspell",
        "Doom Blade",
        "Wrath of God",
        "Murder",
        "Giant Growth",
        "Path to Exile",
        "Arc Lightning",
        "Rampant Growth",
        "Opt",
        "Preordain",
    ] {
        assert_round_trips(name);
    }
    assert_eq!(rendered("Lightning Bolt"), ["~ deals 3 damage to any target."]);
    assert_eq!(rendered("Doom Blade"), ["Destroy target nonblack creature."]);
}

#[test]
fn activated_triggered_and_static_abilities_render() {
    for name in [
        "Llanowar Elves",       // mana ability
        "Prodigal Sorcerer",    // {T}: damage
        "Shivan Dragon",        // pump
        "Gravedigger",          // ETB "you may" with a graveyard target
        "Ajani's Pridemate",    // "whenever you gain life"
        "Glorious Anthem",      // anthem
        "Elvish Archdruid",     // "other Elf creatures", "for each Elf"
        "Thalia, Guardian of Thraben", // cost increase
        "Grim Lavamancer",      // cost with exile from graveyard
        "Rhystic Study",        // "unless that player pays"
        "Exploration",          // additional land play
    ] {
        assert_round_trips(name);
    }
    assert_eq!(rendered("Llanowar Elves"), ["{T}: Add {G}."]);
    assert_eq!(
        rendered("Glorious Anthem"),
        ["Creatures you control get +1/+1."]
    );
}

#[test]
fn tokens_modes_replacements_and_restrictions_render() {
    for name in [
        "Raise the Alarm",     // token
        "Charming Prince",     // modal
        "Rest in Peace",       // replacement, ETB trigger
        "Glacial Fortress",    // "enters tapped unless you control ..."
        "Pacifism",            // enchant, restriction on the enchanted creature
        "Anointed Procession", // token doubling
        "Cyclonic Rift",       // overload
        "Thragtusk",
    ] {
        assert_round_trips(name);
    }
}

#[test]
fn card_kinds_render() {
    for name in [
        "The Eldest Reborn",         // Saga chapters
        "Case of the Uneaten Feast", // Case: "To solve" and "Solved"
        "Wildfire Howl",             // gift
        "Oracle of Mul Daya",
    ] {
        assert_round_trips(name);
    }
    // Saga chapters render as "I — ...".
    assert!(rendered("The Eldest Reborn")[0].starts_with("I — "));
}

#[test]
fn every_ability_of_a_card_with_unknown_custom_behavior_is_a_gap() {
    // A behavior the renderer has no words for is reported, never dropped silently.
    let a = AbilityDef::new(
        AbilityKind::Spell(SpellAbility {
            body: Body::effect(Effect::Custom("no such behavior".into())),
        }),
        "",
    );
    let r = render_ability(&a, &FaceInfo::default());
    assert!(r.is_err(), "{r:?}");
}

// --- The comparison --------------------------------------------------------------

fn same(a: &str, b: &str) -> bool {
    tokens_match(&normalize_unit(a), &normalize_unit(b))
}

#[test]
fn equivalent_wordings_compare_equal() {
    // CR 700.4: "dies" means "is put into a graveyard from the battlefield".
    assert!(same(
        "When ~ dies, return it to its owner's hand.",
        "When ~ is put into a graveyard from the battlefield, return it to its owner's hand."
    ));
    // Numbers, case, punctuation, grammatical number.
    assert!(same("Draw two cards.", "draw 2 card"));
    // A leading duration moves to the end of its sentence.
    assert!(same(
        "Until end of turn, target creature gains flying.",
        "Target creature gains flying until end of turn."
    ));
    // "X ..., where X is V" and "equal to V".
    assert!(same(
        "You gain life equal to its power.",
        "You gain X life, where X is its power."
    ));
}

#[test]
fn different_meanings_compare_different() {
    assert!(!same("Destroy target creature.", "Destroy target nonblack creature."));
    assert!(!same("Exile another target creature.", "Exile target creature."));
    assert!(!same(
        "Creatures you control get +1/+1.",
        "Creatures your opponents control get +1/+1."
    ));
    assert!(!same("Draw a card.", "Target player draws a card."));
    assert!(!same("Return it to its owner's hand.", "Return ~ to its owner's hand."));
    assert!(!same("Tap up to two target creatures.", "Tap two target creatures."));
}

#[test]
fn a_misread_ability_mismatches() {
    // Doom Blade compiled as if "nonblack" had been dropped.
    let mut def: CardDef = (*card("Doom Blade")).clone();
    let face = &mut def.faces[0];
    let mut a = (*face.chars.abilities[0]).clone();
    if let AbilityKind::Spell(s) = &mut a.kind {
        s.body.targets[0].what = TargetKind::Object(Filter::Type(mtg_engine::types::CardType::Creature));
    }
    face.chars.abilities[0] = Arc::new(a);
    let r = check_card(&def);
    assert!(!r.pass);
    assert_eq!(r.unmatched_rendered, ["Destroy target creature."]);
}

// --- Regression: passing cards keep passing ----------------------------------------

#[test]
fn cards_that_round_trip_keep_round_tripping() {
    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/../../docs/roundtrip-passing.txt");
    let list = std::fs::read_to_string(path).expect("docs/roundtrip-passing.txt");
    let names: Vec<&str> = list
        .lines()
        .filter(|l| !l.trim().is_empty() && !l.starts_with('#'))
        .collect();
    assert!(names.len() > 15_000, "the list has {} cards", names.len());
    // A sample keeps the test fast; `mtg-tools roundtrip --check` checks them all.
    let mut failed = Vec::new();
    for name in names.iter().step_by(7) {
        let Some(c) = mtg_engine::card::CardDb::global().get(name) else {
            failed.push(format!("{name}: unknown card"));
            continue;
        };
        let r = check_card(&c);
        if !r.pass {
            failed.push(format!(
                "{name}: oracle {:?} rendered {:?} gaps {:?}",
                r.unmatched_oracle, r.unmatched_rendered, r.gaps
            ));
        }
    }
    assert!(failed.is_empty(), "no longer round-trip:\n{}", failed.join("\n"));
}

// --- Parser bugs the round trip found ----------------------------------------------

/// "Creatures your opponents control enter tapped." was compiled with the probe noun's
/// `Card` ("card your opponents control"), so creature tokens (not cards, CR 108.2b)
/// entered untapped.
#[test]
fn creatures_your_opponents_control_includes_their_tokens() {
    cr!("614.1d", "108.2b");
    assert_round_trips("Kinjalli's Sunwing");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Kinjalli's Sunwing");
    t.lands(P1, "Plains", 2);
    t.set_step(P1, Step::PrecombatMain);
    let alarm = t.hand(P1, "Raise the Alarm");
    t.cast(P1, alarm).go();
    t.resolve_all();
    let soldiers: Vec<ObjectId> = t
        .named_on_battlefield("Soldier Token")
        .into_iter()
        .filter(|id| t.g.obj(*id).is_token())
        .collect();
    assert_eq!(soldiers.len(), 2);
    for s in soldiers {
        assert!(t.g.obj(s).tapped, "an opponent's creature token enters tapped");
    }
}

/// "Colorless creatures you control enter with two additional +1/+1 counters on them."
/// applies to creature tokens too.
#[test]
fn creatures_you_control_enter_with_counters_includes_tokens() {
    cr!("614.1d", "122.6", "108.2b");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Curator Beastie");
    t.enter(P0, "Incubator Drone");
    t.resolve_all();
    let scions: Vec<ObjectId> = t
        .named_on_battlefield("Eldrazi Scion Token")
        .into_iter()
        .filter(|id| t.g.obj(*id).is_token())
        .collect();
    assert_eq!(scions.len(), 1);
    assert_eq!(t.counters(scions[0], "+1/+1"), 2);
}

/// "Seven Dwarves gets +1/+1 for each other creature named Seven Dwarves you control":
/// a token copy named Seven Dwarves counts.
#[test]
fn other_creatures_named_like_it_include_tokens() {
    cr!("201.2", "108.2b");
    let mut t = TestGame::new(2);
    let dwarves = t.battlefield(P0, "Seven Dwarves");
    assert_eq!(t.pt(dwarves), (2, 2));
    // A token named Seven Dwarves (as a token copy would be).
    let token = t.battlefield(P0, "Seven Dwarves");
    t.g.objects[token.0 as usize].kind = mtg_engine::object::ObjKind::Token;
    t.g.dirty = true;
    t.g.recompute();
    assert!(t.g.obj(token).is_token());
    assert_eq!(t.pt(dwarves), (3, 3), "the token counts");
}

/// "Whenever an opponent gains life, you may pay {R}. If you do, return this card from
/// your graveyard to your hand." functions from the graveyard (CR 113.6m).
#[test]
fn return_this_card_from_your_graveyard_triggers_in_the_graveyard() {
    cr!("113.6m");
    let mut t = TestGame::new(2);
    t.lands(P0, "Mountain", 1);
    let fire = t.graveyard(P0, "Punishing Fire");
    t.set_step(P0, Step::PrecombatMain);
    t.answer_yes(P0, true);
    t.g.gain_life(P1, 2);
    t.settle();
    t.resolve_all();
    assert!(!t.g.is_live(fire), "it triggered and returned");
    assert!(t.in_hand(P0, "Punishing Fire"));
}

/// "When this Aura enters, enchanted creature deals damage equal to its power to any other
/// target": "other" is other than the enchanted creature, which can't be the target.
#[test]
fn any_other_target_excludes_the_object_dealing_the_damage() {
    cr!("115.4", "115.1");
    assert_round_trips("Pain for All");
    let mut t = TestGame::new(2);
    let mine = t.battlefield(P0, "Grizzly Bears");
    let theirs = t.battlefield(P1, "Grizzly Bears");
    t.lands(P0, "Mountain", 3);
    t.set_step(P0, Step::PrecombatMain);
    let aura = t.hand(P0, "Pain for All");
    let from = t.asked().len();
    t.answer_targets(P0, &[Entity::Object(mine)]);
    t.answer_targets(P0, &[Entity::Object(theirs)]);
    t.cast(P0, aura).go();
    t.resolve_all();
    let offered: Vec<Vec<Entity>> = t.asked()[from..]
        .iter()
        .filter_map(|(_, d)| match d {
            mtg_engine::decision::Decision::ChooseTargets { candidates, .. } => {
                Some(candidates.clone())
            }
            _ => None,
        })
        .collect();
    assert_eq!(offered.len(), 2, "the Aura's target, then the trigger's");
    assert!(!offered[1].contains(&Entity::Object(mine)));
    assert!(offered[1].contains(&Entity::Object(theirs)));
    assert!(offered[1].contains(&Entity::Player(P1)));
    assert!(!t.on_battlefield(theirs), "dealt 2 damage");
}

/// "Change the target of target spell with a single target unless that spell's
/// controller pays {2}." The "unless" clause used to be dropped (the target qualifier
/// swallowed the rest of the sentence), so the target always changed.
#[test]
fn divert_changes_the_target_unless_its_controller_pays() {
    cr!("118.12a", "115.7a");
    assert_round_trips("Divert");
    for pays in [false, true] {
        let mut t = TestGame::new(2);
        let bears = t.battlefield(P0, "Grizzly Bears");
        let giant = t.battlefield(P1, "Hill Giant");
        t.set_step(P1, Step::PrecombatMain);
        t.lands(P1, "Mountain", if pays { 3 } else { 1 });
        let bolt = t.hand(P1, "Lightning Bolt");
        let s = t.cast(P1, bolt).target(bears).go();
        t.lands(P0, "Island", 1);
        let divert = t.hand(P0, "Divert");
        t.cast(P0, divert).target(s).go();
        t.answer_yes(P1, pays);
        t.answer_targets(P0, &[Entity::Object(giant)]);
        t.resolve_all();
        if pays {
            assert!(!t.on_battlefield(bears), "paid: the Bolt still hits the Bears");
            assert!(t.on_battlefield(giant));
        } else {
            assert!(t.on_battlefield(bears), "not paid: the target changed");
            assert!(!t.on_battlefield(giant));
        }
    }
}

/// "Equip—Pay 3 life. Activate only once each turn." The restriction printed with the
/// keyword applied to crew but was ignored for equip.
#[test]
fn equip_activate_only_once_each_turn() {
    cr!("602.5b", "702.6a");
    let mut t = TestGame::new(2);
    let sword = t.battlefield(P0, "Dark Knight's Greatsword");
    let a = t.battlefield(P0, "Grizzly Bears");
    let b = t.battlefield(P0, "Hill Giant");
    t.set_step(P0, Step::PrecombatMain);
    assert!(t.activate(P0, sword, 0, &[Entity::Object(a)]).is_ok());
    t.resolve_all();
    assert_eq!(t.g.obj(sword).attached_to, Some(Entity::Object(a)));
    assert!(
        t.activate(P0, sword, 0, &[Entity::Object(b)]).is_err(),
        "only once each turn"
    );
    assert_eq!(t.life(P0), 17);
}
