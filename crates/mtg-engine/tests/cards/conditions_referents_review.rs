//! Review tests for `conditions_referents`: intervening-if clauses about the source as it
//! last existed (CR 603.10a), and "it" in the effect after a condition about ~.

use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

fn supported(n: &str) {
    let u = card(n).unsupported_text().join(" | ");
    assert!(u.is_empty(), "{n} has unsupported text: {u}");
}

fn destroy(t: &mut TestGame, id: ObjectId) {
    t.g.destroy_all(vec![id], None, false);
    t.settle();
}

fn count_subtype(t: &TestGame, p: PlayerId, s: &str) -> usize {
    t.g.battlefield
        .iter()
        .filter(|id| t.g.obj(**id).controller == p && t.g.obj(**id).chars.has_subtype(s))
        .count()
}

#[test]
fn when_it_dies_if_it_wasnt_blocking() {
    cr!("603.4", "603.10a");
    // Guildsworn Prowler: "When ~ dies, if it wasn't blocking, draw a card."
    supported("Guildsworn Prowler");
    // Dies while not blocking: a card.
    let mut t = TestGame::new(2);
    let p = t.battlefield(P0, "Guildsworn Prowler");
    let before = t.hand_size(P0);
    destroy(&mut t, p);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), before + 1);
    // Dies blocking: no card.
    let mut t = TestGame::new(2);
    t.set_step(P1, Step::PrecombatMain);
    let wurm = t.battlefield(P1, "Craw Wurm");
    let p = t.battlefield(P0, "Guildsworn Prowler");
    let before = t.hand_size(P0);
    t.attack(&[(wurm, Entity::Player(P0))], &[(p, wurm)]);
    t.resolve_all();
    assert!(!t.on_battlefield(p));
    assert_eq!(t.hand_size(P0), before);
}

#[test]
fn when_it_dies_if_its_power_was_three_or_greater() {
    cr!("603.4", "603.10a");
    // Deathknell Berserker (2/2): "When ~ dies, if its power was 3 or greater, create a
    // 2/2 black Zombie Berserker creature token."
    supported("Deathknell Berserker");
    for (counters, tokens) in [(0, 0), (1, 1)] {
        let mut t = TestGame::new(2);
        let b = t.battlefield(P0, "Deathknell Berserker");
        if counters > 0 {
            t.g.add_counters(Entity::Object(b), "+1/+1", counters, None);
        }
        destroy(&mut t, b);
        t.resolve_all();
        assert_eq!(
            count_subtype(&t, P0, "Zombie"),
            tokens,
            "{counters} counters"
        );
    }
}

#[test]
fn when_it_dies_if_it_was_equipped() {
    cr!("603.4", "603.10a");
    // Gunner Conscript: "When ~ dies, if it was enchanted, create a Junk token. When ~
    // dies, if it was equipped, create a Junk token."
    supported("Gunner Conscript");
    for equipped in [false, true] {
        let mut t = TestGame::new(2);
        let g = t.battlefield(P0, "Gunner Conscript");
        if equipped {
            let e = t.battlefield(P0, "Bonesplitter");
            assert!(t.g.attach(e, Entity::Object(g)));
        }
        destroy(&mut t, g);
        t.resolve_all();
        assert_eq!(
            count_subtype(&t, P0, "Junk"),
            usize::from(equipped),
            "{equipped}"
        );
    }
}

#[test]
fn if_this_has_fewer_than_three_counters_put_one_on_it() {
    cr!("603.4");
    // Adaptive Training Post: "Whenever you cast an instant or sorcery spell, if ~ has
    // fewer than three charge counters on it, put a charge counter on it."
    supported("Adaptive Training Post");
    let mut t = TestGame::new(2);
    let post = t.battlefield(P0, "Adaptive Training Post");
    t.set_step(P0, Step::PrecombatMain);
    for i in 0..4u32 {
        t.lands(P0, "Mountain", 1);
        let bolt = t.hand(P0, "Lightning Bolt");
        t.cast(P0, bolt).target(P1).go();
        t.resolve_all();
        assert_eq!(
            t.counters(post, "charge"),
            (i + 1).min(3),
            "after {} spells",
            i + 1
        );
    }
}

