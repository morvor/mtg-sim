//! Rulings batch P046 — spells and abilities whose targets are illegal as they try to
//! resolve (CR 608.2b): with every target illegal they don't resolve and none of their
//! effects happen; a legal target that isn't destroyed (indestructible, CR 702.12b) still
//! gets the rest of the effect; with some targets legal, the rest happens (CR 608.2b).

use crate::r_p046_common::*;
use crate::r_s01_common::*;
use mtg_engine::decision::Answer;
use mtg_engine::mana::ManaType;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::types::*;
use mtg_engine::*;

/// P0 casts `name` targeting `targets` (P1's permanents, set up by the caller); each of
/// `gone` is returned to its owner's hand in response; then everything resolves.
fn cast_then_remove(t: &mut TestGame, name: &str, targets: &[ObjectId], gone: &[ObjectId]) {
    supported(name);
    let s = t.hand(P0, name);
    let es: Vec<Entity> = targets.iter().map(|o| Entity::Object(*o)).collect();
    assert!(cost_to_cast(t, P0, s, &es).is_some(), "{name}");
    for g in gone {
        bounce(t, *g);
    }
    t.resolve_all();
    assert!(t.in_graveyard(P0, name) || t.in_exile(name), "{name}");
}

#[test]
fn with_its_only_target_illegal_none_of_the_spell_happens() {
    cr!("608.2b");
    // Each: P1's artifact or enchantment is targeted, then leaves; then what didn't happen.
    let case = |name: &str| {
        let mut t = TestGame::new(2);
        let relic = t.battlefield(P1, "Darksteel Relic");
        cast_then_remove(&mut t, name, &[relic], &[relic]);
        t
    };
    ruling!(
        "Springsage Ritual",
        "If the artifact or enchantment becomes an illegal target, Springsage Ritual doesn’t resolve. You won’t gain 4 life."
    );
    assert_eq!(case("Springsage Ritual").life(P0), 20);
    ruling!(
        "Invoke the Divine",
        "If the target artifact or enchantment is an illegal target as Invoke the Divine tries to resolve, it won’t resolve and none of its effects will happen. You won’t gain 4 life."
    );
    ruling!(
        "Invoke the Divine",
        "If the target artifact or enchantment is an illegal target by the time Invoke the Divine tries to resolve, the spell doesn’t resolve. You won’t gain any life."
    );
    assert_eq!(case("Invoke the Divine").life(P0), 20);
    ruling!(
        "Solemn Offering",
        "If the targeted permanent is an illegal target by the time Solemn Offering would resolve, the entire spell doesn’t resolve. You won’t gain any life."
    );
    assert_eq!(case("Solemn Offering").life(P0), 20);
    ruling!(
        "True Love's Kiss",
        "If the target artifact or enchantment is an illegal target by the time True Love's Kiss tries to resolve, the spell won't resolve. You won't draw a card"
    );
    assert_eq!(case("True Love's Kiss").hand_size(P0), 0);
    ruling!(
        "Mental Modulation",
        "If the target permanent is an illegal target as Mental Modulation tries to resolve, it won’t resolve and none of its effects will happen. You won’t draw a card."
    );
    assert_eq!(case("Mental Modulation").hand_size(P0), 0);
    ruling!(
        "Broken Bond",
        "If the target artifact or enchantment is an illegal target by the time Broken Bond tries to resolve, the spell doesn’t resolve. You won’t put a land card onto the battlefield."
    );
    let mut t = TestGame::new(2);
    let relic = t.battlefield(P1, "Darksteel Relic");
    let forest = t.hand(P0, "Forest");
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[Entity::Object(forest)]);
    cast_then_remove(&mut t, "Broken Bond", &[relic], &[relic]);
    assert_eq!(t.zone(forest), Zone::Hand(P0));
}

