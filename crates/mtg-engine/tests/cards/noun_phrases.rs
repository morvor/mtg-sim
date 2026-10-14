//! The noun-phrase grammar (`oracle/phrases.rs`, `oracle/patterns/noun_phrase_*.rs`):
//! alternatives with adjectives of their own ("target land or nonblack creature", "Hero
//! or noncreature spell"), predicates after the noun ("that isn't all colors", "that's
//! attacking you", "with islandwalk", "you own or control"), groups chosen between ("all
//! lands or all creatures"), and elided verbs ("1 damage to target player and 1 damage to
//! target creature that player controls").

use mtg_engine::decision::{Answer, Decision};
use mtg_engine::events::MoveCause;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

fn assert_supported(names: &[&str]) {
    for n in names {
        let u = card(n).unsupported_text().join(" | ");
        assert!(u.is_empty(), "{n} has unsupported text: {u}");
    }
}

/// The candidates of the target decisions asked since `from`.
fn target_candidates(t: &TestGame, from: usize) -> Vec<Vec<Entity>> {
    t.asked()[from..]
        .iter()
        .filter_map(|(_, d)| match d {
            Decision::ChooseTargets { candidates, .. } => Some(candidates.clone()),
            _ => None,
        })
        .collect()
}

#[test]
fn land_or_nonblack_creature_names_two_complete_alternatives() {
    cr!("115.1", "608.2b");
    assert_supported(&["Stream of Acid"]);
    let mut t = TestGame::new(2);
    t.lands(P0, "Swamp", 4);
    let forest = t.battlefield(P1, "Forest");
    let bears = t.battlefield(P1, "Grizzly Bears");
    let knight = t.battlefield(P1, "Black Knight");
    let rock = t.battlefield(P1, "Mind Stone");
    let spell = t.hand(P0, "Stream of Acid");
    let from = t.asked().len();
    t.cast(P0, spell).target(bears).go();
    let c = &target_candidates(&t, from)[0];
    assert!(c.contains(&Entity::Object(forest)));
    assert!(c.contains(&Entity::Object(bears)));
    // "nonblack" describes only the creature alternative; neither describes an artifact.
    assert!(!c.contains(&Entity::Object(knight)));
    assert!(!c.contains(&Entity::Object(rock)));
    t.resolve_all();
    assert!(t.in_graveyard(P1, "Grizzly Bears"));
}

#[test]
fn black_creature_or_black_planeswalker_and_her_controller() {
    cr!("115.1", "608.2h", "205.3j");
    assert_supported(&["Liliana's Defeat"]);
    let mut t = TestGame::new(2);
    t.lands(P0, "Swamp", 1);
    let liliana = t.battlefield(P1, "Liliana of the Veil");
    let gideon = t.battlefield(P1, "Gideon Jura");
    let bears = t.battlefield(P1, "Grizzly Bears");
    let knight = t.battlefield(P1, "Black Knight");
    let spell = t.hand(P0, "Liliana's Defeat");
    let from = t.asked().len();
    t.cast(P0, spell).target(liliana).go();
    let c = &target_candidates(&t, from)[0];
    assert!(c.contains(&Entity::Object(liliana)));
    assert!(c.contains(&Entity::Object(knight)));
    assert!(!c.contains(&Entity::Object(gideon)));
    assert!(!c.contains(&Entity::Object(bears)));
    t.resolve_all();
    assert!(t.in_graveyard(P1, "Liliana of the Veil"));
    // It was a Liliana planeswalker: her controller loses 3 life.
    assert_eq!(t.life(P1), 17);

    // A black creature: no life loss.
    let mut t = TestGame::new(2);
    t.lands(P0, "Swamp", 1);
    let knight = t.battlefield(P1, "Black Knight");
    let spell = t.hand(P0, "Liliana's Defeat");
    t.cast(P0, spell).target(knight).go();
    t.resolve_all();
    assert!(t.in_graveyard(P1, "Black Knight"));
    assert_eq!(t.life(P1), 20);
}

