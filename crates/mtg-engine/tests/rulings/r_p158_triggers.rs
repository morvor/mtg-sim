//! Rulings batch P158 — triggered abilities with self-replacement effects: when they
//! trigger (once per event, CR 603.2c; not for a teammate's life gain, CR 810.9; not for a
//! permanent that entered during the step, CR 603.2b), the order they resolve in relative
//! to the spell that caused them (CR 603.3, 405.5), what they check as they resolve
//! (CR 608.2h), full parties (CR 700.8), the Necrobloom's landfall and dredge
//! (CR 702.52a, 121.2, 614.11a), dice (CR 706.2) and a reflexive trigger (CR 603.12).

use crate::r_p158_common::*;
use mtg_engine::decision::Decision;
use mtg_engine::mana::ManaType;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

const WHITE_MAGE: &str = "The Destined White Mage";

/// The number of The Destined White Mage's life-gain triggers on the stack.
fn mage_triggers(t: &TestGame) -> usize {
    triggers_on_stack(t, "put a +1/+1 counter on target creature you control")
}

#[test]
fn white_mage_triggers_once_per_life_gain_event() {
    cr!("603.2c", "119.9");
    interp!("ruling-contradicting-oracle-gives-way");
    // The ruling calls the life-gain trigger the White Mage's "second ability"; it is its
    // third (Lifelink; "{W}, {T}: ..."; "Whenever you gain life, ..."), as the card's other
    // rulings say: that ordinal gives way to the Oracle text (exempt in
    // docs/rulings-exemptions/rulings-P158.tsv, logged in docs/RULING_DISCREPANCIES.md).
    ruling!(
        "The Destined White Mage",
        "triggers just once for each life-gaining event, whether it's 1 life from Al Bhed Salvagers or 3 life from Balamb T-Rexaur."
    );
    ruling!(
        "The Destined White Mage",
        "If you gain an amount of life \"for each\" of something or \"equal to the number\" of something, that life is gained as one event and The Destined White Mage's third ability triggers only once."
    );
    supported(WHITE_MAGE);
    supported("Congregate");
    for n in [1, 3] {
        let mut t = TestGame::new(2);
        let mage = t.battlefield(P0, WHITE_MAGE);
        gain(&mut t, P0, n);
        assert_eq!(mage_triggers(&t), 1, "{n} life");
        t.answer_targets(P0, &[obj(mage)]);
        t.resolve_all();
        assert_eq!(t.counters(mage, "+1/+1"), 1, "{n} life");
    }
    // Congregate: 2 life for each of three creatures, one event.
    let mut t = TestGame::new(2);
    let mage = t.battlefield(P0, WHITE_MAGE);
    t.battlefield(P0, "Grizzly Bears");
    t.battlefield(P1, "Hill Giant");
    cast_at(&mut t, P0, "Congregate", &[pl(P0)]);
    t.resolve();
    assert_eq!(t.life(P0), 26);
    assert_eq!(mage_triggers(&t), 1);
    t.answer_targets(P0, &[obj(mage)]);
    t.resolve_all();
    assert_eq!(t.counters(mage, "+1/+1"), 1);
}

#[test]
fn white_mage_doesnt_trigger_on_a_teammates_life_gain() {
    cr!("810.9", "119.9");
    ruling!(
        "The Destined White Mage",
        "In a Two-Headed Giant game, life gained by your teammate won't cause The Destined White Mage's third ability to trigger, even though it caused your team's life total to increase."
    );
    supported(WHITE_MAGE);
    let mut t = two_headed_giant();
    t.battlefield(P0, WHITE_MAGE);
    let team = t.life(P0);
    gain(&mut t, P1, 3);
    assert_eq!(t.life(P0), team + 3, "the team's life total rose");
    assert_eq!(mage_triggers(&t), 0);
    gain(&mut t, P0, 3);
    assert_eq!(mage_triggers(&t), 1);
}

