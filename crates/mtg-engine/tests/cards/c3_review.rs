//! Review regressions for cards3-costs-mana-replacements: Primeval Spawn's entry
//! replacement on copies and under Humility, Tazri's granted mana ability with cycling and
//! cards in other zones, and costs that must be paid in full (CR 118.3).

use mtg_engine::decision::Answer;
use mtg_engine::mana::ManaType;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

fn pool(t: &TestGame, p: PlayerId) -> usize {
    t.g.player(p).mana_pool.total()
}

fn tapped(t: &TestGame, ids: &[ObjectId]) -> usize {
    ids.iter().filter(|l| t.g.obj(**l).tapped).count()
}

#[test]
fn primeval_spawn_token_copy_is_exiled() {
    cr!("614.1a", "707.9");
    ruling!(
        "Primeval Spawn",
        "If a creature token that is a copy of Primeval Spawn would be put onto the battlefield, it’s put into exile instead"
    );
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let spawn = t.battlefield(P0, "Primeval Spawn");
    t.lands(P0, "Island", 3);
    let cc = t.hand(P0, "Cackling Counterpart");
    t.cast(P0, cc).target(spawn).go();
    t.resolve_all();
    let spawns =
        t.g.battlefield
            .iter()
            .filter(|o| t.g.obj(**o).chars.name == "Primeval Spawn")
            .count();
    assert_eq!(spawns, 1);
    assert!(t.in_graveyard(P0, "Cackling Counterpart"));
}

#[test]
fn primeval_spawn_copy_of_it_put_onto_the_battlefield_is_exiled() {
    cr!("614.1a", "707.9");
    ruling!(
        "Primeval Spawn",
        "If an effect causes a card to enter the battlefield as a copy of Primeval Spawn and that permanent wasn’t cast"
    );
    // A card that wasn't cast entering as a copy of Primeval Spawn has its ability.
    let mut t = TestGame::new(2);
    let spawn = t.battlefield(P0, "Primeval Spawn");
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[Entity::Object(spawn)]);
    let clone = t.g.create_card_object(card("Clone"), P0, Zone::Hand(P0));
    let moved = t.g.move_object(
        clone,
        Zone::Battlefield,
        mtg_engine::events::MoveCause::Effect,
        Some(P0),
    );
    let now = t.g.current(moved.unwrap_or(clone));
    assert_eq!(t.zone(now), Zone::Exile);
}

#[test]
fn tazri_mana_ability_counts_cycling() {
    cr!("602.1b", "702.29a");
    ruling!(
        "Tazri, Stalwart Survivor",
        "you could tap a creature with cycling and Tazri's ability for mana"
    );
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    t.battlefield(P0, "Tazri, Stalwart Survivor");
    // Street Wraith: swampwalk and cycling (a cost that can't be paid on the battlefield).
    let wraith = t.battlefield(P0, "Street Wraith");
    t.activate(P0, wraith, 0, &[]).unwrap();
    assert_eq!(t.g.player(P0).mana_pool.count(ManaType::B), 1);
}

#[test]
fn tazri_mana_cant_pay_for_abilities_of_creature_cards_in_other_zones() {
    cr!("106.6");
    ruling!(
        "Tazri, Stalwart Survivor",
        "It can't be spent to activate abilities of creature cards in other zones."
    );
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    t.battlefield(P0, "Tazri, Stalwart Survivor");
    let a = t.battlefield(P0, "Rootwalla");
    let b = t.battlefield(P0, "Rootwalla");
    t.activate(P0, a, 1, &[]).unwrap();
    t.activate(P0, b, 1, &[]).unwrap();
    assert_eq!(pool(&t, P0), 2);
    // Hollow One's cycling {2}: a creature card's ability, but not a creature's.
    let hollow = t.hand(P0, "Hollow One");
    let uid =
        t.g.obj(hollow)
            .chars
            .abilities
            .iter()
            .find(|a| a.text == "Cycling")
            .map(|a| a.uid)
            .expect("Hollow One has cycling");
    t.g.turn.priority = Some(P0);
    assert!(t.g.activate_ability(P0, hollow, uid).is_err());
    assert_eq!(pool(&t, P0), 2);
    assert!(t.in_hand(P0, "Hollow One"));
    // Other mana pays it.
    t.lands(P0, "Wastes", 2);
    t.g.turn.priority = Some(P0);
    assert!(t.g.activate_ability(P0, hollow, uid).is_ok());
    assert!(t.in_graveyard(P0, "Hollow One"));
}

