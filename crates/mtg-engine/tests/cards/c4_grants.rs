//! cards4-referents-statics, family `granted-abilities`: abilities granted to exiled
//! cards, tokens and other permanents, with referents fixed as they're granted.

use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

fn supported(name: &str) {
    let def = card(name);
    assert!(
        def.unsupported_text().is_empty(),
        "{name} has unsupported text: {:?}",
        def.unsupported_text()
    );
}

#[test]
fn monk_class_exiled_card_can_be_cast_after_another_spell() {
    cr!("113.6b", "601.3");
    supported("Monk Class");
    let mut t = TestGame::new(2);
    let class = t.battlefield(P0, "Monk Class");
    t.g.obj_mut(class).class_level = 3;
    t.g.dirty = true;
    for _ in 0..3 {
        t.library_top(P0, "Island");
    }
    t.library_top(P0, "Shock");
    t.advance_to(P0, Step::Upkeep);
    t.resolve_all();
    let shock = t
        .g
        .exile
        .iter()
        .copied()
        .find(|o| t.g.obj(*o).chars.name.as_str() == "Shock")
        .expect("exiled");
    t.advance_to(P0, Step::PrecombatMain);
    t.lands(P0, "Mountain", 2);
    // Not before another spell this turn.
    assert!(t.cast(P0, shock).target(Entity::Player(P1)).try_go().is_err());
    let bolt = t.hand(P0, "Lightning Bolt");
    t.cast(P0, bolt).target(Entity::Player(P1)).go();
    t.resolve_all();
    t.cast(P0, shock).target(Entity::Player(P1)).go();
    t.resolve_all();
    assert_eq!(t.life(P1), 20 - 3 - 2, "{}", t.dump_log());
}

#[test]
fn glimpse_the_impossible_spawn_tokens_can_be_sacrificed_for_mana() {
    cr!("111.4", "605.1a");
    supported("Glimpse the Impossible");
    let mut t = TestGame::new(2);
    for _ in 0..3 {
        t.library_top(P0, "Island");
    }
    t.lands(P0, "Mountain", 3);
    let spell = t.hand(P0, "Glimpse the Impossible");
    t.cast(P0, spell).go();
    t.resolve_all();
    t.advance_to(P0, Step::End);
    t.resolve_all();
    let spawn: Vec<ObjectId> = t
        .g
        .battlefield
        .iter()
        .copied()
        .filter(|o| t.g.obj(*o).chars.has_subtype("Spawn"))
        .collect();
    assert_eq!(spawn.len(), 3, "{}", t.dump_log());
    let has_mana_ability = t.g.obj(spawn[0]).chars.abilities.iter().any(|a| {
        matches!(&a.kind, mtg_engine::ability::AbilityKind::Activated(x) if x.is_mana_ability)
    });
    assert!(has_mana_ability);
}

#[test]
fn hellish_rebuke_punishes_damage_to_its_caster_only() {
    cr!("603.2", "608.2h", "113.6");
    ruling!(
        "Hellish Rebuke",
        "Each will trigger only for the player who cast the Hellish Rebuke that granted it the ability."
    );
    supported("Hellish Rebuke");
    let mut t = TestGame::new(3);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let other = t.battlefield(P1, "Hill Giant");
    t.lands(P0, "Swamp", 3);
    let spell = t.hand(P0, "Hellish Rebuke");
    // P1's turn: P0 casts it, then the Bears attack P0 (who cast it) and the Hill Giant
    // attacks P2 (no trigger for that damage).
    t.advance_to(P1, Step::BeginningOfCombat);
    t.cast(P0, spell).go();
    t.resolve_all();
    t.attack(
        &[(bears, Entity::Player(P0)), (other, Entity::Player(P2))],
        &[],
    );
    t.resolve_all();
    assert!(!t.on_battlefield(bears), "{}", t.dump_log());
    assert!(t.on_battlefield(other));
    assert_eq!(t.life(P1), 20 - 2);
    assert_eq!(t.life(P0), 20 - 2);
    assert_eq!(t.life(P2), 20 - 3);
}

#[test]
fn timothar_bat_returns_the_card_exiled_for_it() {
    cr!("607.2a", "608.2h", "400.7");
    ruling!(
        "Timothar, Baron of Bats",
        "you return only that card to the battlefield, not any other card exiled by Timothar"
    );
    supported("Timothar, Baron of Bats");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Timothar, Baron of Bats");
    let a = t.battlefield(P0, "Vampire Nighthawk");
    let b = t.battlefield(P0, "Child of Night");
    t.lands(P0, "Swamp", 2);
    t.answer_yes(P0, true);
    t.g.destroy(a, None);
    t.settle();
    t.resolve_all();
    t.answer_yes(P0, true);
    t.g.destroy(b, None);
    t.settle();
    t.resolve_all();
    assert!(t.in_exile("Vampire Nighthawk"), "{}", t.dump_log());
    assert!(t.in_exile("Child of Night"));
    // The second Bat (created for Child of Night) deals combat damage.
    let bats: Vec<ObjectId> = t
        .g
        .battlefield
        .iter()
        .copied()
        .filter(|o| t.g.obj(*o).chars.has_subtype("Bat"))
        .collect();
    assert_eq!(bats.len(), 2);
    let second = *bats.iter().max_by_key(|o| o.0).expect("bat");
    t.g.obj_mut(second).summoning_sick = false;
    t.advance_to(P0, Step::BeginningOfCombat);
    t.attack(&[(second, Entity::Player(P1))], &[]);
    t.resolve_all();
    assert!(!t.on_battlefield(second));
    assert_eq!(t.named_on_battlefield("Child of Night").len(), 1, "{}", t.dump_log());
    assert!(t.in_exile("Vampire Nighthawk"));
}

#[test]
fn symbiote_spider_man_gives_its_other_abilities_to_the_new_host() {
    cr!("113.6", "602.2", "613.1f");
    supported("Symbiote Spider-Man");
    let mut t = TestGame::new(2);
    let sym = t.graveyard(P0, "Symbiote Spider-Man");
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.lands(P0, "Island", 3);
    for _ in 0..5 {
        t.library_top(P0, "Island");
    }
    t.set_step(P0, Step::PrecombatMain);
    t.activate(P0, sym, 0, &[Entity::Object(bears)]).expect("activates");
    t.resolve_all();
    assert!(t.in_exile("Symbiote Spider-Man"), "{}", t.dump_log());
    assert_eq!(t.counters(bears, "+1/+1"), 1);
    let hand = t.hand_size(P0);
    let gy = t.graveyard_size(P0);
    // 3 combat damage: look at three cards, one into the hand, the rest into the graveyard.
    t.attack(&[(bears, Entity::Player(P1))], &[]);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand + 1, "{}", t.dump_log());
    assert_eq!(t.graveyard_size(P0), gy + 2);
}
