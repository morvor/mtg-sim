//! Rulings batch P205 — delve (CR 702.66), detain (CR 701.35), dethrone (CR 702.105),
//! and domain (an ability word, CR 207.2c).

use crate::r_s01_common::*;
use crate::r_s02_common::can_attack;
use crate::r_s06_common::give_control;
use crate::r_s20_common::to_beginning_of_combat;
use mtg_engine::ability::{Effect, PlayerRef, Value};
use mtg_engine::decision::{Answer, Decision};
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::counters;
use mtg_engine::*;

// ---------------------------------------------------------------------------
// Delve
// ---------------------------------------------------------------------------

/// The delve choices P0 was offered since decision `from`: (candidates, maximum).
fn delve_offers(t: &TestGame, from: usize) -> Vec<(usize, u32)> {
    t.asked()[from..]
        .iter()
        .filter(|(q, _)| *q == P0)
        .filter_map(|(_, d)| match d {
            Decision::ChooseEntities {
                prompt,
                candidates,
                max,
                ..
            } if prompt.contains("delve") => Some((candidates.len(), *max)),
            _ => None,
        })
        .collect()
}

/// P0 casts `name` with eight cards in the graveyard, exiling `exile` of them for delve
/// and paying the rest with `lands` Islands. Returns the spell and the delve offers.
fn cast_with_delve(
    t: &mut TestGame,
    name: &str,
    exile: usize,
    lands: usize,
) -> (ObjectId, Vec<(usize, u32)>) {
    supported(name);
    let cards: Vec<Entity> = (0..8)
        .map(|_| Entity::Object(t.graveyard(P0, "Grizzly Bears")))
        .collect();
    t.lands(P0, "Island", lands);
    let card = t.hand(P0, name);
    t.answer_choose(P0, &cards[..exile]);
    t.answer_targets(P0, &[Entity::Player(P1)]);
    let from = t.asked().len();
    let spell = t.cast(P0, card).go();
    (spell, delve_offers(t, from))
}

#[test]
fn delve_doesnt_change_mana_cost_or_mana_value() {
    cr!("702.66a", "202.3", "601.2f");
    ruling!(
        "Murktide Regent",
        "Delve doesn't change a spell's mana cost or mana value. For example, Murktide Regent's mana value is 7 even if you exiled three cards to cast it."
    );
    ruling!(
        "Sorcerous Squall",
        "Delve doesn't change a spell's mana cost or mana value. For example, Sorcerous Squall's mana value is 9 even if you exiled three cards to cast it."
    );
    for (name, mv, lands) in [("Murktide Regent", 7, 4), ("Sorcerous Squall", 9, 6)] {
        let mut t = TestGame::new(2);
        let (spell, _) = cast_with_delve(&mut t, name, 3, lands);
        assert_eq!(t.graveyard_size(P0), 5, "{name}");
        assert_eq!(
            crate::r_s04_common::untapped_lands(&t, P0),
            0,
            "{name}"
        );
        assert_eq!(t.obj(spell).chars.mana_value(), mv, "{name}");
    }
}

#[test]
fn delve_pays_only_generic_mana_up_to_the_generic_requirement() {
    cr!("702.66a", "702.66b", "601.2f");
    ruling!(
        "Murktide Regent",
        "You can exile cards to pay only for generic mana, and you can't exile more cards than the generic mana requirement of a spell with delve. For example, you can't exile more than five cards from your graveyard to cast Murktide Regent unless an effect has increased its cost."
    );
    ruling!(
        "Sorcerous Squall",
        "You can exile cards to pay only for generic mana, and you can't exile more cards than the generic mana requirement of a spell with delve. For example, you can't exile more than six cards from your graveyard to cast Sorcerous Squall unless an effect has increased its cost."
    );
    for (name, generic, colored) in [("Murktide Regent", 5, 2), ("Sorcerous Squall", 6, 3)] {
        // Exiling the most it can: only the colored mana is left to pay.
        let mut t = TestGame::new(2);
        let (_, offers) = cast_with_delve(&mut t, name, 8, colored);
        assert_eq!(offers, vec![(8, generic)], "{name}");
        assert_eq!(t.graveyard_size(P0), 8 - generic as usize, "{name}");
        // Without the colored mana, it can't be cast at all.
        let mut t = TestGame::new(2);
        (0..8).for_each(|_| {
            t.graveyard(P0, "Grizzly Bears");
        });
        t.lands(P0, "Island", colored - 1);
        let card = t.hand(P0, name);
        t.answer_targets(P0, &[Entity::Player(P1)]);
        assert!(t.cast(P0, card).try_go().is_err(), "{name}");
        assert_eq!(t.graveyard_size(P0), 8, "{name}");
    }
}

