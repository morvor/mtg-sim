//! Rulings batch P091 — landfall and other "whenever a land enters" abilities: targets and
//! resolution, counting, simultaneous entries, multiplayer, and the cards' other
//! abilities (Cosima, Omnath, Undergrowth Champion, The Endstone, Trench Behemoth...).

use crate::r_p091_common::*;
use crate::r_s01_common::*;
use crate::r_s02_common::destroy;
use crate::r_s06_common::{damage, give_control};
use crate::r_s25_common::targets_of;
use mtg_engine::decision::Answer;
use mtg_engine::events::MoveCause;
use mtg_engine::game::{GameConfig, Variant};
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

fn obj(id: ObjectId) -> Entity {
    Entity::Object(id)
}

#[allow(dead_code)]
fn move_to(t: &mut TestGame, id: ObjectId, zone: Zone) -> ObjectId {
    let id = t.g.current(id);
    let new = t.g.move_object(id, zone, MoveCause::Effect, None);
    t.g.flush_events();
    new.unwrap_or(id)
}

#[test]
fn simple_landfall_targets_table() {
    cr!("115.1", "608.2b", "122.1");
    ruling!(
        "Grappling Kraken",
        "You may target a creature that's already tapped with Grappling Kraken's ability. In that case, you'll just put a stun counter on that creature."
    );
    ruling!(
        "Tatyova, Steward of Tides",
        "You may choose to target the land that entered the battlefield with Tatyova, Steward of Tides’s triggered ability."
    );
    ruling!(
        "Omnath, Locus of the Roil",
        "You may choose Omnath as the target of its own second ability."
    );
    supported("Grappling Kraken");
    supported("Tatyova, Steward of Tides");
    supported("Omnath, Locus of the Roil");
    // Grappling Kraken: a tapped target gets a stun counter.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Grappling Kraken");
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.g.tap(bears);
    t.answer_targets(P0, &[obj(bears)]);
    play_from_hand(&mut t, P0, "Island");
    t.resolve_all();
    assert_eq!(t.counters(bears, "stun"), 1);
    // Tatyova: the land that entered becomes a 3/3 with haste.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Tatyova, Steward of Tides");
    t.lands(P0, "Forest", 6);
    let forest = t.hand(P0, "Forest");
    t.play_land(P0, forest).unwrap();
    t.g.flush_events();
    let forest = t.g.current(forest);
    t.answer_targets(P0, &[obj(forest)]);
    t.settle();
    let trig = *t.g.stack.last().unwrap();
    assert_eq!(targets_of(&t, trig), vec![obj(forest)]);
    t.resolve_all();
    assert!(crate::r_s20_common::creature_now(&mut t, forest));
    assert_eq!(t.pt(forest), (3, 3));
    // Omnath, Locus of the Roil targets itself.
    let mut t = TestGame::new(2);
    let omnath = t.battlefield(P0, "Omnath, Locus of the Roil");
    t.answer_targets(P0, &[obj(omnath)]);
    play_from_hand(&mut t, P0, "Island");
    t.resolve_all();
    assert_eq!(t.counters(omnath, "+1/+1"), 1);
}