#[test]
fn spell_or_nonland_permanent_goes_on_top_or_bottom_of_its_owners_library() {
    cr!("115.1");
    ruling!(
        "Sudden Setback",
        "The spell or permanent's owner chooses whether to put it on the top or bottom of their library."
    );
    assert_supported(&["Sudden Setback"]);
    for top in [true, false] {
        let mut t = TestGame::new(2);
        t.lands(P0, "Island", 4);
        let forest = t.battlefield(P1, "Forest");
        let wurm = t.battlefield(P1, "Craw Wurm");
        let spell = t.hand(P0, "Sudden Setback");
        let from = t.asked().len();
        t.answer(P1, DecisionKind::Option, Answer::Index(if top { 0 } else { 1 }));
        t.answer(P1, DecisionKind::YesNo, Answer::Bool(top));
        t.cast(P0, spell).target(wurm).go();
        let c = &target_candidates(&t, from)[0];
        assert!(!c.contains(&Entity::Object(forest)));
        t.resolve_all();
        assert_eq!(t.zone(wurm), Zone::Library(P1), "top: {top}");
        let lib = &t.g.player(P1).library;
        let pos = lib
            .iter()
            .position(|&o| t.g.obj(o).chars.name == "Craw Wurm")
            .expect("in library");
        // The owner (who was asked) chose the top or the bottom.
        assert!(pos == 0 || pos == lib.len() - 1);
    }
}

#[test]
fn token_thats_red_green_and_white() {
    cr!("111.3", "105.2");
    assert_supported(&["Godsire"]);
    let mut t = TestGame::new(2);
    let godsire = t.battlefield(P0, "Godsire");
    t.activate(P0, godsire, 0, &[]).unwrap();
    t.resolve_all();
    let token = t
        .g
        .permanents()
        .find(|o| o.is_token() && o.chars.has_subtype("Beast"))
        .map(|o| o.id)
        .expect("a Beast token");
    let c = &t.obj_now(token).chars;
    assert_eq!(t.pt(token), (8, 8));
    for col in [Color::Red, Color::Green, Color::White] {
        assert!(c.colors.contains(col), "{col:?}");
    }
    assert!(!c.colors.contains(Color::Black) && !c.colors.contains(Color::Blue));
}

#[test]
fn either_group_and_life_for_each_creature_that_died() {
    cr!("608.2d", "700.4");
    assert_supported(&["Reign of Terror", "Catastrophe"]);
    for (choice, dead) in [(0usize, 2), (1, 1)] {
        let mut t = TestGame::new(2);
        t.lands(P0, "Swamp", 5);
        t.battlefield(P1, "Grizzly Bears");
        t.battlefield(P0, "Craw Wurm");
        t.battlefield(P1, "Savannah Lions");
        t.battlefield(P1, "Black Knight");
        let spell = t.hand(P0, "Reign of Terror");
        t.answer(P0, DecisionKind::Option, Answer::Index(choice));
        t.cast(P0, spell).go();
        t.resolve_all();
        let green_dead = t.in_graveyard(P1, "Grizzly Bears");
        assert_eq!(green_dead, choice == 0);
        assert_eq!(t.in_graveyard(P1, "Savannah Lions"), choice == 1);
        assert!(!t.in_graveyard(P1, "Black Knight"));
        assert_eq!(t.life(P0), 20 - 2 * dead, "choice {choice}");
    }
    // Catastrophe: lands or creatures, chosen as it resolves.
    ruling!("Catastrophe", "You choose land or creatures when the spell resolves.");
    let mut t = TestGame::new(2);
    t.lands(P0, "Plains", 6);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let forest = t.battlefield(P1, "Forest");
    let spell = t.hand(P0, "Catastrophe");
    t.answer(P0, DecisionKind::Option, Answer::Index(1));
    t.cast(P0, spell).go();
    t.resolve_all();
    assert!(!t.on_battlefield(bears));
    assert!(t.on_battlefield(forest));
}

#[test]
fn tap_all_lands_its_controller_controls() {
    cr!("603.2", "608.2h");
    assert_supported(&["Tectonic Instability"]);
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Tectonic Instability");
    let mine = t.battlefield(P0, "Forest");
    let theirs = t.battlefield(P1, "Mountain");
    let land = t.hand(P0, "Plains");
    t.play_land(P0, land).unwrap();
    t.resolve_all();
    assert!(t.obj_now(mine).tapped);
    let plains = t.named_on_battlefield("Plains")[0];
    assert!(t.obj_now(plains).tapped);
    assert!(!t.obj_now(theirs).tapped);
}