#[test]
fn delve_can_pay_for_generic_mana_added_by_a_cost_increase() {
    cr!("702.66a", "702.66b", "601.2f");
    ruling!(
        "Sorcerous Squall",
        "You can exile cards to pay only for generic mana, and you can't exile more cards than the generic mana requirement of a spell with delve. For example, you can't exile more than six cards from your graveyard to cast Sorcerous Squall unless an effect has increased its cost."
    );
    // Thalia, Guardian of Thraben: noncreature spells cost {1} more. The total cost is
    // {7}{U}{U}{U}, so delve can pay for seven.
    supported("Thalia, Guardian of Thraben");
    let mut t = TestGame::new(2);
    t.battlefield(P1, "Thalia, Guardian of Thraben");
    let (_, offers) = cast_with_delve(&mut t, "Sorcerous Squall", 8, 3);
    assert_eq!(offers, vec![(8, 7)]);
    assert_eq!(t.graveyard_size(P0), 1);
}

// ---------------------------------------------------------------------------
// Detain
// ---------------------------------------------------------------------------

#[test]
fn lavinia_detains_only_what_opponents_control_as_it_resolves() {
    cr!("701.35a", "611.2c");
    ruling!(
        "Lavinia of the Tenth",
        "Lavinia's ability doesn't affect permanents an opponent gains control of after its enters-the-battlefield ability has resolved. Similarly, if you gain control of a detained permanent after that ability resolves, it will continue to be detained until your next turn."
    );
    supported("Lavinia of the Tenth");
    let mut t = TestGame::new(2);
    let theirs = t.battlefield(P1, "Grizzly Bears");
    let mine = t.battlefield(P0, "Hill Giant");
    let attacker = t.battlefield(P0, "Raging Goblin");
    t.enter(P0, "Lavinia of the Tenth");
    t.resolve_all();
    // P1 gains control of P0's Hill Giant afterward: it isn't detained.
    give_control(&mut t, mine, P1);
    to_beginning_of_combat(&mut t, P0);
    attack_with(&mut t, &[(attacker, Entity::Player(P1))]);
    assert!(t.g.can_block(mine, attacker));
    assert!(!t.g.can_block(theirs, attacker));
    block_and_finish(&mut t, P1, &[]);
    // P0 gains control of the detained Bears: it stays detained during P1's turn.
    give_control(&mut t, theirs, P0);
    let wurm = t.battlefield(P1, "Craw Wurm");
    to_beginning_of_combat(&mut t, P1);
    attack_with(&mut t, &[(wurm, Entity::Player(P0))]);
    assert!(!t.g.can_block(theirs, wurm));
}

#[test]
fn archon_of_the_triumvirate_detains_before_blockers_are_declared() {
    cr!("701.35a", "508.1m", "509.1a");
    ruling!(
        "Archon of the Triumvirate",
        "The nonland permanents will be detained before blockers are chosen. Notably, a creature detained this way can’t block before it is detained."
    );
    supported("Archon of the Triumvirate");
    let mut t = TestGame::new(2);
    let archon = t.battlefield(P0, "Archon of the Triumvirate");
    let spider = t.battlefield(P1, "Giant Spider");
    t.answer_targets(P0, &[Entity::Object(spider)]);
    to_beginning_of_combat(&mut t, P0);
    attack_with(&mut t, &[(archon, Entity::Player(P1))]);
    assert_eq!(triggers_on_stack(&t, "detain"), 1);
    t.resolve_all();
    assert!(!t.g.can_block(spider, archon));
    block_and_finish(&mut t, P1, &[]);
    assert_eq!(t.life(P1), 16);
}

