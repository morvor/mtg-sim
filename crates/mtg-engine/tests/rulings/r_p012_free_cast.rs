//! Rulings batch P012 — casting a spell from hand "without paying its mana cost" while an
//! ability resolves (Omnispell Adept, Oracle of Bones, Maelstrom Archangel) and playing
//! the revealed card with Djinn of Wishes: it's cast (and can be countered), X is 0
//! (CR 107.3b), no alternative cost but additional costs (CR 118.9a), fuse (CR 702.102c),
//! type-based timing ignored but other restrictions kept (CR 608.2g, 601.3), and only
//! while the ability resolves.

use crate::r_p012_common::*;
use crate::r_s01_common::{asked_since, supported, tapped_lands};
use crate::r_s22_common::attack_p1_unblocked;
use mtg_engine::decision::{Answer, Decision};
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

/// The ways of casting a spell from hand without paying its mana cost.
#[derive(Clone, Copy, Debug, PartialEq)]
enum Caster {
    /// "{2}{U}, {T}: You may cast an instant or sorcery spell from your hand without
    /// paying its mana cost."
    Omnispell,
    /// "Tribute 2 ... When this creature enters, if tribute wasn't paid, you may cast an
    /// instant or sorcery spell from your hand without paying its mana cost."
    Oracle,
    /// "Whenever this creature deals combat damage to a player, you may cast a spell from
    /// your hand without paying its mana cost."
    Maelstrom,
}

const CASTERS: [Caster; 3] = [Caster::Omnispell, Caster::Oracle, Caster::Maelstrom];

impl Caster {
    fn name(self) -> &'static str {
        match self {
            Caster::Omnispell => "Omnispell Adept",
            Caster::Oracle => "Oracle of Bones",
            Caster::Maelstrom => "Maelstrom Archangel",
        }
    }
}

/// P0 casts `card` from hand through `c` (choosing to, choosing the card; `queue` queues
/// the rest of the answers), and everything resolves.
fn free_cast(t: &mut TestGame, c: Caster, card: ObjectId, queue: impl FnOnce(&mut TestGame)) {
    supported(c.name());
    match c {
        Caster::Omnispell => {
            let oa = t.battlefield(P0, "Omnispell Adept");
            t.lands(P0, "Island", 3);
            t.answer_yes(P0, true);
            t.answer_choose(P0, &[obj(card)]);
            queue(t);
            t.activate(P0, oa, 0, &[]).unwrap();
            t.resolve_all();
        }
        Caster::Oracle => {
            // P0 chooses P1 for the tribute; P1 doesn't pay it.
            t.answer_choose(P0, &[pl(P1)]);
            t.answer(P1, DecisionKind::YesNo, Answer::Bool(false));
            t.answer_yes(P0, true);
            t.answer_choose(P0, &[obj(card)]);
            queue(t);
            t.enter(P0, "Oracle of Bones");
            t.resolve_all();
        }
        Caster::Maelstrom => {
            let ma = t.battlefield(P0, "Maelstrom Archangel");
            t.answer_yes(P0, true);
            t.answer_choose(P0, &[obj(card)]);
            queue(t);
            attack_p1_unblocked(t, ma);
        }
    }
}

/// P1's life lost to the caster itself (Maelstrom Archangel's 5 combat damage).
fn base_loss(c: Caster) -> i32 {
    if c == Caster::Maelstrom {
        5
    } else {
        0
    }
}

/// Whether `p` was asked how to cast a spell since `from`.
fn casting_way_asked(t: &TestGame, p: PlayerId, from: usize) -> bool {
    asked_since(t, from).iter().any(|(q, d)| {
        *q == p
            && (matches!(d, Decision::ChooseCastingMethod { .. })
                || matches!(d, Decision::ChooseOption { prompt, .. } if prompt.contains("how to cast")))
    })
}