#[test]
fn rime_chill_with_every_target_illegal_doesnt_draw() {
    cr!("608.2b");
    ruling!(
        "Rime Chill",
        "If all of the target creatures are illegal targets as Rime Chill tries to resolve, it won't resolve and none of its effects will happen. You won't draw a card."
    );
    supported("Rime Chill");
    // P0's Rime Chill targets P1's Bears and Hill Giant (one target slot, "up to two").
    let rime_chill = |gone: &[usize]| {
        let mut t = TestGame::new(2);
        let both = [t.battlefield(P1, "Grizzly Bears"), t.battlefield(P1, "Hill Giant")];
        let s = t.hand(P0, "Rime Chill");
        fill_pool(&mut t, P0, 7);
        t.cast(P0, s)
            .targets(&[Entity::Object(both[0]), Entity::Object(both[1])])
            .go();
        for i in gone {
            bounce(&mut t, both[*i]);
        }
        t.resolve_all();
        (t, both[1])
    };
    let (t, _) = rime_chill(&[0, 1]);
    assert_eq!(t.hand_size(P0), 0);
    // One of them still legal: it's tapped and stunned, and P0 draws.
    let (t, b) = rime_chill(&[0]);
    assert_eq!(t.hand_size(P0), 1);
    assert!(t.obj_now(b).tapped);
    assert_eq!(t.counters(b, "stun"), 1);
}

#[test]
fn pest_infestation_with_every_target_illegal_creates_no_pests() {
    cr!("608.2b", "107.3a");
    ruling!(
        "Pest Infestation",
        "If any artifacts or enchantments were chosen as targets, and all of them are illegal targets as Pest Infestation tries to resolve, it won't resolve and none of its effects will happen. No Pests will be created."
    );
    supported("Pest Infestation");
    let pests = |t: &TestGame| with_subtype(t, P0, "Pest").len();
    // X = 2, both targets leave.
    let mut t = TestGame::new(2);
    let a = t.battlefield(P1, "Darksteel Relic");
    let b = t.battlefield(P1, "Glorious Anthem");
    let s = t.hand(P0, "Pest Infestation");
    fill_pool(&mut t, P0, 5);
    t.cast(P0, s).x(2).targets(&[Entity::Object(a), Entity::Object(b)]).go();
    bounce(&mut t, a);
    bounce(&mut t, b);
    t.resolve_all();
    assert_eq!(pests(&t), 0);
    // One stays: four Pests.
    let mut t = TestGame::new(2);
    let a = t.battlefield(P1, "Darksteel Relic");
    let b = t.battlefield(P1, "Glorious Anthem");
    let s = t.hand(P0, "Pest Infestation");
    fill_pool(&mut t, P0, 5);
    t.cast(P0, s).x(2).targets(&[Entity::Object(a), Entity::Object(b)]).go();
    bounce(&mut t, a);
    t.resolve_all();
    assert_eq!(pests(&t), 4);
    assert!(t.in_graveyard(P1, "Glorious Anthem"));
}

#[test]
fn unravel_the_aether_shuffles_only_with_a_legal_target() {
    cr!("608.2b", "701.24a");
    ruling!(
        "Unravel the Aether",
        "If the artifact or enchantment is an illegal target as Unravel the Aether tries to resolve, it won’t resolve and none of its effects will happen. No library will be shuffled."
    );
    supported("Psychogenic Probe");
    // Psychogenic Probe: "Whenever a spell or ability causes a player to shuffle their
    // library, Psychogenic Probe deals 2 damage to that player."
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Psychogenic Probe");
    let anthem = t.battlefield(P1, "Glorious Anthem");
    cast_then_remove(&mut t, "Unravel the Aether", &[anthem], &[anthem]);
    assert_eq!(t.life(P1), 20);
    ruling!(
        "Unravel the Aether",
        "If the artifact or enchantment is a token, it will be shuffled into its owner’s library when Unravel the Aether resolves, then will cease to exist."
    );
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Psychogenic Probe");
    let spec = mtg_engine::tokens::predefined("Treasure").expect("Treasure");
    let treasure = crate::r_p023_common::token_of(&mut t, P1, spec);
    let lib = t.library_size(P1);
    cast_then_remove(&mut t, "Unravel the Aether", &[treasure], &[]);
    assert!(!t.on_battlefield(treasure));
    assert_eq!(t.library_size(P1), lib);
    assert_eq!(t.life(P1), 18);
}