#[test]
fn damage_to_each_of_two_other_target_creatures() {
    cr!("115.3", "120.3");
    ruling!(
        "Betrayal at the Vault",
        "If one of the two other target creatures leaves the battlefield before Betrayal at the Vault resolves, the creature you control will still deal damage to the remaining other target creature."
    );
    assert_supported(&["Betrayal at the Vault"]);
    let mut t = TestGame::new(2);
    t.lands(P0, "Forest", 6);
    let wurm = t.battlefield(P0, "Craw Wurm");
    let giant = t.battlefield(P1, "Hill Giant");
    let bears = t.battlefield(P1, "Grizzly Bears");
    let spell = t.hand(P0, "Betrayal at the Vault");
    let from = t.asked().len();
    t.cast(P0, spell).target(wurm).target(giant).target(bears).go();
    // The other targets can't be the first one.
    let cands = target_candidates(&t, from);
    assert!(cands.len() >= 2);
    assert!(!cands[1].contains(&Entity::Object(wurm)), "{cands:?}");
    t.g.move_object(bears, Zone::Graveyard(P1), MoveCause::Effect, None);
    t.g.flush_events();
    t.resolve_all();
    assert!(t.in_graveyard(P1, "Hill Giant"));
    assert!(t.on_battlefield(wurm));
}

#[test]
fn feint_taps_blockers_and_prevents_their_and_the_attackers_combat_damage() {
    cr!("509.1g", "615.1");
    assert_supported(&["Feint"]);
    for cast in [false, true] {
        let mut t = TestGame::new(2);
        let wurm = t.battlefield(P0, "Craw Wurm");
        let bears = t.battlefield(P1, "Grizzly Bears");
        let lions = t.battlefield(P1, "Savannah Lions");
        t.lands(P1, "Mountain", 1);
        t.answer(
            P0,
            DecisionKind::Attackers,
            Answer::Attackers(vec![(wurm, Entity::Player(P1))]),
        );
        t.answer(P1, DecisionKind::Blockers, Answer::Blockers(vec![(bears, wurm), (lions, wurm)]));
        t.advance_to(P0, Step::DeclareBlockers);
        if cast {
            let feint = t.hand(P1, "Feint");
            t.cast_with(P1, feint, &[Entity::Object(wurm)]).unwrap();
            t.resolve_all();
            assert!(t.obj_now(bears).tapped && t.obj_now(lions).tapped);
        }
        t.advance_to(P0, Step::PostcombatMain);
        // Without Feint, the blockers' 4 damage kills the Wurm and it kills both.
        assert_eq!(t.on_battlefield(wurm), cast);
        assert_eq!(t.on_battlefield(bears), cast);
        assert_eq!(t.on_battlefield(lions), cast);
    }
}

#[test]
fn searing_blaze_landfall_replaces_both_damage_parts() {
    cr!("115.1", "608.2c");
    ruling!(
        "Searing Blaze",
        "The effect of this spell's landfall ability replaces its normal effect."
    );
    assert_supported(&["Searing Blaze"]);
    for landfall in [false, true] {
        let mut t = TestGame::new(2);
        t.lands(P0, "Mountain", 2);
        let wurm = t.battlefield(P1, "Craw Wurm");
        if landfall {
            let land = t.hand(P0, "Mountain");
            t.play_land(P0, land).unwrap();
        }
        let spell = t.hand(P0, "Searing Blaze");
        t.cast_with(P0, spell, &[Entity::Player(P1), Entity::Object(wurm)])
            .unwrap();
        t.resolve_all();
        let n = if landfall { 3 } else { 1 };
        assert_eq!(t.life(P1), 20 - n, "landfall: {landfall}");
        assert_eq!(t.obj_now(wurm).damage, n as u32);
    }
}