#[test]
fn free_casts_pay_additional_costs_but_not_alternative_costs() {
    cr!("118.9", "118.9a", "601.2b", "601.2f", "702.33a");
    ruling!(
        "Maelstrom Archangel",
        "If you cast a card “without paying its mana cost,” you can't choose to cast it for any alternative costs, such as evoke or the alternative cost provided by the morph ability. However, you can pay optional additional costs, such as conspire, and you must still pay mandatory additional costs, such as the one on Goldmeadow Stalwart."
    );
    ruling!(
        "Oracle of Bones",
        "If you cast a card “without paying its mana cost,” you can’t pay alternative costs such as overload costs. You can, however, pay additional costs such as kicker costs. If the card has mandatory additional costs, you must pay those."
    );
    ruling!(
        "Omnispell Adept",
        "If you cast a spell “without paying its mana cost,” you can’t choose to cast it for any alternative costs. You can, however, pay additional costs, such as kicker costs. If the card has any mandatory additional costs, such as that of Severed Strands, those must be paid to cast the card."
    );
    for c in CASTERS {
        // Burst Lightning's kicker {4} (an optional additional cost) may be paid.
        let mut t = TestGame::new(2);
        let burst = t.hand(P0, "Burst Lightning");
        t.lands(P0, "Mountain", 4);
        free_cast(&mut t, c, burst, |t| {
            t.answer(P0, DecisionKind::OptionalCost, Answer::Bool(true));
            t.answer_targets(P0, &[pl(P1)]);
        });
        assert_eq!(t.life(P1), 16 - base_loss(c), "{c:?}");
        // Village Rites' sacrifice (a mandatory additional cost) is paid.
        let mut t = TestGame::new(2);
        let rites = t.hand(P0, "Village Rites");
        let bears = t.battlefield(P0, "Grizzly Bears");
        free_cast(&mut t, c, rites, |t| {
            t.answer_choose(P0, &[obj(bears)]);
        });
        assert!(t.in_graveyard(P0, "Grizzly Bears"), "{c:?}");
        assert!(t.in_graveyard(P0, "Village Rites"), "{c:?}");
        // Invigorate's alternative cost (an opponent gains 3 life) isn't offered or paid.
        let mut t = TestGame::new(2);
        let inv = t.hand(P0, "Invigorate");
        t.battlefield(P0, "Forest");
        let bears = t.battlefield(P0, "Grizzly Bears");
        let from = t.asked().len();
        free_cast(&mut t, c, inv, |t| {
            t.answer_targets(P0, &[obj(bears)]);
        });
        assert_eq!(t.pt(bears), (6, 6), "{c:?}");
        assert_eq!(t.life(P1), 20 - base_loss(c), "{c:?}");
        assert!(!casting_way_asked(&t, P0, from), "{c:?}");
    }
}

#[test]
fn maelstrom_archangels_free_spell_is_cast_and_can_be_countered() {
    cr!("601.2i", "701.6a");
    ruling!(
        "Maelstrom Archangel",
        "If you cast a card this way, you're casting it as a spell. It can be countered."
    );
    // P1's Nullstone Gargoyle counters the first noncreature spell of the turn.
    let mut t = TestGame::new(2);
    t.battlefield(P1, "Nullstone Gargoyle");
    let div = t.hand(P0, "Divination");
    free_cast(&mut t, Caster::Maelstrom, div, |_| {});
    assert_eq!(t.zone(div), Zone::Graveyard(P0));
    assert_eq!(t.hand_size(P0), 0, "countered: no cards drawn");
}

#[test]
fn maelstrom_archangels_free_spell_has_x_0() {
    cr!("107.3b");
    ruling!(
        "Maelstrom Archangel",
        "If you cast a card with X in its cost this way, X must be 0."
    );
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P1, "Hill Giant");
    let blaze = t.hand(P0, "Blaze");
    t.lands(P0, "Mountain", 6);
    free_cast(&mut t, Caster::Maelstrom, blaze, |t| {
        t.answer(P0, DecisionKind::X, Answer::Number(5));
        t.answer_targets(P0, &[obj(giant)]);
    });
    assert!(t.in_graveyard(P0, "Blaze"));
    assert_eq!(t.obj_now(giant).damage, 0);
    assert_eq!(tapped_lands(&t, P0), 0);
}

#[test]
fn oracle_of_bones_may_cast_both_halves_of_a_fuse_card() {
    cr!("702.102a", "702.102c", "118.9");
    ruling!(
        "Oracle of Bones",
        "If you choose to cast a split card with fuse without paying its mana cost, you may cast both halves."
    );
    let mut t = TestGame::new(2);
    let wt = t.hand(P0, "Wear // Tear");
    let thopter = t.battlefield(P1, "Ornithopter");
    let anthem = t.battlefield(P1, "Glorious Anthem");
    let from = t.asked().len();
    free_cast(&mut t, Caster::Oracle, wt, |t| {
        t.answer(P0, DecisionKind::Option, Answer::Index(2));
        t.answer_targets(P0, &[obj(thopter)]);
        t.answer_targets(P0, &[obj(anthem)]);
    });
    let options: Vec<String> = asked_since(&t, from)
        .into_iter()
        .map(|(p, d)| format!("{p:?} {d:?}").chars().take(200).collect())
        .collect();
    assert!(!t.on_battlefield(thopter), "{options:?}");
    assert!(!t.on_battlefield(anthem), "{options:?}");
    assert_eq!(tapped_lands(&t, P0), 0);
}