#[test]
fn illegal_targets_stop_the_whole_effect() {
    cr!("608.2b", "701.10a");
    ruling!(
        "Omnath, Locus of the Roil",
        "If the target Elemental is an illegal target by the time Omnath's second ability tries to resolve, the ability doesn't resolve. You don't draw a card if you control eight or more lands. If the target is legal but can't have a +1/+1 counter put on it (most likely because an opponent controls Blightbeetle), you do draw a card if you control eight or more lands."
    );
    ruling!(
        "Choco-Comet",
        "If the target is illegal when Choco-Comet tries to resolve, it won't resolve and none of its effects will happen. You won't create a Bird token."
    );
    supported("Choco-Comet");
    supported("Blightbeetle");
    // Omnath: eight lands; the target Elemental leaves: no card.
    for case in ["gone", "blightbeetle"] {
        let mut t = TestGame::new(2);
        let omnath = t.battlefield(P0, "Omnath, Locus of the Roil");
        t.lands(P0, "Island", 7);
        if case == "blightbeetle" {
            t.battlefield(P1, "Blightbeetle");
        }
        t.answer_targets(P0, &[obj(omnath)]);
        play_from_hand(&mut t, P0, "Island");
        if case == "gone" {
            destroy(&mut t, omnath);
        }
        let hand = t.hand_size(P0);
        t.resolve_all();
        if case == "gone" {
            assert_eq!(t.hand_size(P0), hand);
        } else {
            assert_eq!(t.counters(omnath, "+1/+1"), 0);
            assert_eq!(t.hand_size(P0), hand + 1);
        }
    }
    // Choco-Comet: the target leaves; no token. Positive control: it resolves otherwise.
    for leaves in [true, false] {
        let mut t = TestGame::new(2);
        let bears = t.battlefield(P1, "Grizzly Bears");
        t.lands(P0, "Mountain", 3);
        let comet = t.hand(P0, "Choco-Comet");
        t.cast(P0, comet).x(1).target(obj(bears)).go();
        if leaves {
            destroy(&mut t, bears);
        }
        t.resolve_all();
        assert_eq!(tokens(&t, P0).len(), usize::from(!leaves), "{leaves}");
    }
}

#[test]
fn counting_on_resolution_includes_the_source() {
    cr!("608.2h", "603.6a");
    ruling!(
        "Omnath, Locus of the Roil",
        "The number of Elementals you control is counted as Omnath's first ability resolves. If Omnath is still on the battlefield, it will count itself."
    );
    ruling!(
        "Canyon Jerboa",
        "Canyon Jerboa's ability affects only creatures you control at the time it resolves. Creatures you begin to control later in the turn won't get +1/+1."
    );
    ruling!(
        "Field of the Dead",
        "Field of the Dead's last ability counts itself and the land entering the battlefield in addition to whichever other lands you control."
    );
    supported("Canyon Jerboa");
    supported("Field of the Dead");
    // Omnath: itself plus an Elemental that arrives in response: 2 damage.
    let mut t = TestGame::new(2);
    t.answer_targets(P0, &[Entity::Player(P1)]);
    put_onto_battlefield(&mut t, P0, "Omnath, Locus of the Roil");
    t.battlefield(P0, "Air Elemental");
    t.resolve_all();
    assert_eq!(t.life(P1), 18);
    // Canyon Jerboa: creatures you control at resolution.
    let mut t = TestGame::new(2);
    let jerboa = t.battlefield(P0, "Canyon Jerboa");
    play_from_hand(&mut t, P0, "Plains");
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.resolve_all();
    assert_eq!(t.pt(jerboa), (2, 3));
    assert_eq!(t.pt(bears), (3, 3));
    let later = t.battlefield(P0, "Hill Giant");
    t.g.recompute();
    assert_eq!(t.pt(later), (3, 3));
    // Field of the Dead: five other differently named lands + Field + the land entering
    // = seven names.
    let mut t = TestGame::new(2);
    for l in ["Plains", "Island", "Swamp", "Mountain", "Forest"] {
        t.battlefield(P0, l);
    }
    let field = t.battlefield(P0, "Field of the Dead");
    play_from_hand(&mut t, P0, "Wastes");
    assert_eq!(triggers_of(&t, field), 1);
    t.resolve_all();
    assert_eq!(tokens(&t, P0).len(), 1);
}