#[test]
fn a_detain_lasts_until_the_leaving_players_next_turn_would_have_begun() {
    cr!("701.35a", "800.4m");
    ruling!(
        "Tax Collector",
        "When a player leaves a multiplayer game, any continuous effects with durations that last until that player’s next turn or until a specific point in that turn (such as the effect of being detained) will last until that turn would have begun. They neither expire immediately nor last indefinitely."
    );
    supported("Tax Collector");
    let mut t = TestGame::new(3);
    let bears = t.battlefield(P1, "Grizzly Bears");
    // Arrest: detain target creature an opponent controls.
    t.answer(P0, DecisionKind::Modes, Answer::Indices(vec![1]));
    t.answer_targets(P0, &[Entity::Object(bears)]);
    t.enter(P0, "Tax Collector");
    t.resolve_all();
    // P0 leaves the game during P1's upkeep: the Bears are still detained.
    t.advance_to(P1, Step::Upkeep);
    t.g.player_loses(P0);
    t.settle();
    assert!(!t.g.player(P0).in_game());
    t.advance_to(P1, Step::BeginningOfCombat);
    assert!(!can_attack(&mut t, bears));
    // Still during P2's turn; P0's next turn would have begun before P1's next turn.
    t.advance_to(P2, Step::BeginningOfCombat);
    assert!(!t.g.can_block_at_all(bears));
    t.advance_to(P1, Step::BeginningOfCombat);
    assert!(can_attack(&mut t, bears));
}

// ---------------------------------------------------------------------------
// Dethrone
// ---------------------------------------------------------------------------

#[test]
fn multiple_instances_of_dethrone_trigger_separately() {
    cr!("702.105a", "702.105b");
    ruling!(
        "Marchesa, the Black Rose",
        "If a creature has multiple instances of dethrone, each triggers separately."
    );
    supported("Marchesa, the Black Rose");
    supported("Marchesa's Emissary");
    // Marchesa's Emissary has dethrone; Marchesa gives it another.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Marchesa, the Black Rose");
    let emissary = t.battlefield(P0, "Marchesa's Emissary");
    to_beginning_of_combat(&mut t, P0);
    attack_with(&mut t, &[(emissary, Entity::Player(P1))]);
    assert_eq!(triggers_on_stack(&t, "Dethrone"), 2);
    t.resolve_all();
    assert_eq!(t.counters(emissary, counters::PLUS1), 2);
}

#[test]
fn scourge_of_the_throne_rechecks_its_intervening_if_but_dethrone_doesnt() {
    cr!("603.4", "702.105a", "506.4");
    ruling!(
        "Scourge of the Throne",
        "Scourge of the Throne's ability has an intervening \"if\" clause. It must be attacking the player with the most life or tied with the most life both when it's declared as an attacker and as it starts to resolve for it to have any effect."
    );
    supported("Scourge of the Throne");
    for lose_life in [false, true] {
        let mut t = TestGame::new(2);
        let scourge = t.battlefield(P0, "Scourge of the Throne");
        to_beginning_of_combat(&mut t, P0);
        attack_with(&mut t, &[(scourge, Entity::Player(P1))]);
        assert_eq!(triggers_on_stack(&t, "untap all attacking creatures"), 1);
        assert_eq!(triggers_on_stack(&t, "Dethrone"), 1);
        if lose_life {
            // P1 is no longer the player with the most life.
            let mut ctx = mtg_engine::eval::Ctx::new(None, P0);
            t.g.exec(
                &Effect::LoseLife {
                    who: PlayerRef::Player(P1),
                    n: Value::Const(5),
                },
                &mut ctx,
            );
            t.settle();
        }
        t.resolve_all();
        // Dethrone checked only as it triggered: the counter either way.
        assert_eq!(t.counters(scourge, counters::PLUS1), 1);
        // The other ability does nothing if P1 isn't tied for the most life any more.
        assert_eq!(t.obj(scourge).tapped, lose_life, "lost life: {lose_life}");
    }
}

// ---------------------------------------------------------------------------
// Domain
// ---------------------------------------------------------------------------

#[test]
fn leyline_bindings_domain_reduction_doesnt_change_its_mana_value() {
    cr!("207.2c", "202.3", "601.2f");
    ruling!(
        "Leyline Binding",
        "Leyline Binding's domain ability doesn't change its mana value, which is always 6."
    );
    supported("Leyline Binding");
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P1, "Hill Giant");
    // Five basic land types: it costs {W}.
    for land in ["Plains", "Island", "Swamp", "Mountain", "Forest"] {
        t.battlefield(P0, land);
    }
    let card = t.hand(P0, "Leyline Binding");
    let spell = t.cast(P0, card).go();
    assert_eq!(crate::r_s04_common::untapped_lands(&t, P0), 4);
    assert_eq!(t.obj(spell).chars.mana_value(), 6);
    t.answer_targets(P0, &[Entity::Object(giant)]);
    t.resolve_all();
    assert_eq!(t.zone(giant), mtg_engine::object::Zone::Exile);
}