#[test]
fn white_mage_cant_save_a_creature_dealt_lethal_damage_as_the_life_is_gained() {
    cr!("704.3", "704.5g", "510.2", "702.15b");
    interp!("ruling-contradicting-oracle-gives-way");
    // The ruling's "second ability" is the White Mage's third (see
    // `white_mage_triggers_once_per_life_gain_event`); the rest is cited.
    ruling!(
        "The Destined White Mage",
        "If a creature you control is dealt lethal damage at the same time that you gain life, it can't receive a counter from"
    );
    supported(WHITE_MAGE);
    // The Mage (2/3 lifelink) attacks into a Hill Giant: it deals 2 (P0 gains 2) and is
    // dealt 3 at the same time. It dies to state-based actions before its trigger is put
    // on the stack, so the trigger can only target the Bears, which get the counter.
    let mut t = TestGame::new(2);
    let mage = t.battlefield(P0, WHITE_MAGE);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let giant = t.battlefield(P1, "Hill Giant");
    let from = t.asked().len();
    t.attack(&[(mage, pl(P1))], &[(giant, mage)]);
    t.resolve_all();
    assert_eq!(t.life(P0), 22);
    assert!(!t.on_battlefield(mage));
    assert!(t.in_graveyard(P0, WHITE_MAGE));
    // The trigger was put on the stack with the Bears as its only choice.
    let choices = target_choices(&t, P0, from, "");
    assert!(!choices.is_empty(), "the trigger was put on the stack");
    for cands in &choices {
        assert!(cands.contains(&obj(bears)), "{cands:?}");
        assert!(!cands.contains(&obj(mage)) && !cands.contains(&obj(t.g.current(mage))), "{cands:?}");
    }
    assert_eq!(t.counters(bears, "+1/+1"), 1);
}

#[test]
fn a_full_party_is_four_creatures_in_the_party() {
    cr!("700.8", "700.8c", "614.1a");
    ruling!(
        "Coveted Prize",
        "Some cards refer to you having a \"full party.\" This is true if the number of creatures in your party is four."
    );
    ruling!(
        "The Destined Black Mage",
        "Some cards refer to you having a \"full party.\" This is true if the number of creatures in your party is four."
    );
    ruling!(
        "The Destined Thief",
        "Some cards refer to you having a \"full party.\" This is true if the number of creatures in your party is four."
    );
    ruling!(
        "The Destined Warrior",
        "Some cards refer to you having a \"full party.\" This is true if the number of creatures in your party is four."
    );
    ruling!(
        "The Destined White Mage",
        "Some cards refer to you having a \"full party.\" This is true if the number of creatures in your party is four."
    );
    ruling!(
        "The Destined Thief",
        "If you have a full party, you won't discard at all when The Destined Thief's last ability resolves."
    );
    supported("Coveted Prize");
    supported("The Destined Black Mage");
    supported("The Destined Thief");
    supported("The Destined Warrior");
    // Each card with a party of three (not full) and of four: the four vanilla party
    // members, minus one whose type the tested card doesn't supply itself (the White Mage
    // is a Cleric, the Thief a Rogue, the Warrior a Warrior, the Black Mage a Wizard).
    for full in [false, true] {
        let party = |t: &mut TestGame, drop: usize| {
            let ids = full_party(t, P0);
            if !full {
                kill(t, ids[drop]);
            }
        };
        // The White Mage: one counter, or three.
        let mut t = TestGame::new(2);
        party(&mut t, 1);
        let mage = t.battlefield(P0, WHITE_MAGE);
        t.answer_targets(P0, &[obj(mage)]);
        gain(&mut t, P0, 1);
        t.resolve_all();
        assert_eq!(t.counters(mage, "+1/+1"), if full { 3 } else { 1 }, "full {full}");
        // The Black Mage: 1 damage to each opponent, or 3.
        let mut t = TestGame::new(2);
        party(&mut t, 1);
        t.battlefield(P0, "The Destined Black Mage");
        cast_at(&mut t, P0, "Opt", &[]);
        t.resolve_all();
        assert_eq!(t.life(P1), if full { 17 } else { 19 }, "full {full}");
        // The Warrior: +1/+0 at the beginning of combat, or +3/+0.
        let mut t = TestGame::new(2);
        party(&mut t, 1);
        t.battlefield(P0, "The Destined Warrior");
        let bears = t.battlefield(P0, "Grizzly Bears");
        t.advance_to(P0, Step::BeginningOfCombat);
        t.resolve_all();
        assert_eq!(t.pt(bears), (if full { 5 } else { 3 }, 2), "full {full}");
        // The Thief: a creature deals combat damage to P1: draw then discard, or draw three.
        let mut t = TestGame::new(2);
        party(&mut t, 2);
        let thief = t.battlefield(P0, "The Destined Thief");
        let from = t.asked().len();
        t.attack(&[(thief, pl(P1))], &[]);
        t.resolve_all();
        assert_eq!(t.hand_size(P0), if full { 3 } else { 0 }, "full {full}");
        let discarded = asked_since(&t, from)
            .iter()
            .any(|(p, d)| *p == P0 && matches!(d, Decision::ChooseEntities { prompt, .. } if prompt.to_lowercase().contains("discard")));
        assert_eq!(discarded, !full, "full {full}: {:?}", asked_since(&t, from));
        // Coveted Prize: with a full party, a Grizzly Bears from the hand may be cast free.
        let mut t = TestGame::new(2);
        t.set_step(P0, Step::PrecombatMain);
        party(&mut t, 1);
        let bears = t.hand(P0, "Grizzly Bears");
        let tutored = t.library_top(P0, "Hill Giant");
        lots(&mut t, P0);
        let prize = t.hand(P0, "Coveted Prize");
        t.answer_choose(P0, &[obj(tutored)]);
        t.answer_yes(P0, true);
        t.answer_choose(P0, &[obj(bears)]);
        t.cast(P0, prize).go();
        t.resolve_all();
        assert!(t.in_hand(P0, "Hill Giant"), "full {full}");
        assert_eq!(t.on_battlefield(bears), full, "full {full}");
    }
}

