//! Rulings batch S07 — evoke (CR 702.74): "You may cast this card by paying [cost] rather
//! than paying its mana cost" and "When this permanent enters, if its evoke cost was paid,
//! its controller sacrifices it."

use crate::r_s01_common::*;
use crate::r_s02_common::*;
use crate::r_s04_common::*;
use crate::r_s05_common::*;
use crate::r_s07_common::*;
use mtg_engine::ability::*;
use mtg_engine::decision::Answer;
use mtg_engine::game::Game;
use mtg_engine::keywords::KeywordKind;
use mtg_engine::object::CastMethod;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

const EVOKE: CastMethod = CastMethod::Keyword(KeywordKind::Evoke);

#[test]
fn the_sacrifice_trigger_and_the_enters_trigger_go_on_the_stack_in_either_order() {
    cr!("603.3b", "702.74a");
    ruling!(
        "Ingot Chewer",
        "If you cast this card for its evoke cost, you may put the sacrifice trigger and the regular enters-the-battlefield trigger on the stack in either order. The one put on the stack last will resolve first."
    );
    supported("Ingot Chewer");
    // Ingot Chewer: {4}{R} 3/3, "When this creature enters, destroy target artifact.";
    // evoke {R}.
    let mut tops = Vec::new();
    for order in [vec![0, 1], vec![1, 0]] {
        let mut t = TestGame::new(2);
        t.battlefield(P1, "Millstone");
        t.lands(P0, "Mountain", 1);
        let chewer = t.hand(P0, "Ingot Chewer");
        t.answer(P0, DecisionKind::Order, Answer::Indices(order));
        t.cast(P0, chewer).method(EVOKE).go();
        t.resolve();
        assert_eq!(t.stack_len(), 2);
        assert_eq!(on_stack(&t, "Evoke"), 1);
        let evoke_on_top = stack_items(&t).last().unwrap().contains("Evoke");
        tops.push(evoke_on_top);
        // The one put on the stack last resolves first.
        t.resolve();
        if evoke_on_top {
            assert!(t.named_on_battlefield("Ingot Chewer").is_empty());
            assert!(t.on_battlefield(t.named_on_battlefield("Millstone")[0]));
        } else {
            assert_eq!(t.named_on_battlefield("Ingot Chewer").len(), 1);
            assert!(t.in_graveyard(P1, "Millstone"));
        }
        t.resolve_all();
        assert!(t.in_graveyard(P0, "Ingot Chewer"));
        assert!(t.in_graveyard(P1, "Millstone"));
    }
    assert_ne!(tops[0], tops[1]);
}

#[test]
fn a_spell_cast_without_paying_its_mana_cost_cant_be_evoked() {
    cr!("118.9a", "702.74a", "702.85a");
    ruling!(
        "Offalsnout",
        "If you're casting a spell \"without paying its mana cost,\" you can't use its evoke ability."
    );
    supported("Offalsnout");
    supported("Bloodbraid Elf");
    // Bloodbraid Elf cascades into Offalsnout ({2}{B} 2/2 flash; evoke {B}): it's cast
    // without paying its mana cost, so its evoke cost isn't paid and it stays.
    let mut t = TestGame::new(2);
    stack_library(&mut t, P0, &["Offalsnout"]);
    give_mana_for(&mut t, P0, "Bloodbraid Elf");
    let elf = t.hand(P0, "Bloodbraid Elf");
    t.cast(P0, elf).go();
    t.settle();
    t.answer_yes(P0, true);
    t.resolve();
    let snout = *t.g.stack.last().unwrap();
    assert_eq!(t.g.obj(snout).chars.name, "Offalsnout");
    let cast = &t.g.obj(snout).stack.as_ref().unwrap().cast;
    assert_eq!(cast.method, CastMethod::Free);
    assert!(!cast.paid.iter().any(|p| p == "evoke"));
    t.resolve();
    assert_eq!(on_stack(&t, "Evoke"), 0);
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Offalsnout").len(), 1);
}

