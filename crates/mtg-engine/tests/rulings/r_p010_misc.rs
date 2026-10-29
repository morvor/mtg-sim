//! Rulings batch P010 — burn with each set's mechanic: legendary Oaths, Desert
//! conditions, draft notes, dice, Dragons, Rooms, escalate, -1/-1 counters, Two-Headed
//! Giant and a few more.

use crate::r_p009_common::*;
use crate::r_s01_common::{supported, tokens, triggers_on_stack};
use crate::r_s25_common::cast_new;
use mtg_engine::events::Event;
use mtg_engine::game::{GameConfig, Variant};
use mtg_engine::keywords::KeywordKind;
use mtg_engine::object::{CastMethod, Zone};
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

#[test]
fn touch_of_the_void_exiles_a_damaged_creature_that_dies_later() {
    cr!("614.1a", "614.6");
    ruling!(
        "Touch of the Void",
        "A creature doesn’t necessarily have to be dealt lethal damage by Touch of the Void to be exiled. After being dealt damage, if it would die for any reason that turn, it’ll be exiled instead."
    );
    supported("Touch of the Void");
    let mut t = TestGame::new(2);
    let wurm = t.battlefield(P1, "Craw Wurm");
    cast_new(&mut t, P0, "Touch of the Void", &[obj(wurm)]);
    t.resolve_all();
    assert_eq!(dmg(&t, wurm), 3);
    kill(&mut t, wurm);
    assert!(t.in_exile("Craw Wurm"));
    assert!(!t.in_graveyard(P1, "Craw Wurm"));
}

#[test]
fn oaths_are_legendary_with_different_names() {
    cr!("704.5j", "201.2a");
    ruling!(
        "Oath of Gideon",
        "Each of the Oaths is legendary: if you control more than one of any particular Oath, you choose one to remain on the battlefield and the others are put into your graveyard."
    );
    ruling!(
        "Oath of Gideon",
        "The “legend rule” checks the full names of legendary permanents. You can control Oath of Gideon and Oath of Chandra at the same time, for example."
    );
    ruling!(
        "Oath of Gideon",
        "Even if you control a particular Oath, you can cast another of that Oath to take advantage of its enters-the-battlefield ability."
    );
    for n in ["Oath of Gideon", "Oath of Chandra"] {
        supported(n);
    }
    let mut t = TestGame::new(2);
    let first = t.battlefield(P0, "Oath of Gideon");
    let chandra = t.battlefield(P0, "Oath of Chandra");
    // A second Oath of Gideon is cast and enters: its ability triggers, P0 keeps one,
    // and the ability still resolves.
    let spell = cast_new(&mut t, P0, "Oath of Gideon", &[]);
    t.resolve();
    let second = t.g.current(spell);
    assert_eq!(
        [first, second].iter().filter(|o| t.on_battlefield(**o)).count(),
        1
    );
    assert!(t.on_battlefield(chandra));
    assert_eq!(t.stack_len(), 1);
    t.resolve_all();
    assert_eq!(tokens(&t, P0).len(), 2);
    assert!(t.in_graveyard(P0, "Oath of Gideon"));
}

