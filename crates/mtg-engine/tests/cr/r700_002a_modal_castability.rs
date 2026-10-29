//! CR 700.2a, 601.2c: a modal spell can be cast only if one of its modes can be chosen,
//! which needs legal targets for that mode. The list of legal actions offered with
//! priority agrees with casting: checked for every fully supported modal instant and
//! sorcery card, on an empty battlefield.

use mtg_engine::ability::AbilityKind;
use mtg_engine::card::card;
use mtg_engine::decision::Action;
use mtg_engine::mana::ManaType;
use mtg_engine::testing::*;
use mtg_engine::*;

/// Whether P0's legal actions include casting `c`.
fn listed(t: &mut TestGame, c: ObjectId) -> bool {
    t.g.turn.priority = Some(P0);
    t.g.recompute();
    t.g.legal_actions(P0)
        .iter()
        .any(|a| matches!(a, Action::Cast { card, .. } if *card == c))
}

#[test]
fn a_modal_spell_with_no_choosable_mode_isnt_a_legal_action() {
    cr!("700.2a", "601.2c");
    let mut checked = 0;
    let mut uncastable = vec![];
    for sc in mtg_data::cards().iter() {
        let text = sc.oracle_text.as_deref().unwrap_or("");
        let types = sc.type_line.as_deref().unwrap_or("");
        if !(types.contains("Instant") || types.contains("Sorcery"))
            || !text.contains("Choose ")
            || !text.contains('•')
        {
            continue;
        }
        let def = card(&sc.name);
        let modal = def.faces[0].chars.abilities.iter().any(
            |a| matches!(&a.kind, AbilityKind::Spell(s) if s.body.modal.is_some()),
        );
        if !def.is_fully_supported() || !modal {
            continue;
        }
        let mut t = TestGame::new(2);
        for ty in [
            ManaType::W,
            ManaType::U,
            ManaType::B,
            ManaType::R,
            ManaType::G,
            ManaType::C,
        ] {
            t.g.players[0].mana_pool.add_type(ty, 8);
        }
        let c = t.hand(P0, &sc.name);
        let offered = listed(&mut t, c);
        // X is 0 (a greater default X could need more targets than there are).
        match t.cast(P0, c).x(0).try_go() {
            Err(e) if format!("{e:?}").contains("no legal targets / modes") => {
                assert!(!offered, "{}: listed but no mode can be chosen", sc.name);
                uncastable.push(sc.name.clone());
            }
            Ok(_) => assert!(offered, "{}: castable but not listed", sc.name),
            Err(_) => {}
        }
        checked += 1;
    }
    assert!(checked > 100, "only {checked} modal spells checked");
    // Bant Charm (destroy target artifact; put target creature on the bottom; counter
    // target instant spell) is among those that can't be cast here.
    assert!(uncastable.iter().any(|n| n == "Bant Charm"), "{uncastable:?}");
}

#[test]
fn default_modes_repeat_a_mode_that_may_be_chosen_more_than_once() {
    cr!("700.2d", "700.2a");
    interp!("default-first-listed-choice");
    // Eldrazi Confluence: "Choose three. You may choose the same mode more than once." With
    // no nonland permanent on the battlefield, only "Create a 1/1 colorless Eldrazi Scion
    // creature token ..." can be chosen: by default, three times.
    let mut t = TestGame::new(2);
    for ty in [ManaType::C, ManaType::U, ManaType::B] {
        t.g.players[0].mana_pool.add_type(ty, 2);
    }
    let c = t.hand(P0, "Eldrazi Confluence");
    t.cast(P0, c).go();
    t.resolve_all();
    let scions = t
        .g
        .permanents()
        .filter(|o| o.is_token() && o.chars.has_subtype("Scion"))
        .count();
    assert_eq!(scions, 3);
}