#[test]
fn if_it_wasnt_attacking_uses_last_known_information() {
    cr!("608.2c", "608.2h");
    // Prophesied End: "Destroy target creature. If it wasn't attacking, its controller
    // draws a card."
    supported("Prophesied End");
    for attacking in [false, true] {
        let mut t = TestGame::new(2);
        t.set_step(P1, Step::BeginningOfCombat);
        let bears = t.battlefield(P1, "Grizzly Bears");
        if attacking {
            t.answer(
                P1,
                DecisionKind::Attackers,
                mtg_engine::decision::Answer::Attackers(vec![(bears, Entity::Player(P0))]),
            );
        }
        t.advance_to(P1, Step::DeclareAttackers);
        assert_eq!(t.g.is_attacking(bears), attacking);
        t.lands(P0, "Plains", 2);
        let s = t.hand(P0, "Prophesied End");
        let before = t.hand_size(P1);
        t.cast(P0, s).target(bears).go();
        t.resolve_all();
        assert!(!t.on_battlefield(bears));
        assert_eq!(
            t.hand_size(P1),
            before + usize::from(!attacking),
            "{attacking}"
        );
    }
}

#[test]
fn whenever_a_creature_deals_combat_damage_draw_if_it_has_a_counter_otherwise_counter() {
    cr!("510.3a", "608.2c");
    // Marcus, Mutant Mayor: "Whenever a creature you control deals combat damage to a
    // player, draw a card if that creature has a +1/+1 counter on it. If it doesn't, put a
    // +1/+1 counter on it."
    supported("Marcus, Mutant Mayor");
    for countered in [false, true] {
        let mut t = TestGame::new(2);
        t.set_step(P0, Step::PrecombatMain);
        t.battlefield(P0, "Marcus, Mutant Mayor");
        let bears = t.battlefield(P0, "Grizzly Bears");
        if countered {
            t.g.add_counters(Entity::Object(bears), "+1/+1", 1, None);
        }
        let before = t.hand_size(P0);
        t.attack(&[(bears, Entity::Player(P1))], &[]);
        t.resolve_all();
        assert_eq!(
            t.hand_size(P0),
            before + usize::from(countered),
            "{countered}"
        );
        assert_eq!(t.counters(bears, "+1/+1"), 1, "{countered}");
    }
}

#[test]
fn exile_it_and_if_it_doesnt_have_suspend_it_gains_suspend() {
    cr!("702.62a", "608.2c");
    // Suspend: "Exile target creature and put two time counters on it. If it doesn't have
    // suspend, it gains suspend."
    supported("Suspend");
    let mut t = TestGame::new(2);
    t.lands(P0, "Island", 1);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let s = t.hand(P0, "Suspend");
    t.cast(P0, s).target(bears).go();
    t.resolve_all();
    let exiled = t.g.current(bears);
    assert!(t.in_exile("Grizzly Bears"));
    assert_eq!(t.counters(exiled, "time"), 2);
    assert!(t
        .obj_now(exiled)
        .chars
        .has_keyword(mtg_engine::keywords::KeywordKind::Suspend));
}

#[test]
fn if_it_has_unearth_exile_it_then_return_that_card_to_its_owners_hand() {
    cr!("400.7j", "608.2c");
    // Meticulous Excavation: "{2}{W}: Return target permanent you control to its owner's
    // hand. If it has unearth, instead exile it, then return that card to its owner's
    // hand. Activate only during your turn."
    supported("Meticulous Excavation");
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    t.lands(P0, "Plains", 3);
    let me = t.battlefield(P0, "Meticulous Excavation");
    let h = t.battlefield(P0, "Hellspark Elemental");
    t.activate(P0, me, 0, &[Entity::Object(h)]).unwrap();
    t.resolve_all();
    assert!(t.in_hand(P0, "Hellspark Elemental"));
    assert!(!t.in_exile("Hellspark Elemental"));
}

#[test]
fn otherwise_after_two_conditions_about_the_exiled_card() {
    cr!("608.2c");
    ruling!(
        "Misfortune Teller",
        "you will create both a Rogue token and a Treasure token"
    );
    // Misfortune Teller: "Whenever ~ enters or deals combat damage to a player, exile
    // target card from a graveyard. If it was a creature card, create a 2/2 black Rogue
    // creature token. If it was a land card, create a Treasure token. Otherwise, you gain
    // 3 life."
    supported("Misfortune Teller");
    for (c, rogues, treasures, life) in [
        ("Grizzly Bears", 1, 0, 20),
        ("Forest", 0, 1, 20),
        ("Lightning Bolt", 0, 0, 23),
        ("Dryad Arbor", 1, 1, 20),
    ] {
        let mut t = TestGame::new(2);
        let card = t.graveyard(P1, c);
        t.answer_targets(P0, &[Entity::Object(card)]);
        t.enter(P0, "Misfortune Teller");
        t.resolve_all();
        assert!(t.in_exile(c), "{c}");
        assert_eq!(count_subtype(&t, P0, "Rogue"), rogues, "{c}");
        assert_eq!(count_subtype(&t, P0, "Treasure"), treasures, "{c}");
        assert_eq!(t.life(P0), life, "{c}");
    }
}
