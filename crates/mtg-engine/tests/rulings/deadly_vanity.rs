//! Rulings: Deadly Vanity (the back face of Selfless Glyphweaver): "Choose a creature or
//! planeswalker, then destroy all other creatures and planeswalkers."

use mtg_engine::decision::Decision;
use mtg_engine::mana::ManaType;
use mtg_engine::object::CastMethod;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::PlayerId;
use mtg_engine::*;

const CARD: &str = "Selfless Glyphweaver // Deadly Vanity";

fn add_mana(t: &mut TestGame, p: PlayerId, m: ManaType, n: u32) {
    t.g.players[p.idx()].mana_pool.add_type(m, n);
}

/// The candidates P0 was asked to choose among.
fn choices(t: &TestGame) -> Vec<Vec<Entity>> {
    t.asked()
        .into_iter()
        .filter_map(|(p, d)| match d {
            Decision::ChooseEntities { candidates, .. } if p == P0 => Some(candidates),
            _ => None,
        })
        .collect()
}

#[test]
fn the_chosen_permanent_is_saved_and_all_others_are_destroyed() {
    cr!("115.10", "115.10a");
    ruling!(
        "Selfless Glyphweaver // Deadly Vanity",
        "As Deadly Vanity resolves, you choose which creature or planeswalker you are saving."
    );
    ruling!(
        "Selfless Glyphweaver // Deadly Vanity",
        "Because Deadly Vanity does not target the permanent you choose to save, you may choose creatures with hexproof or protection from black."
    );
    ruling!(
        "Selfless Glyphweaver // Deadly Vanity",
        "You may choose any creature or planeswalker, even one controlled by an opponent."
    );
    assert!(card(CARD).unsupported_text().is_empty());
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let scout = t.battlefield(P1, "Gladecover Scout");
    let giant = t.battlefield(P1, "Hill Giant");
    let jace = t.battlefield(P1, "Jace Beleren");
    let forest = t.battlefield(P1, "Forest");
    let vanity = t.hand(P0, CARD);
    add_mana(&mut t, P0, ManaType::B, 8);
    // The opponent's hexproof creature is saved: it isn't a target.
    t.answer_choose(P0, &[Entity::Object(scout)]);
    t.cast(P0, vanity).method(CastMethod::Half(1)).go();
    assert!(choices(&t).is_empty());
    t.resolve_all();
    let asked = choices(&t);
    assert_eq!(asked.len(), 1);
    for c in [bears, scout, giant, jace] {
        assert!(asked[0].contains(&Entity::Object(c)));
    }
    assert!(t.on_battlefield(scout));
    assert!(!t.on_battlefield(bears) && !t.on_battlefield(giant) && !t.on_battlefield(jace));
    assert!(t.on_battlefield(forest));
}

#[test]
fn a_creature_or_planeswalker_must_be_chosen() {
    cr!("115.10");
    ruling!(
        "Selfless Glyphweaver // Deadly Vanity",
        "You must choose a creature or planeswalker if there is one on the battlefield."
    );
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    t.battlefield(P0, "Grizzly Bears");
    t.battlefield(P1, "Hill Giant");
    let vanity = t.hand(P0, CARD);
    add_mana(&mut t, P0, ManaType::B, 8);
    // Choosing nothing isn't allowed: one of them is still saved.
    t.answer_choose(P0, &[]);
    t.cast(P0, vanity).method(CastMethod::Half(1)).go();
    t.resolve_all();
    let left = t.named_on_battlefield("Grizzly Bears").len()
        + t.named_on_battlefield("Hill Giant").len();
    assert_eq!(left, 1);
}
