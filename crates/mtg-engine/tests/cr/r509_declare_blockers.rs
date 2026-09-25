//! CR 509.1–509.2: declaring blockers, block restrictions/requirements/costs.

use crate::r506_common::*;
use mtg_engine::ability::*;
use mtg_engine::combat::{block_declaration_legal, block_options};
use mtg_engine::decision::Decision;
use mtg_engine::keywords::KeywordKind;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

fn eager_blocker() -> CardDef {
    custom_card(
        "Eager Sentry",
        "Creature — Human Soldier",
        Some((1, 4)),
        "This creature blocks each combat if able.",
    )
}

/// "Creatures can't block unless their controller pays [cost] for each blocking creature
/// they control."
fn block_tax(cost: Cost) -> CardDef {
    custom_with(
        "Blockade Tax",
        "Enchantment",
        None,
        vec![restriction(Restriction::BlockCost {
            blockers: Filter::creature(),
            cost,
        })],
    )
}

fn one_generic() -> Cost {
    Cost::mana(mtg_engine::mana::ManaCost::parse("{1}").unwrap())
}

#[test]
fn illegal_block_declarations_are_undone() {
    cr!("509.1");
    let mut t = TestGame::new(2);
    let a = t.battlefield(P0, "Grizzly Bears");
    let b = t.battlefield(P1, "Hill Giant");
    let c = t.battlefield(P1, "Grizzly Bears");
    declare(&mut t, &[(a, Entity::Player(P1))]);
    // Blocking the same attacker twice with one creature is illegal.
    block(&mut t, P1, &[(b, a), (b, a), (c, a)]);
    go_to(&mut t, Step::DeclareBlockers);
    assert!(t.g.blockers().is_empty());
    assert!(!is_blocked(&t, a));
    // Turn-based action: nothing on the stack.
    assert_eq!(t.stack_len(), 0);
}

#[test]
fn blockers_must_be_untapped_nonbattles_blocking_creatures_attacking_their_controller() {
    cr!("509.1a");
    let mut t = TestGame::new(3);
    let a1 = t.battlefield(P0, "Grizzly Bears");
    let a2 = t.battlefield(P0, "Hill Giant");
    let ready = t.battlefield(P1, "Craw Wurm");
    let tapped = t.battlefield(P1, "Grizzly Bears");
    t.g.tap(tapped);
    let odd = bf(
        &mut t,
        P1,
        with_counters_base(
            custom_with("Odd Siege", "Battle Creature — Siege", Some((3, 3)), vec![]),
            None,
            Some(3),
        ),
    );
    declare(&mut t, &[(a1, Entity::Player(P1)), (a2, Entity::Player(P2))]);
    go_to(&mut t, Step::DeclareAttackers);
    let opts = block_options(&t.g, &[P1]);
    let blockers: Vec<ObjectId> = opts.iter().map(|(b, _)| *b).collect();
    assert_eq!(blockers, vec![ready]);
    assert!(!blockers.contains(&tapped) && !blockers.contains(&odd));
    // P1's creature can block only the creature attacking P1.
    assert_eq!(opts[0].1, vec![a1]);
}