#[test]
fn free_casts_ignore_type_timing_but_not_other_restrictions() {
    cr!("608.2g", "307.1", "601.3");
    ruling!(
        "Maelstrom Archangel",
        "If you want to cast a card this way, you cast it as part of the resolution of Maelstrom Angel's triggered ability. Timing restrictions based on the card's type (such as creature or sorcery) are ignored. Other casting restrictions are not (such as “Cast [this card] only before attackers are declared”)."
    );
    ruling!(
        "Oracle of Bones",
        "If you want to cast a card this way, you cast it as part of the resolution of the triggered ability. Timing restrictions based on the card’s type (such as sorcery) are ignored. Other casting restrictions (such as “Cast [this card] only before attackers are declared”) are not."
    );
    ruling!(
        "Omnispell Adept",
        "You may cast a sorcery this way even if it’s not your turn."
    );
    // A sorcery mid-combat (Maelstrom Archangel) or on P1's turn (Omnispell Adept).
    let mut t = TestGame::new(2);
    let div = t.hand(P0, "Divination");
    free_cast(&mut t, Caster::Maelstrom, div, |_| {});
    assert!(t.in_graveyard(P0, "Divination"));
    assert_eq!(t.hand_size(P0), 2);
    let mut t = TestGame::new(2);
    t.set_step(P1, Step::PrecombatMain);
    let div = t.hand(P0, "Divination");
    free_cast(&mut t, Caster::Omnispell, div, |_| {});
    assert!(t.in_graveyard(P0, "Divination"));
    assert_eq!(t.hand_size(P0), 2);
    // A creature card mid-combat.
    let mut t = TestGame::new(2);
    let bears = t.hand(P0, "Grizzly Bears");
    free_cast(&mut t, Caster::Maelstrom, bears, |_| {});
    assert!(t.on_battlefield(bears));
    // Gorilla War Cry ("Cast this spell only during combat before blockers are declared"):
    // not in the combat damage step, not in a main phase.
    for c in [Caster::Maelstrom, Caster::Oracle] {
        let mut t = TestGame::new(2);
        let cry = t.hand(P0, "Gorilla War Cry");
        free_cast(&mut t, c, cry, |_| {});
        assert_eq!(t.zone(cry), Zone::Hand(P0), "{c:?}");
    }
}

/// P0's Djinn of Wishes activates with `top` on top of P0's library; `then` queues the
/// answers for playing it. Everything resolves.
fn wish(t: &mut TestGame, top: &str, then: impl FnOnce(&mut TestGame)) -> ObjectId {
    supported("Djinn of Wishes");
    let djinn = t.battlefield(P0, "Djinn of Wishes");
    t.g.add_counters(obj(djinn), "wish", 3, None);
    let card = t.library_top(P0, top);
    t.lands(P0, "Island", 4);
    then(t);
    t.activate(P0, djinn, 0, &[]).unwrap();
    t.resolve_all();
    card
}

#[test]
fn djinn_of_wishes_plays_the_card_only_as_its_ability_resolves() {
    cr!("608.2g", "307.1", "601.3");
    ruling!(
        "Djinn of Wishes",
        "If you wish to play the exiled card, you must play it while the last ability of Djinn of Wishes is resolving. You can't play it later in the turn. A spell cast this way may be cast at a time you normally wouldn't be able to cast a spell of that type, but other restrictions (such as \"Cast this spell only during combat\") are enforced."
    );
    // A sorcery on P1's turn.
    let mut t = TestGame::new(2);
    t.set_step(P1, Step::PrecombatMain);
    wish(&mut t, "Divination", |t| {
        t.answer_yes(P0, true);
    });
    assert!(t.in_graveyard(P0, "Divination"));
    assert_eq!(t.hand_size(P0), 2);
    // Gorilla War Cry in a main phase can't be cast: it's exiled.
    let mut t = TestGame::new(2);
    let cry = wish(&mut t, "Gorilla War Cry", |t| {
        t.answer_yes(P0, true);
    });
    assert_eq!(t.zone(cry), Zone::Exile);
    // Not played as the ability resolves: exiled, and it can't be played later.
    let mut t = TestGame::new(2);
    let bolt = wish(&mut t, "Lightning Bolt", |t| {
        t.answer_yes(P0, false);
    });
    assert_eq!(t.zone(bolt), Zone::Exile);
    let bolt = t.g.current(bolt);
    t.lands(P0, "Mountain", 1);
    assert!(crate::r_s08_common::legal_cast_methods(&mut t, P0, bolt).is_empty());
}
