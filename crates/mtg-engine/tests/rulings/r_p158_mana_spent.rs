//! Rulings batch P158 — what was spent to cast a spell: a spell cast "without paying its
//! mana cost" can't be paid for instead (CR 118.9a, 601.2f), cost reductions can't be
//! waived, "as though it were mana of any color" doesn't change what mana was spent
//! (CR 609.4b), mana value ignores alternative costs (CR 202.3), and cost reductions
//! that reduce only generic mana (CR 601.2f) and apply once per spell.

use crate::r_p158_common::*;
use mtg_engine::decision::{Answer, Decision};
use mtg_engine::mana::ManaType;
use mtg_engine::object::CastMethod;
use mtg_engine::testing::*;
use mtg_engine::types::*;
use mtg_engine::*;

/// Whether `p` can cast `card` by `method` now.
fn can_cast(t: &mut TestGame, p: PlayerId, card: ObjectId, method: CastMethod) -> bool {
    t.g.turn.priority = Some(p);
    t.g.legal_actions(p)
        .contains(&Action::Cast { card, method })
}

/// Whether `p` was offered a choice of casting method since decision `from`.
fn method_asked(t: &TestGame, p: PlayerId, from: usize) -> bool {
    asked_since(t, from)
        .iter()
        .any(|(q, d)| *q == p && matches!(d, Decision::ChooseCastingMethod { .. }))
}

/// P0 resolves Electrodominance with X = `x` at P1 and casts `card` from their hand by
/// its permission, with `pool_red` red mana (plus whatever the pool already held) left in
/// their pool; `then` queues the answers for the free cast.
fn electrodominance(t: &mut TestGame, x: u32, card: ObjectId, pool_red: u32, then: impl FnOnce(&mut TestGame)) {
    supported("Electrodominance");
    let e = t.hand(P0, "Electrodominance");
    let held = t.g.players[0].mana_pool.total();
    mana(t, P0, ManaType::R, x + 2 + pool_red);
    t.answer(P0, DecisionKind::X, Answer::Number(x as i64));
    t.answer_targets(P0, &[pl(P1)]);
    t.answer_choose(P0, &[obj(card)]);
    then(t);
    t.cast(P0, e).go();
    assert_eq!(t.g.players[0].mana_pool.total(), held + pool_red as usize);
    t.resolve_all();
}

const FIGHT: &str = "fights up to one target creature";

#[test]
fn cast_without_paying_its_mana_cost_cant_be_paid_for_instead() {
    cr!("118.9a", "601.2f", "601.2g");
    ruling!(
        "Sundering Stroke",
        "If an effect allows you to cast Sundering Stroke without paying its mana cost, you can't choose to cast it and pay its cost unless another rule or effect allows you to cast it for a cost."
    );
    ruling!(
        "Mythos of Illuna",
        "If an effect allows you to cast a spell without paying its mana cost, you can't choose to cast it and pay unless another rule or effect allows you to cast that spell for a cost. Similarly, you can't waive a cost reduction unless that effect says you may. "
    );
    supported("Sundering Stroke");
    supported("Mythos of Illuna");
    // Sundering Stroke cast by Electrodominance's permission with seven red mana in the
    // pool: it's cast without paying, no mana is spent, so it deals 7 damage divided
    // (4 to P1 and 3 to P2), not 7 to each.
    let mut t = TestGame::new(3);
    let stroke = t.hand(P0, "Sundering Stroke");
    let from = t.asked().len();
    electrodominance(&mut t, 7, stroke, 7, |t| {
        divide(t, P0, &[4, 3]);
        t.answer_targets(P0, &[pl(P1), pl(P2)]);
    });
    assert!(!method_asked(&t, P0, from), "no choice of paying");
    assert_eq!(t.g.players[0].mana_pool.total(), 7, "no mana was spent");
    assert_eq!((t.life(P1), t.life(P2)), (20 - 7 - 4, 17));
    // Mythos of Illuna cast that way with {R}{G} in the pool: the token has no fight
    // ability.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let mythos = t.hand(P0, "Mythos of Illuna");
    let before = t.g.battlefield.clone();
    mana(&mut t, P0, ManaType::G, 1);
    electrodominance(&mut t, 4, mythos, 1, |t| {
        t.answer_targets(P0, &[obj(bears)]);
    });
    assert_eq!(t.g.players[0].mana_pool.total(), 2, "no mana was spent");
    let toks = new_tokens(&t, P0, &before);
    assert_eq!(toks.len(), 1);
    assert!(!has_ability(&t, toks[0], FIGHT));
    // A cost reduction can't be waived: under Sapphire Medallion Mythos costs {1}{U}{U};
    // with {U}{U}{R}{G} in the pool only three mana are spent, so {R}{G} wasn't.
    supported("Sapphire Medallion");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Sapphire Medallion");
    let bears = t.battlefield(P1, "Grizzly Bears");
    let before = t.g.battlefield.clone();
    mana(&mut t, P0, ManaType::U, 2);
    mana(&mut t, P0, ManaType::R, 1);
    mana(&mut t, P0, ManaType::G, 1);
    let mythos = t.hand(P0, "Mythos of Illuna");
    t.cast(P0, mythos).target(obj(bears)).go();
    assert_eq!(t.g.players[0].mana_pool.total(), 1);
    t.resolve_all();
    let toks = new_tokens(&t, P0, &before);
    assert_eq!(toks.len(), 1);
    assert!(!has_ability(&t, toks[0], FIGHT));
}