#[test]
fn target_attacking_vampire_that_isnt_a_demon_becomes_one() {
    cr!("205.1b", "508.1", "115.1");
    assert_supported(&["Clavileño, First of the Blessed"]);
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Clavileño, First of the Blessed");
    let vamp = t.battlefield(P0, "Vampire Interloper");
    // Already a Demon (a Vampire Demon), and not a Vampire: not legal targets.
    let demon = t.battlefield(P0, "Bloodletter of Aclazotz");
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.answer(
        P0,
        DecisionKind::Attackers,
        Answer::Attackers(vec![
            (vamp, Entity::Player(P1)),
            (demon, Entity::Player(P1)),
            (bears, Entity::Player(P1)),
        ]),
    );
    t.answer_targets(P0, &[Entity::Object(vamp)]);
    let from = t.asked().len();
    t.advance_to(P0, Step::DeclareAttackers);
    t.resolve_all();
    let c = &target_candidates(&t, from)[0];
    assert!(c.contains(&Entity::Object(vamp)));
    assert!(!c.contains(&Entity::Object(demon)));
    assert!(!c.contains(&Entity::Object(bears)));
    let c = &t.obj_now(vamp).chars;
    assert!(c.has_subtype("Demon") && c.has_subtype("Vampire"));
    // It gained "When this creature dies, draw a card and create a tapped 4/3 ... token".
    t.library_top(P0, "Forest");
    let hand = t.hand_size(P0);
    t.g.move_object(vamp, Zone::Graveyard(P0), MoveCause::Effect, None);
    t.g.flush_events();
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand + 1);
    let token = t
        .g
        .permanents()
        .find(|o| o.is_token() && o.chars.has_subtype("Demon"))
        .map(|o| o.id)
        .expect("a Vampire Demon token");
    assert_eq!(t.pt(token), (4, 3));
    assert!(t.obj_now(token).tapped);
}

#[test]
fn hunting_kavu_exiles_a_creature_without_flying_attacking_you() {
    cr!("115.1", "508.1b", "608.2b");
    ruling!(
        "Hunting Kavu",
        "Being “without flying” is a targeting restriction."
    );
    assert_supported(&["Hunting Kavu"]);
    let mut t = TestGame::new(2);
    let kavu = t.battlefield(P1, "Hunting Kavu");
    t.lands(P1, "Mountain", 2);
    t.lands(P1, "Forest", 1);
    let giant = t.battlefield(P0, "Hill Giant");
    let angel = t.battlefield(P0, "Serra Angel");
    let idle = t.battlefield(P0, "Grizzly Bears");
    t.answer(
        P0,
        DecisionKind::Attackers,
        Answer::Attackers(vec![(giant, Entity::Player(P1)), (angel, Entity::Player(P1))]),
    );
    t.advance_to(P0, Step::DeclareAttackers);
    let from = t.asked().len();
    t.activate(P1, kavu, 0, &[Entity::Object(giant)]).unwrap();
    let c = &target_candidates(&t, from)[0];
    assert!(c.contains(&Entity::Object(giant)));
    assert!(!c.contains(&Entity::Object(angel)));
    assert!(!c.contains(&Entity::Object(idle)));
    t.resolve_all();
    assert!(t.in_exile("Hill Giant") && t.in_exile("Hunting Kavu"));
}

#[test]
fn creature_with_islandwalk_is_not_any_landwalk() {
    cr!("702.14a", "115.1");
    assert_supported(&["Merfolk Assassin"]);
    let mut t = TestGame::new(2);
    let assassin = t.battlefield(P0, "Merfolk Assassin");
    let leviathan = t.battlefield(P1, "Segovian Leviathan");
    let bears = t.battlefield(P1, "Grizzly Bears");
    let from = t.asked().len();
    t.activate(P0, assassin, 0, &[Entity::Object(leviathan)]).unwrap();
    let c = &target_candidates(&t, from)[0];
    assert!(c.contains(&Entity::Object(leviathan)));
    assert!(!c.contains(&Entity::Object(bears)));
    t.resolve_all();
    assert!(t.in_graveyard(P1, "Segovian Leviathan"));
}

#[test]
fn permanent_you_own_or_control() {
    cr!("108.3", "110.2", "115.1");
    assert_supported(&["Telim'Tor's Edict"]);
    let mut t = TestGame::new(2);
    t.lands(P0, "Mountain", 1);
    let mine = t.battlefield(P0, "Grizzly Bears");
    // Owned by P0, controlled by P1.
    let lent = t.battlefield(P1, "Hill Giant");
    t.g.obj_mut(lent).owner = P0;
    let theirs = t.battlefield(P1, "Savannah Lions");
    let spell = t.hand(P0, "Telim'Tor's Edict");
    let from = t.asked().len();
    t.cast(P0, spell).target(lent).go();
    let c = &target_candidates(&t, from)[0];
    assert!(c.contains(&Entity::Object(mine)));
    assert!(c.contains(&Entity::Object(lent)));
    assert!(!c.contains(&Entity::Object(theirs)));
    t.resolve_all();
    assert!(t.in_exile("Hill Giant"));
}

