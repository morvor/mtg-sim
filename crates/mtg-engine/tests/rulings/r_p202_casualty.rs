//! Rulings batch P202 — casualty (CR 702.153) on Grisly Sigil: "Choose target creature or
//! planeswalker. If it was dealt noncombat damage this turn, Grisly Sigil deals 3 damage to
//! it and you gain 3 life. Otherwise, Grisly Sigil deals 1 damage to it and you gain 1
//! life."

use crate::r_s01_common::*;
use mtg_engine::decision::Answer;
use mtg_engine::testing::*;
use mtg_engine::*;

/// P0 casts Grisly Sigil targeting `target`, sacrificing `sacrifice` for casualty if given.
fn cast_sigil(t: &mut TestGame, target: ObjectId, sacrifice: Option<ObjectId>) {
    supported("Grisly Sigil");
    give_mana_for(t, P0, "Grisly Sigil");
    let c = t.hand(P0, "Grisly Sigil");
    t.answer(
        P0,
        DecisionKind::OptionalCost,
        Answer::Bool(sacrifice.is_some()),
    );
    if let Some(s) = sacrifice {
        t.answer_choose(P0, &[Entity::Object(s)]);
    }
    t.cast(P0, c).target(Entity::Object(target)).go();
    t.settle();
}

#[test]
fn grisly_sigils_casualty_copy_deals_damage_first_so_the_original_deals_three() {
    cr!("702.153a", "707.10", "608.1", "120.2");
    ruling!(
        "Grisly Sigil",
        "the copy will deal 1 damage and you will gain 1 life. Then, the original spell will resolve. It will deal 3 damage, and you will gain 3 life."
    );
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let colossus = t.battlefield(P1, "Darksteel Colossus");
    cast_sigil(&mut t, colossus, Some(bears));
    // The casualty trigger copies the spell; the copy resolves first, while the Colossus
    // hasn't been dealt noncombat damage yet.
    t.resolve();
    assert_eq!(t.obj_now(colossus).damage, 0);
    t.resolve();
    assert_eq!(t.obj_now(colossus).damage, 1);
    assert_eq!(t.life(P0), 21);
    // Now it has: the original deals 3.
    t.resolve_all();
    assert_eq!(t.obj_now(colossus).damage, 4);
    assert_eq!(t.life(P0), 24);
}

#[test]
fn grisly_sigil_counts_only_noncombat_damage() {
    cr!("120.2", "510.2");
    // Without noncombat damage (no damage at all, or only combat damage): 1 damage.
    let mut t = TestGame::new(2);
    let colossus = t.battlefield(P1, "Darksteel Colossus");
    cast_sigil(&mut t, colossus, None);
    t.resolve_all();
    assert_eq!(t.obj_now(colossus).damage, 1);
    assert_eq!(t.life(P0), 21);

    // Combat damage this turn: still 1.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let attacker = t.battlefield(P0, "Craw Wurm");
    t.attack(&[(attacker, Entity::Player(P1))], &[(bears, attacker)]);
    let wurm = t.g.current(attacker);
    assert_eq!(t.obj_now(wurm).damage, 2);
    t.advance_to(P0, mtg_engine::turn::Step::PostcombatMain);
    cast_sigil(&mut t, wurm, None);
    t.resolve_all();
    assert_eq!(t.obj_now(wurm).damage, 3);
    assert_eq!(t.life(P0), 21);

    // Noncombat damage earlier this turn: 3.
    let mut t = TestGame::new(2);
    let colossus = t.battlefield(P1, "Darksteel Colossus");
    t.lands(P0, "Mountain", 1);
    let shock = t.hand(P0, "Shock");
    t.cast(P0, shock).target(Entity::Object(colossus)).go();
    t.resolve_all();
    cast_sigil(&mut t, colossus, None);
    t.resolve_all();
    assert_eq!(t.obj_now(colossus).damage, 5);
    assert_eq!(t.life(P0), 23);
}