#[test]
fn oath_of_chandra_end_step_trigger() {
    cr!("603.4");
    ruling!(
        "Oath of Chandra",
        "Oath of Chandra’s last ability will trigger even if the planeswalker that entered the battlefield under your control is no longer on the battlefield, no longer under your control, or no longer a planeswalker. This is also true if Oath of Chandra wasn’t on the battlefield when the planeswalker entered."
    );
    ruling!(
        "Oath of Chandra",
        "The last ability triggers only once per turn, even if more than one planeswalker entered the battlefield under your control."
    );
    ruling!(
        "Oath of Liliana",
        "The last ability triggers only once per turn, even if more than one planeswalker entered the battlefield under your control."
    );
    supported("Oath of Liliana");
    // A planeswalker entered, then died, before Oath of Chandra was on the battlefield.
    let mut t = TestGame::new(2);
    let pw = t.enter(P0, "Chandra, Torch of Defiance");
    t.settle();
    kill(&mut t, pw);
    t.battlefield(P0, "Oath of Chandra");
    t.advance_to(P0, Step::End);
    t.resolve_all();
    assert_eq!(t.life(P1), 18);
    // Two planeswalkers: once each.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Oath of Chandra");
    t.battlefield(P0, "Oath of Liliana");
    t.enter(P0, "Chandra, Torch of Defiance");
    t.enter(P0, "Jace Beleren");
    t.settle();
    t.advance_to(P0, Step::End);
    t.resolve_all();
    assert_eq!(t.life(P1), 18);
    let zombies = tokens(&t, P0)
        .into_iter()
        .filter(|z| t.obj_now(*z).chars.has_subtype("Zombie"))
        .count();
    assert_eq!(zombies, 1);
    // No planeswalker: no trigger.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Oath of Chandra");
    t.advance_to(P0, Step::End);
    t.resolve_all();
    assert_eq!(t.life(P1), 20);
}

#[test]
fn desert_condition_can_be_met_differently_on_resolution() {
    cr!("603.4");
    ruling!(
        "Sand Strangler",
        "For abilities that trigger only if you control a Desert or there is a Desert card in your graveyard, one condition must be true as the ability triggers and one must be true as the ability resolves. They don’t have to be the same condition, though."
    );
    ruling!(
        "Wretched Camel",
        "For abilities that trigger only if you control a Desert or there is a Desert card in your graveyard, one condition must be true as the ability triggers and one must be true as the ability resolves. They don’t have to be the same condition, though."
    );
    for n in ["Sand Strangler", "Wretched Camel", "Desert"] {
        supported(n);
    }
    // Sand Strangler: P0 controls a Desert as it triggers; it's in the graveyard as the
    // ability resolves.
    let mut t = TestGame::new(2);
    let wurm = t.battlefield(P1, "Craw Wurm");
    let desert = t.battlefield(P0, "Desert");
    t.answer_targets(P0, &[obj(wurm)]);
    t.answer_yes(P0, true);
    t.enter(P0, "Sand Strangler");
    t.settle();
    assert_eq!(t.stack_len(), 1);
    kill(&mut t, desert);
    t.resolve_all();
    assert_eq!(dmg(&t, wurm), 3);
    // ... and if the Desert card leaves the graveyard too, nothing happens.
    let mut t = TestGame::new(2);
    let wurm = t.battlefield(P1, "Craw Wurm");
    let desert = t.battlefield(P0, "Desert");
    t.answer_targets(P0, &[obj(wurm)]);
    t.answer_yes(P0, true);
    t.enter(P0, "Sand Strangler");
    t.settle();
    kill(&mut t, desert);
    let card = t.g.current(desert);
    t.g.move_object(card, Zone::Exile, mtg_engine::events::MoveCause::Effect, None);
    t.resolve_all();
    assert_eq!(dmg(&t, wurm), 0);

    // Wretched Camel: a Desert card in the graveyard as it dies; a Desert on the
    // battlefield (and none in the graveyard) as the ability resolves.
    let mut t = TestGame::new(2);
    let camel = t.battlefield(P0, "Wretched Camel");
    let desert = t.graveyard(P0, "Desert");
    t.hand(P1, "Grizzly Bears");
    t.answer_targets(P0, &[pl(P1)]);
    kill(&mut t, camel);
    assert_eq!(t.stack_len(), 1);
    t.g.move_object(desert, Zone::Exile, mtg_engine::events::MoveCause::Effect, None);
    t.battlefield(P0, "Desert");
    t.resolve_all();
    assert_eq!(t.hand_size(P1), 0);
}

