//! Rulings batch P074 — lifelink on spells and life-gain interactions: instant and sorcery
//! spells with lifelink (Heartflame Duelist, Radiant Scrollwielder, Judith), "for each"
//! life gain as one event (Vito, Firesong and Sunspeaker), The Wind Crystal's doubling,
//! Everlasting Torment and life totals that "become" a number, and Two-Headed Giant.

use crate::r_s01_common::{give_mana_for, supported};
use crate::r_s02_common::target_candidates;
use crate::r_s06_common::damage;
use mtg_engine::decision::Answer;
use mtg_engine::game::GameConfig;
use mtg_engine::object::{CastMethod, Zone};
use mtg_engine::testing::*;
use mtg_engine::*;

fn pl(p: PlayerId) -> Entity {
    Entity::Player(p)
}

fn obj(id: ObjectId) -> Entity {
    Entity::Object(id)
}

/// P0 casts `name` from hand (with the mana it needs) with `targets`, and the stack
/// resolves.
fn cast_resolve(t: &mut TestGame, name: &str, targets: &[Entity]) {
    give_mana_for(t, P0, name);
    let c = t.hand(P0, name);
    let mut cb = t.cast(P0, c);
    for e in targets {
        cb = cb.target(*e);
    }
    cb.go();
    t.resolve_all();
}

#[test]
fn lifelink_spell_dealing_damage_to_you_leaves_life_unchanged() {
    cr!("702.15b", "120.3f", "119.9");
    ruling!(
        "Heartflame Duelist // Heartflame Slash",
        "If an instant or sorcery spell you control with lifelink deals damage to you, you gain and lose that much life simultaneously. In this case, your life total doesn't change."
    );
    ruling!(
        "Radiant Scrollwielder",
        "If an instant or sorcery spell you control with lifelink deals damage to you, you gain and lose that much life simultaneously. In this case, your life total doesn’t change."
    );
    supported("Heartflame Duelist // Heartflame Slash");
    supported("Radiant Scrollwielder");
    // Heartflame Duelist: its own Adventure, Heartflame Slash, aimed at you.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Heartflame Duelist // Heartflame Slash");
    t.lands(P0, "Mountain", 3);
    let slash = t.hand(P0, "Heartflame Duelist // Heartflame Slash");
    t.cast(P0, slash)
        .method(CastMethod::Half(1))
        .target(pl(P0))
        .go();
    t.resolve_all();
    assert_eq!(t.zone(slash), Zone::Exile);
    assert_eq!(t.life(P0), 20);
    // The same spell aimed at an opponent gains life (positive control).
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Heartflame Duelist // Heartflame Slash");
    t.lands(P0, "Mountain", 3);
    let slash = t.hand(P0, "Heartflame Duelist // Heartflame Slash");
    t.cast(P0, slash)
        .method(CastMethod::Half(1))
        .target(pl(P1))
        .go();
    t.resolve_all();
    assert_eq!((t.life(P0), t.life(P1)), (23, 17));
    // Radiant Scrollwielder: a Lightning Bolt to your own face; and a Flame Rift that
    // deals 4 damage to each player nets you 4 life (gain 8, lose 4 simultaneously).
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Radiant Scrollwielder");
    cast_resolve(&mut t, "Lightning Bolt", &[pl(P0)]);
    assert!(t.in_graveyard(P0, "Lightning Bolt"));
    assert_eq!(t.life(P0), 20);
    cast_resolve(&mut t, "Flame Rift", &[]);
    assert_eq!((t.life(P0), t.life(P1)), (24, 16));
}

#[test]
fn lifelink_spell_that_makes_creatures_fight_gains_no_life() {
    cr!("702.15b", "701.14a", "120.3f");
    ruling!(
        "Heartflame Duelist // Heartflame Slash",
        "Some instant and sorcery spells instruct other things to deal damage rather than deal damage themselves. In these cases, you won't gain life for that damage (unless the other things also have lifelink)."
    );
    ruling!(
        "Radiant Scrollwielder",
        "Some instant and sorcery spells instruct other things to deal damage rather than deal damage themselves. In these cases, you won’t gain life for that damage (unless the other things also have lifelink)."
    );
    ruling!(
        "Judith, Carnage Connoisseur",
        "Similarly, an instant or sorcery spell with lifelink must actually deal damage in order to cause its controller to gain life. If the spell instructs another object to deal damage, its controller won't gain life when that damage is dealt."
    );
    supported("Judith, Carnage Connoisseur");
    for source in [
        "Heartflame Duelist // Heartflame Slash",
        "Radiant Scrollwielder",
        "Judith, Carnage Connoisseur",
    ] {
        let mut t = TestGame::new(2);
        t.battlefield(P0, source);
        let bears = t.battlefield(P0, "Grizzly Bears");
        let elves = t.battlefield(P1, "Llanowar Elves");
        // Judith's mode "That spell gains deathtouch and lifelink".
        t.answer(P0, DecisionKind::Modes, Answer::Indices(vec![0]));
        cast_resolve(&mut t, "Prey Upon", &[obj(bears), obj(elves)]);
        assert!(t.in_graveyard(P1, "Llanowar Elves"), "{source}");
        assert_eq!(t.obj_now(bears).damage, 1, "{source}");
        assert_eq!(
            t.life(P0),
            20,
            "{source}: the fight's damage isn't the spell's"
        );
        // Positive control: a spell that deals the damage itself gains life.
        t.answer(P0, DecisionKind::Modes, Answer::Indices(vec![0]));
        cast_resolve(&mut t, "Lightning Bolt", &[pl(P1)]);
        assert_eq!(t.life(P0), 23, "{source}");
    }
}

