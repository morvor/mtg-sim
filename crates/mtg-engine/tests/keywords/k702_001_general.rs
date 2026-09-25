//! CR 702.1 Keyword abilities: general.

use super::k702_001_010_common::*;
use mtg_engine::ability::{AbilityKind, Filter};
use mtg_engine::keywords::KeywordKind;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

#[test]
fn keyword_lines_compile_to_keyword_abilities_and_reminder_text_is_ignored() {
    cr!("702.1");
    // "Haste (This creature can attack and {T} as soon as it comes under your control.)"
    let goblin = card("Raging Goblin");
    assert!(goblin.is_fully_supported());
    let abilities = &goblin.faces[0].chars.abilities;
    assert_eq!(abilities.len(), 1);
    assert!(matches!(&abilities[0].kind, AbilityKind::Keyword(k) if k.kind == KeywordKind::Haste));
    // One line may list several keywords: "Deathtouch, lifelink".
    let aetherborn = card("Gifted Aetherborn");
    let kinds: Vec<KeywordKind> = aetherborn.faces[0].chars.keywords().map(|k| k.kind).collect();
    assert_eq!(kinds, vec![KeywordKind::Deathtouch, KeywordKind::Lifelink]);
    // The keyword stands for its rules: the Aetherborn destroys what it damages and its
    // controller gains life.
    let mut t = TestGame::new(2);
    let a = t.battlefield(P0, "Gifted Aetherborn");
    let giant = t.battlefield(P1, "Hill Giant");
    t.set_step(P0, Step::BeginningOfCombat);
    declare(&mut t, &[(a, Entity::Player(P1))]);
    block(&mut t, P1, &[(giant, a)]);
    go_to(&mut t, Step::EndOfCombat);
    assert!(t.in_graveyard(P1, "Hill Giant"));
    assert_eq!(t.life(P0), 22);
}

#[test]
fn a_keyword_cost_is_only_the_keywords_own_cost() {
    cr!("702.1a");
    let mut t = TestGame::new(2);
    // Auriok Steelshaper: "Equip costs you pay cost {1} less."
    t.battlefield(P0, "Auriok Steelshaper");
    // Ring of Evos Isle: "{2}: Equipped creature gains hexproof until end of turn." and
    // "Equip {1}".
    let ring = t.battlefield(P0, "Ring of Evos Isle");
    let bears = t.battlefield(P0, "Grizzly Bears");
    // The equip cost {1} costs {0}: no mana needed.
    t.activate(P0, ring, 1, &[Entity::Object(bears)]).unwrap();
    t.resolve_all();
    assert_eq!(t.obj_now(ring).attached_to, Some(Entity::Object(bears)));
    // The Equipment's other ability isn't an equip ability: it still costs {2}.
    t.lands(P0, "Island", 1);
    assert!(t.activate(P0, ring, 0, &[]).is_err());
    t.lands(P0, "Island", 1);
    t.activate(P0, ring, 0, &[]).unwrap();
    t.resolve_all();
    assert!(t.obj_now(bears).has_keyword(KeywordKind::Hexproof));
    // Nor is the cost to cast an Equipment spell an equip cost.
    let sword = t.hand(P0, "Short Sword");
    assert!(t.cast(P0, sword).try_go().is_err());
}

#[test]
fn a_granted_keyword_variable_is_constantly_reevaluated() {
    cr!("702.1b", "702.45a");
    let mut t = TestGame::new(2);
    // Fumiko the Lowblood: 3/2, "Fumiko has bushido X, where X is the number of
    // attacking creatures."
    let fumiko = t.battlefield(P1, "Fumiko the Lowblood");
    let bushido_n = |t: &TestGame| {
        t.obj_now(fumiko)
            .chars
            .keywords()
            .find(|k| k.kind == KeywordKind::Bushido)
            .and_then(|k| k.n)
    };
    assert_eq!(bushido_n(&t), Some(0));
    let b1 = t.battlefield(P0, "Grizzly Bears");
    let b2 = t.battlefield(P0, "Grizzly Bears");
    let b3 = t.battlefield(P0, "Hill Giant");
    t.set_step(P0, Step::BeginningOfCombat);
    declare(
        &mut t,
        &[
            (b1, Entity::Player(P1)),
            (b2, Entity::Player(P1)),
            (b3, Entity::Player(P1)),
        ],
    );
    go_to(&mut t, Step::DeclareAttackers);
    assert_eq!(bushido_n(&t), Some(3));
    block(&mut t, P1, &[(fumiko, b3)]);
    go_to(&mut t, Step::DeclareBlockers);
    t.resolve_all();
    // +3/+3: a 6/5 Fumiko kills the Hill Giant and survives.
    assert_eq!(t.pt(fumiko), (6, 5));
    go_to(&mut t, Step::EndOfCombat);
    assert!(t.on_battlefield(fumiko));
    assert!(!t.on_battlefield(b3));
    t.advance_to(P1, Step::Upkeep);
    assert_eq!(bushido_n(&t), Some(0));
}

