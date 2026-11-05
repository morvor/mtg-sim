//! Rulings batch P033 — "Spells you cast with mana value 5 or greater cost {1} less to cast
//! and can't be countered" (Thryx, the Sudden Storm) and "Spells with flash you cast cost
//! {1} less to cast and can't be countered" (Cunning Nightbonder): the cost is locked in
//! as the spell is cast, with no player getting priority in between (CR 601.2f, 601.2i),
//! while "can't be countered" applies to the spells on the stack only while the permanent
//! is on the battlefield, including spells cast before it entered (CR 611.3a).

use crate::r_p033_common::*;
use crate::r_p046_common::others_asked_since;
use crate::r_s01_common::supported;
use mtg_engine::mana::ManaType;
use mtg_engine::testing::*;
use mtg_engine::types::*;
use mtg_engine::*;

/// P1 casts Cancel ({1}{U}{U}: "Counter target spell.") targeting `spell` and it resolves.
fn p1_cancels(t: &mut TestGame, spell: ObjectId) {
    let cancel = t.hand(P1, "Cancel");
    add(t, P1, ManaType::U, 3);
    t.cast(P1, cancel).target(obj(spell)).go();
    t.resolve();
}

/// P1 casts Unsummon ({U}: "Return target creature to its owner's hand.") targeting
/// `creature` and it resolves.
fn p1_unsummons(t: &mut TestGame, creature: ObjectId) {
    let un = t.hand(P1, "Unsummon");
    add(t, P1, ManaType::U, 1);
    t.cast(P1, un).target(obj(creature)).go();
    t.resolve();
}

#[test]
fn thryx_makes_a_spell_already_on_the_stack_uncounterable_without_a_discount() {
    cr!("611.3a", "601.2f", "701.6a", "113.6");
    ruling!(
        "Thryx, the Sudden Storm",
        "If Thryx enters the battlefield while a spell with mana value 5 or greater that you cast is on the stack, that spell can't be countered, even though you don't get the {1} discount."
    );
    ruling!(
        "Thryx, the Sudden Storm",
        "Thryx's last ability doesn't apply to itself while it's a spell."
    );
    supported("Thryx, the Sudden Storm");
    supported("Cancel");
    let mut t = main_phase(2);
    // Craw Wurm (mana value 6) cast first costs {4}{G}{G}.
    let wurm = t.hand(P0, "Craw Wurm");
    let wurm = cast_normal(&mut t, P0, wurm);
    assert_spent(&wurm.spent, "{4}{G}{G}", "Craw Wurm");
    // Thryx ({3}{U}{U}, flash) in response costs its full cost, and resolves.
    let thryx = t.hand(P0, "Thryx, the Sudden Storm");
    let thryx = cast_normal(&mut t, P0, thryx);
    assert_spent(&thryx.spent, "{3}{U}{U}", "Thryx");
    t.resolve();
    assert!(t.on_battlefield(thryx.spell));
    // P1's Cancel can't counter the Wurm.
    p1_cancels(&mut t, wurm.spell);
    assert!(t.in_graveyard(P1, "Cancel"));
    t.resolve_all();
    assert!(t.on_battlefield(wurm.spell));
    // Thryx itself, as a spell, can be countered.
    let mut t = main_phase(2);
    let thryx = t.hand(P0, "Thryx, the Sudden Storm");
    let thryx = cast_normal(&mut t, P0, thryx);
    p1_cancels(&mut t, thryx.spell);
    assert!(t.in_graveyard(P0, "Thryx, the Sudden Storm"));
}

#[test]
fn removing_thryx_or_nightbonder_doesnt_change_the_cost_but_lets_spells_be_countered() {
    cr!("601.2f", "601.2i", "611.3a", "701.6a");
    ruling!(
        "Thryx, the Sudden Storm",
        "Players can't attempt to remove Thryx to raise the cost of a spell you wish to cast, but they can remove it to allow their spells and abilities to counter your spells."
    );
    ruling!(
        "Cunning Nightbonder",
        "Players can't attempt to remove Cunning Nightbonder to raise the cost of a spell you wish to cast, but they can remove it to allow their spells and abilities to counter your spells."
    );
    supported("Cunning Nightbonder");
    supported("Unsummon");
    // (reducer, spell, what it costs with the reducer)
    for (reducer, spell, cost) in [
        // Craw Wurm ({4}{G}{G}).
        ("Thryx, the Sudden Storm", "Craw Wurm", "{3}{G}{G}"),
        // Boon Satyr ({1}{G}{G}, flash).
        ("Cunning Nightbonder", "Boon Satyr", "{G}{G}"),
    ] {
        let mut t = main_phase(2);
        let r = t.battlefield(P0, reducer);
        let card = t.hand(P0, spell);
        // Nobody gets priority while the spell is being cast: its cost is locked in with
        // the reducer on the battlefield.
        let from = t.asked().len();
        let paid = cast_normal(&mut t, P0, card);
        assert_spent(&paid.spent, cost, reducer);
        assert!(
            others_asked_since(&t, from, P0).is_empty(),
            "{reducer}: asked during casting"
        );
        // With the reducer still there, Cancel doesn't counter it; once P1 has bounced
        // it, Cancel does.
        let mut t2 = main_phase(2);
        t2.battlefield(P0, reducer);
        let c2 = t2.hand(P0, spell);
        let s2 = cast_normal(&mut t2, P0, c2).spell;
        p1_cancels(&mut t2, s2);
        assert!(
            t2.obj_now(s2).zone == mtg_engine::object::Zone::Stack,
            "{reducer}: countered"
        );
        p1_unsummons(&mut t, r);
        assert!(t.in_hand(P0, reducer));
        // What was paid stays paid.
        assert_eq!(
            t.obj(paid.spell)
                .stack
                .as_ref()
                .unwrap()
                .cast
                .mana_spent
                .len(),
            paid.spent.len()
        );
        p1_cancels(&mut t, paid.spell);
        assert!(
            t.in_graveyard(P0, spell),
            "{reducer}: not countered after it left"
        );
    }
}