fn garbage_fire_damage(notes: &[i64]) -> u32 {
    let mut t = TestGame::new(2);
    t.g.start.draft = mtg_engine::draft::DraftInfo {
        notes: notes
            .iter()
            .map(|n| mtg_engine::draft::DraftNote {
                player: P0,
                card: "Garbage Fire".into(),
                number: Some(*n),
                noted_player: None,
                noted_name: None,
                noted_types: vec![],
                colors: vec![],
                guessed_correctly: None,
            })
            .collect(),
        removed: vec![],
    };
    let wurm = t.battlefield(P1, "Colossal Dreadmaw");
    t.battlefield(P1, "Glorious Anthem");
    t.battlefield(P1, "Glorious Anthem");
    cast_new(&mut t, P0, "Garbage Fire", &[obj(wurm)]);
    t.resolve_all();
    if t.on_battlefield(wurm) {
        dmg(&t, wurm)
    } else {
        u32::MAX
    }
}

#[test]
fn garbage_fire_uses_the_highest_noted_number() {
    cr!("905.2b");
    ruling!(
        "Garbage Fire",
        "For example, if you draft Garbage Fire as the third card in a draft round and then another one as the twelfth card in a draft round, each Garbage Fire will deal 12 damage."
    );
    ruling!(
        "Garbage Fire",
        "If you somehow cast, copy, or gain control of Garbage Fire but you didn't draft one, the highest noted number is considered to be 0, so Garbage Fire won't deal any damage."
    );
    supported("Garbage Fire");
    // A 6/6 with two anthems is an 8/8: 3 damage doesn't kill it, 12 does.
    assert_eq!(garbage_fire_damage(&[3]), 3);
    assert_eq!(garbage_fire_damage(&[3, 12]), u32::MAX);
    assert_eq!(garbage_fire_damage(&[]), 0);
}

fn roll(t: &mut TestGame, p: PlayerId, sides: u32, result: u32) {
    t.g.dice.loaded.push_back(result);
    mtg_engine::dice::roll_dice(
        &mut t.g,
        p,
        1,
        sides,
        mtg_engine::dice::DiceIgnore::None,
        0,
        None,
    );
    t.g.flush_events();
    t.settle();
}

#[test]
fn eelectrocute_any_die_that_rolled_a_six() {
    cr!("706.2", "601.3");
    ruling!(
        "Eelectrocute",
        "For the last ability, it doesn't matter how many pips the die you rolled a 6 on had, whether that was a six-sided die or a d20."
    );
    supported("Eelectrocute");
    for (sides, result, castable) in [(6, 5, false), (20, 6, true), (6, 6, true)] {
        let mut t = TestGame::new(2);
        roll(&mut t, P0, sides, result);
        t.lands(P0, "Mountain", 2);
        let e = t.graveyard(P0, "Eelectrocute");
        let r = t.cast(P0, e).target(pl(P1)).try_go();
        t.clear_answers();
        assert_eq!(r.is_ok(), castable, "d{sides} rolled {result}");
        if castable {
            t.resolve_all();
            assert_eq!(t.life(P1), 18);
            // Cast this way, it's exiled.
            assert!(t.in_exile("Eelectrocute"));
        }
    }
}

#[test]
fn dragon_tempest_both_abilities_and_a_dragon_that_left() {
    cr!("603.2", "113.7a");
    ruling!(
        "Dragon Tempest",
        "The two abilities aren’t mutually exclusive. If a Dragon with flying (which is most of them) enters the battlefield under your control, both abilities will trigger."
    );
    ruling!(
        "Dragon Tempest",
        "If a Dragon entering the battlefield causes Dragon Tempest’s second ability to trigger but leaves the battlefield before that ability resolves, that Dragon still deals damage."
    );
    supported("Dragon Tempest");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Dragon Tempest");
    t.battlefield(P0, "Shivan Dragon");
    t.answer_targets(P0, &[pl(P1)]);
    let whelp = t.enter(P0, "Dragon Whelp");
    t.settle();
    assert_eq!(t.stack_len(), 2);
    assert_eq!(triggers_on_stack(&t, "haste"), 1);
    // The Whelp leaves; X counts the Shivan Dragon, and the Whelp is the source.
    kill(&mut t, whelp);
    t.resolve_all();
    assert_eq!(t.life(P1), 19);
    assert!(t.g.log.iter().any(|l| l.text.contains("Dragon Whelp")
        && l.text.contains("deals 1 damage to Player(PlayerId(1))")));
}