#[test]
fn simultaneous_lands_are_all_counted() {
    cr!("603.2c", "603.6a", "603.4");
    ruling!(
        "Tatyova, Steward of Tides",
        "If multiple lands enter the battlefield simultaneously, all of those lands are counted. For example, if you control six lands and two lands enter the battlefield at the same time, Tatyova’s ability will trigger twice."
    );
    ruling!(
        "Field of the Dead",
        "If multiple lands enter the battlefield simultaneously, possibly including Field of the Dead itself, all of those lands are counted."
    );
    // Tatyova: six lands; two enter together: two triggers.
    let mut t = TestGame::new(2);
    let tat = t.battlefield(P0, "Tatyova, Steward of Tides");
    t.lands(P0, "Forest", 5);
    let a = t.hand(P0, "Forest");
    let b = t.hand(P0, "Island");
    crate::r_s31_common::put_together(&mut t, &[a, b]);
    assert_eq!(triggers_of(&t, tat), 2);
    // Field of the Dead and five lands with different names enter together with one other
    // land: each of the two Fields' triggers... here one Field: six triggers (one per land).
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Wastes");
    let names = ["Field of the Dead", "Plains", "Island", "Swamp", "Mountain", "Forest"];
    let cards: Vec<ObjectId> = names.iter().map(|n| t.hand(P0, n)).collect();
    crate::r_s31_common::put_together(&mut t, &cards);
    let field = t.g.current(cards[0]);
    assert_eq!(triggers_of(&t, field), 6);
    t.resolve_all();
    assert_eq!(tokens(&t, P0).len(), 6);
}

#[test]
fn ankh_of_mishra_uses_the_controller_on_resolution_and_triggers_on_any_land() {
    cr!("603.10a", "608.2h", "113.7a");
    ruling!(
        "Ankh of Mishra",
        "It determines the land's controller at the time the ability resolves. If the land leaves the battlefield before the ability resolves, the land's last controller before it left is used."
    );
    ruling!(
        "Ankh of Mishra",
        "This triggers on any land entering. This includes playing a land or putting a land onto the battlefield using a spell or ability."
    );
    supported("Ankh of Mishra");
    // Played by P0; P1 gains control of it before resolution: P1 is dealt 2.
    let mut t = TestGame::new(2);
    let ankh = t.battlefield(P0, "Ankh of Mishra");
    let land = play_from_hand(&mut t, P0, "Forest");
    assert_eq!(triggers_of(&t, ankh), 1);
    give_control(&mut t, land, P1);
    t.resolve_all();
    assert_eq!((t.life(P0), t.life(P1)), (20, 18));
    // Put onto the battlefield by P1's effect, then gone before resolution: last
    // controller P1.
    let mut t = TestGame::new(2);
    let ankh = t.battlefield(P0, "Ankh of Mishra");
    let land = put_onto_battlefield(&mut t, P1, "Forest");
    assert_eq!(triggers_of(&t, ankh), 1);
    give_control(&mut t, land, P0);
    give_control(&mut t, land, P1);
    destroy(&mut t, land);
    t.resolve_all();
    assert_eq!((t.life(P0), t.life(P1)), (20, 18));
}

#[test]
fn hidden_herd_only_triggers_on_played_lands() {
    cr!("305.1", "603.2");
    ruling!(
        "Hidden Herd",
        "Does not trigger on lands put onto the battlefield by a spell or ability."
    );
    supported("Hidden Herd");
    let mut t = TestGame::new(2);
    let herd = t.battlefield(P0, "Hidden Herd");
    t.set_step(P1, Step::PrecombatMain);
    put_onto_battlefield(&mut t, P1, "Tropical Island");
    assert_eq!(triggers_of(&t, herd), 0);
    let turf = t.hand(P1, "Tropical Island");
    t.play_land(P1, turf).unwrap_or_else(|e| panic!("{e:?}"));
    t.g.flush_events();
    t.settle();
    assert_eq!(triggers_of(&t, herd), 1);
    t.resolve_all();
    assert!(crate::r_s20_common::creature_now(&mut t, herd));
}