#[test]
fn mila_triggers_once_per_attack_and_resolves_before_the_spell() {
    cr!("603.2c", "508.3d", "603.3", "405.5");
    ruling!(
        "Mila, Crafty Companion // Lukka, Wayward Bonder",
        "Mila’s first ability triggers only once no matter how many creatures are attacking or how many planeswalkers are being attacked."
    );
    ruling!(
        "Mila, Crafty Companion // Lukka, Wayward Bonder",
        "Mila’s second ability will resolve before the spell or ability that caused it to trigger."
    );
    supported("Mila, Crafty Companion // Lukka, Wayward Bonder");
    supported("Chandra, Torch of Defiance");
    supported("Jace, Unraveler of Secrets");
    // Two creatures attack two planeswalkers: one trigger, one counter on each.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Mila, Crafty Companion // Lukka, Wayward Bonder");
    let chandra = t.battlefield(P0, "Chandra, Torch of Defiance");
    let jace = t.battlefield(P0, "Jace, Unraveler of Secrets");
    let (c0, j0) = (t.counters(chandra, "loyalty"), t.counters(jace, "loyalty"));
    let a = t.battlefield(P1, "Grizzly Bears");
    let b = t.battlefield(P1, "Hill Giant");
    t.set_step(P1, Step::PrecombatMain);
    attack_with(&mut t, &[(a, obj(chandra)), (b, obj(jace))]);
    assert_eq!(triggers_on_stack(&t, "loyalty counter"), 1);
    t.resolve_all();
    assert_eq!(t.counters(chandra, "loyalty"), c0 + 1);
    assert_eq!(t.counters(jace, "loyalty"), j0 + 1);
    // P1's Shock at P0's Bears: Mila's trigger resolves (P0 draws) before Shock does.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Mila, Crafty Companion // Lukka, Wayward Bonder");
    let bears = t.battlefield(P0, "Grizzly Bears");
    lots(&mut t, P1);
    let shock = t.hand(P1, "Shock");
    let shock = t.cast(P1, shock).target(obj(bears)).go();
    t.settle();
    assert_eq!(t.stack_len(), 2);
    t.answer_yes(P0, true);
    t.resolve();
    assert_eq!(t.hand_size(P0), 1, "drew");
    assert!(t.g.stack.contains(&shock) && t.on_battlefield(bears));
    t.resolve();
    assert!(!t.on_battlefield(bears));
}