#[test]
fn nonland_permanents_that_arent_historic_return() {
    cr!("700.6", "608.2c");
    assert_supported(&["Desynchronization"]);
    let mut t = TestGame::new(2);
    t.lands(P0, "Island", 4);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let isamaru = t.battlefield(P1, "Isamaru, Hound of Konda");
    let rock = t.battlefield(P1, "Mind Stone");
    let forest = t.battlefield(P1, "Forest");
    let spell = t.hand(P0, "Desynchronization");
    t.cast(P0, spell).go();
    t.resolve_all();
    assert!(t.in_hand(P1, "Grizzly Bears"));
    assert!(!t.on_battlefield(bears));
    for kept in [isamaru, rock, forest] {
        assert!(t.on_battlefield(kept));
    }
}

#[test]
fn an_additional_damage_to_each_green_creature() {
    cr!("120.3", "608.2c");
    ruling!(
        "Kaervek's Hex",
        "If a creature is both black and green, Kaervek’s Hex will deal 1 damage to that creature."
    );
    assert_supported(&["Kaervek's Hex"]);
    let mut t = TestGame::new(2);
    t.lands(P0, "Swamp", 4);
    let wurm = t.battlefield(P1, "Craw Wurm");
    let giant = t.battlefield(P1, "Hill Giant");
    let knight = t.battlefield(P1, "Black Knight");
    let mut colors = ColorSet::single(Color::Black);
    colors.insert(Color::Green);
    let golgari = card::CardDef::custom(object::Characteristics {
        name: "Golgari Test Creature".into(),
        card_types: CardTypeSet::single(CardType::Creature),
        colors,
        power: Some(5),
        toughness: Some(5),
        ..Default::default()
    });
    let both = t.custom(P1, golgari, Zone::Battlefield);
    let spell = t.hand(P0, "Kaervek's Hex");
    t.cast(P0, spell).go();
    t.resolve_all();
    assert_eq!(t.obj_now(wurm).damage, 2);
    assert_eq!(t.obj_now(giant).damage, 1);
    assert_eq!(t.obj_now(knight).damage, 0);
    assert_eq!(t.obj_now(both).damage, 1);
}

#[test]
fn any_target_of_an_opponents_choice() {
    cr!("115.1", "601.2c");
    assert_supported(&["Cuombajj Witches"]);
    let mut t = TestGame::new(2);
    let witches = t.battlefield(P0, "Cuombajj Witches");
    let lions = t.battlefield(P1, "Savannah Lions");
    let elves = t.battlefield(P0, "Llanowar Elves");
    t.answer_targets(P1, &[Entity::Object(elves)]);
    t.activate(P0, witches, 0, &[Entity::Object(lions)]).unwrap();
    t.resolve_all();
    assert!(t.in_graveyard(P1, "Savannah Lions"));
    assert!(t.in_graveyard(P0, "Llanowar Elves"));
}

/// A {0} instant with `text`, compiled with the real compiler.
fn instant(name: &str, text: &str) -> card::CardDef {
    let tl = TypeLine::parse("Instant");
    let ctx = oracle::CompileContext {
        card_name: name,
        full_name: name,
        type_line: &tl,
        layout: card::Layout::Normal,
        face_index: 0,
        keywords: &[],
        power: None,
        toughness: None,
    };
    let compiled = oracle::compile(text, &ctx);
    assert!(compiled.unsupported.is_empty(), "{:?}", compiled.unsupported);
    card::CardDef::custom(object::Characteristics {
        name: name.into(),
        mana_cost: mana::ManaCost::parse("{0}"),
        card_types: tl.card_types,
        abilities: compiled.abilities,
        rules_text: std::sync::Arc::from(text),
        ..Default::default()
    })
}