#[test]
fn legions_landing_counts_only_declared_attackers() {
    cr!("508.1", "506.3", "603.2");
    ruling!(
        "Legion's Landing // Adanto, the First Fort",
        "The last ability of Legion's Landing only counts creatures that you declare as attacking creatures. Creatures that enter the battlefield attacking won't count."
    );
    supported("Legion's Landing // Adanto, the First Fort");
    supported("Hero of Bladehold");
    for third in [false, true] {
        let mut t = TestGame::new(2);
        let landing = t.battlefield(P0, "Legion's Landing // Adanto, the First Fort");
        let hero = t.battlefield(P0, "Hero of Bladehold");
        let bears = t.battlefield(P0, "Grizzly Bears");
        let mut decl = vec![(hero, Entity::Player(P1)), (bears, Entity::Player(P1))];
        if third {
            let giant = t.battlefield(P0, "Hill Giant");
            decl.push((giant, Entity::Player(P1)));
        }
        attack_with(&mut t, &decl);
        t.resolve_all();
        // Hero's battle cry trigger made two tokens enter attacking.
        let attackers = crate::r_s09_common::attacks_now(&t).len();
        assert_eq!(attackers, decl.len() + 2);
        let transformed = t
            .named_on_battlefield("Adanto, the First Fort")
            .contains(&t.g.current(landing));
        assert_eq!(transformed, third, "three declared: {third}");
    }
}

#[test]
fn undergrowth_champion_removes_one_counter_per_prevention_and_even_unpreventable() {
    cr!("615.1a", "510.2", "615.12");
    ruling!(
        "Undergrowth Champion",
        "If Undergrowth Champion is dealt damage from multiple sources at the same time (for example, if it's blocked by multiple creatures), its prevention effect will apply once to all that damage. You'll remove only one +1/+1 counter."
    );
    ruling!(
        "Undergrowth Champion",
        "If damage that can't be prevented is dealt to Undergrowth Champion while it has a +1/+1 counter on it, you'll still remove a +1/+1 counter on it."
    );
    supported("Undergrowth Champion");
    supported("Skullcrack");
    // Blocked by two Bears: all damage prevented, one counter removed.
    let mut t = TestGame::new(2);
    let champ = t.battlefield(P0, "Undergrowth Champion");
    crate::r_s29_common::put_counters(&mut t, champ, "+1/+1", 2);
    let a = t.battlefield(P1, "Grizzly Bears");
    let b = t.battlefield(P1, "Grizzly Bears");
    attack_with(&mut t, &[(champ, Entity::Player(P1))]);
    t.answer(P1, DecisionKind::Damage, Answer::Default);
    block_and_finish(&mut t, P1, &[(a, champ), (b, champ)]);
    assert_eq!(t.counters(champ, "+1/+1"), 1);
    assert_eq!(crate::r_s07_common::damage_on(&t, champ), 0);
    // Skullcrack: damage can't be prevented; Lightning Bolt still removes a counter.
    let mut t = TestGame::new(2);
    let champ = t.battlefield(P0, "Undergrowth Champion");
    crate::r_s29_common::put_counters(&mut t, champ, "+1/+1", 3);
    t.lands(P0, "Mountain", 3);
    let crack = t.hand(P0, "Skullcrack");
    t.cast(P0, crack).target(Entity::Player(P1)).go();
    t.resolve_all();
    let bolt = t.hand(P0, "Lightning Bolt");
    t.cast(P0, bolt).target(obj(champ)).go();
    t.resolve_all();
    assert_eq!(t.counters(champ, "+1/+1"), 2);
    assert_eq!(crate::r_s07_common::damage_on(&t, champ), 3);
    assert!(t.on_battlefield(champ));
}

#[test]
fn omnath_locus_of_rage_triggers_for_each_elemental_dying_with_it() {
    cr!("603.10a", "603.6c");
    ruling!(
        "Omnath, Locus of Rage",
        "If Omnath dies at the same time as any other Elementals you control, Omnath's last ability will trigger for each of them (and once for itself)."
    );
    supported("Omnath, Locus of Rage");
    supported("Wrath of God");
    let mut t = TestGame::new(2);
    let omnath = t.battlefield(P0, "Omnath, Locus of Rage");
    t.battlefield(P0, "Air Elemental");
    t.battlefield(P0, "Fire Elemental");
    t.battlefield(P0, "Grizzly Bears");
    t.lands(P0, "Plains", 4);
    let wrath = t.hand(P0, "Wrath of God");
    for _ in 0..3 {
        t.answer_targets(P0, &[Entity::Player(P1)]);
    }
    t.cast(P0, wrath).go();
    t.resolve();
    assert_eq!(triggers_of(&t, omnath), 3);
    t.resolve_all();
    assert_eq!(t.life(P1), 11);
}