#[test]
fn spending_mana_as_though_another_color_doesnt_change_what_was_spent() {
    cr!("609.4b", "601.2g");
    ruling!(
        "Sundering Stroke",
        "Sundering Stroke checks what mana was actually spent to cast it. If an effect allows you to spend mana “as though it were mana” of any color or type, that will allow you to spend mana you couldn't otherwise spend, but it won't change what mana you spent to cast the spell."
    );
    ruling!(
        "Mythos of Illuna",
        "The ability checks what mana was actually spent to cast a spell. If an effect allows you to spend mana \"as though it were mana\" of any color or type, that allows you to spend mana you couldn't otherwise spend, but it doesn't change what mana you spent to cast the spell."
    );
    supported("Sunglasses of Urza");
    supported("Chromatic Orrery");
    // Sunglasses of Urza ("You may spend white mana as though it were red mana"): seven
    // white mana cast Sundering Stroke, but seven red mana wasn't spent: damage divided.
    let mut t = TestGame::new(3);
    t.battlefield(P0, "Sunglasses of Urza");
    mana(&mut t, P0, ManaType::W, 7);
    let s = t.hand(P0, "Sundering Stroke");
    assert!(can_cast(&mut t, P0, s, CastMethod::Normal));
    divide(&mut t, P0, &[4, 3]);
    t.cast(P0, s).targets(&[pl(P1), pl(P2)]).go();
    assert_eq!(t.g.players[0].mana_pool.total(), 0);
    t.resolve_all();
    assert_eq!((t.life(P1), t.life(P2)), (16, 17));
    // Without the Sunglasses, white mana can't cast it at all.
    let mut t = TestGame::new(3);
    mana(&mut t, P0, ManaType::W, 7);
    let s = t.hand(P0, "Sundering Stroke");
    assert!(!can_cast(&mut t, P0, s, CastMethod::Normal));
    // Chromatic Orrery ("You may spend mana as though it were mana of any color"): four
    // colorless mana cast Mythos of Illuna, but {R}{G} wasn't spent: no fight ability.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Chromatic Orrery");
    let bears = t.battlefield(P1, "Grizzly Bears");
    let before = t.g.battlefield.clone();
    mana(&mut t, P0, ManaType::C, 4);
    let m = t.hand(P0, "Mythos of Illuna");
    t.cast(P0, m).target(obj(bears)).go();
    assert_eq!(t.g.players[0].mana_pool.total(), 0);
    t.resolve_all();
    let toks = new_tokens(&t, P0, &before);
    assert_eq!(toks.len(), 1);
    assert!(!has_ability(&t, toks[0], FIGHT));
}

