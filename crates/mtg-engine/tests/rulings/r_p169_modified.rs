//! Rulings batch P169 — modified creatures (CR 700.9) and the Adventure distributions of
//! Ondu Knotmaster and Elusive Otter (CR 715, 601.2d, 608.2b).

use crate::r_p169_common::*;
use mtg_engine::decision::{Action, Answer, Decision};
use mtg_engine::keywords::KeywordKind;
use mtg_engine::mana::ManaType;
use mtg_engine::object::{CastMethod, Zone};
use mtg_engine::testing::*;
use mtg_engine::types::*;
use mtg_engine::*;

/// P1's Bonesplitter attached to the creature `to` (controlled by P0).
fn opponents_equipment_on(t: &mut TestGame, to: ObjectId) -> ObjectId {
    let eq = t.battlefield(P1, "Bonesplitter");
    assert!(t.g.attach(eq, obj(to)));
    t.g.recompute();
    t.g.flush_events();
    assert_eq!(t.obj_now(eq).controller, P1);
    eq
}

#[test]
fn a_creature_equipped_by_an_opponents_equipment_is_modified() {
    cr!("700.9", "301.5c");
    ruling!(
        "Upriser Renegade",
        "A creature that is equipped is considered modified no matter who controls the Equipment that's attached to it."
    );
    ruling!(
        "Obstinate Gargoyle",
        "A creature that is equipped is considered modified no matter who controls the Equipment that's attached to it."
    );
    ruling!(
        "Envoy of the Ancestors",
        "A creature that is equipped is considered modified no matter who controls the Equipment that's attached to it."
    );
    ruling!(
        "Lion Umbra",
        "A creature that is equipped is considered modified no matter who controls the Equipment that's attached to it."
    );
    ruling!(
        "Kami's Flare",
        "A creature that is equipped is considered modified no matter who controls the Equipment that's attached to it."
    );
    // "for each other modified creature you control".
    supported("Upriser Renegade");
    let mut t = TestGame::new(2);
    let upriser = t.battlefield(P0, "Upriser Renegade");
    let bears = t.battlefield(P0, "Grizzly Bears");
    assert_eq!(t.pt(upriser), (1, 3));
    opponents_equipment_on(&mut t, bears);
    assert_eq!(t.pt(upriser), (3, 3));
    assert_eq!(t.pt(bears), (4, 2));

    // "as long as it's modified".
    supported("Obstinate Gargoyle");
    let mut t = TestGame::new(2);
    let garg = t.battlefield(P0, "Obstinate Gargoyle");
    assert!(!has_kw(&mut t, garg, KeywordKind::Flying));
    opponents_equipment_on(&mut t, garg);
    assert!(has_kw(&mut t, garg, KeywordKind::Flying));

    // "Modified creatures you control have lifelink".
    supported("Envoy of the Ancestors");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Envoy of the Ancestors");
    let bears = t.battlefield(P0, "Grizzly Bears");
    assert!(!has_kw(&mut t, bears, KeywordKind::Lifelink));
    opponents_equipment_on(&mut t, bears);
    assert!(has_kw(&mut t, bears, KeywordKind::Lifelink));

    // "Enchant modified creature": a legal target only once equipped.
    supported("Lion Umbra");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let umbra = t.hand(P0, "Lion Umbra");
    t.lands(P0, "Forest", 2);
    assert!(!castable(&mut t, P0, umbra));
    opponents_equipment_on(&mut t, bears);
    assert!(castable(&mut t, P0, umbra));
    t.cast(P0, umbra).target(bears).go();
    t.resolve_all();
    assert_eq!(t.pt(bears), (7, 5));

    // "if you control a modified creature".
    supported("Kami's Flare");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let giant = t.battlefield(P1, "Hill Giant");
    opponents_equipment_on(&mut t, bears);
    cast_new(&mut t, P0, "Kami's Flare", &[obj(giant)]);
    t.resolve_all();
    assert!(t.in_graveyard(P1, "Hill Giant"));
    assert_eq!(t.life(P1), 18);
}

#[test]
fn ondu_knotmaster_is_a_creature_card_in_the_graveyard() {
    cr!("715.4", "115.1");
    ruling!(
        "Ondu Knotmaster // Throw a Line",
        "An adventurer card is a permanent card in every zone except the stack, as well as while on the stack if not cast as an Adventure. Ignore its alternative characteristics in those cases. For example, while it's in your graveyard, Ondu Knotmaster is a white and black creature card whose mana value is 4. It can't be the target of Pinnacle Monk's last ability (\"When Pinnacle Monk enters the battlefield, return target instant or sorcery card from your graveyard to your hand\")."
    );
    supported("Ondu Knotmaster // Throw a Line");
    supported("Pinnacle Monk // Mystic Peak");
    let mut t = TestGame::new(2);
    let knot = t.graveyard(P0, "Ondu Knotmaster // Throw a Line");
    let o = t.obj_now(knot).chars.clone();
    assert!(o.card_types.contains(CardType::Creature));
    assert!(!o.card_types.contains(CardType::Sorcery));
    assert_eq!(
        o.colors,
        ColorSet::single(Color::White).union(ColorSet::single(Color::Black))
    );
    assert_eq!(o.mana_value(), 4);
    let opt = t.graveyard(P0, "Opt");
    let from = t.asked().len();
    t.enter(P0, "Pinnacle Monk // Mystic Peak");
    t.settle();
    let offered = crate::r_s02_common::target_candidates(&t, P0, from);
    assert_eq!(offered, vec![vec![obj(opt)]]);
}

