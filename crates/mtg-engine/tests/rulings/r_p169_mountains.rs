//! Rulings batch P169 — Mountain matters (land types, CR 205.3i, 305.6), Arena of
//! Glory's haste mana (CR 106.6), values locked in as a spell is cast (CR 601.2d), static
//! abilities in a graveyard (CR 613.7a), and "attack with total power" triggers.

use crate::r_p169_common::*;
use mtg_engine::decision::Decision;
use mtg_engine::keywords::KeywordKind;
use mtg_engine::mana::ManaType;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

#[test]
fn desert_were_worm_counts_power_only_as_attackers_are_declared() {
    cr!("508.1", "603.2");
    ruling!(
        "Desert Were-Worm",
        "Desert Were-Worm's second ability will only trigger if creatures you attacked with had power 12 or greater at the time you attacked. Increases to the power of attacking creatures after they attack, such as from attack triggers, will not contribute to the 12 power needed for this ability to trigger."
    );
    supported("Desert Were-Worm");
    supported("Brazen Wolves");
    // `mountains` Mountains: the Worm is 2*mountains/5; Brazen Wolves is 2/3 (4/3 once
    // its attack trigger resolves).
    let run = |mountains: usize| {
        let mut t = TestGame::new(2);
        t.lands(P0, "Mountain", mountains);
        let worm = t.battlefield(P0, "Desert Were-Worm");
        let wolves = t.battlefield(P0, "Brazen Wolves");
        attack_with(
            &mut t,
            &[(worm, Entity::Player(P1)), (wolves, Entity::Player(P1))],
        );
        let worm_triggers = stack_triggers_from(&t, worm).len();
        t.resolve_all();
        let total = t.pt(worm).0 + t.pt(wolves).0;
        (worm_triggers, total)
    };
    // 10 power as declared, 12 after Brazen Wolves's trigger: no trigger.
    assert_eq!(run(4), (0, 12));
    // Positive control: 12 as declared.
    assert_eq!(run(5), (1, 14));
}

/// P0's Arena of Glory, exerted for {R}{R} (with {R} from the pool for its cost).
fn arena_mana(t: &mut TestGame) -> ObjectId {
    let arena = t.battlefield(P0, "Arena of Glory");
    mana(t, P0, ManaType::R, 1);
    t.activate(P0, arena, 1, &[]).unwrap();
    assert_eq!(t.g.player(P0).mana_pool.total(), 2);
    arena
}

#[test]
fn arena_of_glory_mana_on_a_noncreature_spell_gives_no_haste() {
    cr!("106.6", "302.6");
    ruling!(
        "Arena of Glory",
        "If mana generated with Arena of Glory's last ability is spent on a noncreature spell that becomes a creature later in the turn, that creature won't have haste."
    );
    ruling!(
        "Riddleform",
        "If Riddleform becomes a creature the same turn it enters the battlefield, you can't attack with it unless it gains haste."
    );
    supported("Arena of Glory");
    supported("Riddleform");
    let mut t = TestGame::new(2);
    arena_mana(&mut t);
    mana(&mut t, P0, ManaType::U, 1);
    let card = t.hand(P0, "Riddleform");
    t.cast(P0, card).go();
    t.resolve_all();
    assert_eq!(t.g.player(P0).mana_pool.total(), 1);
    let riddle = t.g.current(card);
    t.answer_yes(P0, true);
    cast_opt(&mut t, P0);
    t.resolve_all();
    assert!(t.obj_now(riddle).is(CardType::Creature));
    assert_eq!(t.pt(riddle), (3, 3));
    t.set_step(P0, Step::BeginningOfCombat);
    assert!(!can_attack(&mut t, riddle));
    // A Riddleform that has been under P0's control since the turn began can attack.
    let mut t = TestGame::new(2);
    let riddle = t.battlefield(P0, "Riddleform");
    t.answer_yes(P0, true);
    cast_opt(&mut t, P0);
    t.resolve_all();
    t.set_step(P0, Step::BeginningOfCombat);
    assert!(can_attack(&mut t, riddle));
    // One that entered this turn can attack once it has haste (Fervor).
    supported("Fervor");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Fervor");
    mana(&mut t, P0, ManaType::U, 2);
    let card = t.hand(P0, "Riddleform");
    t.cast(P0, card).go();
    t.resolve_all();
    let riddle = t.g.current(card);
    t.set_step(P0, Step::BeginningOfCombat);
    assert!(!can_attack(&mut t, riddle));
    t.set_step(P0, Step::PrecombatMain);
    t.answer_yes(P0, true);
    cast_opt(&mut t, P0);
    t.resolve_all();
    assert!(t.obj_now(riddle).is(CardType::Creature));
    t.set_step(P0, Step::BeginningOfCombat);
    assert!(can_attack(&mut t, riddle));
}