#[test]
fn aether_spike_controller_must_pay_the_full_amount() {
    cr!("118.3", "118.12");
    // Two {E} paid: one more Forest than the Bears needed isn't enough to pay {2}.
    let mut t = TestGame::new(2);
    t.set_step(P1, Step::PrecombatMain);
    t.lands(P1, "Forest", 2);
    let bears = t.hand(P1, "Grizzly Bears");
    t.cast(P1, bears).go();
    let extra = t.lands(P1, "Forest", 1);
    t.lands(P0, "Island", 2);
    let spike = t.hand(P0, "Aether Spike");
    let spell = t.g.current(bears);
    t.cast(P0, spike).target(spell).go();
    t.answer(P0, DecisionKind::X, Answer::Number(2));
    t.answer_yes(P1, true);
    t.resolve_all();
    assert!(t.in_graveyard(P1, "Grizzly Bears"));
    assert_eq!(tapped(&t, &extra), 0);
    // With two Forests, both are tapped to pay it.
    let mut t = TestGame::new(2);
    t.set_step(P1, Step::PrecombatMain);
    t.lands(P1, "Forest", 2);
    let bears = t.hand(P1, "Grizzly Bears");
    t.cast(P1, bears).go();
    let extra = t.lands(P1, "Forest", 2);
    t.lands(P0, "Island", 2);
    let spike = t.hand(P0, "Aether Spike");
    let spell = t.g.current(bears);
    t.cast(P0, spike).target(spell).go();
    t.answer(P0, DecisionKind::X, Answer::Number(2));
    t.answer_yes(P1, true);
    t.resolve_all();
    assert!(t.on_battlefield(t.g.current(bears)));
    assert_eq!(tapped(&t, &extra), 2);
}

#[test]
fn taste_of_paradise_additional_costs_are_paid() {
    cr!("601.2b", "118.3");
    // Two additional {1}{G}: all eight Forests are tapped.
    let mut t = TestGame::new(2);
    let forests = t.lands(P0, "Forest", 8);
    let taste = t.hand(P0, "Taste of Paradise");
    t.answer(P0, DecisionKind::OptionalCost, Answer::Number(2));
    t.cast(P0, taste).go();
    assert_eq!(tapped(&t, &forests), 8);
    t.resolve_all();
    assert_eq!(t.life(P0), 29);
}

#[test]
fn errant_minion_mana_paid_is_spent() {
    cr!("118.3", "615.7");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let minion = t.battlefield(P0, "Errant Minion");
    t.g.attach(minion, Entity::Object(bears));
    let islands = t.lands(P1, "Island", 3);
    t.answer(P1, DecisionKind::X, Answer::Number(2));
    t.advance_to(P1, Step::Upkeep);
    t.resolve_all();
    assert_eq!(tapped(&t, &islands), 2);
    assert_eq!(t.life(P1), 20);
}

#[test]
fn primeval_spawn_under_humility_enters() {
    cr!("614.12");
    ruling!(
        "Primeval Spawn",
        "then its first ability won’t cause it to be exiled"
    );
    let mut t = TestGame::new(2);
    t.battlefield(P1, "Humility");
    let spawn = t.enter(P0, "Primeval Spawn");
    assert!(t.on_battlefield(spawn));
}