#[test]
fn horribly_awry_checks_the_mana_value_not_the_cost_paid() {
    cr!("202.3", "118.9", "115.5");
    ruling!(
        "Horribly Awry",
        "The mana value of a spell doesn't change if its controller paid an alternative cost (such as an evoke cost) to cast it."
    );
    supported("Horribly Awry");
    supported("Mulldrifter");
    // Mulldrifter ({4}{U}, evoke {2}{U}) cast for its evoke cost has mana value 5:
    // Horribly Awry can't target it; a Grizzly Bears spell it can.
    let mut t = TestGame::new(2);
    t.set_step(P1, mtg_engine::turn::Step::PrecombatMain);
    lots(&mut t, P1);
    let m = t.hand(P1, "Mulldrifter");
    let m = t
        .cast(P1, m)
        .method(CastMethod::Keyword(mtg_engine::keywords::KeywordKind::Evoke))
        .go();
    lots(&mut t, P0);
    let h = t.hand(P0, "Horribly Awry");
    assert!(t.cast(P0, h).target(obj(m)).try_go().is_err());
    assert!(t.g.stack.contains(&m));
    let mut t = TestGame::new(2);
    t.set_step(P1, mtg_engine::turn::Step::PrecombatMain);
    let b = cast_at(&mut t, P1, "Grizzly Bears", &[]);
    lots(&mut t, P0);
    let h = t.hand(P0, "Horribly Awry");
    t.cast(P0, h).target(obj(b)).go();
    t.resolve();
    assert!(t.in_exile("Grizzly Bears"));
}

#[test]
fn rowan_and_will_reduce_only_generic_mana() {
    cr!("601.2f");
    ruling!(
        "Rowan, Scholar of Sparks // Will, Scholar of Frost",
        "Neither face’s first ability can reduce colored mana requirements of spells you cast."
    );
    supported("Rowan, Scholar of Sparks // Will, Scholar of Frost");
    let rowan = |t: &mut TestGame| {
        t.battlefield(P0, "Rowan, Scholar of Sparks // Will, Scholar of Frost");
    };
    let will = |t: &mut TestGame| {
        crate::r_p079_common::back_face(t, P0, "Rowan, Scholar of Sparks // Will, Scholar of Frost");
    };
    for face in [&rowan as &dyn Fn(&mut TestGame), &will] {
        // Opt ({U}): still {U}; Divination ({2}{U}): {1}{U}.
        crate::r_p194_common::assert_total_cost(face, P0, "Opt", &[(ManaType::U, 1)], 0);
        crate::r_p194_common::assert_total_cost(face, P0, "Divination", &[(ManaType::U, 1)], 1);
        // Counterspell ({U}{U}): castable for {U}{U}, not for {U} plus one colorless.
        for (blue, castable) in [(2, true), (1, false)] {
            let mut t = TestGame::new(2);
            face(&mut t);
            lots(&mut t, P1);
            let shock = t.hand(P1, "Shock");
            let shock = t.cast(P1, shock).target(pl(P0)).go();
            mana(&mut t, P0, ManaType::U, blue);
            mana(&mut t, P0, ManaType::C, 2 - blue);
            let c = t.hand(P0, "Counterspell");
            assert_eq!(t.cast(P0, c).target(obj(shock)).try_go().is_ok(), castable, "{blue} blue");
            if castable {
                assert_eq!(t.g.players[0].mana_pool.total(), 0);
            }
        }
    }
}

#[test]
fn the_destined_warrior_reduces_generic_mana_once_per_party_spell() {
    cr!("601.2f");
    ruling!(
        "The Destined Warrior",
        "The cost reduction applies only once to any spell you cast that's a Cleric, Rogue, Warrior, or Wizard. A spell with more than one of these subtypes will still have its cost reduced by just {1}."
    );
    ruling!(
        "The Destined Warrior",
        "The cost reduction applies only to generic mana in the total cost of Cleric, Rogue, Warrior, and Wizard spells you cast."
    );
    supported("The Destined Warrior");
    supported("Samut, Hazoret's Champion");
    let warrior = |t: &mut TestGame| {
        t.battlefield(P0, "The Destined Warrior");
    };
    // Samut, Hazoret's Champion ({1}{R}, a Warrior Cleric): {R}, not free.
    crate::r_p194_common::assert_total_cost(&warrior, P0, "Samut, Hazoret's Champion", &[(ManaType::R, 1)], 0);
    // Fugitive Wizard ({U}): still {U}; Cabal Evangel ({1}{B}, a Cleric): {B}.
    crate::r_p194_common::assert_total_cost(&warrior, P0, "Fugitive Wizard", &[(ManaType::U, 1)], 0);
    crate::r_p194_common::assert_total_cost(&warrior, P0, "Cabal Evangel", &[(ManaType::B, 1)], 0);
    // Grizzly Bears ({1}{G}, a Bear): not reduced.
    crate::r_p194_common::assert_total_cost(&warrior, P0, "Grizzly Bears", &[(ManaType::G, 1)], 1);
}