#[test]
fn arena_of_glory_mana_on_a_token_making_spell_gives_no_haste() {
    cr!("106.6", "302.6");
    ruling!(
        "Arena of Glory",
        "If mana generated with Arena of Glory's last ability is spent to cast a noncreature spell that creates tokens, those tokens won't gain haste."
    );
    ruling!(
        "Arena of Glory",
        "The mana generated with Arena of Glory's last ability can be spent on anything, not just creature spells."
    );
    let mut t = TestGame::new(2);
    arena_mana(&mut t);
    mana(&mut t, P0, ManaType::W, 1);
    let card = t.hand(P0, "Raise the Alarm");
    t.cast(P0, card).go();
    t.resolve_all();
    // Arena mana paid the generic part.
    assert_eq!(t.g.player(P0).mana_pool.total(), 1);
    let soldiers = tokens(&t, P0);
    assert_eq!(soldiers.len(), 2);
    // The other {R} pays for Shock.
    let bears = t.battlefield(P1, "Grizzly Bears");
    let shock = t.hand(P0, "Shock");
    t.cast(P0, shock).target(bears).go();
    t.resolve_all();
    assert!(t.in_graveyard(P1, "Grizzly Bears"));
    assert_eq!(t.g.player(P0).mana_pool.total(), 0);
    t.set_step(P0, Step::BeginningOfCombat);
    for s in soldiers {
        assert!(!has_kw(&mut t, s, KeywordKind::Haste));
        assert!(!can_attack(&mut t, s));
    }
}

#[test]
fn arena_of_glory_mana_on_any_part_of_a_creature_spells_cost_gives_haste() {
    cr!("106.6", "601.2f", "702.33a");
    ruling!(
        "Arena of Glory",
        "If mana generated with Arena of Glory's last ability is spent to pay any part of a creature spell's cost, including an alternative or additional cost, that creature spell (and the resulting creature) will gain haste until end of turn."
    );
    // One Arena {R} pays the generic part of Grizzly Bears.
    let mut t = TestGame::new(2);
    arena_mana(&mut t);
    mana(&mut t, P0, ManaType::G, 1);
    let card = t.hand(P0, "Grizzly Bears");
    let spell = t.cast(P0, card).go();
    assert!(
        has_kw(&mut t, spell, KeywordKind::Haste),
        "the spell has haste"
    );
    t.resolve_all();
    let bears = t.g.current(card);
    t.set_step(P0, Step::BeginningOfCombat);
    assert!(can_attack(&mut t, bears));
    // Kavu Titan kicked: Forests pay its mana cost, and Arena mana is needed for the
    // kicker.
    supported("Kavu Titan");
    let mut t = TestGame::new(2);
    arena_mana(&mut t);
    mana(&mut t, P0, ManaType::G, 2);
    mana(&mut t, P0, ManaType::C, 1);
    let card = t.hand(P0, "Kavu Titan");
    t.cast(P0, card).kicked(true).go();
    t.resolve_all();
    let titan = t.g.current(card);
    assert_eq!(t.counters(titan, counters::PLUS1), 3);
    t.set_step(P0, Step::BeginningOfCombat);
    assert!(can_attack(&mut t, titan));
    // Positive control: a creature cast without Arena mana can't attack.
    let mut t = TestGame::new(2);
    mana(&mut t, P0, ManaType::G, 2);
    let card = t.hand(P0, "Grizzly Bears");
    t.cast(P0, card).go();
    t.resolve_all();
    t.set_step(P0, Step::BeginningOfCombat);
    assert!(!can_attack(&mut t, card));
}