#[test]
fn hero_or_noncreature_spell_narrows_both_alternatives() {
    cr!("115.1", "205.3m");
    // Scarlet Witch, Chaotic Avenger: "cast a Hero or noncreature spell"; the same phrase
    // as a target: a Hero spell or a noncreature spell (not a Hero permanent).
    assert_supported(&["Scarlet Witch, Chaotic Avenger"]);
    for (name, legal) in [("K'un-Lun Warrior", true), ("Divination", true), ("Grizzly Bears", false)] {
        let counter = instant("Hero Denial", "Counter target Hero or noncreature spell.");
        let mut t = TestGame::new(2);
        t.set_step(P1, Step::PrecombatMain);
        t.lands(P1, "Mountain", 1);
        t.lands(P1, "Island", 1);
        t.lands(P1, "Forest", 1);
        let hero_on_battlefield = t.battlefield(P1, "K'un-Lun Warrior");
        let spell = t.hand(P1, name);
        t.cast(P1, spell).go();
        let on_stack = t.g.current(spell);
        let c = t.custom(P0, counter, Zone::Hand(P0));
        t.answer_targets(P0, &[Entity::Object(on_stack)]);
        t.g.turn.priority = Some(P0);
        let r = t.g.cast_spell(P0, c, object::CastMethod::Normal);
        assert_eq!(r.is_ok(), legal, "{name}");
        // A Hero permanent isn't a Hero spell.
        let cands: Vec<Vec<Entity>> = target_candidates(&t, 0);
        assert!(cands.iter().all(|c| !c.contains(&Entity::Object(hero_on_battlefield))));
        if legal {
            t.resolve();
            assert!(t.in_graveyard(P1, name), "{name}");
        }
    }
}

#[test]
fn spell_or_nonland_permanent_isnt_narrowed_to_permanent_spells() {
    cr!("115.1");
    let bounce = instant(
        "Spell Bounce",
        "Return target spell or nonland permanent to its owner's hand.",
    );
    let mut t = TestGame::new(2);
    t.set_step(P1, Step::PrecombatMain);
    t.lands(P1, "Island", 3);
    let divination = t.hand(P1, "Divination");
    t.cast(P1, divination).go();
    let bears = t.battlefield(P1, "Grizzly Bears");
    let forest = t.battlefield(P1, "Forest");
    let c = t.custom(P0, bounce, Zone::Hand(P0));
    let from = t.asked().len();
    let div_spell = t.g.current(divination);
    t.answer_targets(P0, &[Entity::Object(div_spell)]);
    t.g.turn.priority = Some(P0);
    t.g.cast_spell(P0, c, object::CastMethod::Normal).unwrap();
    let cands = &target_candidates(&t, from)[0];
    assert!(cands.contains(&Entity::Object(div_spell)));
    assert!(cands.contains(&Entity::Object(bears)));
    assert!(!cands.contains(&Entity::Object(forest)));
    t.resolve();
    assert!(t.in_hand(P1, "Divination"));
}

/// P0 attacks with `attacker`; P1 blocks with `blockers`; the game waits in the declare
/// blockers step with P0 holding priority (the "becomes blocked" triggers on the stack).
fn blocked_by(t: &mut TestGame, attacker: ObjectId, blockers: &[ObjectId]) {
    t.answer(
        P0,
        DecisionKind::Attackers,
        Answer::Attackers(vec![(attacker, Entity::Player(P1))]),
    );
    t.answer(
        P1,
        DecisionKind::Blockers,
        Answer::Blockers(blockers.iter().map(|b| (*b, attacker)).collect()),
    );
    t.advance_to(P0, Step::DeclareBlockers);
}

#[test]
fn sacrificed_attackers_blockers_are_known_from_last_known_information() {
    cr!("608.2h", "509.1g", "601.2h");
    assert_supported(&["Urborg Panther"]);
    let mut t = TestGame::new(2);
    let panther = t.battlefield(P0, "Urborg Panther");
    t.lands(P0, "Swamp", 1);
    let giant = t.battlefield(P1, "Hill Giant");
    let idle = t.battlefield(P1, "Grizzly Bears");
    blocked_by(&mut t, panther, &[giant]);
    let from = t.asked().len();
    t.activate(P0, panther, 0, &[Entity::Object(giant)]).unwrap();
    let c = &target_candidates(&t, from)[0];
    assert!(!c.contains(&Entity::Object(idle)));
    // The Panther is gone; the Giant is still a creature that was blocking it.
    assert!(t.in_graveyard(P0, "Urborg Panther"));
    t.resolve_all();
    assert!(t.in_graveyard(P1, "Hill Giant"));
    assert!(t.on_battlefield(idle));
}