#[test]
fn evoke_doesnt_change_when_the_creature_can_be_cast() {
    cr!("302.1", "702.74a");
    ruling!(
        "Reveillark",
        "Evoke doesn't change the timing of when you can cast the creature that has it."
    );
    supported("Reveillark");
    // Reveillark ({4}{W}, evoke {5}{W}) has no flash.
    let mut t = TestGame::new(2);
    t.lands(P0, "Plains", 6);
    let lark = t.hand(P0, "Reveillark");
    assert!(can_cast(&mut t, P0, lark, EVOKE));
    // Not in the opponent's turn...
    t.set_step(P1, Step::PrecombatMain);
    assert!(!can_cast(&mut t, P0, lark, EVOKE));
    // ...nor with a spell on the stack, nor in combat.
    t.set_step(P0, Step::PrecombatMain);
    let bolt = t.hand(P1, "Lightning Bolt");
    t.lands(P1, "Mountain", 1);
    t.cast(P1, bolt).target(Entity::Player(P0)).go();
    assert!(!can_cast(&mut t, P0, lark, EVOKE));
    t.resolve_all();
    t.set_step(P0, Step::BeginningOfCombat);
    assert!(!can_cast(&mut t, P0, lark, EVOKE));
    // Offalsnout has flash: it can be evoked in the opponent's turn.
    t.set_step(P1, Step::PrecombatMain);
    t.lands(P0, "Swamp", 1);
    let snout = t.hand(P0, "Offalsnout");
    assert!(can_cast(&mut t, P0, snout, EVOKE));
}

#[test]
fn evoking_a_spell_doesnt_change_its_mana_cost() {
    cr!("118.9c", "202.3", "702.74a");
    ruling!(
        "Spitebellows",
        "When you cast a spell by paying its evoke cost, its mana cost doesn't change. You just pay the evoke cost instead."
    );
    supported("Spitebellows");
    // Spitebellows: {5}{R} 6/1; evoke {1}{R}{R}.
    let mut t = TestGame::new(2);
    t.lands(P0, "Mountain", 3);
    let sb = t.hand(P0, "Spitebellows");
    let spell = t.cast(P0, sb).method(EVOKE).go();
    assert_eq!(untapped_lands(&t, P0), 0);
    let o = t.g.obj(spell);
    assert_eq!(
        o.chars.mana_cost,
        mtg_engine::mana::ManaCost::parse("{5}{R}")
    );
    assert_eq!(t.g.mana_value_of(spell), 6);
}

#[test]
fn a_reduced_evoke_cost_still_counts_as_the_evoke_cost_being_paid() {
    cr!("601.2f", "702.74a");
    ruling!(
        "Walker of the Grove",
        "Whether evoke's sacrifice ability triggers when the creature enters depends on whether the spell's controller chose to pay the evoke cost, not whether they actually paid it"
    );
    supported("Walker of the Grove");
    supported("Heartless Summoning");
    // Walker of the Grove: {6}{G}{G} 7/7, "When this creature leaves the battlefield,
    // create a 4/4 green Elemental creature token."; evoke {4}{G}. Heartless Summoning:
    // "Creature spells you cast cost {2} less to cast. Creatures you control get -1/-1."
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Heartless Summoning");
    t.lands(P0, "Forest", 3);
    let walker = t.hand(P0, "Walker of the Grove");
    // {4}{G} - {2}: three lands.
    t.cast(P0, walker).method(EVOKE).go();
    assert_eq!(untapped_lands(&t, P0), 0);
    t.resolve();
    assert_eq!(on_stack(&t, "Evoke"), 1);
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Walker of the Grove"));
    assert_eq!(tokens_with_subtype(&t, P0, "Elemental").len(), 1);
}

#[test]
fn cost_increases_and_reductions_apply_to_the_evoke_cost() {
    cr!("601.2f", "118.9d", "702.74a");
    ruling!(
        "Reveillark",
        "Effects that cause you to pay more or less to cast a spell will cause you to pay that much more or less while casting it for its evoke cost, too."
    );
    supported("Sphere of Resistance");
    // Sphere of Resistance: "Spells cost {1} more to cast." Evoke {5}{W} + {1}.
    let mut t = TestGame::new(2);
    t.battlefield(P1, "Sphere of Resistance");
    t.lands(P0, "Plains", 6);
    let lark = t.hand(P0, "Reveillark");
    assert!(!can_cast(&mut t, P0, lark, EVOKE));
    t.lands(P0, "Plains", 1);
    assert!(can_cast(&mut t, P0, lark, EVOKE));
    t.cast(P0, lark).method(EVOKE).go();
    assert_eq!(untapped_lands(&t, P0), 0);
    // Heartless Summoning: {5}{W} - {2}.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Heartless Summoning");
    t.lands(P0, "Plains", 3);
    let lark = t.hand(P0, "Reveillark");
    assert!(!can_cast(&mut t, P0, lark, EVOKE));
    t.lands(P0, "Plains", 1);
    t.cast(P0, lark).method(EVOKE).go();
    assert_eq!(untapped_lands(&t, P0), 0);
}

