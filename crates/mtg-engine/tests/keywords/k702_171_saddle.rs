//! CR 702.171 Saddle.

use crate::common_k702_011_017::assert_supported;
use crate::common_k702_140_152::*;
use crate::common_k702_168_177::*;
use mtg_engine::ability::*;
use mtg_engine::keywords::KeywordKind;
use mtg_engine::kw::saddle::is_saddled;
use mtg_engine::mana::ManaType;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

/// Activates the saddle ability of `mount`, tapping `saddlers`, and resolves it.
fn saddle(t: &mut TestGame, mount: ObjectId, saddlers: &[ObjectId]) {
    let uid = ability_uid(t, mount, "Saddle");
    let es: Vec<Entity> = saddlers.iter().map(|c| Entity::Object(*c)).collect();
    t.answer_choose(P0, &es);
    activate_uid(t, P0, mount, uid).expect("saddle");
    t.resolve_all();
}

#[test]
fn saddle_taps_other_creatures_with_enough_power_as_a_sorcery() {
    cr!("702.171", "702.171a");
    assert_supported("Brightfield Mustang");
    ruling!(
        "Brightfield Mustang",
        "“Saddle N” means “Tap any number of other untapped creatures you control with total power N or greater: This permanent becomes saddled until end of turn. Activate only as a sorcery.”"
    );
    ruling!(
        "Archmage's Newt",
        "\"Saddle N\" means \"Tap any number of other untapped creatures you control with total power N or greater: This permanent becomes saddled until end of turn. Activate only as a sorcery.\""
    );
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    // Giant Beaver: Saddle 3.
    let beaver = t.battlefield(P0, "Giant Beaver");
    let saddle_uid = ability_uid(&mut t, beaver, "Saddle");
    // It can't tap itself, and total power 2 isn't enough.
    let bears = t.battlefield(P0, "Grizzly Bears");
    assert!(!activatable(&mut t, P0, beaver, saddle_uid));
    // Nor can a tapped creature, or a creature an opponent controls, help.
    let elves = t.battlefield(P0, "Llanowar Elves");
    t.g.tap(elves);
    t.battlefield(P1, "Hill Giant");
    assert!(!activatable(&mut t, P0, beaver, saddle_uid));
    t.g.untap(elves);
    assert!(activatable(&mut t, P0, beaver, saddle_uid));
    // Only as a sorcery: not during combat, not in an opponent's turn, not with something
    // on the stack.
    t.set_step(P0, Step::BeginningOfCombat);
    assert!(!activatable(&mut t, P0, beaver, saddle_uid));
    t.set_step(P1, Step::PrecombatMain);
    assert!(!activatable(&mut t, P0, beaver, saddle_uid));
    t.set_step(P0, Step::PrecombatMain);
    let bolt = t.hand(P1, "Lightning Bolt");
    add_mana(&mut t, P1, ManaType::R, 1);
    t.cast(P1, bolt).target(Entity::Player(P0)).go();
    assert!(!activatable(&mut t, P0, beaver, saddle_uid));
    t.resolve_all();
    // Tapping 2 + 1 power: the Beaver becomes saddled as the ability resolves.
    t.answer_choose(P0, &[Entity::Object(bears), Entity::Object(elves)]);
    activate_uid(&mut t, P0, beaver, saddle_uid).unwrap();
    assert!(t.obj(bears).tapped && t.obj(elves).tapped);
    assert!(!is_saddled(&t.g, beaver));
    t.resolve_all();
    assert!(is_saddled(&t.g, beaver));
    assert!(!t.obj(beaver).tapped);
}