#[test]
fn sacrificed_aura_destroys_non_wall_creatures_blocking_enchanted_creature() {
    cr!("608.2h", "303.4", "509.1g");
    assert_supported(&["Coils of the Medusa"]);
    let mut t = TestGame::new(2);
    let wurm = t.battlefield(P0, "Craw Wurm");
    let coils = t.battlefield(P0, "Coils of the Medusa");
    t.g.attach(coils, Entity::Object(wurm));
    let bears = t.battlefield(P1, "Grizzly Bears");
    let wall = t.battlefield(P1, "Wall of Wood");
    let idle = t.battlefield(P1, "Savannah Lions");
    blocked_by(&mut t, wurm, &[bears, wall]);
    t.activate(P0, coils, 0, &[]).unwrap();
    t.resolve_all();
    assert!(t.in_graveyard(P1, "Grizzly Bears"));
    assert!(t.on_battlefield(wall));
    assert!(t.on_battlefield(idle));
}

#[test]
fn sacrificed_goblin_deals_damage_to_each_creature_blocking_it() {
    cr!("608.2h", "509.1g");
    ruling!(
        "Ib Halfheart, Goblin Tactician",
        "If you don’t actually sacrifice the Goblin"
    );
    assert_supported(&["Ib Halfheart, Goblin Tactician"]);
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Ib Halfheart, Goblin Tactician");
    let goblin = t.battlefield(P0, "Raging Goblin");
    let wurm = t.battlefield(P1, "Craw Wurm");
    let bears = t.battlefield(P1, "Grizzly Bears");
    let idle = t.battlefield(P1, "Hill Giant");
    blocked_by(&mut t, goblin, &[wurm, bears]);
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Raging Goblin"));
    // 4 damage to each: the 6/4 Wurm and the Bears die.
    assert!(!t.on_battlefield(wurm));
    assert!(t.in_graveyard(P1, "Craw Wurm") && t.in_graveyard(P1, "Grizzly Bears"));
    assert_eq!(t.obj_now(idle).damage, 0);

    // Removed before the ability resolves: nothing is sacrificed, no damage is dealt.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Ib Halfheart, Goblin Tactician");
    let goblin = t.battlefield(P0, "Raging Goblin");
    let wurm = t.battlefield(P1, "Craw Wurm");
    blocked_by(&mut t, goblin, &[wurm]);
    t.settle();
    t.g.move_object(goblin, Zone::Exile, MoveCause::Effect, None);
    t.g.flush_events();
    t.resolve_all();
    assert_eq!(t.obj_now(wurm).damage, 0);
}

#[test]
fn scarlet_witch_casts_a_hero_or_noncreature_card_she_exiled() {
    cr!("115.1", "205.3m", "601.2");
    assert_supported(&["Scarlet Witch, Chaotic Avenger"]);
    for (other, castable) in [("Divination", true), ("K'un-Lun Warrior", true), ("Hill Giant", false)] {
        let mut t = TestGame::new(2);
        let witch = t.battlefield(P0, "Scarlet Witch, Chaotic Avenger");
        t.library_top(P0, "Grizzly Bears");
        t.library_top(P0, other);
        t.answer(
            P0,
            DecisionKind::Attackers,
            Answer::Attackers(vec![(witch, Entity::Player(P1))]),
        );
        let from = t.asked().len();
        t.advance_to(P0, Step::PostcombatMain);
        let names: Vec<Vec<String>> = t.asked()[from..]
            .iter()
            .filter_map(|(_, d)| match d {
                Decision::ChooseEntities { candidates, .. } => Some(
                    candidates
                        .iter()
                        .filter_map(|e| match e {
                            Entity::Object(o) => Some(
                                // Exiled face down: the card's own name.
                                t.g.face_characteristics(*o, object::FaceState::Front)
                                    .name
                                    .to_string(),
                            ),
                            _ => None,
                        })
                        .collect(),
                ),
                _ => None,
            })
            .collect();
        let offered: Vec<&String> = names.iter().flatten().collect();
        // A Hero spell or a noncreature spell; Grizzly Bears is neither.
        assert!(!offered.iter().any(|n| *n == "Grizzly Bears"), "{names:?}");
        assert_eq!(offered.iter().any(|n| *n == other), castable, "{other}: {names:?}");
    }
}