/// Taps P0's Prismatic Geoscope for mana; the amount added.
fn geoscope_mana(t: &mut TestGame) -> u32 {
    supported("Prismatic Geoscope");
    let scope = t.battlefield(P0, "Prismatic Geoscope");
    assert!(crate::r_s20_common::tap_for_mana(t, P0, scope, "Add X"));
    assert_eq!(t.stack_len(), 0);
    t.g.player(P0).mana_pool.total() as u32
}

#[test]
fn domain_counts_basic_land_types_not_lands() {
    cr!("207.2c", "305.6");
    ruling!(
        "Prismatic Geoscope",
        "How many lands you control of a particular basic land type is irrelevant to a domain ability, as long as that number is greater than zero. As far as domain is concerned, ten Forests are the same as one Forest."
    );
    let mut t = TestGame::new(2);
    t.lands(P0, "Forest", 10);
    assert_eq!(geoscope_mana(&mut t), 1);
    let mut t = TestGame::new(2);
    t.lands(P0, "Forest", 10);
    t.battlefield(P0, "Island");
    assert_eq!(geoscope_mana(&mut t), 2);
}

#[test]
fn domain_counts_basic_land_types_of_nonbasic_lands() {
    cr!("207.2c", "305.6", "305.8");
    ruling!(
        "Prismatic Geoscope",
        "Some nonbasic lands do have basic land types. Domain abilities don't count the number of lands you control—they count the number of basic land types among lands you control, even if that means checking the same land twice. For example, if you control a Tundra, a Blood Crypt, and a Forest, you'll have a Plains, Island, Swamp, Mountain, and Forest among the lands you control. Your domain abilities will be maxed out."
    );
    let mut t = TestGame::new(2);
    for land in ["Tundra", "Blood Crypt", "Forest"] {
        t.battlefield(P0, land);
    }
    assert_eq!(geoscope_mana(&mut t), 5);
}

// ---------------------------------------------------------------------------
// Delirium: Demonic Counsel
// ---------------------------------------------------------------------------

#[test]
fn demonic_counsels_delirium_search_doesnt_reveal() {
    cr!("701.23e", "207.2c", "608.2c");
    ruling!(
        "Demonic Counsel",
        "In the case where Demonic Counsel's delirium ability allows you to search your library for any card, you won't have to reveal that card."
    );
    supported("Demonic Counsel");
    let reveals = |t: &TestGame| {
        t.g.turn_events
            .iter()
            .filter(|e| {
                matches!(e, mtg_engine::events::Event::Custom { name, .. }
                    if name == mtg_engine::reveal::REVEALED)
            })
            .count()
    };
    // With delirium: any card (Hill Giant), not revealed.
    let mut t = TestGame::new(2);
    bury(&mut t, P0, &["Forest", "Grizzly Bears", "Mind Stone", "Lightning Bolt"]);
    let giant = t.library_top(P0, "Hill Giant");
    t.library_top(P0, "Lord of the Pit");
    give_mana_for(&mut t, P0, "Demonic Counsel");
    let card = t.hand(P0, "Demonic Counsel");
    t.answer_choose(P0, &[Entity::Object(giant)]);
    t.cast(P0, card).go();
    t.resolve_all();
    assert!(t.in_hand(P0, "Hill Giant"));
    assert_eq!(t.hand_size(P0), 1);
    assert_eq!(reveals(&t), 0);
    // Without: only a Demon, and it's revealed.
    let mut t = TestGame::new(2);
    let giant = t.library_top(P0, "Hill Giant");
    t.library_top(P0, "Lord of the Pit");
    give_mana_for(&mut t, P0, "Demonic Counsel");
    let card = t.hand(P0, "Demonic Counsel");
    t.answer_choose(P0, &[Entity::Object(giant)]);
    t.cast(P0, card).go();
    t.resolve_all();
    assert!(t.in_hand(P0, "Lord of the Pit"));
    assert!(!t.in_hand(P0, "Hill Giant"));
    assert_eq!(reveals(&t), 1);

}

fn bury(t: &mut TestGame, p: PlayerId, names: &[&str]) {
    for n in names {
        t.graveyard(p, n);
    }
}