#[test]
fn a_saddled_mount_may_be_saddled_again_and_mounts_attack_normally() {
    cr!("702.171a");
    ruling!(
        "Brightfield Mustang",
        "You may activate a permanent’s saddle ability even if that permanent is already saddled."
    );
    ruling!(
        "Archmage's Newt",
        "You may activate a permanent's saddle ability even if that permanent is already saddled."
    );
    ruling!(
        "Brightfield Mustang",
        "Creatures with saddle can attack or block as normal even if they aren’t saddled."
    );
    ruling!(
        "Archmage's Newt",
        "Creatures with saddle can attack or block as normal even if they aren't saddled."
    );
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let mustang = t.battlefield(P0, "Brightfield Mustang");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let elves = t.battlefield(P0, "Llanowar Elves");
    saddle(&mut t, mustang, &[elves]);
    assert!(is_saddled(&t.g, mustang));
    let uid = ability_uid(&mut t, mustang, "Saddle");
    assert!(activatable(&mut t, P0, mustang, uid));
    saddle(&mut t, mustang, &[bears]);
    assert!(is_saddled(&t.g, mustang));
    // Unsaddled, a Mount attacks as normal (without its "while saddled" trigger).
    let mut t = TestGame::new(2);
    let mustang = t.battlefield(P0, "Brightfield Mustang");
    t.set_step(P0, Step::BeginningOfCombat);
    t.attack(&[(mustang, Entity::Player(P1))], &[]);
    assert_eq!(t.life(P1), 17);
    assert_eq!(t.counters(mustang, "+1/+1"), 0);
}

#[test]
fn saddled_lasts_until_end_of_turn_or_until_it_leaves_the_battlefield() {
    cr!("702.171b", "514.2");
    ruling!(
        "Giant Beaver",
        "“Saddled” isn’t an ability that a creature has. It’s just something true about that creature. It won’t stop being saddled until the turn ends or it leaves the battlefield."
    );
    ruling!(
        "Archmage's Newt",
        "\"Saddled\" isn't an ability that a creature has. It's just something true about that creature. It won't stop being saddled until the turn ends or it leaves the battlefield."
    );
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let mustang = t.battlefield(P0, "Brightfield Mustang");
    let bears = t.battlefield(P0, "Grizzly Bears");
    saddle(&mut t, mustang, &[bears]);
    assert!(is_saddled(&t.g, mustang));
    // It isn't an ability: losing all abilities doesn't end it.
    run_effect_on(
        &mut t,
        mustang,
        Effect::Modify {
            what: Sel::Target(0),
            mods: vec![Modification::RemoveAllAbilities],
            duration: Duration::EndOfTurn,
        },
    );
    assert!(!t.obj(mustang).chars.has_keyword(KeywordKind::Saddle));
    assert!(is_saddled(&t.g, mustang));
    // Still saddled in the end step; no longer after the cleanup step.
    t.advance_to(P0, Step::End);
    assert!(is_saddled(&t.g, mustang));
    t.advance_to(P1, Step::Upkeep);
    assert!(!is_saddled(&t.g, mustang));
    // A saddled permanent that leaves the battlefield is a new object, not saddled.
    t.advance_to(P0, Step::PrecombatMain);
    let bears = t.g.current(bears);
    saddle(&mut t, mustang, &[bears]);
    assert!(is_saddled(&t.g, mustang));
    let back = flicker(&mut t, mustang);
    assert!(!is_saddled(&t.g, back));
}

#[test]
fn a_copy_of_a_saddled_mount_isnt_saddled() {
    cr!("702.171b", "707.2");
    ruling!(
        "Brightfield Mustang",
        "If a permanent becomes a copy of a saddled Mount, the copy won’t be saddled."
    );
    ruling!(
        "Archmage's Newt",
        "If a permanent becomes a copy of a saddled Mount, the copy won't be saddled."
    );
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let mustang = t.battlefield(P0, "Brightfield Mustang");
    let bears = t.battlefield(P0, "Grizzly Bears");
    saddle(&mut t, mustang, &[bears]);
    let giant = t.battlefield(P0, "Hill Giant");
    let mut ctx = mtg_engine::eval::Ctx::new(None, P0);
    ctx.targets = vec![vec![Entity::Object(giant)], vec![Entity::Object(mustang)]];
    t.g.exec(
        &Effect::BecomeCopy {
            what: Sel::Target(0),
            of: Sel::Target(1),
            duration: Duration::Permanent,
        },
        &mut ctx,
    );
    t.g.recompute();
    assert_eq!(t.obj(giant).chars.name, "Brightfield Mustang");
    assert!(!is_saddled(&t.g, giant));
    assert!(is_saddled(&t.g, mustang));
}

