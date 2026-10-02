//! In-game tests for compiler misreads found by the Oracle round trip (follow-up item
//! `roundtrip-parser-bugs-1`): each shows the behavior the corrected compilation has and
//! the old one didn't.

use mtg_engine::card::card;
use mtg_engine::testing::*;
use mtg_engine::*;

fn supported(name: &str) {
    let c = card(name);
    assert!(
        c.unsupported_text().is_empty(),
        "{name}: {:?}",
        c.unsupported_text()
    );
}

#[test]
fn hedron_matrix_uses_the_equipped_creatures_mana_value() {
    cr!("301.5a", "202.3");
    // "Equipped creature gets +X/+X, where X is its mana value.": "its" is the equipped
    // creature's (it was read as the Equipment's own mana value, 4).
    supported("Hedron Matrix");
    let mut t = TestGame::new(2);
    let matrix = t.battlefield(P0, "Hedron Matrix");
    let bears = t.battlefield(P0, "Grizzly Bears"); // mana value 2
    assert!(t.g.attach(matrix, Entity::Object(bears)));
    t.settle();
    assert_eq!(t.pt(bears), (4, 4));
    let wurm = t.battlefield(P0, "Craw Wurm"); // mana value 6
    assert!(t.g.attach(matrix, Entity::Object(wurm)));
    t.settle();
    assert_eq!(t.pt(wurm), (12, 10));
    assert_eq!(t.pt(bears), (2, 2));
}

#[test]
fn where_x_is_its_power_after_a_target_is_the_targets_power() {
    cr!("107.3c", "608.2c");
    // Winged Temple of Orazca: "{1}{G}{U}, {T}: Target creature you control gains flying
    // and gets +X/+X until end of turn, where X is its power." "Its" is the target's
    // (it was read as the land's own power, so the creature got +0/+0).
    supported("Hadana's Climb // Winged Temple of Orazca");
    let mut t = TestGame::new(2);
    let temple = t.battlefield(P0, "Hadana's Climb // Winged Temple of Orazca");
    assert!(mtg_engine::dfc::transform(&mut t.g, temple));
    let temple = t.g.current(temple);
    let wurm = t.battlefield(P0, "Craw Wurm"); // 6/4
    t.lands(P0, "Forest", 2);
    t.lands(P0, "Island", 1);
    t.activate(P0, temple, 1, &[Entity::Object(wurm)]).unwrap();
    t.resolve_all();
    assert_eq!(t.pt(wurm), (12, 10));
}

#[test]
fn a_sentence_after_a_this_turn_trigger_continues_it() {
    cr!("603.7b", "603.7c", "714.2b");
    // The Last Ronin, chapter III: "Whenever a creature you control attacks alone this
    // turn, put three +1/+1 counters on it. It gains trample, lifelink, and indestructible
    // until end of turn." The second sentence is part of the delayed trigger's effect (it
    // was compiled as the chapter ability's own instruction, giving the Saga the keywords).
    use mtg_engine::keywords::KeywordKind;
    supported("The Last Ronin");
    let mut t = TestGame::new(2);
    t.set_step(P0, mtg_engine::turn::Step::PrecombatMain);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let saga = t.battlefield(P0, "The Last Ronin");
    t.g.objects[saga.0 as usize]
        .counters
        .insert("lore".into(), 2);
    t.g.add_counters(Entity::Object(saga), "lore", 1, None);
    t.g.flush_events();
    t.resolve_all();
    let bears = t.g.current(bears);
    assert!(!t.obj_now(bears).chars.has_keyword(KeywordKind::Trample));
    t.answer(
        P0,
        DecisionKind::Attackers,
        Answer::Attackers(vec![(bears, Entity::Player(P1))]),
    );
    t.advance_to(P0, mtg_engine::turn::Step::DeclareBlockers);
    t.resolve_all();
    assert_eq!(t.counters(bears, "+1/+1"), 3);
    let c = &t.obj_now(bears).chars;
    assert!(c.has_keyword(KeywordKind::Trample));
    assert!(c.has_keyword(KeywordKind::Lifelink));
    assert!(c.has_keyword(KeywordKind::Indestructible));
}

#[test]
fn damage_equal_to_that_cards_mana_value_is_the_exiled_cards() {
    cr!("406.1", "120.3");
    // Undying Flames: "Exile cards from the top of your library until you exile a nonland
    // card. Undying Flames deals damage to any target equal to that card's mana value."
    // "That card" is the exiled card (it was read as the damage's target, a player here,
    // whose "mana value" is 0).
    supported("Undying Flames");
    let mut t = TestGame::new(2);
    t.library_top(P0, "Grizzly Bears"); // mana value 2
    t.library_top(P0, "Forest");
    t.lands(P0, "Mountain", 6);
    let s = t.hand(P0, "Undying Flames");
    t.cast(P0, s).target(Entity::Player(P1)).go();
    t.resolve_all();
    assert!(t.in_exile("Grizzly Bears"));
    assert_eq!(t.life(P1), 18);
}

#[test]
fn ricochet_trap_needs_only_one_opponent_to_have_cast_a_blue_spell() {
    cr!("118.9", "601.2b");
    // Ricochet Trap: "If an opponent cast a blue spell this turn, you may pay {R} rather
    // than pay this spell's mana cost." In a three-player game, one opponent's blue spell
    // is enough.
    use mtg_engine::object::CastMethod;
    supported("Ricochet Trap");
    let mut t = TestGame::new(3);
    t.set_step(P0, mtg_engine::turn::Step::PrecombatMain);
    let trap = t.hand(P0, "Ricochet Trap");
    t.lands(P0, "Mountain", 1);
    let alt_castable = |t: &mut TestGame| {
        t.cast_options(P0, trap)
            .into_iter()
            .any(|o| matches!(o.method, CastMethod::Alternative(_)))
    };
    assert!(!alt_castable(&mut t));
    let opt = t.hand(P1, "Opt");
    t.lands(P1, "Island", 1);
    t.cast(P1, opt).go();
    t.resolve_all();
    assert!(alt_castable(&mut t));
}

#[test]
fn enters_from_your_graveyard_is_only_your_own_graveyard() {
    cr!("400.3", "603.2");
    // Dredging Claw: "Whenever a creature enters from your graveyard, you may attach this
    // Equipment to it." A creature an opponent returns from their own graveyard doesn't
    // trigger it (it was read as "from a graveyard").
    use mtg_engine::events::MoveCause;
    use mtg_engine::object::Zone;
    supported("Dredging Claw");
    for (owner, triggers) in [(P1, false), (P0, true)] {
        let mut t = TestGame::new(2);
        t.set_step(P0, mtg_engine::turn::Step::PrecombatMain);
        t.battlefield(P0, "Dredging Claw");
        let bears = t.graveyard(owner, "Grizzly Bears");
        t.g.move_object(bears, Zone::Battlefield, MoveCause::Effect, Some(owner));
        t.g.flush_events();
        t.settle();
        assert_eq!(t.stack_len(), usize::from(triggers), "owner {owner:?}");
    }
}