#[test]
fn a_granted_keyword_cost_variable_is_reevaluated() {
    cr!("702.1b", "702.21a");
    let mut t = TestGame::new(2);
    // Minthara, Merciless Soul: "Minthara has ward {X}, where X is the number of
    // experience counters you have."
    let minthara = t.battlefield(P0, "Minthara, Merciless Soul");
    let ward_mv = |t: &TestGame| {
        t.obj_now(minthara)
            .chars
            .keywords()
            .find(|k| k.kind == KeywordKind::Ward)
            .and_then(|k| k.cost.as_ref().and_then(|c| c.mana.as_ref()).map(|m| m.mana_value()))
    };
    assert_eq!(ward_mv(&t), Some(0));
    t.g.add_counters(Entity::Player(P0), "experience", 2, None);
    t.g.recompute();
    assert_eq!(ward_mv(&t), Some(2));
    // An opponent's Shock targeting it is countered unless they pay {2}; they can't.
    t.lands(P1, "Mountain", 1);
    let shock = t.hand(P1, "Shock");
    t.set_step(P1, Step::PrecombatMain);
    t.cast(P1, shock).target(minthara).go();
    t.resolve_all();
    assert!(t.on_battlefield(minthara));
    assert!(t.in_graveyard(P1, "Shock"));
}

#[test]
fn the_same_is_true_for_grants_every_variant_of_each_listed_keyword() {
    cr!("702.1c");
    ruling!(
        "Cairn Wanderer",
        "Cairn Wanderer’s ability looks at all cards in all graveyards. It gains any landwalk abilities and any protection abilities."
    );
    assert_supported("Cairn Wanderer");
    let mut t = TestGame::new(2);
    let wanderer = t.battlefield(P0, "Cairn Wanderer");
    assert!(!t.obj_now(wanderer).has_keyword(KeywordKind::Flying));
    t.graveyard(P1, "Serra Angel");
    t.graveyard(P1, "Wind Drake");
    t.graveyard(P0, "White Knight");
    t.graveyard(P0, "Silver Knight");
    t.graveyard(P1, "Bog Wraith");
    t.g.recompute();
    let o = t.obj_now(wanderer);
    // Flying and vigilance from Serra Angel (flying only once for two flying cards).
    assert_eq!(instances(&t, wanderer, KeywordKind::Flying), 1);
    assert!(o.has_keyword(KeywordKind::Vigilance));
    assert!(o.has_keyword(KeywordKind::FirstStrike));
    // Protection from black and from red: each variant.
    let prot: Vec<Option<Filter>> = o
        .chars
        .keywords()
        .filter(|k| k.kind == KeywordKind::Protection)
        .map(|k| k.filter.clone())
        .collect();
    assert_eq!(prot.len(), 2);
    assert!(prot
        .iter()
        .any(|f| matches!(f, Some(Filter::Color(Color::Black)))));
    assert!(prot.iter().any(|f| matches!(f, Some(Filter::Color(Color::Red)))));
    // Swampwalk: the landwalk variant.
    let walk: Vec<Option<Filter>> = o
        .chars
        .keywords()
        .filter(|k| k.kind == KeywordKind::Landwalk)
        .map(|k| k.filter.clone())
        .collect();
    assert_eq!(walk.len(), 1);
    assert!(matches!(&walk[0], Some(Filter::Subtype(s)) if s == "Swamp"));
    // Keywords that aren't listed aren't granted, and noncreature cards don't count.
    t.graveyard(P0, "Typhoid Rats");
    t.g.recompute();
    assert!(t.obj_now(wanderer).has_keyword(KeywordKind::Deathtouch));
    // The protection works: it can't be blocked by black creatures.
    let rats = t.battlefield(P1, "Typhoid Rats");
    t.set_step(P0, Step::BeginningOfCombat);
    declare(&mut t, &[(wanderer, Entity::Player(P1))]);
    go_to(&mut t, Step::DeclareAttackers);
    assert!(!t.g.can_block(rats, wanderer));
}

#[test]
fn with_a_keyword_means_with_that_keyword_ability() {
    cr!("702.1d");
    let mut t = TestGame::new(2);
    let drake = t.battlefield(P1, "Wind Drake");
    let bears = t.battlefield(P1, "Grizzly Bears");
    let spider = t.battlefield(P1, "Giant Spider");
    t.lands(P0, "Forest", 2);
    // Plummet: "Destroy target creature with flying."
    let plummet = t.hand(P0, "Plummet");
    t.cast(P0, plummet).target(drake).go();
    let candidates = last_target_candidates(&t);
    assert_eq!(candidates, vec![Entity::Object(drake)]);
    let _ = (bears, spider);
    t.resolve();
    assert!(t.in_graveyard(P1, "Wind Drake"));
    // A creature that gains flying from an effect has a flying ability too.
    t.battlefield(P1, "Levitation");
    t.lands(P0, "Forest", 2);
    let plummet = t.hand(P0, "Plummet");
    t.cast(P0, plummet).target(bears).go();
    let candidates = last_target_candidates(&t);
    assert!(candidates.contains(&Entity::Object(bears)));
    assert!(candidates.contains(&Entity::Object(spider)));
    t.resolve();
    assert!(t.in_graveyard(P1, "Grizzly Bears"));
}