#[test]
fn judith_trigger_resolves_first_even_if_the_spell_is_countered() {
    cr!("603.3", "601.2i", "701.6a");
    ruling!(
        "Judith, Carnage Connoisseur",
        "Judith, Carnage Connoisseur's triggered ability resolves before the spell that caused it to trigger. The ability will resolve even if that spell is countered."
    );
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Judith, Carnage Connoisseur");
    // The Imp token mode.
    t.answer(P0, DecisionKind::Modes, Answer::Indices(vec![1]));
    give_mana_for(&mut t, P0, "Lightning Bolt");
    let bolt = t.hand(P0, "Lightning Bolt");
    let spell = t.cast(P0, bolt).target(pl(P1)).go();
    t.settle();
    assert_eq!(t.stack_len(), 2);
    // P1 counters the Bolt in response to the trigger.
    give_mana_for(&mut t, P1, "Counterspell");
    let cs = t.hand(P1, "Counterspell");
    t.cast(P1, cs).target(obj(spell)).go();
    t.resolve();
    assert!(t.in_graveyard(P0, "Lightning Bolt"));
    assert_eq!(
        crate::r_s05_common::tokens_with_subtype(&t, P0, "Imp").len(),
        0
    );
    t.resolve_all();
    assert_eq!(
        crate::r_s05_common::tokens_with_subtype(&t, P0, "Imp").len(),
        1
    );
    assert_eq!(t.life(P1), 20);
}

#[test]
fn for_each_life_gain_is_one_event() {
    cr!("119.9", "603.2c");
    ruling!(
        "Vito, Thorn of the Dusk Rose",
        "If you gain an amount of life \"for each\" of something, that life is gained as one event and Vito's first ability triggers only once."
    );
    ruling!(
        "Firesong and Sunspeaker",
        "If you gain an amount of life \"for each\" of something, that life is gained as one event and Firesong and Sunspeaker's last ability triggers only once."
    );
    supported("Vito, Thorn of the Dusk Rose");
    supported("Firesong and Sunspeaker");
    // Festival of Trokin: "You gain 2 life for each creature you control." Three creatures.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Vito, Thorn of the Dusk Rose");
    t.battlefield(P0, "Grizzly Bears");
    t.battlefield(P0, "Llanowar Elves");
    t.answer_targets(P0, &[pl(P1)]);
    cast_resolve(&mut t, "Festival of Trokin", &[]);
    assert_eq!(t.life(P0), 26);
    assert_eq!(t.life(P1), 14, "one trigger for 6, not three for 2");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Firesong and Sunspeaker");
    t.battlefield(P0, "Grizzly Bears");
    give_mana_for(&mut t, P0, "Festival of Trokin");
    let f = t.hand(P0, "Festival of Trokin");
    t.cast(P0, f).go();
    t.answer_targets(P0, &[pl(P1)]);
    t.resolve();
    assert_eq!(t.life(P0), 24);
    assert_eq!(t.stack_len(), 1, "Firesong triggered once");
    t.resolve_all();
    assert_eq!(t.life(P1), 17);
}

#[test]
fn firesong_cant_target_planeswalkers_or_battles() {
    cr!("115.1", "603.3d");
    ruling!(
        "Firesong and Sunspeaker",
        "The last ability of Firesong and Sunspeaker can't target a planeswalker or battle."
    );
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Firesong and Sunspeaker");
    let jace = t.battlefield(P1, "Jace Beleren");
    let bears = t.battlefield(P1, "Grizzly Bears");
    let from = t.asked().len();
    t.answer_targets(P0, &[obj(bears)]);
    cast_resolve(&mut t, "Festival of Trokin", &[]);
    let cands = target_candidates(&t, P0, from);
    assert_eq!(cands.len(), 1);
    assert!(cands[0].contains(&obj(bears)) && cands[0].contains(&pl(P1)));
    assert!(!cands[0].contains(&obj(jace)));
    assert!(!t.on_battlefield(bears));
}

#[test]
fn vito_and_exquisite_blood_loop_until_a_player_wins() {
    cr!("119.9", "104.4b", "603.2");
    ruling!(
        "Vito, Thorn of the Dusk Rose",
        "If an ability triggers whenever an opponent loses life and causes you to gain life, such as the ability of Exquisite Blood, this will loop until either you win the game or a player takes an action to break the loop."
    );
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Vito, Thorn of the Dusk Rose");
    t.battlefield(P0, "Exquisite Blood");
    t.g.players[P1.idx()].life = 5;
    t.g.gain_life(P0, 1);
    t.g.flush_events();
    t.resolve_all();
    assert!(t.has_lost(P1));
    assert_eq!(t.life(P0), 25);
}