/// P1's indestructible Darksteel Relic is targeted by P0's `name` and isn't destroyed.
fn indestructible_target(name: &str) -> TestGame {
    let mut t = TestGame::new(2);
    let relic = t.battlefield(P1, "Darksteel Relic");
    cast_then_remove(&mut t, name, &[relic], &[]);
    assert!(t.on_battlefield(relic), "{name}");
    t
}

#[test]
fn a_legal_target_that_isnt_destroyed_still_gets_the_rest() {
    cr!("608.2b", "702.12b");
    ruling!(
        "Natural End",
        "If the target is legal but not destroyed (most likely because it has indestructible), you do gain life."
    );
    assert_eq!(indestructible_target("Natural End").life(P0), 23);
    ruling!(
        "Natural End",
        "If the target artifact or enchantment is an illegal target by the time Natural End tries to resolve, the spell doesn’t resolve. You don’t gain life."
    );
    let mut t = TestGame::new(2);
    let relic = t.battlefield(P1, "Darksteel Relic");
    cast_then_remove(&mut t, "Natural End", &[relic], &[relic]);
    assert_eq!(t.life(P0), 20);
    ruling!(
        "Nature's Claim",
        "If the target is legal but not destroyed (most likely because it has indestructible), its controller does gain 4 life."
    );
    assert_eq!(indestructible_target("Nature's Claim").life(P1), 24);
    ruling!(
        "Nature's Claim",
        "If the target artifact or enchantment is an illegal target by the time Nature's Claim tries to resolve, the spell doesn't resolve. No player gains 4 life."
    );
    let mut t = TestGame::new(2);
    let relic = t.battlefield(P1, "Darksteel Relic");
    cast_then_remove(&mut t, "Nature's Claim", &[relic], &[relic]);
    assert_eq!((t.life(P0), t.life(P1)), (20, 20));
    ruling!(
        "Slice in Twain",
        "If, on the other hand, the target is a legal target but isn’t destroyed (most likely because it has indestructible), you’ll draw a card."
    );
    assert_eq!(indestructible_target("Slice in Twain").hand_size(P0), 1);
    ruling!(
        "Slice in Twain",
        "If the target artifact or enchantment is an illegal target by the time Slice in Twain resolves, the entire spell doesn’t resolve. You won’t draw a card."
    );
    let mut t = TestGame::new(2);
    let relic = t.battlefield(P1, "Darksteel Relic");
    cast_then_remove(&mut t, "Slice in Twain", &[relic], &[relic]);
    assert_eq!(t.hand_size(P0), 0);
}

#[test]
fn cindervines_damage_needs_a_legal_target() {
    cr!("608.2b", "702.12b");
    ruling!(
        "Cindervines",
        "If the target permanent is an illegal target by the time Cindervines's activated ability tries to resolve, the ability doesn't resolve. No player is dealt damage. If the target is legal but not destroyed (most likely because it has indestructible), its controller is dealt damage."
    );
    supported("Cindervines");
    let activate = |gone: bool| {
        let mut t = TestGame::new(2);
        let vines = t.battlefield(P0, "Cindervines");
        let relic = t.battlefield(P1, "Darksteel Relic");
        t.g.players[0].mana_pool.add_type(ManaType::C, 1);
        t.activate(P0, vines, 0, &[Entity::Object(relic)]).unwrap();
        if gone {
            bounce(&mut t, relic);
        }
        t.resolve_all();
        t.life(P1)
    };
    assert_eq!(activate(true), 20);
    assert_eq!(activate(false), 18);
}