#[test]
fn the_endstone_rulings() {
    cr!("119.5", "603.3", "119.3");
    ruling!(
        "The Endstone",
        "For your life total to become a certain value, you will gain or lose the appropriate amount of life. For example, if your starting life total is 20 and your life total is 17 when The Endstone’s last ability resolves, you will lose 7 life and your life total will become 10. Other abilities that interact with life gain or life loss will interact with this effect accordingly."
    );
    ruling!(
        "The Endstone",
        "The Endstone’s first ability resolves before the spell that caused it to trigger. It resolves even if that spell is countered or otherwise leaves the stack without resolving."
    );
    supported("Exquisite Blood");
    supported("Ajani's Pridemate");
    // 17 → 10: P0 loses 7; P1's Exquisite Blood gains P1 7.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "The Endstone");
    t.battlefield(P1, "Exquisite Blood");
    t.g.players[0].life = 17;
    t.advance_to(P0, Step::End);
    t.resolve_all();
    assert_eq!((t.life(P0), t.life(P1)), (10, 27));
    // 5 → 10: P0 gains 5; Ajani's Pridemate gets a counter.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "The Endstone");
    let pm = t.battlefield(P0, "Ajani's Pridemate");
    t.g.players[0].life = 5;
    t.advance_to(P0, Step::End);
    t.resolve_all();
    assert_eq!(t.life(P0), 10);
    assert_eq!(t.counters(pm, "+1/+1"), 1);
    // Cast Grizzly Bears; the draw trigger is above it, and resolves although the Bears
    // are countered.
    let mut t = TestGame::new(2);
    let stone = t.battlefield(P0, "The Endstone");
    t.lands(P0, "Forest", 2);
    let bears = t.hand(P0, "Grizzly Bears");
    let spell = t.cast(P0, bears).go();
    t.settle();
    assert_eq!(triggers_of(&t, stone), 1);
    assert_ne!(*t.g.stack.last().unwrap(), spell);
    t.lands(P1, "Island", 2);
    let cs = t.hand(P1, "Counterspell");
    t.cast(P1, cs).target(obj(spell)).go();
    let hand = t.hand_size(P0);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand + 1);
    assert!(t.in_graveyard(P0, "Grizzly Bears"));
}

#[test]
fn ghost_spider_triggers_on_casting_a_copy_from_exile() {
    cr!("707.12", "601.2a", "603.2");
    ruling!(
        "Gwen Stacy // Ghost-Spider",
        "Ghost-Spider's triggered ability will trigger even if the spell you cast isn't represented by a card. For example, if an effect copies a card in exile and allows you to cast the copy, casting that copy will cause Ghost-Spider's triggered ability to trigger."
    );
    supported("Gwen Stacy // Ghost-Spider");
    let mut t = TestGame::new(2);
    let spider = crate::r_s17_common::enter_transformed(&mut t, P0, "Gwen Stacy // Ghost-Spider");
    t.resolve_all();
    let bolt = t.hand(P0, "Lightning Bolt");
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[obj(bolt)]);
    let scepter = t.enter(P0, "Isochron Scepter");
    t.resolve_all();
    assert_eq!(t.zone(bolt), Zone::Exile);
    t.lands(P0, "Wastes", 2);
    t.answer_yes(P0, true);
    t.answer_yes(P0, true);
    t.answer_targets(P0, &[Entity::Player(P1)]);
    crate::r_s06_common::activate_containing(&mut t, P0, scepter, "{2}").unwrap();
    t.resolve_all();
    assert_eq!(t.life(P1), 17);
    assert_eq!(t.counters(spider, "+1/+1"), 1);
}