#[test]
fn eowyn_reduces_equip_abilities_but_not_reconfigure() {
    cr!("702.6a", "702.151a", "601.2f");
    ruling!(
        "Éowyn, Lady of Rohan",
        "Some Equipment creature cards in other sets have reconfigure, a different ability that attaches them to a creature. Reconfigure is not an equip ability, and reconfigure costs are not reduced by Éowyn, Lady of Rohan's last ability. Equip variants or equip abilities with restrictions, like the \"equip Human\" ability of Dúnedain Blade, are still equip abilities, and their costs will be reduced by Éowyn, Lady of Rohan's last ability."
    );
    supported("Éowyn, Lady of Rohan");
    supported("Lizard Blades");
    supported("Dúnedain Blade");
    // (equipment, index of the activated ability, mana it costs with Éowyn)
    for (equipment, index, ability, cost) in [
        ("Lizard Blades", 0, "Reconfigure {2}", 2),
        ("Dúnedain Blade", 0, "Equip Human {1}", 0),
        ("Dúnedain Blade", 1, "Equip {3}", 2),
    ] {
        for short in [false, true] {
            if short && cost == 0 {
                continue;
            }
            let mut t = TestGame::new(2);
            let eowyn = t.battlefield(P0, "Éowyn, Lady of Rohan");
            let e = t.battlefield(P0, equipment);
            assert!(has_ability(&t, e, ability), "{equipment}: {ability}");
            let available = if short { cost - 1 } else { cost };
            mana(&mut t, P0, ManaType::C, available);
            let ok = t.activate(P0, e, index, &[obj(eowyn)]).is_ok();
            assert_eq!(ok, !short, "{equipment}: {ability} for {available}");
            if ok {
                assert_eq!(t.g.players[0].mana_pool.total(), 0, "{equipment}: {ability}");
                t.resolve_all();
                assert_eq!(crate::r_s06_common::attached_to(&t, e), Some(obj(eowyn)), "{equipment}");
            }
        }
    }
}

#[test]
fn denials_require_four_however_many_birds_or_flyers() {
    cr!("614.1a", "118.12a");
    ruling!(
        "Dazzling Denial",
        "If you control more than one Bird, the controller of the target spell still only needs to pay {4}."
    );
    ruling!(
        "Lofty Denial",
        "If you control more than one creature with flying, Lofty Denial still requires only {4}."
    );
    supported("Dazzling Denial");
    supported("Lofty Denial");
    for (denial, creature) in [("Dazzling Denial", "Storm Crow"), ("Lofty Denial", "Welkin Tern")] {
        for (available, countered) in [(4u32, false), (3, true)] {
            let mut t = TestGame::new(2);
            t.set_step(P1, mtg_engine::turn::Step::PrecombatMain);
            t.battlefield(P0, creature);
            t.battlefield(P0, creature);
            let bears = cast_at(&mut t, P1, "Grizzly Bears", &[]);
            t.g.players[1].mana_pool.mana.clear();
            mana(&mut t, P1, ManaType::C, available);
            cast_at(&mut t, P0, denial, &[obj(bears)]);
            t.answer_yes(P1, true);
            t.resolve();
            assert_eq!(!t.g.stack.contains(&bears), countered, "{denial} with {available}");
            if !countered {
                assert_eq!(t.g.players[1].mana_pool.total(), 0, "{denial}: paid {available}");
            }
        }
    }
}