#[test]
fn survive_with_an_empty_graveyard_still_shuffles() {
    cr!("701.24a", "702.127a");
    ruling!(
        "Struggle // Survive",
        "If a player has no cards in their graveyard when Survive resolves, that player just shuffles their library."
    );
    supported("Struggle // Survive");
    let mut t = TestGame::new(2);
    t.graveyard(P1, "Grizzly Bears");
    let card = t.graveyard(P0, "Struggle // Survive");
    t.lands(P0, "Forest", 1);
    t.lands(P0, "Wastes", 1);
    t.cast(P0, card)
        .method(CastMethod::Keyword(KeywordKind::Aftermath))
        .go();
    let before = t.g.turn_events.len();
    t.resolve_all();
    let shuffled: Vec<PlayerId> = t.g.turn_events[before..]
        .iter()
        .filter_map(|e| match e {
            Event::Shuffled { player } => Some(*player),
            _ => None,
        })
        .collect();
    assert!(shuffled.contains(&P0), "{shuffled:?}");
    assert!(shuffled.contains(&P1));
    assert_eq!(t.graveyard_size(P1), 0);
    assert_eq!(t.library_size(P1), 31);
}

#[test]
fn steaming_sauna_and_null_profusion_apply_in_timestamp_order() {
    cr!("613.7");
    interp!("room-door-unlock-timestamp");
    ruling!(
        "Roaring Furnace // Steaming Sauna",
        "If multiple effects modify your hand size, apply them in timestamp order."
    );
    supported("Roaring Furnace // Steaming Sauna");
    supported("Null Profusion");
    // Unlock Steaming Sauna, then Null Profusion: two.
    let mut t = TestGame::new(2);
    let room = t.battlefield(P0, "Roaring Furnace // Steaming Sauna");
    assert!(mtg_engine::rooms::unlock(&mut t.g, room, 1, P0));
    t.g.flush_events();
    t.resolve_all();
    t.g.recompute();
    assert_eq!(t.g.player(P0).max_hand_size, None);
    add(&mut t, P0, "Null Profusion");
    t.g.recompute();
    assert_eq!(t.g.player(P0).max_hand_size, Some(2));
    // Null Profusion, then unlock Steaming Sauna: no maximum.
    let mut t = TestGame::new(2);
    let room = t.battlefield(P0, "Roaring Furnace // Steaming Sauna");
    add(&mut t, P0, "Null Profusion");
    t.g.recompute();
    assert_eq!(t.g.player(P0).max_hand_size, Some(2));
    assert!(mtg_engine::rooms::unlock(&mut t.g, room, 1, P0));
    t.g.flush_events();
    t.resolve_all();
    t.g.recompute();
    assert_eq!(t.g.player(P0).max_hand_size, None);
}

#[test]
fn savage_alliance_modes_on_the_same_creature_are_two_damage_events() {
    cr!("603.2c");
    ruling!(
        "Savage Alliance",
        "If you choose the second and third modes, and the targeted creature is controlled by the targeted opponent, that creature will take 2 plus 1 damage for a total of 3 damage. An ability that triggers whenever that creature is dealt damage will trigger twice."
    );
    supported("Savage Alliance");
    supported("Boros Reckoner");
    let mut t = TestGame::new(2);
    let reckoner = t.battlefield(P1, "Boros Reckoner");
    t.lands(P0, "Mountain", 1);
    t.lands(P0, "Wastes", 3);
    let sa = t.hand(P0, "Savage Alliance");
    t.answer_targets(P1, &[pl(P0)]);
    t.answer_targets(P1, &[pl(P0)]);
    t.cast(P0, sa)
        .modes(&[1, 2])
        .targets(&[obj(reckoner)])
        .targets(&[pl(P1)])
        .go();
    t.resolve();
    assert!(!t.on_battlefield(reckoner));
    assert_eq!(triggers_on_stack(&t, "dealt damage"), 2);
    t.resolve_all();
    assert_eq!(t.life(P0), 17);
}