#[test]
fn attacks_while_saddled_triggers_only_if_saddled_as_it_attacks() {
    cr!("702.171b");
    assert_supported("Brightfield Mustang");
    ruling!(
        "Brightfield Mustang",
        "An ability that triggers when a creature “attacks while saddled” will trigger only if that creature was saddled when it was declared as an attacker."
    );
    ruling!(
        "Archmage's Newt",
        "An ability that triggers when a creature \"attacks while saddled\" will trigger only if that creature was saddled when it was declared as an attacker."
    );
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let mustang = t.battlefield(P0, "Brightfield Mustang");
    let bears = t.battlefield(P0, "Grizzly Bears");
    saddle(&mut t, mustang, &[bears]);
    t.set_step(P0, Step::BeginningOfCombat);
    t.attack(&[(mustang, Entity::Player(P1))], &[]);
    // "Untap it and put a +1/+1 counter on it."
    assert_eq!(t.counters(mustang, "+1/+1"), 1);
    assert!(!t.obj(mustang).tapped);
    assert_eq!(t.life(P1), 16);
}

#[test]
fn creatures_tapped_for_a_saddle_ability_saddle_that_mount() {
    cr!("702.171c");
    assert_supported("Giant Beaver");
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    // Giant Beaver: "Whenever this creature attacks while saddled, put a +1/+1 counter on
    // target creature that saddled it this turn."
    let beaver = t.battlefield(P0, "Giant Beaver");
    let giant = t.battlefield(P0, "Hill Giant");
    let bears = t.battlefield(P0, "Grizzly Bears");
    saddle(&mut t, beaver, &[giant]);
    t.set_step(P0, Step::BeginningOfCombat);
    t.answer(
        P0,
        DecisionKind::Attackers,
        mtg_engine::decision::Answer::Attackers(vec![(beaver, Entity::Player(P1))]),
    );
    t.advance_to(P0, Step::DeclareAttackers);
    t.settle();
    // Only the Hill Giant saddled it: the Bears can't be targeted.
    let candidates = pending_targets(&t);
    assert!(candidates.contains(&Entity::Object(giant)));
    assert!(!candidates.contains(&Entity::Object(bears)));
    t.resolve_all();
    assert_eq!(t.counters(giant, "+1/+1"), 1);
    assert_eq!(t.counters(bears, "+1/+1"), 0);
}

#[test]
fn whenever_a_creature_saddles_a_mount() {
    cr!("702.171c");
    assert_supported("Canyon Vaulter");
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    // Canyon Vaulter: "Whenever this creature saddles a Mount or crews a Vehicle during
    // your main phase, that Mount or Vehicle gains flying until end of turn."
    let mustang = t.battlefield(P0, "Brightfield Mustang");
    let vaulter = t.battlefield(P0, "Canyon Vaulter");
    saddle(&mut t, mustang, &[vaulter]);
    assert!(t.obj(mustang).chars.has_keyword(KeywordKind::Flying));
    assert!(!t.obj(vaulter).chars.has_keyword(KeywordKind::Flying));
    // Saddled by another creature: no trigger.
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let mustang = t.battlefield(P0, "Brightfield Mustang");
    t.battlefield(P0, "Canyon Vaulter");
    let bears = t.battlefield(P0, "Grizzly Bears");
    saddle(&mut t, mustang, &[bears]);
    assert!(!t.obj(mustang).chars.has_keyword(KeywordKind::Flying));
}