#[test]
fn cast_triggers_resolve_before_the_spell_even_if_it_is_countered() {
    cr!("603.3", "405.5", "601.2i");
    ruling!(
        "Prismari Pianist",
        "Prismari Pianist's ability resolves before the spell that caused it to trigger. It resolves even if that spell is countered or otherwise leaves the stack without resolving."
    );
    ruling!(
        "The Destined Black Mage",
        "The Destined Black Mage's last ability resolves before the spell that caused it to trigger. It resolves even if that spell is countered or otherwise leaves the stack."
    );
    supported("Prismari Pianist");
    supported("The Destined Black Mage");
    for (source, check) in [
        (
            "Prismari Pianist",
            (&|t: &TestGame| tokens(t, P0).len() == 1 && t.life(P1) == 20) as &dyn Fn(&TestGame) -> bool,
        ),
        ("The Destined Black Mage", &|t: &TestGame| t.life(P1) == 19),
    ] {
        // Resolves before the spell.
        let mut t = TestGame::new(2);
        t.battlefield(P0, source);
        let bears = t.battlefield(P1, "Grizzly Bears");
        let shock = cast_at(&mut t, P0, "Shock", &[obj(bears)]);
        t.settle();
        assert_eq!(t.stack_len(), 2, "{source}");
        t.resolve();
        assert!(t.g.stack.contains(&shock), "{source}: the trigger resolved first");
        assert!(check(&t), "{source}");
        // Resolves even when the spell is countered.
        let mut t = TestGame::new(2);
        t.battlefield(P0, source);
        let shock = cast_at(&mut t, P0, "Shock", &[pl(P1)]);
        t.settle();
        cast_at(&mut t, P1, "Counterspell", &[obj(shock)]);
        t.resolve();
        assert_eq!(t.zone(shock), Zone::Graveyard(P0), "{source}");
        t.resolve_all();
        assert!(check(&t), "{source}");
    }
}

#[test]
fn tezzerets_emblem_puts_a_permanent_in_after_the_end_step_began() {
    cr!("603.2b", "603.2", "513.1");
    ruling!(
        "Tezzeret, Artifice Master",
        "If you put a permanent onto the battlefield with Tezzeret’s emblem’s ability, any triggered abilities of that permanent that trigger during your end step won’t trigger during that same end step."
    );
    supported("Tezzeret, Artifice Master");
    supported("Voltaic Servant");
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let tez = t.battlefield(P0, "Tezzeret, Artifice Master");
    crate::r_s29_common::put_counters(&mut t, tez, "loyalty", 4);
    act(&mut t, P0, tez, 2, &[]);
    assert_eq!(t.g.find_in_zone(Zone::Command, "").len(), 1, "the emblem");
    // Voltaic Servant ("At the beginning of your end step, untap target artifact") comes
    // in during the end step: no trigger this end step.
    let servant = t.library_top(P0, "Voltaic Servant");
    let rock = t.battlefield(P0, "Sunglasses of Urza");
    t.g.tap(rock);
    t.answer_choose(P0, &[obj(servant)]);
    t.advance_to(P0, Step::End);
    t.resolve_all();
    assert!(t.on_battlefield(servant));
    assert_eq!(triggers_on_stack(&t, "untap target artifact"), 0);
    assert!(t.obj_now(rock).tapped, "no untap this end step");
    // The next end step, it triggers.
    pass_turn(&mut t);
    pass_turn(&mut t);
    t.answer_targets(P0, &[obj(rock)]);
    t.advance_to(P0, Step::End);
    t.resolve_all();
    assert!(!t.obj_now(rock).tapped);
}

/// P0 cycles the Yidaro card `card` (from their hand); its trigger goes on the stack.
fn cycle(t: &mut TestGame, card: ObjectId) {
    mana(t, P0, ManaType::R, 2);
    crate::r_s06_common::activate_containing(t, P0, card, "Cycling").expect("cycled");
    t.settle();
}

