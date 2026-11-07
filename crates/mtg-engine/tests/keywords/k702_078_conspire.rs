//! CR 702.78 Conspire.

use crate::common_k702_011_017::assert_supported;
use crate::common_k702_027_037::optional_costs_offered;
use crate::common_k702_038_051::{spell_copies_on_stack, triggers_named};
use mtg_engine::decision::Answer;
use mtg_engine::keywords::KeywordKind;
use mtg_engine::testing::*;
use mtg_engine::*;

fn pay_conspire(t: &mut TestGame, p: PlayerId, tap: &[ObjectId]) {
    t.answer(p, DecisionKind::OptionalCost, Answer::Bool(true));
    let es: Vec<Entity> = tap.iter().map(|o| Entity::Object(*o)).collect();
    t.answer_choose(p, &es);
}

#[test]
fn conspire_taps_two_creatures_sharing_a_color_to_copy_the_spell() {
    cr!("702.78", "702.78a");
    assert_supported("Burn Trail");
    let mut t = TestGame::new(2);
    let g1 = t.battlefield(P0, "Raging Goblin");
    let g2 = t.battlefield(P0, "Raging Goblin");
    t.lands(P0, "Mountain", 4);
    let trail = t.hand(P0, "Burn Trail");
    pay_conspire(&mut t, P0, &[g1, g2]);
    t.cast(P0, trail).target(P1).go();
    assert!(t.obj_now(g1).tapped && t.obj_now(g2).tapped);
    t.settle();
    assert_eq!(triggers_named(&t, "Conspire").len(), 1);
    t.resolve();
    assert_eq!(spell_copies_on_stack(&t, "Burn Trail"), 1);
    t.resolve_all();
    assert_eq!(t.life(P1), 14);
    // The copy wasn't cast.
    assert_eq!(t.g.history.spells_cast.len(), 1);
}

#[test]
fn without_paying_the_conspire_cost_nothing_is_copied() {
    cr!("702.78a");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Raging Goblin");
    t.battlefield(P0, "Raging Goblin");
    t.lands(P0, "Mountain", 4);
    let trail = t.hand(P0, "Burn Trail");
    t.answer(P0, DecisionKind::OptionalCost, Answer::Bool(false));
    t.cast(P0, trail).target(P1).go();
    t.settle();
    assert!(triggers_named(&t, "Conspire").is_empty());
    t.resolve_all();
    assert_eq!(t.life(P1), 17);
}

#[test]
fn the_tapped_creatures_must_each_share_a_color_with_the_spell() {
    cr!("702.78a");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Raging Goblin");
    t.battlefield(P0, "Grizzly Bears");
    t.lands(P0, "Mountain", 4);
    let trail = t.hand(P0, "Burn Trail");
    t.cast(P0, trail).target(P1).go();
    // A red spell: only one red creature, so conspire can't be paid.
    assert!(optional_costs_offered(&t, P0).is_empty());
    // A hybrid green-or-white spell: a green and a white creature each share a color.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let lions = t.battlefield(P0, "Savannah Lions");
    t.lands(P0, "Forest", 1);
    let blessing = t.hand(P0, "Barkshell Blessing");
    pay_conspire(&mut t, P0, &[bears, lions]);
    t.cast(P0, blessing).target(bears).go();
    t.resolve_all();
    assert_eq!(t.pt(bears), (6, 6));
}

#[test]
fn creatures_with_summoning_sickness_can_be_tapped_for_conspire() {
    cr!("702.78a");
    let mut t = TestGame::new(2);
    let g1 = t.battlefield_sick(P0, "Grizzly Bears");
    let g2 = t.battlefield_sick(P0, "Grizzly Bears");
    t.lands(P0, "Forest", 1);
    let blessing = t.hand(P0, "Barkshell Blessing");
    pay_conspire(&mut t, P0, &[g1, g2]);
    t.cast(P0, blessing).target(g1).go();
    t.resolve_all();
    assert_eq!(t.pt(g1), (6, 6));
}

#[test]
fn the_copy_may_have_a_new_target() {
    cr!("702.78a");
    let mut t = TestGame::new(2);
    let g1 = t.battlefield(P0, "Raging Goblin");
    let g2 = t.battlefield(P0, "Raging Goblin");
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.lands(P0, "Mountain", 4);
    let trail = t.hand(P0, "Burn Trail");
    pay_conspire(&mut t, P0, &[g1, g2]);
    t.cast(P0, trail).target(P1).go();
    t.answer_yes(P0, true);
    t.answer_targets(P0, &[Entity::Object(bears)]);
    t.resolve_all();
    assert_eq!(t.life(P1), 17);
    assert!(!t.on_battlefield(bears));
}