#[test]
fn kogla_abilities_with_illegal_targets() {
    cr!("608.2b", "701.14b");
    ruling!(
        "Kogla, the Titan Ape",
        "If the target creature is an illegal target when Kogla's first ability tries to resolve, the ability doesn't resolve. If it's a legal target but Kogla is no longer on the battlefield when the ability resolves, the target creature won't deal or be dealt damage."
    );
    supported("Kogla, the Titan Ape");
    // The target leaves: Kogla isn't dealt damage.
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P1, "Hill Giant");
    t.answer_targets(P0, &[Entity::Object(giant)]);
    let kogla = t.enter(P0, "Kogla, the Titan Ape");
    t.settle();
    bounce(&mut t, giant);
    t.resolve_all();
    assert_eq!(t.obj_now(kogla).damage, 0);
    // Kogla leaves: the Giant isn't dealt damage.
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P1, "Hill Giant");
    t.answer_targets(P0, &[Entity::Object(giant)]);
    let kogla = t.enter(P0, "Kogla, the Titan Ape");
    t.settle();
    bounce(&mut t, kogla);
    t.resolve_all();
    assert!(t.on_battlefield(giant));
    assert_eq!(t.obj_now(giant).damage, 0);
    ruling!(
        "Kogla, the Titan Ape",
        "If the target Human is an illegal target by the time Kogla's last ability tries to resolve, the ability won't resolve. Kogla won't gain indestructible."
    );
    let has_indestructible = |human_leaves: bool| {
        let mut t = TestGame::new(2);
        let kogla = t.battlefield(P0, "Kogla, the Titan Ape");
        let human = t.battlefield(P0, "Order of the Sacred Bell");
        t.g.players[0].mana_pool.add_type(ManaType::G, 2);
        t.activate(P0, kogla, 0, &[Entity::Object(human)]).unwrap();
        if human_leaves {
            bounce(&mut t, human);
        }
        t.resolve_all();
        t.obj_now(kogla)
            .has_keyword(mtg_engine::keywords::KeywordKind::Indestructible)
    };
    assert!(!has_indestructible(true));
    assert!(has_indestructible(false));
}

#[test]
fn crush_contraband_exiles_the_other_target() {
    cr!("608.2b", "700.2");
    ruling!(
        "Crush Contraband",
        "If you've chosen both modes and one of Crush Contraband's targets becomes an illegal target, the other is still exiled."
    );
    supported("Crush Contraband");
    let mut t = TestGame::new(2);
    let relic = t.battlefield(P1, "Darksteel Relic");
    let anthem = t.battlefield(P1, "Glorious Anthem");
    let s = t.hand(P0, "Crush Contraband");
    fill_pool(&mut t, P0, 4);
    t.cast(P0, s)
        .modes(&[0, 1])
        .target(relic)
        .target(anthem)
        .go();
    bounce(&mut t, relic);
    t.resolve_all();
    assert!(t.in_exile("Glorious Anthem"));
    assert_eq!(t.zone(relic), Zone::Hand(P1));
}

#[test]
fn titan_of_industry_with_its_only_target_illegal_does_nothing() {
    cr!("608.2b", "700.2");
    ruling!(
        "Titan of Industry",
        "If one of those modes is chosen with either the third or fourth mode and the only target is an illegal target when the ability would resolve, the ability does not resolve and the other chosen mode has no effect."
    );
    supported("Titan of Industry");
    let rhinos = |gone: bool| {
        let mut t = TestGame::new(2);
        let anthem = t.battlefield(P1, "Glorious Anthem");
        t.answer(P0, DecisionKind::Modes, Answer::Indices(vec![0, 2]));
        t.answer_targets(P0, &[Entity::Object(anthem)]);
        t.enter(P0, "Titan of Industry");
        t.settle();
        if gone {
            bounce(&mut t, anthem);
        }
        t.resolve_all();
        with_subtype(&t, P0, "Rhino").len()
    };
    assert_eq!(rhinos(true), 0);
    assert_eq!(rhinos(false), 1);
}