#[test]
fn yidaros_trigger_without_yidaro_in_the_graveyard_still_shuffles_unless_the_fourth_time() {
    cr!("702.29c", "701.24c", "400.7", "614.1a");
    ruling!(
        "Yidaro, Wandering Monster",
        "If you cycle Yidaro but it isn't in your graveyard as the triggered ability resolves, you can't put it onto the battlefield or shuffle it into your library. You'll still shuffle your library if you haven't cycled Yidaro four times this game."
    );
    supported("Yidaro, Wandering Monster");
    // The first time, exiled in response (the trigger is on the stack above the cycling
    // ability's draw): the library is still shuffled.
    let mut t = TestGame::new(2);
    let y = t.hand(P0, "Yidaro, Wandering Monster");
    cycle(&mut t, y);
    assert_eq!(t.stack_len(), 2);
    assert_eq!(t.zone(y), Zone::Graveyard(P0));
    crate::r_p198_common::exile(&mut t, y);
    let from = t.g.turn_events.len();
    t.resolve_all();
    assert!(crate::r_s33_common::shuffled_since(&t, from, P0));
    assert!(t.in_exile("Yidaro, Wandering Monster"));
    // The fourth time: it would be put onto the battlefield; exiled in response, nothing
    // happens and the library isn't shuffled.
    let mut t = TestGame::new(2);
    for _ in 0..3 {
        let y = t.hand(P0, "Yidaro, Wandering Monster");
        cycle(&mut t, y);
        t.resolve_all();
        assert!(!t.in_graveyard(P0, "Yidaro, Wandering Monster"));
    }
    let y = t.hand(P0, "Yidaro, Wandering Monster");
    cycle(&mut t, y);
    crate::r_p198_common::exile(&mut t, y);
    let from = t.g.turn_events.len();
    t.resolve_all();
    assert!(!crate::r_s33_common::shuffled_since(&t, from, P0));
    assert!(t.in_exile("Yidaro, Wandering Monster"));
    assert!(t.named_on_battlefield("Yidaro, Wandering Monster").is_empty());
    // (Left alone the fourth time, it enters.)
    let mut t = TestGame::new(2);
    for _ in 0..4 {
        let y = t.hand(P0, "Yidaro, Wandering Monster");
        cycle(&mut t, y);
        t.resolve_all();
    }
    assert_eq!(t.named_on_battlefield("Yidaro, Wandering Monster").len(), 1);
}

#[test]
fn shilgengar_uses_the_sacrificed_angels_toughness_as_it_last_existed() {
    cr!("608.2h", "701.21a");
    ruling!(
        "Shilgengar, Sire of Famine",
        "If you sacrificed an Angel to pay the cost of Shilgengar's second ability, use the toughness of that Angel as it last existed on the battlefield to determine how many Blood tokens to create."
    );
    supported("Shilgengar, Sire of Famine");
    for (pumped, blood) in [(false, 4), (true, 7)] {
        let mut t = TestGame::new(2);
        let s = t.battlefield(P0, "Shilgengar, Sire of Famine");
        let angel = t.battlefield(P0, "Serra Angel");
        if pumped {
            cast_at(&mut t, P0, "Giant Growth", &[obj(angel)]);
            t.resolve_all();
        }
        t.answer_choose(P0, &[obj(angel)]);
        act(&mut t, P0, s, 0, &[]);
        assert!(!t.on_battlefield(angel));
        assert_eq!(with_subtype(&t, P0, "Blood").len(), blood, "pumped {pumped}");
    }
}

const NECROBLOOM: &str = "The Necrobloom";

/// The tokens `p` controls with the subtype `ty` (The Necrobloom itself is a Plant).
fn tokens_typed(t: &TestGame, p: PlayerId, ty: &str) -> usize {
    tokens(t, p)
        .into_iter()
        .filter(|id| crate::r_p027_common::has_subtype(t, *id, ty))
        .count()
}