#[test]
fn each_instance_of_conspire_is_paid_separately_and_triggers_on_its_own_payment() {
    cr!("702.78b");
    ruling!(
        "Wort, the Raidmother",
        "Each conspire ability triggers only if you tap two creatures specifically for that ability."
    );
    assert_supported("Wort, the Raidmother");
    // Wort gives Burn Trail a second instance of conspire.
    for (paid, copies) in [(0usize, 0usize), (1, 1), (2, 2)] {
        let mut t = TestGame::new(2);
        t.battlefield(P0, "Wort, the Raidmother");
        let goblins: Vec<ObjectId> = (0..4)
            .map(|_| t.battlefield(P0, "Raging Goblin"))
            .collect();
        t.lands(P0, "Mountain", 4);
        let trail = t.hand(P0, "Burn Trail");
        for i in 0..2 {
            if i < paid {
                pay_conspire(&mut t, P0, &goblins[2 * i..2 * i + 2]);
            } else {
                t.answer(P0, DecisionKind::OptionalCost, Answer::Bool(false));
            }
        }
        let spell = t.cast(P0, trail).target(P1).go();
        assert_eq!(
            t.g.obj(spell).chars.keyword_count(KeywordKind::Conspire),
            2
        );
        assert_eq!(optional_costs_offered(&t, P0).len(), 2);
        t.settle();
        assert_eq!(triggers_named(&t, "Conspire").len(), copies);
        t.resolve_all();
        assert_eq!(t.life(P1), 20 - 3 * (1 + copies as i32));
        assert_eq!(
            goblins.iter().filter(|g| t.obj_now(**g).tapped).count(),
            2 * paid
        );
    }
}

#[test]
fn conspire_can_be_paid_for_a_spell_cast_with_flashback() {
    cr!("702.78a");
    ruling!(
        "Wort, the Raidmother",
        "You may pay additional costs, such as conspire."
    );
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Wort, the Raidmother");
    let g1 = t.battlefield(P0, "Raging Goblin");
    let g2 = t.battlefield(P0, "Raging Goblin");
    t.lands(P0, "Mountain", 5);
    // Firebolt: 2 damage to any target; flashback {4}{R}.
    let bolt = t.graveyard(P0, "Firebolt");
    pay_conspire(&mut t, P0, &[g1, g2]);
    t.cast(P0, bolt)
        .method(mtg_engine::object::CastMethod::Keyword(KeywordKind::Flashback))
        .target(P1)
        .go();
    t.resolve_all();
    assert_eq!(t.life(P1), 16);
    assert!(t.in_exile("Firebolt"));
}

#[test]
fn conspire_given_to_spells_cast_from_exile() {
    cr!("702.78a");
    // Rassilon's "Each noncreature spell you cast from exile has conspire."
    let def = crate::common_k702_011_017::custom_card(
        "Exile Schemer",
        "Creature — Human",
        Some((2, 2)),
        "Each noncreature spell you cast from exile has conspire.",
    );
    let mut t = TestGame::new(2);
    t.custom(P0, def, mtg_engine::object::Zone::Battlefield);
    let g1 = t.battlefield(P0, "Raging Goblin");
    let g2 = t.battlefield(P0, "Raging Goblin");
    t.lands(P0, "Mountain", 2);
    // From the hand: no conspire.
    let from_hand = t.hand(P0, "Lightning Bolt");
    let spell = t.cast(P0, from_hand).target(P1).go();
    assert_eq!(t.g.obj(spell).chars.keyword_count(KeywordKind::Conspire), 0);
    t.resolve_all();
    // From exile (with a permission to cast it): conspire.
    let exiled = t.exile(P0, "Lightning Bolt");
    crate::common_k702_052_066::run_effect(
        &mut t,
        None,
        P0,
        mtg_engine::ability::Effect::GrantPlayPermission {
            who: mtg_engine::ability::PlayerRef::You,
            what: mtg_engine::ability::Sel::Target(0),
            duration: mtg_engine::ability::Duration::EndOfTurn,
            free: false,
        },
        &[Entity::Object(exiled)],
    );
    pay_conspire(&mut t, P0, &[g1, g2]);
    let spell = t.cast(P0, exiled).target(P1).go();
    assert_eq!(t.g.obj(spell).chars.keyword_count(KeywordKind::Conspire), 1);
    t.resolve_all();
    // 3 from the first Bolt, 3 + 3 from the second and its copy.
    assert_eq!(t.life(P1), 11);
}