#[test]
fn trample_counts_damage_already_marked_on_the_blocker() {
    cr!("702.19b", "510.1c");
    ruling!(
        "Savage Alliance",
        "When determining how much damage a creature with trample may assign to the player or planeswalker it's attacking, take into account damage that has already been dealt to the blocking creatures."
    );
    let mut t = TestGame::new(2);
    let baloth = t.battlefield(P0, "Rumbling Baloth");
    let blocker = t.battlefield(P1, "Rumbling Baloth");
    t.lands(P0, "Mountain", 1);
    t.lands(P0, "Wastes", 2);
    let sa = t.hand(P0, "Savage Alliance");
    t.cast(P0, sa).modes(&[0]).targets(&[pl(P0)]).go();
    t.resolve();
    assert!(t.obj_now(baloth).has_keyword(KeywordKind::Trample));
    t.g.deal_damage(baloth, obj(blocker), 2, false);
    t.settle();
    t.attack(&[(baloth, pl(P1))], &[(blocker, baloth)]);
    assert!(!t.on_battlefield(blocker));
    assert_eq!(t.life(P1), 18);
}

#[test]
fn puncture_bolt_damage_then_counter() {
    cr!("704.5f", "704.5g", "701.19a", "702.79a");
    ruling!(
        "Puncture Bolt",
        "If you target a creature with 1 toughness, Puncture Bolt deals 1 damage to it, then puts a -1/-1 counter on it, which reduces it to 0 toughness. The creature will be put into its owner’s graveyard as a result of having 0 toughness. Regeneration can’t save it."
    );
    ruling!(
        "Puncture Bolt",
        "If you target a creature with 2 toughness, Puncture Bolt deals 1 damage to it, then puts a -1/-1 counter on it, which reduces it to 1 toughness. The creature will be destroyed as a result of lethal damage. Regeneration could save it."
    );
    ruling!(
        "Puncture Bolt",
        "Puncture Bolt will resolve fully (and give the creature a -1/-1 counter) even if the 1 damage would be enough to destroy it."
    );
    supported("Puncture Bolt");
    supported("Kitchen Finks");
    // A regeneration shield on a 1/1: it dies anyway.
    let mut t = TestGame::new(2);
    let elves = t.battlefield(P1, "Llanowar Elves");
    cast_new(&mut t, P1, "Regenerate", &[obj(elves)]);
    t.resolve();
    cast_new(&mut t, P0, "Puncture Bolt", &[obj(elves)]);
    t.resolve_all();
    assert!(t.in_graveyard(P1, "Llanowar Elves"));
    // On a 2/2, regeneration saves it.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    cast_new(&mut t, P1, "Regenerate", &[obj(bears)]);
    t.resolve();
    cast_new(&mut t, P0, "Puncture Bolt", &[obj(bears)]);
    t.resolve_all();
    assert!(t.on_battlefield(bears));
    assert!(t.obj_now(bears).tapped);
    assert_eq!(t.pt(bears), (1, 1));
    // Kitchen Finks (3/2, persist) with 1 damage: the 1 damage is lethal, but the -1/-1
    // counter is put on it before it dies, so persist doesn't return it.
    let mut t = TestGame::new(2);
    let finks = t.battlefield(P1, "Kitchen Finks");
    let src = t.battlefield(P0, "Prodigal Sorcerer");
    t.g.deal_damage(src, obj(finks), 1, false);
    t.settle();
    cast_new(&mut t, P0, "Puncture Bolt", &[obj(finks)]);
    t.resolve_all();
    assert!(t.in_graveyard(P1, "Kitchen Finks"));
    assert!(t.named_on_battlefield("Kitchen Finks").is_empty());
}

