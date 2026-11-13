//! Having or gaining the activated abilities of other objects
//! (`src/oracle/patterns/grant_activated_abilities.rs`, `Modification::AddAbilitiesOf`;
//! CR 113.6, 602.5c, 611.2c) and spending mana as though it were any color to activate an
//! object's own abilities (CR 609.4b).

use mtg_engine::ability::AbilityKind;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

fn assert_supported(names: &[&str]) {
    for n in names {
        let u = card(n).unsupported_text().join(" | ");
        assert!(u.is_empty(), "{n} has unsupported text: {u}");
    }
}

fn activated(t: &TestGame, id: ObjectId) -> Vec<String> {
    t.obj_now(id)
        .chars
        .abilities
        .iter()
        .filter(|a| matches!(a.kind, AbilityKind::Activated(_)))
        .map(|a| a.text.to_string())
        .collect()
}

fn index_of(t: &TestGame, id: ObjectId, needle: &str) -> usize {
    activated(t, id)
        .iter()
        .position(|a| a.contains(needle))
        .unwrap_or_else(|| panic!("no ability with {needle:?}: {:?}", activated(t, id)))
}

#[test]
fn quicksilver_elemental_gains_abilities_and_pays_them_with_blue_mana() {
    cr!("611.2c", "609.4b", "113.6");
    ruling!(
        "Quicksilver Elemental",
        "gains only activated abilities. It doesn't gain keyword abilities"
    );
    assert_supported(&["Quicksilver Elemental"]);
    let mut t = TestGame::new(2);
    let q = t.battlefield(P0, "Quicksilver Elemental");
    let dragon = t.battlefield(P1, "Shivan Dragon");
    t.lands(P0, "Island", 3);
    t.set_step(P0, Step::PrecombatMain);
    t.activate(P0, q, 0, &[Entity::Object(dragon)]).unwrap();
    t.resolve();
    assert_eq!(activated(&t, q).len(), 2);
    // Not flying: a keyword.
    assert!(!t
        .obj_now(q)
        .has_keyword(mtg_engine::keywords::KeywordKind::Flying));
    // The Dragon leaving doesn't take the ability away (determined as it resolved).
    t.g.destroy_all(vec![dragon], None, false);
    t.settle();
    // "{R}: ~ gets +1/+0": paid with blue mana.
    let i = index_of(&t, q, "{R}");
    t.activate(P0, q, i, &[]).unwrap();
    t.resolve();
    t.activate(P0, q, i, &[]).unwrap();
    t.resolve();
    assert_eq!(t.pt(q), (5, 4));
    // Until end of turn.
    t.advance_to(P1, Step::Upkeep);
    assert_eq!(activated(&t, q).len(), 1);
}

#[test]
fn robaran_mercenaries_has_its_legendary_creatures_abilities() {
    cr!("113.6", "602.5c");
    ruling!(
        "Robaran Mercenaries",
        "The costs of activated abilities that Robaran Mercenaries gains must be paid as normal"
    );
    assert_supported(&["Robaran Mercenaries"]);
    let mut t = TestGame::new(2);
    let r = t.battlefield(P0, "Robaran Mercenaries");
    assert!(activated(&t, r).is_empty());
    let k = t.battlefield(P0, "Kamahl, Pit Fighter");
    // An opponent's legendary creature doesn't count.
    t.battlefield(P1, "Kamahl, Pit Fighter");
    t.settle();
    assert_eq!(activated(&t, r).len(), 1);
    // The gained ability names Robaran: Robaran taps and deals the damage.
    t.activate(P0, r, 0, &[Entity::Player(P1)]).unwrap();
    t.resolve();
    assert_eq!(t.life(P1), 17);
    assert!(t.obj_now(r).tapped);
    assert!(!t.obj_now(k).tapped);
    // Gone when the legendary creature is.
    t.g.destroy_all(vec![k], None, false);
    t.settle();
    assert!(activated(&t, r).is_empty());
}

#[test]
fn manascape_refractor_has_every_lands_mana_abilities() {
    cr!("113.6", "305.6");
    ruling!(
        "Manascape Refractor",
        "Lands with a basic land type have an intrinsic activated mana ability"
    );
    assert_supported(&["Manascape Refractor"]);
    let mut t = TestGame::new(2);
    let m = t.battlefield(P0, "Manascape Refractor");
    t.battlefield(P0, "Forest");
    t.battlefield(P1, "Island");
    t.battlefield(P1, "Island");
    t.settle();
    // "{T}: Add {G}" and "{T}: Add {U}" twice (one from each Island).
    assert_eq!(activated(&t, m).len(), 3);
}

#[test]
fn territory_forge_has_the_exiled_cards_abilities() {
    cr!("607.2a", "113.6");
    ruling!(
        "Territory Forge",
        "Territory Forge gains only activated abilities"
    );
    assert_supported(&["Territory Forge"]);
    let mut t = TestGame::new(2);
    let stone = t.battlefield(P1, "Mind Stone");
    t.lands(P0, "Mountain", 5);
    let f = t.hand(P0, "Territory Forge");
    t.set_step(P0, Step::PrecombatMain);
    t.answer(
        P0,
        DecisionKind::Targets,
        mtg_engine::decision::Answer::Entities(vec![Entity::Object(stone)]),
    );
    t.cast(P0, f).go();
    t.resolve_all();
    let f = t.named_on_battlefield("Territory Forge")[0];
    assert!(!t.on_battlefield(stone));
    assert_eq!(activated(&t, f).len(), 2);
}

#[test]
fn a_keyword_that_is_an_activated_ability_is_gained_once() {
    cr!("702.151a", "113.6");
    ruling!(
        "Quicksilver Elemental",
        "unless those keyword abilities are activated"
    );
    let mut t = TestGame::new(2);
    let q = t.battlefield(P0, "Quicksilver Elemental");
    // "{W}: Exile target card from a graveyard. ..." and reconfigure {2}.
    let sash = t.battlefield(P1, "Lion Sash");
    t.lands(P0, "Island", 1);
    t.set_step(P0, Step::PrecombatMain);
    let before = activated(&t, q).len();
    t.activate(P0, q, 0, &[Entity::Object(sash)]).unwrap();
    t.resolve();
    assert_eq!(activated(&t, q).len(), before + activated(&t, sash).len());
    // Reconfigure is two activated abilities (CR 702.151a).
    assert_eq!(activated(&t, q).len(), before + 3, "{:?}", activated(&t, q));
}
