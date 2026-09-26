//! CR 700.8–700.12: party, activated this turn, descended, outlaws.

use crate::r700_common::*;
use mtg_engine::ability::*;
use mtg_engine::decision::Decision;
use mtg_engine::events::MoveCause;
use mtg_engine::game_terms::party_size;
use mtg_engine::keywords::KeywordKind;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

#[test]
fn a_party_is_up_to_one_each_of_cleric_rogue_warrior_and_wizard() {
    cr!("700.8", "700.8a");
    supported("Malakir Blood-Priest");
    let mut t = TestGame::new(2);
    // Two Clerics count once; the Bears aren't in any party.
    t.battlefield(P0, "Keepers of the Faith");
    t.battlefield(P0, "Bane Alley Blackguard");
    t.battlefield(P0, "Oreskos Swiftclaw");
    t.battlefield(P0, "Grizzly Bears");
    // An opponent's Wizard isn't in P0's party.
    t.battlefield(P1, "Fugitive Wizard");
    assert_eq!(party_size(&t.g, P0), 3);
    // "each opponent loses X life and you gain X life, where X is the number of creatures
    // in your party": the Blood-Priest (a Cleric) doesn't add to it.
    t.enter(P0, "Malakir Blood-Priest");
    t.resolve_all();
    assert_eq!(t.life(P1), 17);
    assert_eq!(t.life(P0), 23);
}

#[test]
fn a_creature_with_several_party_types_is_counted_for_the_highest_result() {
    cr!("700.8b");
    let rogue_wizard = oracle_card(
        "Masked Magus",
        "Creature — Human Rogue Wizard",
        "{1}{U}",
        Some((1, 1)),
        "",
    );
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Keepers of the Faith");
    t.custom(P0, rogue_wizard.clone(), Zone::Battlefield);
    assert_eq!(party_size(&t.g, P0), 2);
    // With a Wizard, the Rogue Wizard counts as the Rogue: three, not two.
    t.battlefield(P0, "Fugitive Wizard");
    assert_eq!(party_size(&t.g, P0), 3);
    // A second Rogue Wizard can't fill two roles at once either.
    t.custom(P0, rogue_wizard, Zone::Battlefield);
    assert_eq!(party_size(&t.g, P0), 3);
}

#[test]
fn a_full_party_has_four_creatures() {
    cr!("700.8c");
    supported("Archpriest of Iona");
    let mut t = TestGame::new(2);
    // "Archpriest of Iona's power is equal to the number of creatures in your party. At
    // the beginning of combat on your turn, if you have a full party, target creature gets
    // +1/+1 and gains flying until end of turn."
    let priest = t.battlefield(P0, "Archpriest of Iona");
    t.battlefield(P0, "Bane Alley Blackguard");
    t.battlefield(P0, "Fugitive Wizard");
    assert_eq!(t.pt(priest).0, 3);
    t.set_step(P0, Step::PrecombatMain);
    t.advance_to_step(Step::BeginningOfCombat);
    t.settle();
    assert_eq!(t.stack_len(), 0);
    // A Warrior completes the party.
    let mut t = TestGame::new(2);
    let priest = t.battlefield(P0, "Archpriest of Iona");
    t.battlefield(P0, "Bane Alley Blackguard");
    t.battlefield(P0, "Fugitive Wizard");
    t.battlefield(P0, "Oreskos Swiftclaw");
    assert_eq!(t.pt(priest).0, 4);
    t.answer_targets(P0, &[Entity::Object(priest)]);
    t.set_step(P0, Step::PrecombatMain);
    t.advance_to_step(Step::BeginningOfCombat);
    t.settle();
    t.resolve_all();
    assert_eq!(t.pt(priest).0, 5);
    assert!(t.obj_now(priest).has_keyword(KeywordKind::Flying));
}