#[test]
fn the_necrobloom_counts_lands_with_different_names_including_those_entering_together() {
    cr!("603.2c", "608.2h", "201.2a");
    ruling!(
        "The Necrobloom",
        "If you control multiple lands with the same name, only one of those lands will count toward the seven or more required to create a Zombie. For example, if you control four lands named Forest, two named Plains, and one named Swamp, you control three lands with different names."
    );
    ruling!(
        "The Necrobloom",
        "If multiple lands enter the battlefield under your control simultaneously, all of those lands are counted. For example, if you sacrifice seven lands while resolving Scapeshift and search your library for seven lands with different names, you'll create seven Zombie tokens."
    );
    supported(NECROBLOOM);
    supported("Scapeshift");
    // Four Forests, two Plains, a Swamp, then a Mountain enters: four names, a Plant.
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    t.battlefield(P0, NECROBLOOM);
    t.lands(P0, "Forest", 4);
    t.lands(P0, "Plains", 2);
    t.lands(P0, "Swamp", 1);
    let m = t.hand(P0, "Mountain");
    t.play_land(P0, m).expect("land");
    t.resolve_all();
    assert_eq!(tokens_typed(&t, P0, "Plant"), 1);
    assert_eq!(tokens_typed(&t, P0, "Zombie"), 0);
    // Six names, then a seventh: a Zombie.
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    t.battlefield(P0, NECROBLOOM);
    for l in ["Forest", "Plains", "Swamp", "Mountain", "Island", "Wastes"] {
        t.lands(P0, l, 1);
    }
    let m = t.hand(P0, "Snow-Covered Forest");
    t.play_land(P0, m).expect("land");
    t.resolve_all();
    assert_eq!(tokens_typed(&t, P0, "Plant"), 0);
    assert_eq!(tokens_typed(&t, P0, "Zombie"), 1);
    // Scapeshift: seven Forests sacrificed, seven lands with different names enter at
    // once: seven Zombies.
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    t.battlefield(P0, NECROBLOOM);
    let forests = t.lands(P0, "Forest", 7);
    let found = stack_library(
        &mut t,
        P0,
        &["Plains", "Island", "Swamp", "Mountain", "Wastes", "Snow-Covered Forest", "Snow-Covered Plains"],
    );
    lots(&mut t, P0);
    let s = t.hand(P0, "Scapeshift");
    let sac: Vec<Entity> = forests.iter().map(|f| obj(*f)).collect();
    t.answer_choose(P0, &sac);
    let get: Vec<Entity> = found.iter().map(|f| obj(*f)).collect();
    t.answer_choose(P0, &get);
    t.cast(P0, s).go();
    t.resolve_all();
    assert!(found.iter().all(|f| t.on_battlefield(*f)));
    assert_eq!(tokens_typed(&t, P0, "Zombie"), 7);
    assert_eq!(tokens_typed(&t, P0, "Plant"), 0);
}

#[test]
fn necrobloom_dredge_replaces_each_draw_in_turn_with_no_actions_in_between() {
    cr!("702.52a", "121.2", "614.11a", "614.6");
    ruling!(
        "The Necrobloom",
        "If you're drawing multiple cards, each draw is performed one at a time. For example, if you're instructed to draw two cards and you replace the first draw with a dredge ability, another card with a dredge ability (including one that was put into your graveyard by the first dredge ability) may be used to replace the second draw."
    );
    ruling!(
        "The Necrobloom",
        "Once you've announced that you're applying a card's dredge ability to replace a draw, players can't take any action until you've put that card into your hand and put the top cards of your library into your graveyard."
    );
    supported(NECROBLOOM);
    // A Plains in the graveyard; the library: Opt, Forest, Shock, Divination, ... Drawing
    // two: the first draw dredges the Plains (milling Opt and the Forest), the second
    // dredges the Forest (milling Shock and Divination).
    let mut t = TestGame::new(2);
    t.battlefield(P0, NECROBLOOM);
    let plains = t.graveyard(P0, "Plains");
    let top = stack_library(&mut t, P0, &["Opt", "Forest", "Shock", "Divination", "Hill Giant"]);
    let forest = top[1];
    let from = t.asked().len();
    t.answer_yes(P0, true);
    t.answer_yes(P0, true);
    t.answer_choose_moved(P0, &[obj(forest)]);
    t.g.draw_cards(P0, 2);
    t.g.flush_events();
    t.settle();
    assert!(t.in_hand(P0, "Plains") && t.in_hand(P0, "Forest"), "{}", t.dump_log());
    assert_eq!(t.hand_size(P0), 2);
    assert!(t.in_graveyard(P0, "Opt") && t.in_graveyard(P0, "Shock") && t.in_graveyard(P0, "Divination"));
    assert_eq!(crate::r_s33_common::library_top_first(&t, P0)[0], top[4]);
    assert_eq!(crate::r_p051_common::drawn_this_turn(&t, P0), 0);
    assert!(t.g.is_live(t.g.current(plains)));
    // Nobody but P0 decided anything meanwhile.
    assert!(asked_since(&t, from).iter().all(|(p, _)| *p == P0), "{:?}", asked_since(&t, from));
}