#[test]
fn eladamri_can_cast_ondu_knotmaster_but_not_throw_a_line_from_the_top() {
    cr!("715.3a", "601.3e");
    ruling!(
        "Ondu Knotmaster // Throw a Line",
        "If you cast an adventurer card as an Adventure, use only its alternative characteristics to determine whether it's legal to cast that spell. For example, if you control Eladamri, Korvecdal (\"You may cast creature spells from the top of your library\") and Ondu Knotmaster is on top of your library, you can cast Ondu Knotmaster, but not Throw a Line."
    );
    supported("Eladamri, Korvecdal");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Eladamri, Korvecdal");
    t.battlefield(P0, "Grizzly Bears");
    t.lands(P0, "Plains", 2);
    t.lands(P0, "Swamp", 2);
    let knot = t.library_top(P0, "Ondu Knotmaster // Throw a Line");
    t.g.turn.priority = Some(P0);
    t.g.recompute();
    let methods: Vec<CastMethod> = t
        .g
        .legal_actions(P0)
        .into_iter()
        .filter_map(|a| match a {
            Action::Cast { card, method } if card == knot => Some(method),
            _ => None,
        })
        .collect();
    assert_eq!(methods, vec![CastMethod::Normal]);
    // Positive control: from the hand, Throw a Line can be cast.
    let in_hand = t.hand(P0, "Ondu Knotmaster // Throw a Line");
    let bears = t.named_on_battlefield("Grizzly Bears")[0];
    t.answer_targets(P0, &[obj(bears)]);
    t.cast(P0, in_hand).method(CastMethod::Half(1)).go();
}

/// The Divide decisions asked of P0 since decision `from`: (total, recipients, min).
fn divides_since(t: &TestGame, from: usize) -> Vec<(u32, usize, u32)> {
    t.asked()[from..]
        .iter()
        .filter_map(|(_, d)| match d {
            Decision::Divide {
                total,
                recipients,
                min_each,
                ..
            } => Some((*total, recipients.len(), *min_each)),
            _ => None,
        })
        .collect()
}

#[test]
fn throw_a_line_divides_as_cast_and_an_illegal_target_loses_its_counter() {
    cr!("601.2d", "608.2b", "115.7");
    ruling!(
        "Ondu Knotmaster // Throw a Line",
        "You choose how the counters will be distributed as you cast Throw a Line. Each target must receive at least one +1/+1 counter."
    );
    ruling!(
        "Ondu Knotmaster // Throw a Line",
        "If you choose two targets for Throw a Line, and one of the creatures is an illegal target as Throw a Line tries to resolve, the original distribution of counters still applies and the counter that would have been put on the illegal target are lost."
    );
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let giant = t.battlefield(P1, "Hill Giant");
    mana(&mut t, P0, ManaType::W, 1);
    mana(&mut t, P0, ManaType::B, 1);
    let card = t.hand(P0, "Ondu Knotmaster // Throw a Line");
    let from = t.asked().len();
    t.answer_targets(P0, &[obj(bears), obj(giant)]);
    // Asking for 2 and 0 is illegal (each target must get one): the default split is
    // used.
    t.answer(P0, DecisionKind::Divide, Answer::Numbers(vec![2, 0]));
    t.cast(P0, card).method(CastMethod::Half(1)).go();
    // The division was chosen while casting, before anything resolved.
    assert_eq!(divides_since(&t, from), vec![(2, 2, 1)]);
    // The Giant becomes an illegal target.
    destroy(&mut t, giant);
    t.resolve_all();
    assert_eq!(t.counters(bears, counters::PLUS1), 1);
    assert_eq!(t.zone(card), Zone::Exile);

    // With one target, it gets both.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    mana(&mut t, P0, ManaType::W, 1);
    mana(&mut t, P0, ManaType::B, 1);
    let card = t.hand(P0, "Ondu Knotmaster // Throw a Line");
    t.answer_targets(P0, &[obj(bears)]);
    t.cast(P0, card).method(CastMethod::Half(1)).go();
    t.resolve_all();
    assert_eq!(t.counters(bears, counters::PLUS1), 2);
}

#[test]
fn groves_bounty_keeps_the_original_distribution_when_a_target_is_illegal() {
    cr!("601.2d", "608.2b");
    ruling!(
        "Elusive Otter // Grove's Bounty",
        "If some of the creatures are illegal targets as Grove's Bounty tries to resolve, the original distribution of counters still applies and the counters that would have been put on illegal targets are lost."
    );
    supported("Elusive Otter // Grove's Bounty");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let giant = t.battlefield(P0, "Hill Giant");
    mana(&mut t, P0, ManaType::G, 4);
    let card = t.hand(P0, "Elusive Otter // Grove's Bounty");
    t.answer_targets(P0, &[obj(bears), obj(giant)]);
    t.answer(P0, DecisionKind::Divide, Answer::Numbers(vec![1, 2]));
    t.cast(P0, card).method(CastMethod::Half(1)).x(3).go();
    // The Giant (to get two) leaves the battlefield.
    let g = t.g.current(giant);
    t.g.move_object(g, Zone::Hand(P0), mtg_engine::events::MoveCause::Effect, None);
    t.g.flush_events();
    t.resolve_all();
    assert_eq!(t.counters(bears, counters::PLUS1), 1);
    assert_eq!(t.pt(bears), (3, 3));
}