#[test]
fn evasion_restrictions_are_cumulative_and_fixed_once_blocks_are_declared() {
    cr!("509.1b");
    // CR 509.1b example: an attacker with flying and shadow can't be blocked by a creature
    // with flying but without shadow.
    let mut t = TestGame::new(2);
    let both = bf(
        &mut t,
        P0,
        custom_card("Shade Hawk", "Creature — Bird", Some((2, 2)), "Flying\nShadow"),
    );
    let flyer = t.battlefield(P1, "Serra Angel");
    let shadow = bf(
        &mut t,
        P1,
        custom_card("Umbra Walker", "Creature — Spirit", Some((2, 2)), "Shadow"),
    );
    let shadow_flyer = bf(
        &mut t,
        P1,
        custom_card("Umbra Hawk", "Creature — Bird", Some((1, 1)), "Flying\nShadow"),
    );
    declare(&mut t, &[(both, Entity::Player(P1))]);
    go_to(&mut t, Step::DeclareAttackers);
    assert!(!t.g.can_block(flyer, both));
    assert!(!t.g.can_block(shadow, both));
    assert!(t.g.can_block(shadow_flyer, both));

    // Gaining evasion after a legal block doesn't undo the block.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let giant = t.battlefield(P1, "Hill Giant");
    declare(&mut t, &[(bears, Entity::Player(P1))]);
    block(&mut t, P1, &[(giant, bears)]);
    go_to(&mut t, Step::DeclareBlockers);
    apply(
        &mut t,
        P0,
        Effect::Modify {
            what: Sel::Target(0),
            mods: vec![Modification::AddKeyword(
                mtg_engine::keywords::Keyword::new(KeywordKind::Flying),
            )],
            duration: Duration::EndOfTurn,
        },
        &[bears],
    );
    assert!(t.obj_now(bears).has_keyword(KeywordKind::Flying));
    assert!(is_blocked(&t, bears) && t.g.is_blocking(giant));
    go_to(&mut t, Step::EndOfCombat);
    assert!(!t.on_battlefield(bears));
    assert_eq!(t.life(P1), 20);
}

#[test]
fn blocking_requirements_are_maximized() {
    cr!("509.1c");
    // CR 509.1c example: one creature "blocks if able", another with no abilities, and a
    // creature with menace attacks: the player must block with both.
    let mut t = TestGame::new(2);
    let menace = bf(
        &mut t,
        P0,
        custom_card("Brute", "Creature — Ogre", Some((3, 3)), "Menace"),
    );
    let eager = bf(&mut t, P1, eager_blocker());
    let plain = t.battlefield(P1, "Grizzly Bears");
    declare(&mut t, &[(menace, Entity::Player(P1))]);
    go_to(&mut t, Step::DeclareAttackers);
    let opts = block_options(&t.g, &[P1]);
    assert!(block_declaration_legal(&t.g, &opts, &[(eager, menace), (plain, menace)]));
    assert!(!block_declaration_legal(&t.g, &opts, &[(eager, menace)]));
    assert!(!block_declaration_legal(&t.g, &opts, &[(plain, menace)]));
    assert!(!block_declaration_legal(&t.g, &opts, &[]));
    // P1 tries not to block; the engine declares the legal block.
    block(&mut t, P1, &[]);
    go_to(&mut t, Step::DeclareBlockers);
    assert!(t.g.is_blocking(eager) && t.g.is_blocking(plain));
}

#[test]
fn all_creatures_able_to_block_do_so() {
    cr!("509.1c");
    // Lure: "All creatures able to block enchanted creature do so."
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let giant = t.battlefield(P0, "Hill Giant");
    let lure = t.battlefield(P0, "Lure");
    t.g.attach(lure, Entity::Object(bears));
    let b1 = t.battlefield(P1, "Craw Wurm");
    let b2 = t.battlefield(P1, "Grizzly Bears");
    declare(
        &mut t,
        &[(bears, Entity::Player(P1)), (giant, Entity::Player(P1))],
    );
    block(&mut t, P1, &[(b1, giant)]);
    go_to(&mut t, Step::DeclareBlockers);
    // Blocking the giant instead is illegal; both block the lured creature.
    let c = t.g.combat.as_ref().unwrap();
    assert_eq!(c.blocking(b1), vec![bears]);
    assert_eq!(c.blocking(b2), vec![bears]);
}

#[test]
fn block_costs_arent_required_to_obey_requirements() {
    cr!("509.1c");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let eager = bf(&mut t, P1, eager_blocker());
    t.lands(P1, "Plains", 2);
    bf(&mut t, P0, block_tax(one_generic()));
    declare(&mut t, &[(bears, Entity::Player(P1))]);
    go_to(&mut t, Step::DeclareAttackers);
    let opts = block_options(&t.g, &[P1]);
    assert!(block_declaration_legal(&t.g, &opts, &[]));
    assert!(block_declaration_legal(&t.g, &opts, &[(eager, bears)]));
    go_to(&mut t, Step::DeclareBlockers);
    assert!(!t.g.is_blocking(eager));
}