/// Paper cards with MTGO-only Oracle printings (Masters Edition IV, Tempest Remastered)
/// whose texts this item's redirection grammar compiles.
#[test]
fn jade_monolith_redirects_the_chosen_sources_next_damage_to_you() {
    cr!("614.9", "609.7a");
    let mut t = TestGame::new(2);
    let monolith = t.battlefield(P0, "Jade Monolith");
    let giant = t.battlefield(P0, "Hill Giant");
    t.lands(P0, "Wastes", 1);
    t.lands(P1, "Mountain", 1);
    let bolt = t.hand(P1, "Lightning Bolt");
    let spell = t.cast(P1, bolt).target(giant).go();
    t.answer_choose(P0, &[Entity::Object(spell)]);
    t.activate(P0, monolith, 0, &[Entity::Object(giant)])
        .unwrap();
    t.resolve_all();
    assert!(t.on_battlefield(giant));
    assert_eq!(t.g.obj(giant).damage, 0);
    assert_eq!(t.life(P0), 17);
    assert_eq!(t.life(P1), 20);
}

#[test]
fn soltari_guerrillas_redirects_its_next_combat_damage_to_a_creature() {
    cr!("614.9", "510.2");
    let mut t = TestGame::new(2);
    let soltari = t.battlefield(P0, "Soltari Guerrillas");
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.set_step(P0, Step::PrecombatMain);
    t.activate(P0, soltari, 0, &[Entity::Object(bears)])
        .unwrap();
    t.resolve();
    t.set_step(P0, Step::BeginningOfCombat);
    t.attack(&[(soltari, Entity::Player(P1))], &[]);
    assert_eq!(t.life(P1), 20);
    assert!(t.in_graveyard(P1, "Grizzly Bears"));
}

#[test]
fn pulse_of_llanowar_one_color_for_all_the_mana_chosen_each_time() {
    cr!("106.12b", "616.1");
    ruling!(
        "Pulse of Llanowar",
        "It just makes all the mana generated by the basic land be of a single color of your choice."
    );
    ruling!(
        "Pulse of Llanowar",
        "You can choose a different color each time a basic land is tapped."
    );
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    t.battlefield(P0, "Pulse of Llanowar");
    t.battlefield(P0, "Mana Reflection");
    let a = t.battlefield(P0, "Plains");
    let b = t.battlefield(P0, "Plains");
    // Each tap: the order of the two replacements (if asked), then the color.
    let pick = |t: &mut TestGame, ty: ManaType| {
        let i = [
            ManaType::W,
            ManaType::U,
            ManaType::B,
            ManaType::R,
            ManaType::G,
        ]
        .iter()
        .position(|x| *x == ty)
        .unwrap();
        t.answer(P0, DecisionKind::Option, Answer::Index(i));
    };
    pick(&mut t, ManaType::G);
    t.activate(P0, a, 0, &[]).unwrap();
    assert_eq!(t.g.player(P0).mana_pool.count(ManaType::G), 2);
    assert_eq!(pool(&t, P0), 2);
    pick(&mut t, ManaType::U);
    t.activate(P0, b, 0, &[]).unwrap();
    assert_eq!(t.g.player(P0).mana_pool.count(ManaType::U), 2);
    assert_eq!(pool(&t, P0), 4);
}

#[test]
fn stunning_reversal_applies_to_any_loss_and_you_may_lose_right_after() {
    cr!("104.3c", "614.1a", "704.5b");
    ruling!(
        "Stunning Reversal",
        "Stunning Reversal’s effect applies any time you would lose the game, even if you’re not losing due to your life total being 0 or less."
    );
    ruling!(
        "Stunning Reversal",
        "If you have fewer than seven cards in your library, you’ll lose the game immediately after applying"
    );
    // Drawing from an empty library: replaced; three cards are left for the seven draws.
    let mut t = TestGame::new(2);
    t.lands(P0, "Swamp", 4);
    let sr = t.hand(P0, "Stunning Reversal");
    t.cast(P0, sr).go();
    t.resolve_all();
    t.g.players[P0.idx()].library.clear();
    for _ in 0..3 {
        t.library_top(P0, "Forest");
    }
    t.g.players[P0.idx()].life = 5;
    let hand = t.hand_size(P0);
    // Four draws: the fourth is from an empty library.
    t.g.draw_cards(P0, 4);
    t.settle();
    // The loss was replaced (life 1), then the seven draws emptied the library again:
    // P0 loses at the next check.
    assert_eq!(t.life(P0), 1);
    assert!(t.has_lost(P0));
    assert_eq!(t.hand_size(P0), hand + 3);
}