#[test]
fn mr_house_triggers_on_four_or_more_on_any_die() {
    cr!("706.2", "603.2");
    ruling!(
        "Mr. House, President and CEO",
        "Some cards from other releases may instruct you to roll dice other than six-sided dice. Results of 4 or higher on those dice cause Mr. House's first ability to trigger."
    );
    supported("Mr. House, President and CEO");
    supported("Contact Other Plane");
    // A d20 (Contact Other Plane): 3 is nothing; 4 is a Robot; 20 is a Robot and a
    // Treasure.
    for (natural, robots, treasures) in [(3, 0, 0), (4, 1, 0), (20, 1, 1)] {
        let mut t = TestGame::new(2);
        t.battlefield(P0, "Mr. House, President and CEO");
        crate::r_s35_common::load_dice(&mut t, &[natural]);
        cast_at(&mut t, P0, "Contact Other Plane", &[]);
        t.resolve_all();
        assert_eq!(crate::r_s35_common::die_rolls(&t), vec![(P0, 20, natural, natural)]);
        assert_eq!(with_subtype(&t, P0, "Robot").len(), robots, "{natural}");
        assert_eq!(with_subtype(&t, P0, "Treasure").len(), treasures, "{natural}");
    }
}

#[test]
fn surtland_flinger_targets_with_a_reflexive_trigger_after_the_sacrifice() {
    cr!("603.12", "603.12a", "115.1");
    ruling!(
        "Surtland Flinger",
        "Surtland Flinger’s triggered ability goes on the stack without a target. While that ability is resolving, you may sacrifice another creature. When you do, the reflexive triggered ability triggers and you pick a target to be dealt damage. This is different from effects that say “If you do” in that players may take actions after you’ve sacrificed the creature but before damage is dealt."
    );
    supported("Surtland Flinger");
    for (fodder, dmg) in [("Grizzly Bears", 2), ("Lowland Giant", 8)] {
        let mut t = TestGame::new(2);
        let flinger = t.battlefield(P0, "Surtland Flinger");
        let fodder = t.battlefield(P0, fodder);
        attack_with(&mut t, &[(flinger, pl(P1))]);
        let trig = *t.g.stack.last().unwrap();
        assert!(targets_of(&t, trig).is_empty(), "no target yet");
        t.answer_yes(P0, true);
        t.answer_choose(P0, &[obj(fodder)]);
        t.answer_targets(P0, &[pl(P1)]);
        t.resolve();
        assert!(!t.on_battlefield(fodder));
        // The reflexive trigger waits on the stack with its target; P1 can act first.
        assert_eq!(t.stack_len(), 1);
        let reflexive = *t.g.stack.last().unwrap();
        assert_eq!(targets_of(&t, reflexive), vec![pl(P1)]);
        assert_eq!(t.life(P1), 20);
        lots(&mut t, P1);
        let s = t.hand(P1, "Shock");
        t.cast(P1, s).target(pl(P0)).go();
        t.resolve();
        assert_eq!(t.life(P0), 18);
        t.resolve();
        assert_eq!(t.life(P1), 20 - dmg, "{dmg}");
    }
}