#[test]
fn blocks_if_able_this_turn_applies_in_each_combat() {
    cr!("509.1c");
    let mut t = TestGame::new(2);
    let angel = t.battlefield(P0, "Serra Angel");
    let wall = bf(
        &mut t,
        P1,
        custom_card("Sky Wall", "Creature — Wall", Some((0, 6)), "Defender\nReach"),
    );
    apply(
        &mut t,
        P0,
        Effect::AddRestriction {
            restriction: Restriction::MustBlock(Filter::In(Box::new(Sel::Target(0)))),
            duration: Duration::EndOfTurn,
        },
        &[wall],
    );
    declare(&mut t, &[(angel, Entity::Player(P1))]);
    go_to(&mut t, Step::DeclareBlockers);
    assert!(t.g.is_blocking(wall));
    t.g.add_extra_combat(true);
    declare(&mut t, &[(angel, Entity::Player(P1))]);
    go_to(&mut t, Step::EndOfCombat);
    go_to(&mut t, Step::DeclareBlockers);
    assert_eq!(t.g.turn.combat_phases, 2);
    assert!(t.g.is_blocking(wall));
}

#[test]
fn block_costs_are_totaled_and_paid_with_mana_abilities() {
    cr!("509.1d", "509.1e");
    let mut t = TestGame::new(2);
    let a = t.battlefield(P0, "Grizzly Bears");
    let b = t.battlefield(P0, "Hill Giant");
    let x = t.battlefield(P1, "Craw Wurm");
    let y = t.battlefield(P1, "Grizzly Bears");
    let lands = t.lands(P1, "Plains", 3);
    bf(&mut t, P0, block_tax(one_generic()));
    declare(&mut t, &[(a, Entity::Player(P1)), (b, Entity::Player(P1))]);
    block(&mut t, P1, &[(x, b), (y, a)]);
    go_to(&mut t, Step::DeclareAttackers);
    let costs = mtg_engine::combat::block_costs_by_player(&t.g, &[(x, b), (y, a)]);
    assert_eq!(costs.len(), 1);
    assert_eq!(costs[0].0, P1);
    assert_eq!(costs[0].1.mana.as_ref().unwrap().mana_value(), 2);
    go_to(&mut t, Step::DeclareBlockers);
    assert!(t.g.is_blocking(x) && t.g.is_blocking(y));
    assert_eq!(lands.iter().filter(|l| t.obj_now(**l).tapped).count(), 2);
}

#[test]
fn partial_payment_of_block_costs_isnt_allowed() {
    cr!("509.1f");
    let mut t = TestGame::new(2);
    let a = t.battlefield(P0, "Grizzly Bears");
    let b = t.battlefield(P0, "Hill Giant");
    let x = t.battlefield(P1, "Craw Wurm");
    let y = t.battlefield(P1, "Grizzly Bears");
    let land = t.battlefield(P1, "Plains");
    bf(&mut t, P0, block_tax(one_generic()));
    declare(&mut t, &[(a, Entity::Player(P1)), (b, Entity::Player(P1))]);
    block(&mut t, P1, &[(x, b), (y, a)]);
    go_to(&mut t, Step::DeclareBlockers);
    assert!(t.g.blockers().is_empty());
    assert!(!t.obj_now(land).tapped);
}