#[test]
fn arena_of_glory_enters_tapped_alongside_a_mountain() {
    cr!("614.12", "614.1d");
    ruling!(
        "Arena of Glory",
        "You must already control a Mountain as Arena of Glory enters the battlefield for it to enter untapped. If it enters the battlefield at the same time as a Mountain when you control no other Mountains, it will enter tapped."
    );
    let mut t = TestGame::new(2);
    let both = enter_together(&mut t, P0, &["Arena of Glory", "Mountain"]);
    let arena = both
        .iter()
        .copied()
        .find(|id| t.obj_now(*id).chars.name == "Arena of Glory")
        .unwrap();
    assert!(t.obj_now(arena).tapped);
    // With a Mountain already: untapped.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Mountain");
    let arena = t.enter(P0, "Arena of Glory");
    assert!(!t.obj_now(arena).tapped);
}

#[test]
fn mountain_types_not_names() {
    cr!("205.3i", "305.6");
    ruling!(
        "Earth Servant",
        "The ability cares about lands with the land type Mountain, not necessarily lands named Mountain."
    );
    ruling!(
        "Wild Nacatl",
        "Wild Nacatl's abilities check for lands with the subtypes Mountain and Plains. Those lands don't need to be named Mountain and Plains. Wild Nacatl will get both bonuses if those two subtypes are contained among the lands you control, even if they're from the same land (such as Plateau or Sacred Foundry)."
    );
    supported("Earth Servant");
    supported("Wild Nacatl");
    let mut t = TestGame::new(2);
    let servant = t.battlefield(P0, "Earth Servant");
    let nacatl = t.battlefield(P0, "Wild Nacatl");
    assert_eq!(t.pt(servant), (4, 4));
    assert_eq!(t.pt(nacatl), (1, 1));
    t.battlefield(P0, "Stomping Ground");
    assert_eq!(t.pt(servant), (4, 5));
    assert_eq!(t.pt(nacatl), (2, 2));
    let mut t = TestGame::new(2);
    let nacatl = t.battlefield(P0, "Wild Nacatl");
    t.battlefield(P0, "Plateau");
    assert_eq!(t.pt(nacatl), (3, 3));
    let mut t = TestGame::new(2);
    let nacatl = t.battlefield(P0, "Wild Nacatl");
    t.battlefield(P0, "Sacred Foundry");
    assert_eq!(t.pt(nacatl), (3, 3));
}

#[test]
fn jaws_of_stone_locks_in_and_divides_as_it_is_cast() {
    cr!("601.2d");
    ruling!(
        "Jaws of Stone",
        "The amount of damage is locked in as you cast Jaws of Stone. It won’t change, even if the number of Mountains you control changes."
    );
    ruling!(
        "Jaws of Stone",
        "You divide the damage among the targets as you cast Jaws of Stone. Each target must be assigned at least 1 damage."
    );
    supported("Jaws of Stone");
    let mut t = TestGame::new(2);
    let mountains = t.lands(P0, "Mountain", 3);
    t.lands(P0, "Wastes", 3);
    let giant = t.battlefield(P1, "Hill Giant");
    let card = t.hand(P0, "Jaws of Stone");
    let from = t.asked().len();
    t.answer_targets(P0, &[obj(giant), Entity::Player(P1)]);
    t.answer(
        P0,
        DecisionKind::Divide,
        mtg_engine::decision::Answer::Numbers(vec![2, 1]),
    );
    let spell = t.cast(P0, card).go();
    let divides: Vec<(u32, u32)> = t.asked()[from..]
        .iter()
        .filter_map(|(_, d)| match d {
            Decision::Divide {
                total, min_each, ..
            } => Some((*total, *min_each)),
            _ => None,
        })
        .collect();
    assert_eq!(divides, vec![(3, 1)]);
    assert!(t.g.stack.contains(&spell));
    // A Mountain leaves and two more arrive before it resolves: still 3 damage, 2 and 1.
    destroy(&mut t, mountains[0]);
    t.lands(P0, "Mountain", 2);
    t.resolve_all();
    assert_eq!(marked(&t, giant), 2);
    assert_eq!(t.life(P1), 19);
    // A division that leaves a target without damage isn't accepted (P0 asks for 3 and 0):
    // each target is assigned at least 1.
    let mut t = TestGame::new(2);
    t.lands(P0, "Mountain", 3);
    t.lands(P0, "Wastes", 3);
    let giant = t.battlefield(P1, "Hill Giant");
    let card = t.hand(P0, "Jaws of Stone");
    t.answer_targets(P0, &[obj(giant), Entity::Player(P1)]);
    t.answer(
        P0,
        DecisionKind::Divide,
        mtg_engine::decision::Answer::Numbers(vec![3, 0]),
    );
    t.cast(P0, card).go();
    t.resolve_all();
    let to_giant = marked(&t, giant) as i32;
    let to_p1 = 20 - t.life(P1);
    assert!(t.on_battlefield(giant));
    assert!(to_giant >= 1 && to_p1 >= 1, "{to_giant} and {to_p1}");
    assert_eq!(to_giant + to_p1, 3);
}

