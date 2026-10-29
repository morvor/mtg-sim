//! cards4-referents-statics, families `instead-alternatives` and `excess-damage`.

use mtg_engine::decision::Answer;
use mtg_engine::testing::*;
use mtg_engine::*;

fn supported(name: &str) {
    let def = card(name);
    assert!(
        def.unsupported_text().is_empty(),
        "{name} has unsupported text: {:?}",
        def.unsupported_text()
    );
}

fn vigil(creature_died: bool, to_battlefield: bool) -> (TestGame, ObjectId) {
    let mut t = TestGame::new(2);
    let forest = t.library_top(P0, "Forest");
    if creature_died {
        let bears = t.battlefield(P1, "Grizzly Bears");
        t.g.destroy(bears, None);
        t.settle();
    }
    t.lands(P0, "Forest", 1);
    let spell = t.hand(P0, "Caravan Vigil");
    t.answer_choose(P0, &[Entity::Object(forest)]);
    t.answer(
        P0,
        DecisionKind::Option,
        Answer::Index(if to_battlefield { 1 } else { 0 }),
    );
    t.cast(P0, spell).go();
    t.resolve_all();
    (t, forest)
}

#[test]
fn caravan_vigil_puts_the_land_onto_the_battlefield_only_with_morbid() {
    cr!("608.2c", "701.23a");
    supported("Caravan Vigil");
    // A creature died: you may put it onto the battlefield.
    let (t, _) = vigil(true, true);
    assert_eq!(t.named_on_battlefield("Forest").len(), 2, "{}", t.dump_log());
    // No creature died: it goes to your hand (no choice).
    let (t, _) = vigil(false, true);
    assert_eq!(t.named_on_battlefield("Forest").len(), 1);
    assert!(t.in_hand(P0, "Forest"));
}

#[test]
fn caravan_vigil_may_still_put_the_land_into_your_hand() {
    cr!("608.2c");
    ruling!(
        "Caravan Vigil",
        "You can choose to put the basic land card into your hand even if a creature died"
    );
    let (t, _) = vigil(true, false);
    assert_eq!(t.named_on_battlefield("Forest").len(), 1);
    assert!(t.in_hand(P0, "Forest"));
}

fn agni_kai(victim: &str) -> TestGame {
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P0, "Hill Giant");
    let v = t.battlefield(P1, victim);
    t.lands(P0, "Mountain", 2);
    let spell = t.hand(P0, "The Last Agni Kai");
    t.cast(P0, spell)
        .targets(&[Entity::Object(giant), Entity::Object(v)])
        .go();
    t.resolve_all();
    t
}

#[test]
fn the_last_agni_kai_adds_red_mana_equal_to_the_excess_fight_damage() {
    cr!("120.10", "701.14a");
    supported("The Last Agni Kai");
    // Hill Giant (3 power) fights a 1/1: 2 excess damage, {R}{R}.
    let t = agni_kai("Llanowar Elves");
    assert_eq!(t.g.player(P0).mana_pool.total(), 2, "{}", t.dump_log());
    // ... a 2/2: 1 excess.
    let t = agni_kai("Grizzly Bears");
    assert_eq!(t.g.player(P0).mana_pool.total(), 1);
    // ... a 0/4: none.
    let t = agni_kai("Wall of Wood");
    assert_eq!(t.g.player(P0).mana_pool.total(), 0);
}

#[test]
fn overclocked_electromancer_gets_energy_equal_to_the_excess_damage() {
    cr!("120.10", "122.1");
    interp!("that-excess-damage-this-turn");
    supported("Overclocked Electromancer");
    let mut t = TestGame::new(2);
    let em = t.battlefield(P0, "Overclocked Electromancer");
    let elves = t.battlefield(P1, "Llanowar Elves");
    // It attacks (power doubled to 4) and is blocked by a 1/1: 3 excess damage.
    t.attack(&[(em, Entity::Player(P1))], &[(elves, em)]);
    t.resolve_all();
    assert_eq!(
        t.g.player(P0).counters.get("energy").copied().unwrap_or(0),
        3,
        "{}",
        t.dump_log()
    );
}

#[test]
fn megatron_deals_the_excess_damage_to_the_creatures_controller_and_converts() {
    cr!("120.4a", "701.28a");
    supported("Megatron, Tyrant // Megatron, Destructive Force");
    let mut t = TestGame::new(2);
    let m = t.battlefield(P0, "Megatron, Tyrant // Megatron, Destructive Force");
    mtg_engine::dfc::transform(&mut t.g, m);
    t.g.dirty = true;
    let stone = t.battlefield(P0, "Mind Stone");
    let elves = t.battlefield(P1, "Llanowar Elves");
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[Entity::Object(stone)]);
    t.answer_targets(P0, &[Entity::Object(elves)]);
    t.attack(&[(m, Entity::Player(P1))], &[]);
    t.resolve_all();
    // 2 damage to a 1/1: 1 to it, the excess 1 to its controller; Megatron converts.
    assert!(!t.on_battlefield(elves), "{}", t.dump_log());
    assert!(!t.on_battlefield(stone));
    assert_eq!(t.obj_now(m).face, mtg_engine::object::FaceState::Front);
    let combat = t.pt(m).0;
    assert_eq!(t.life(P1), 20 - 1 - combat);
}

#[test]
fn megatron_without_excess_damage_doesnt_convert() {
    cr!("120.4a");
    let mut t = TestGame::new(2);
    let m = t.battlefield(P0, "Megatron, Tyrant // Megatron, Destructive Force");
    mtg_engine::dfc::transform(&mut t.g, m);
    t.g.dirty = true;
    let stone = t.battlefield(P0, "Mind Stone");
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[Entity::Object(stone)]);
    t.answer_targets(P0, &[Entity::Object(bears)]);
    t.attack(&[(m, Entity::Player(P1))], &[]);
    t.resolve_all();
    assert!(!t.on_battlefield(bears), "{}", t.dump_log());
    assert_eq!(t.obj_now(m).face, mtg_engine::object::FaceState::Back);
    assert_eq!(t.life(P1), 20 - t.pt(m).0);
}