#[test]
fn choosing_a_party_chooses_up_to_one_creature_of_each_type() {
    cr!("700.8d");
    supported("Stick Together");
    let mut t = TestGame::new(2);
    t.lands(P0, "Plains", 5);
    let cleric = t.battlefield(P0, "Keepers of the Faith");
    let rogue = t.battlefield(P0, "Bane Alley Blackguard");
    let w1 = t.battlefield(P0, "Oreskos Swiftclaw");
    let w2 = t.battlefield(P0, "Oreskos Swiftclaw");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let theirs = t.battlefield(P1, "Fugitive Wizard");
    let their_bears = t.battlefield(P1, "Grizzly Bears");
    // P0 chooses the Cleric, no Rogue, and the second Warrior.
    t.answer_choose(P0, &[Entity::Object(cleric)]);
    t.answer_choose(P0, &[]);
    t.answer_choose(P0, &[Entity::Object(w2)]);
    let s = t.hand(P0, "Stick Together");
    t.cast(P0, s).go();
    t.resolve();
    assert!(t.on_battlefield(cleric));
    assert!(t.on_battlefield(w2));
    assert!(!t.g.is_live(rogue));
    assert!(!t.g.is_live(w1));
    assert!(!t.g.is_live(bears));
    // P1 keeps their Wizard (by default, the largest party).
    assert!(t.on_battlefield(theirs));
    assert!(!t.g.is_live(their_bears));
    // One choice per creature type P0 had creatures of.
    let asked: Vec<String> = t
        .asked()
        .iter()
        .filter_map(|(p, d)| match d {
            Decision::ChooseEntities { prompt, .. } if *p == P0 => Some(prompt.clone()),
            _ => None,
        })
        .collect();
    assert_eq!(asked.len(), 3);
}

#[test]
fn a_permanent_that_was_activated_this_turn() {
    cr!("700.10");
    supported("Cut Short");
    supported("Jace Beleren");
    let mut t = TestGame::new(2);
    let jace = t.battlefield(P1, "Jace Beleren");
    let idle = t.battlefield(P1, "Liliana of the Veil");
    t.g.objects[idle.0 as usize]
        .counters
        .insert("loyalty".into(), 3);
    // P1 activates the first Jace's +2 on their turn.
    t.set_step(P1, Step::PrecombatMain);
    t.activate(P1, jace, 0, &[]).unwrap();
    t.resolve();
    // It loses all its abilities: it was still activated this turn.
    run(
        &mut t,
        P1,
        None,
        Effect::Modify {
            what: Sel::All(Filter::Objects(vec![jace])),
            mods: vec![Modification::RemoveAllAbilities],
            duration: Duration::EndOfTurn,
        },
    );
    t.lands(P0, "Plains", 3);
    let cut = t.hand(P0, "Cut Short");
    t.set_step(P1, Step::End);
    t.cast(P0, cut).target(jace).go();
    // The planeswalker that wasn't activated wasn't a legal target.
    let cands = t
        .asked()
        .iter()
        .find_map(|(p, d)| match d {
            Decision::ChooseTargets { candidates, .. } if *p == P0 => Some(candidates.clone()),
            _ => None,
        })
        .unwrap();
    assert!(cands.contains(&Entity::Object(jace)));
    assert!(!cands.contains(&Entity::Object(idle)));
    t.resolve();
    assert!(!t.g.is_live(jace));
    assert!(t.on_battlefield(idle));
}

#[test]
fn descending_is_putting_a_permanent_card_into_your_graveyard() {
    cr!("700.11");
    let def = oracle_card(
        "Depth Counter",
        "Creature — Fungus",
        "{2}",
        Some((1, 1)),
        "At the beginning of your end step, you gain X life, where X is the number of times you descended this turn.",
    );
    supported("Deep Goblin Skulltaker");
    let mut t = TestGame::new(2);
    t.custom(P0, def, Zone::Battlefield);
    let skulltaker = t.battlefield(P0, "Deep Goblin Skulltaker");
    let bears = t.battlefield(P0, "Grizzly Bears");
    // A creature card from the battlefield and a land card from the hand: two descents.
    t.g.destroy(bears, None);
    let land = t.hand(P0, "Forest");
    t.g.move_object(land, Zone::Graveyard(P0), MoveCause::Discard, Some(P0));
    // An instant card isn't a permanent card; a token isn't a card; another player's
    // permanent cards don't count for P0.
    let bolt = t.hand(P0, "Lightning Bolt");
    t.g.move_object(bolt, Zone::Graveyard(P0), MoveCause::Discard, Some(P0));
    run(
        &mut t,
        P0,
        None,
        Effect::KeywordAction {
            action: KeywordAction::Investigate,
            who: PlayerRef::You,
            what: Sel::None,
            n: Value::c(1),
        },
    );
    let token = *t.g.battlefield.last().unwrap();
    t.g.destroy(token, None);
    let theirs = t.battlefield(P1, "Grizzly Bears");
    t.g.destroy(theirs, None);
    t.g.flush_events();
    // The cards needn't still be there.
    let gy = t.g.player(P0).graveyard.clone();
    for c in gy {
        t.g.move_object(c, Zone::Exile, MoveCause::Effect, None);
    }
    t.g.flush_events();
    assert_eq!(mtg_engine::game_terms::times_descended(&t.g, P0), 2);
    t.advance_to_step(Step::End);
    t.resolve_all();
    assert_eq!(t.life(P0), 22);
    assert_eq!(t.counters(skulltaker, "+1/+1"), 1);
}