#[test]
fn the_wind_crystal_doubles_life_from_a_life_total_becoming_higher() {
    cr!("119.5", "614.1a");
    ruling!(
        "The Wind Crystal",
        "If an effect would set your life total to a specific number that's higher than your current life total, that effect causes you to gain life equal to the difference."
    );
    supported("The Wind Crystal");
    supported("Blessed Wind");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "The Wind Crystal");
    t.g.players[P0.idx()].life = 3;
    cast_resolve(&mut t, "Blessed Wind", &[pl(P0)]);
    // 17 life gained, doubled to 34.
    assert_eq!(t.life(P0), 37);
}

#[test]
fn everlasting_torment_stops_life_total_becoming_higher_but_not_lower() {
    cr!("119.5", "119.7");
    ruling!(
        "Everlasting Torment",
        "If an effect says to set your life total to a certain number, and that number is higher than your current life total, that effect will normally cause you to gain life equal to the difference. With Everlasting Torment on the battlefield, that part of the effect won't do anything."
    );
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Everlasting Torment");
    t.g.players[P0.idx()].life = 3;
    cast_resolve(&mut t, "Blessed Wind", &[pl(P0)]);
    assert_eq!(t.life(P0), 3);
    // Lower works as normal: P1 at 25 becomes 20.
    t.g.players[P1.idx()].life = 25;
    cast_resolve(&mut t, "Blessed Wind", &[pl(P1)]);
    assert_eq!(t.life(P1), 20);
}

#[test]
fn two_wind_crystals_multiply_by_four() {
    cr!("614.5", "616.1");
    ruling!(
        "The Wind Crystal",
        "In the rare case where you control two of The Wind Crystal, life you gain is multiplied by four."
    );
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Mirror Gallery");
    t.battlefield(P0, "The Wind Crystal");
    t.battlefield(P0, "The Wind Crystal");
    t.settle();
    assert_eq!(t.named_on_battlefield("The Wind Crystal").len(), 2);
    t.g.gain_life(P0, 1);
    assert_eq!(t.life(P0), 24);
    t.battlefield(P0, "The Wind Crystal");
    t.settle();
    t.g.gain_life(P0, 1);
    assert_eq!(t.life(P0), 32);
}

/// P0 and P1 against P2 and P3.
fn thg() -> TestGame {
    TestGame::with_config(4, GameConfig::two_headed_giant(vec![0, 0, 1, 1]))
}

#[test]
fn the_wind_crystal_in_two_headed_giant_only_doubles_your_own_gain() {
    cr!("810.9", "614.1a");
    ruling!(
        "The Wind Crystal",
        "In a Two-Headed Giant game, life gained by your teammate won't cause The Wind Crystal's second ability to apply, even though it will cause your team's life total to increase."
    );
    let mut t = thg();
    t.battlefield(P0, "The Wind Crystal");
    t.g.gain_life(P1, 3);
    assert_eq!(t.life(P0), 33);
    t.g.gain_life(P0, 3);
    assert_eq!(t.life(P0), 39);
}

#[test]
fn vizkopa_guildmage_in_two_headed_giant_drains_twice() {
    cr!("810.9", "603.7a");
    ruling!(
        "Vizkopa Guildmage",
        "In a Two-Headed Giant game, Vizkopa Guildmage’s delayed triggered ability causes the opposing team to lose twice the life you gained."
    );
    supported("Vizkopa Guildmage");
    let mut t = thg();
    let mage = t.battlefield(P0, "Vizkopa Guildmage");
    t.lands(P0, "Plains", 2);
    t.lands(P0, "Swamp", 1);
    crate::r_s06_common::activate_containing(&mut t, P0, mage, "each opponent loses").unwrap();
    t.resolve_all();
    t.g.gain_life(P0, 3);
    t.g.flush_events();
    t.resolve_all();
    assert_eq!(t.life(P0), 33);
    assert_eq!(t.life(P2), 24);
}

#[test]
fn nectar_faerie_lifelink_on_an_opponents_creature_gains_them_life() {
    cr!("702.15b");
    ruling!(
        "Nectar Faerie",
        "If you grant lifelink to a creature controlled by another player, that other player will gain life when the creature deals damage."
    );
    supported("Nectar Faerie");
    let mut t = TestGame::new(2);
    let nf = t.battlefield(P0, "Nectar Faerie");
    t.lands(P0, "Swamp", 1);
    let sprite = t.battlefield(P1, "Zephyr Sprite");
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.answer_targets(P0, &[obj(sprite)]);
    crate::r_s06_common::activate_containing(&mut t, P0, nf, "lifelink").unwrap();
    t.resolve_all();
    damage(&mut t, sprite, 1, bears);
    assert_eq!((t.life(P0), t.life(P1)), (20, 21));
}