/// Casts Reveillark for its evoke cost for P0 and resolves the spell (the evoke trigger
/// is then on the stack). P0 and P1 each have a Grizzly Bears card in their graveyard.
fn evoke_reveillark(t: &mut TestGame) -> ObjectId {
    t.graveyard(P0, "Grizzly Bears");
    t.graveyard(P1, "Grizzly Bears");
    t.lands(P0, "Plains", 6);
    let lark = t.hand(P0, "Reveillark");
    t.cast(P0, lark).method(EVOKE).go()
}

#[test]
fn a_creature_that_changed_controllers_is_still_sacrificed() {
    cr!("603.3a", "702.74a");
    ruling!(
        "Reveillark",
        "if a creature cast with evoke changes controllers after it enters but before its sacrifice ability resolves, it will still be sacrificed."
    );
    let mut t = TestGame::new(2);
    let spell = evoke_reveillark(&mut t);
    let p1_bears = t.g.player(P1).graveyard[0];
    t.resolve();
    let lark = t.g.current(spell);
    assert!(t.on_battlefield(lark));
    assert_eq!(on_stack(&t, "Evoke"), 1);
    // In response, P1 gains control of Reveillark.
    run_from(
        &mut t,
        P1,
        None,
        Effect::GainControl {
            what: Sel::Target(0),
            who: PlayerRef::You,
            duration: Duration::EndOfTurn,
        },
        &[Entity::Object(lark)],
    );
    assert_eq!(t.obj_now(lark).controller, P1);
    // The evoke trigger resolves: Reveillark is sacrificed (by P1). Its leaves-the-
    // battlefield ability is controlled by P1: it can return P1's Bears, not P0's.
    let from = t.asked().len();
    t.answer_targets(P1, &[Entity::Object(p1_bears)]);
    t.resolve();
    assert!(t.in_graveyard(P0, "Reveillark"));
    assert_eq!(
        target_candidates(&t, P1, from),
        vec![vec![Entity::Object(p1_bears)]]
    );
    t.resolve_all();
    let bears = named_of(&t, P1, "Grizzly Bears");
    assert_eq!(bears.len(), 1);
    assert_eq!(t.obj(bears[0]).owner, P1);
    assert!(t.in_graveyard(P0, "Grizzly Bears"));
}

/// Whether Night Incarnate is on the battlefield with an ability on the stack.
fn incarnate_waiting(g: &Game) -> bool {
    !g.find_in_zone(mtg_engine::object::Zone::Battlefield, "Night Incarnate")
        .is_empty()
        && g.stack.iter().any(|s| !g.obj(*s).is_spell())
}

#[test]
fn players_may_respond_to_the_evoke_sacrifice_trigger() {
    cr!("603.3", "702.74a", "117.3b");
    ruling!(
        "Night Incarnate",
        "The ability that causes you to sacrifice an evoked creature is a triggered ability. Players may respond to this triggered ability while the creature is still on the battlefield."
    );
    supported("Night Incarnate");
    // Night Incarnate: {4}{B} 3/4 deathtouch, "When this creature leaves the battlefield,
    // all creatures get -3/-3 until end of turn."; evoke {3}{B}.
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P1, "Hill Giant");
    t.lands(P0, "Swamp", 4);
    let ni = t.hand(P0, "Night Incarnate");
    let seen1 = watch(&mut t, P1, is_priority, incarnate_waiting);
    t.cast(P0, ni).method(EVOKE).go();
    // Both players pass: the spell resolves, the evoke trigger goes on the stack, and P1
    // gets priority with Night Incarnate still on the battlefield.
    let ok = t.g.run_until(1000, |g| {
        g.stack.is_empty()
            && g.find_in_zone(mtg_engine::object::Zone::Battlefield, "Night Incarnate")
                .is_empty()
    });
    assert!(ok);
    assert!(seen1.lock().unwrap().iter().any(|x| *x));
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Night Incarnate"));
    assert!(!t.on_battlefield(giant));
}