#[test]
fn whenever_it_becomes_saddled_for_the_first_time_each_turn() {
    cr!("702.171a", "702.171b");
    assert_supported("Stubborn Burrowfiend");
    ruling!(
        "Stubborn Burrowfiend",
        "The value of X is determined as Stubborn Burrowfiend’s triggered ability resolves."
    );
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    for _ in 0..4 {
        t.library_top(P0, "Grizzly Bears");
    }
    let fiend = t.battlefield(P0, "Stubborn Burrowfiend");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let giant = t.battlefield(P0, "Hill Giant");
    saddle(&mut t, fiend, &[bears]);
    // Mill two creature cards: +2/+2.
    assert_eq!(t.graveyard_size(P0), 2);
    assert_eq!(t.pt(fiend), (4, 4));
    // Only the first time each turn.
    saddle(&mut t, fiend, &[giant]);
    assert_eq!(t.graveyard_size(P0), 2);
    assert_eq!(t.pt(fiend), (4, 4));
}

#[test]
fn an_effect_can_make_a_mount_saddled() {
    cr!("702.171b");
    assert_supported("Guidelight Matrix");
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    // Guidelight Matrix: "{2}, {T}: Target Mount you control becomes saddled until end of
    // turn. Activate only as a sorcery."
    let matrix = t.battlefield(P0, "Guidelight Matrix");
    let mustang = t.battlefield(P0, "Brightfield Mustang");
    add_mana(&mut t, P0, ManaType::C, 2);
    let uid = ability_uid(&mut t, matrix, "{2}, {T}: Target Mount");
    t.answer_targets(P0, &[Entity::Object(mustang)]);
    activate_uid(&mut t, P0, matrix, uid).unwrap();
    t.resolve_all();
    assert!(is_saddled(&t.g, mustang));
    t.set_step(P0, Step::BeginningOfCombat);
    t.attack(&[(mustang, Entity::Player(P1))], &[]);
    assert_eq!(t.counters(mustang, "+1/+1"), 1);
}

#[test]
fn when_it_enters_and_whenever_it_attacks_while_saddled() {
    cr!("702.171a", "702.171b");
    assert_supported("Autarch Mammoth");
    // Autarch Mammoth: "When this creature enters and whenever it attacks while saddled,
    // create a 3/3 green Elephant creature token." Saddle 5.
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let mammoth = t.enter(P0, "Autarch Mammoth");
    t.resolve_all();
    assert_eq!(creature_tokens(&t, P0).len(), 1);
    t.g.objects[mammoth.0 as usize].summoning_sick = false;
    // Attacking unsaddled: no token.
    t.set_step(P0, Step::BeginningOfCombat);
    t.attack(&[(mammoth, Entity::Player(P1))], &[]);
    assert_eq!(creature_tokens(&t, P0).len(), 1);
    // Saddled by the Elephant (3) and a Hill Giant (3): a token.
    t.advance_to(P1, Step::Upkeep);
    t.advance_to(P0, Step::PrecombatMain);
    let elephant = creature_tokens(&t, P0)[0];
    let giant = t.battlefield(P0, "Hill Giant");
    saddle(&mut t, mammoth, &[elephant, giant]);
    t.set_step(P0, Step::BeginningOfCombat);
    t.attack(&[(mammoth, Entity::Player(P1))], &[]);
    assert_eq!(creature_tokens(&t, P0).len(), 2);
}

/// Runs `effect` with `target` in target slot 0.
fn run_effect_on(t: &mut TestGame, target: ObjectId, effect: Effect) {
    let mut ctx = mtg_engine::eval::Ctx::new(None, P0);
    ctx.targets = vec![vec![Entity::Object(target)]];
    t.g.exec(&effect, &mut ctx);
    t.g.recompute();
}

/// The candidates of the most recent target choice asked.
fn pending_targets(t: &TestGame) -> Vec<Entity> {
    t.asked()
        .into_iter()
        .rev()
        .find_map(|(_, d)| match d {
            mtg_engine::decision::Decision::ChooseTargets { candidates, .. } => Some(candidates),
            _ => None,
        })
        .unwrap_or_default()
}