#[test]
fn anger_timestamp_is_when_it_went_to_the_graveyard() {
    cr!("613.7a", "613.1f", "613.7");
    ruling!(
        "Anger",
        "The timestamp for the \"in your graveyard\" ability is set at the time that this card goes to your graveyard, regardless of whether you control a Mountain at that time."
    );
    supported("Anger");
    supported("Turn to Frog");
    // Anger goes to the graveyard (no Mountain), then Turn to Frog makes the Bears lose
    // all abilities, then P0 gets a Mountain: Anger's effect is older, so the Bears end up
    // without haste.
    let mut t = TestGame::new(2);
    let anger = t.battlefield(P0, "Anger");
    destroy(&mut t, anger);
    assert!(t.in_graveyard(P0, "Anger"));
    let bears = t.battlefield_sick(P0, "Grizzly Bears");
    cast_new(&mut t, P0, "Turn to Frog", &[obj(bears)]);
    t.resolve_all();
    t.battlefield(P0, "Mountain");
    assert!(!has_kw(&mut t, bears, KeywordKind::Haste));
    // Positive control: Anger put into the graveyard after Turn to Frog grants haste.
    let mut t = TestGame::new(2);
    let anger = t.battlefield(P0, "Anger");
    let bears = t.battlefield_sick(P0, "Grizzly Bears");
    cast_new(&mut t, P0, "Turn to Frog", &[obj(bears)]);
    t.resolve_all();
    destroy(&mut t, anger);
    t.battlefield(P0, "Mountain");
    assert!(has_kw(&mut t, bears, KeywordKind::Haste));
}

#[test]
fn summit_apes_menace_matters_only_as_blockers_are_declared() {
    cr!("509.1b", "509.1h", "702.111b");
    ruling!(
        "Summit Apes",
        "Whether you control a Mountain matters only as the defending player declares blockers. Once it’s blocked, gaining or losing control of a Mountain won’t affect Summit Apes’s blockers."
    );
    supported("Summit Apes");
    // Gaining a Mountain after a single creature blocked it.
    let mut t = TestGame::new(2);
    let apes = t.battlefield(P0, "Summit Apes");
    let bears = t.battlefield(P1, "Grizzly Bears");
    crate::r_p076_common::unblockable_after_blocked(&mut t, apes, bears, |t| {
        t.battlefield(P0, "Mountain");
        assert!(has_kw(t, apes, KeywordKind::Menace));
    });
    // Losing the Mountain after two creatures blocked it: both still block.
    let mut t = TestGame::new(2);
    let mountain = t.battlefield(P0, "Mountain");
    let apes = t.battlefield(P0, "Summit Apes");
    let a = t.battlefield(P1, "Grizzly Bears");
    let b = t.battlefield(P1, "Grizzly Bears");
    crate::r_s03_common::to_blockers(
        &mut t,
        &[(apes, Entity::Player(P1))],
        &[(a, apes), (b, apes)],
    );
    assert_eq!(crate::r_s21_common::blocks_now(&t).len(), 2);
    destroy(&mut t, mountain);
    assert!(!has_kw(&mut t, apes, KeywordKind::Menace));
    let blocks = crate::r_s21_common::blocks_now(&t);
    assert_eq!(blocks.len(), 2);
}

#[test]
fn civic_saber_counts_the_colors() {
    cr!("105.2", "613.4c");
    ruling!(
        "Civic Saber",
        "Civic Saber's bonus can range from +0/+0 (for a colorless creature) to +5/+0 (for a creature that's all five colors)."
    );
    supported("Civic Saber");
    for (name, base, bonus) in [("Ornithopter", (0, 2), 0), ("Child of Alara", (6, 6), 5)] {
        let mut t = TestGame::new(2);
        let c = t.battlefield(P0, name);
        let saber = t.battlefield(P0, "Civic Saber");
        assert!(t.g.attach(saber, obj(c)));
        t.g.recompute();
        assert_eq!(t.pt(c), (base.0 + bonus, base.1), "{name}");
    }
}