#[test]
fn outlaws_are_assassins_mercenaries_pirates_rogues_and_warlocks() {
    cr!("700.12");
    supported("Hellspur Posse Boss");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Hellspur Posse Boss");
    let mk = |t: &mut TestGame, ty: &str| {
        let def = oracle_card(
            &format!("Test {ty}"),
            &format!("Creature — Human {ty}"),
            "{1}",
            Some((1, 1)),
            "",
        );
        t.custom(P0, def, Zone::Battlefield)
    };
    for ty in ["Assassin", "Mercenary", "Pirate", "Rogue", "Warlock"] {
        let c = mk(&mut t, ty);
        assert!(t.obj_now(c).has_keyword(KeywordKind::Haste), "{ty}");
    }
    for ty in ["Warrior", "Wizard", "Soldier"] {
        let c = mk(&mut t, ty);
        assert!(!t.obj_now(c).has_keyword(KeywordKind::Haste), "{ty}");
    }
}

#[test]
fn outlaws_you_control_are_outlaw_permanents() {
    cr!("700.12a");
    let def = oracle_card(
        "Outlaw Counter",
        "Creature — Human",
        "{1}",
        Some((1, 1)),
        "When this creature enters, you gain 1 life for each outlaw you control.",
    );
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Bane Alley Blackguard");
    t.battlefield(P1, "Bane Alley Blackguard");
    // Outlaw cards in hand or graveyard aren't counted.
    t.hand(P0, "Bane Alley Blackguard");
    t.graveyard(P0, "Bane Alley Blackguard");
    let card = t.custom(P0, def, Zone::Hand(P0));
    t.g.move_object(card, Zone::Battlefield, MoveCause::Effect, Some(P0));
    t.settle();
    t.resolve_all();
    assert_eq!(t.life(P0), 21);
}

/// Index of Mjölnir's equip ability among its activated abilities.
fn equip_index(t: &TestGame, id: ObjectId) -> usize {
    t.g.obj(id)
        .chars
        .abilities
        .iter()
        .filter(|a| matches!(a.kind, AbilityKind::Activated(_)))
        .position(|a| a.text.starts_with("Equip"))
        .expect("no equip ability")
}

#[test]
fn a_worthy_creature_is_legendary_non_villain_and_red_or_white() {
    cr!("700.16");
    supported("Mjölnir, Hammer of Thor");
    let villain = oracle_card(
        "Test Villain",
        "Legendary Creature — Human Villain",
        "{R}",
        Some((2, 2)),
        "",
    );
    // "Equip worthy {1}": only a worthy creature can be equipped.
    for (name, worthy) in [
        ("Grizzly Bears", false),
        ("Yeva, Nature's Herald", false),
        ("Test Villain", false),
        ("Isamaru, Hound of Konda", true),
    ] {
        let mut t = TestGame::new(2);
        t.lands(P0, "Plains", 1);
        let mj = t.battlefield(P0, "Mjölnir, Hammer of Thor");
        let c = if name == "Test Villain" {
            t.custom(P0, villain.clone(), Zone::Battlefield)
        } else {
            t.battlefield(P0, name)
        };
        let i = equip_index(&t, mj);
        let ok = t.activate(P0, mj, i, &[Entity::Object(c)]).is_ok();
        assert_eq!(ok, worthy, "{name}");
        if ok {
            t.resolve();
            assert_eq!(t.obj(mj).attached_to, Some(Entity::Object(c)));
            // "Double all damage equipped creature would deal."
            t.set_step(P0, Step::BeginningOfCombat);
            t.attack(&[(c, Entity::Player(P1))], &[]);
            assert_eq!(t.life(P1), 16);
        }
    }
}