#[test]
fn the_creatures_own_abilities_can_resolve_before_the_evoke_ability() {
    cr!("603.3b", "702.74a", "601.2h");
    ruling!(
        "Wistfulness",
        "If you pay the evoke cost, you can have the creature's own triggered ability or abilities resolve before the evoke triggered ability. You can cast spells after that ability resolves but before you have to sacrifice the creature."
    );
    supported("Wistfulness");
    // Wistfulness: {3}{G/U}{G/U} 6/5; "When this creature enters, if {G}{G} was spent to
    // cast it, exile target artifact or enchantment an opponent controls." and "When this
    // creature enters, if {U}{U} was spent to cast it, draw two cards, then discard a
    // card."; evoke {G/U}{G/U}. Evoked with two Forests: only the first ability triggers.
    let mut tested = 0;
    for order in [vec![0, 1], vec![1, 0]] {
        let mut t = TestGame::new(2);
        let stone = t.battlefield(P1, "Millstone");
        t.lands(P0, "Forest", 2);
        let w = t.hand(P0, "Wistfulness");
        let hand = t.hand_size(P0);
        t.answer(P0, DecisionKind::Order, Answer::Indices(order));
        t.answer_targets(P0, &[Entity::Object(stone)]);
        t.cast(P0, w).method(EVOKE).go();
        t.resolve();
        assert_eq!(t.stack_len(), 2);
        if stack_items(&t).last().unwrap().contains("Evoke") {
            continue;
        }
        tested += 1;
        // The exile ability resolves first, with Wistfulness still on the battlefield.
        t.resolve();
        assert!(!t.on_battlefield(stone));
        let wist = t.named_on_battlefield("Wistfulness");
        assert_eq!(wist.len(), 1);
        // P0 can cast a spell before the evoke ability resolves.
        t.lands(P0, "Forest", 1);
        let gg = t.hand(P0, "Giant Growth");
        t.cast(P0, gg).target(wist[0]).go();
        t.resolve();
        assert_eq!(t.pt(wist[0]), (9, 8));
        assert_eq!(on_stack(&t, "Evoke"), 1);
        t.resolve_all();
        assert!(t.in_graveyard(P0, "Wistfulness"));
        // No blue mana was spent: no cards were drawn.
        assert_eq!(t.hand_size(P0), hand - 1);
    }
    assert_eq!(tested, 1);
}

#[test]
fn the_incarnations_check_the_mana_spent_on_their_total_cost() {
    cr!("601.2h", "702.74a");
    ruling!(
        "Wistfulness",
        "Wistfulness's first and second abilities care about what mana was spent to pay its total cost, not just what mana was spent to pay the hybrid mana symbols in its cost."
    );
    ruling!(
        "Wistfulness",
        "Wistfulness's first and second abilities check to see if at least two mana of the appropriate colors were spent to pay Wistfulness's cost."
    );
    // Cast for {3}{G/U}{G/U} with two Forests and three Islands: at least two green and two
    // blue mana were spent, so both abilities trigger.
    let mut t = TestGame::new(2);
    let stone = t.battlefield(P1, "Millstone");
    t.lands(P0, "Forest", 2);
    t.lands(P0, "Island", 3);
    let w = t.hand(P0, "Wistfulness");
    t.answer_targets(P0, &[Entity::Object(stone)]);
    t.cast(P0, w).go();
    let hand = t.hand_size(P0);
    t.resolve();
    assert_eq!(t.stack_len(), 2);
    t.resolve_all();
    assert!(!t.on_battlefield(stone));
    assert_eq!(t.hand_size(P0), hand + 1);
    assert_eq!(t.named_on_battlefield("Wistfulness").len(), 1);
    // With one Island and four Forests, only one blue mana was spent: the draw ability
    // doesn't trigger.
    let mut t = TestGame::new(2);
    t.battlefield(P1, "Millstone");
    t.lands(P0, "Forest", 4);
    t.lands(P0, "Island", 1);
    let w = t.hand(P0, "Wistfulness");
    t.cast(P0, w).go();
    t.resolve();
    assert_eq!(t.stack_len(), 1);
    // Put onto the battlefield without being cast, no mana was spent: neither triggers.
    let mut t = TestGame::new(2);
    t.battlefield(P1, "Millstone");
    enter(&mut t, P0, "Wistfulness");
    assert_eq!(t.stack_len(), 0);
}