#[test]
fn chosen_creatures_still_controlled_become_blocking() {
    cr!("509.1g");
    // "Creatures can't block unless their controller sacrifices a creature for each
    // blocking creature they control."
    let sac = Cost::default().with(CostPart::Sacrifice {
        filter: Filter::creature(),
        count: Value::c(1),
    });
    let mut t = TestGame::new(2);
    let a = t.battlefield(P0, "Grizzly Bears");
    let b = t.battlefield(P0, "Hill Giant");
    let x = t.battlefield(P1, "Craw Wurm");
    let y = t.battlefield(P1, "Grizzly Bears");
    let z = t.battlefield(P1, "Grizzly Bears");
    bf(&mut t, P0, block_tax(sac));
    declare(&mut t, &[(a, Entity::Player(P1)), (b, Entity::Player(P1))]);
    block(&mut t, P1, &[(x, b), (y, a)]);
    // Two blockers: two sacrifices. P1 sacrifices one of the chosen blockers (and another
    // creature): it doesn't become a blocker.
    t.answer_choose(P1, &[Entity::Object(y)]);
    t.answer_choose(P1, &[Entity::Object(z)]);
    go_to(&mut t, Step::DeclareBlockers);
    assert!(!t.on_battlefield(y));
    assert!(t.g.is_blocking(x));
    assert!(!is_blocked(&t, a));
    assert_eq!(t.g.combat.as_ref().unwrap().blocking(x), vec![b]);
    // It remains a blocking creature until combat ends.
    go_to(&mut t, Step::EndOfCombat);
    assert!(t.g.is_blocking(x));
    go_to(&mut t, Step::PostcombatMain);
    assert!(!t.g.is_blocking(x));
}

#[test]
fn attacker_stays_blocked_if_its_blockers_leave() {
    cr!("509.1h");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let unblocked = t.battlefield(P0, "Hill Giant");
    let wurm = t.battlefield(P1, "Craw Wurm");
    declare(
        &mut t,
        &[(bears, Entity::Player(P1)), (unblocked, Entity::Player(P1))],
    );
    block(&mut t, P1, &[(wurm, bears)]);
    go_to(&mut t, Step::DeclareBlockers);
    let ctx = mtg_engine::eval::Ctx::new(None, P0);
    assert!(t.g.matches(bears, &Filter::Blocked, &ctx));
    assert!(t.g.matches(unblocked, &Filter::Unblocked, &ctx));
    t.g.destroy(wurm, None);
    t.g.recompute();
    assert!(t.g.matches(bears, &Filter::Blocked, &ctx));
    go_to(&mut t, Step::EndOfCombat);
    // The blocked bears dealt no damage; the unblocked giant did.
    assert_eq!(t.life(P1), 17);
}

#[test]
fn abilities_trigger_on_blockers_being_declared() {
    cr!("509.1i", "509.2a");
    let mut t = TestGame::new(2);
    let a = bf(
        &mut t,
        P0,
        custom_card(
            "Proud Knight",
            "Creature — Human Knight",
            Some((2, 2)),
            "Whenever this creature becomes blocked, you gain 1 life.",
        ),
    );
    let b = bf(
        &mut t,
        P1,
        custom_card(
            "Watchful Guard",
            "Creature — Human Soldier",
            Some((1, 4)),
            "Whenever this creature blocks, you gain 1 life.",
        ),
    );
    declare(&mut t, &[(a, Entity::Player(P1))]);
    block(&mut t, P1, &[(b, a)]);
    go_to(&mut t, Step::DeclareBlockers);
    assert_eq!(t.stack_len(), 0);
    t.script.lock().unwrap().asked.clear();
    // Both triggers are put on the stack before the active player gets priority.
    t.g.advance();
    assert!(matches!(t.asked().first(), Some((P0, Decision::Priority { .. }))));
    assert_eq!(t.stack_len(), 2);
    t.resolve_all();
    assert_eq!(t.life(P0), 21);
    assert_eq!(t.life(P1), 21);
}

#[test]
fn active_player_gets_priority_in_declare_blockers_step() {
    cr!("509.2");
    let mut t = TestGame::new(2);
    let a = t.battlefield(P0, "Grizzly Bears");
    declare(&mut t, &[(a, Entity::Player(P1))]);
    let order = priority_order_in(&mut t, Step::DeclareBlockers);
    assert_eq!(order, vec![P0, P1]);
}