fn two_headed() -> TestGame {
    TestGame::with_config(
        4,
        GameConfig {
            variant: Variant::TwoHeadedGiant,
            teams: Some(vec![0, 0, 1, 1]),
            ..Default::default()
        },
    )
}

#[test]
fn two_headed_giant_each_opponent_loses_or_is_dealt_damage() {
    cr!("810.9");
    ruling!(
        "Cut // Ribbons",
        "In a Two-Headed Giant game, Ribbons causes the opposing team to lose two times X life."
    );
    ruling!(
        "The Elder Dragon War",
        "In a Two-Headed Giant game, the first chapter ability deals to damage to each player on the opposing team, reducing that team's life total by 4."
    );
    supported("Cut // Ribbons");
    supported("The Elder Dragon War");
    let mut t = two_headed();
    let start = t.life(P2);
    let ribbons = t.graveyard(P0, "Cut // Ribbons");
    t.lands(P0, "Swamp", 2);
    t.lands(P0, "Wastes", 3);
    t.cast(P0, ribbons)
        .method(CastMethod::Keyword(KeywordKind::Aftermath))
        .x(3)
        .go();
    t.resolve_all();
    assert_eq!(t.life(P2), start - 6);
    assert_eq!(t.life(P3), start - 6);
    assert_eq!(t.life(P0), start);

    let mut t = two_headed();
    let start = t.life(P2);
    t.enter(P0, "The Elder Dragon War");
    t.resolve_all();
    assert_eq!(t.life(P2), start - 4);
    assert_eq!(t.life(P0), start);
}

#[test]
fn sonic_assault_on_a_tapped_creature_still_deals_damage() {
    cr!("701.26a", "608.2c");
    ruling!(
        "Sonic Assault",
        "Sonic Assault can target a tapped creature. The creature can't be tapped again, but Sonic Assault still deals 2 damage to that creature's controller."
    );
    let mut t = TestGame::new(2);
    let wurm = t.battlefield(P1, "Craw Wurm");
    t.g.tap(wurm);
    cast_new(&mut t, P0, "Sonic Assault", &[obj(wurm)]);
    t.resolve_all();
    assert_eq!(t.life(P1), 18);
}

#[test]
fn eelectrocute_cast_with_another_permission_isnt_exiled() {
    cr!("702.81a", "601.3");
    ruling!(
        "Eelectrocute",
        "If you cast Eelectrocute from your graveyard using another permission, you won't exile it instead (unless that permission also states that you exile it)."
    );
    // Retrace granted by a permanent (the text of Wrenn and Six's emblem), with a 6
    // rolled this turn too.
    let emblem = crate::r_s01_common::custom_card(
        "Retrace Granter",
        "Enchantment",
        "",
        None,
        "Instant and sorcery cards in your graveyard have retrace.",
    );
    for rolled in [false, true] {
        let mut t = TestGame::new(2);
        t.custom(P0, emblem.clone(), Zone::Battlefield);
        if rolled {
            roll(&mut t, P0, 6, 6);
        }
        t.lands(P0, "Mountain", 2);
        t.hand(P0, "Forest");
        let e = t.graveyard(P0, "Eelectrocute");
        t.cast(P0, e)
            .method(CastMethod::Keyword(KeywordKind::Retrace))
            .target(pl(P1))
            .go();
        t.resolve_all();
        assert_eq!(t.life(P1), 18);
        assert!(t.in_graveyard(P0, "Eelectrocute"), "rolled: {rolled}");
        assert!(!t.in_exile("Eelectrocute"));
    }
}
