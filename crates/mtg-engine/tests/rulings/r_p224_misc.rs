//! Rulings batch P224 — split second, station, sunburst, support, surge, swampwalk and
//! threshold.

use crate::r_p224_common::*;
use crate::r_s01_common::*;
use crate::r_s02_common::can_cast;
use crate::r_s04_common::crew;
use crate::r_s05_common::{colors, move_to};
use crate::r_s08_common::mana_value;
use crate::r_s16_common::add_charge;
use mtg_engine::decision::Decision;
use mtg_engine::keywords::KeywordKind;
use mtg_engine::object::{CastMethod, Zone};
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

const SURGE: CastMethod = CastMethod::Keyword(KeywordKind::Surge);

#[test]
fn split_second_doesnt_stop_chalice_of_the_void_from_triggering() {
    cr!("702.61a", "702.61b", "603.3");
    ruling!(
        "Sudden Edict",
        "Split second doesn't stop triggered abilities from triggering, such as that of Chalice of the Void. If one does, its controller puts it on the stack and chooses targets for it, if any. Those abilities will resolve as normal"
    );
    supported("Sudden Edict");
    supported("Chalice of the Void");
    // Chalice of the Void with two charge counters: "Whenever a player casts a spell with
    // mana value equal to the number of charge counters on this artifact, counter that
    // spell."
    let mut t = TestGame::new(2);
    let chalice = t.battlefield(P1, "Chalice of the Void");
    add_charge(&mut t, chalice, 2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    // Sudden Edict ({1}{B}, split second): "Target player sacrifices a creature of their
    // choice."
    t.lands(P0, "Swamp", 2);
    let edict = t.hand(P0, "Sudden Edict");
    t.cast(P0, edict).target(P1).go();
    t.settle();
    assert_eq!(t.stack_len(), 2);
    t.resolve_all();
    assert!(t.on_battlefield(bears));
    assert!(t.in_graveyard(P0, "Sudden Edict"));
}

/// Synthesizer Labship (with two charge counters) and, for P0, `target` (an artifact):
/// at the beginning of combat, Labship's ability makes `target` an artifact creature with
/// base power and toughness 2/2 and flying until end of turn.
fn labship_animates(t: &mut TestGame, target: ObjectId) {
    let labship = t.battlefield(P0, "Synthesizer Labship");
    add_charge(t, labship, 2);
    t.answer_targets(P0, &[Entity::Object(target)]);
    t.advance_to(P0, Step::BeginningOfCombat);
    t.resolve_all();
    let o = t.obj_now(target);
    assert!(o.chars.is(CardType::Creature) && o.chars.is(CardType::Artifact));
    assert!(o.chars.has_keyword(KeywordKind::Flying));
    assert_eq!(t.pt(target), (2, 2));
}

#[test]
fn labships_base_2_2_isnt_overwritten_by_crewing_or_station_counters() {
    cr!("613.4b", "613.7", "721.2b", "702.122a");
    ruling!(
        "Synthesizer Labship",
        "If Synthesizer Labship’s second ability causes a Vehicle to become an artifact creature, its base power and toughness will be set to 2/2. Crewing that Vehicle will not restore its power and toughness."
    );
    ruling!(
        "Atmospheric Greenhouse",
        "If an effect causes a permanent with station to become a creature by some means other than having the appropriate number of charge counters on it, it won’t use any power and toughness values printed in its text box. Instead, it will use whatever base power and toughness was set by the effect that made it a creature."
    );
    supported("Synthesizer Labship");
    supported("Atmospheric Greenhouse");
    supported("Smuggler's Copter");
    // A Vehicle (Smuggler's Copter, 3/3, crew 1): crewing it keeps it 2/2.
    let mut t = TestGame::new(2);
    let copter = t.battlefield(P0, "Smuggler's Copter");
    let bears = t.battlefield(P0, "Grizzly Bears");
    labship_animates(&mut t, copter);
    assert!(crew(&mut t, P0, copter, &[bears]));
    t.resolve_all();
    assert_eq!(t.pt(copter), (2, 2));
    // A Spacecraft (Atmospheric Greenhouse, 5/4 at 8+): with no charge counters it's 2/2,
    // and still 2/2 once it has eight (it gains the abilities of that striation).
    let mut t = TestGame::new(2);
    let greenhouse = t.battlefield(P0, "Atmospheric Greenhouse");
    labship_animates(&mut t, greenhouse);
    add_charge(&mut t, greenhouse, 8);
    assert_eq!(t.pt(greenhouse), (2, 2));
    assert!(t
        .obj_now(greenhouse)
        .chars
        .has_keyword(KeywordKind::Trample));
}

#[test]
fn a_station_symbol_beside_a_pt_box_makes_it_a_creature_with_that_base_pt() {
    cr!("721.2b");
    ruling!(
        "Atmospheric Greenhouse",
        "If that station symbol is in the same striation as a power and toughness box, it instead means “As long as this permanent has N or more charge counters on it, it has [abilities] and is a creature with base power and toughness [P/T] in addition to its other types.”"
    );
    let mut t = TestGame::new(2);
    let ship = t.battlefield(P0, "Atmospheric Greenhouse");
    add_charge(&mut t, ship, 7);
    assert!(!t.obj_now(ship).chars.is(CardType::Creature));
    add_charge(&mut t, ship, 1);
    let o = t.obj_now(ship);
    assert!(o.chars.is(CardType::Creature) && o.chars.is(CardType::Artifact));
    assert!(o.chars.has_subtype("Spacecraft"));
    assert!(o.chars.has_keyword(KeywordKind::Flying) && o.chars.has_keyword(KeywordKind::Trample));
    assert_eq!(t.pt(ship), (5, 4));
    // Base power and toughness: a +1/+1 counter applies on top of it.
    t.g.add_counters(Entity::Object(ship), counters::PLUS1, 1, None);
    t.g.recompute();
    assert_eq!(t.pt(ship), (6, 5));
}

#[test]
fn sunburst_counts_the_colors_of_mana_actually_spent() {
    cr!("702.44a", "702.44b", "609.4b");
    ruling!(
        "Pentad Prism",
        "Sunburst checks what mana was actually spent to cast the spell. If an effect allows you to spend mana \"as though it were mana\" of any color or type, that allows you to spend mana you couldn't otherwise spend, but it doesn't change what mana you spent to cast the spell."
    );
    supported("Pentad Prism");
    supported("Chromatic Orrery");
    // Chromatic Orrery: "You may spend mana as though it were mana of any color."
    let prism = |lands: &[&str]| {
        let mut t = TestGame::new(2);
        t.battlefield(P0, "Chromatic Orrery");
        for l in lands {
            t.lands(P0, l, 1);
        }
        let c = t.hand(P0, "Pentad Prism");
        t.cast(P0, c).go();
        t.resolve_all();
        t.counters(t.g.current(c), counters::CHARGE)
    };
    assert_eq!(prism(&["Mountain", "Forest"]), 2);
    assert_eq!(prism(&["Mountain", "Mountain"]), 1);
    // Colorless mana spent "as though it were mana of any color" is still colorless.
    assert_eq!(prism(&["Wastes", "Wastes"]), 0);
}

#[test]
fn press_into_service_last_target_can_be_any_creature() {
    cr!("115.3", "701.41a");
    ruling!(
        "Press into Service",
        "The last target of Press into Service can be any creature, even one that’s untapped, one you already control, or one targeted by the support part of the spell."
    );
    // Press into Service ({4}{R}): "Support 2. Gain control of target creature until end
    // of turn. Untap that creature. It gains haste until end of turn."
    let run = |owner: PlayerId, tapped: bool| {
        let mut t = TestGame::new(2);
        let bears = t.battlefield_sick(owner, "Grizzly Bears");
        t.g.objects[bears.0 as usize].tapped = tapped;
        t.lands(P0, "Mountain", 5);
        let spell = t.hand(P0, "Press into Service");
        t.cast(P0, spell)
            .targets(&[bears.into()])
            .target(bears)
            .go();
        t.resolve_all();
        let o = t.obj_now(bears);
        assert_eq!(o.controller, P0);
        assert!(!o.tapped);
        assert!(o.chars.has_keyword(KeywordKind::Haste));
        assert_eq!(t.counters(bears, counters::PLUS1), 1);
    };
    // An untapped creature P1 controls, and one P0 already controls (also the target of
    // the support part).
    run(P1, false);
    run(P0, false);
    run(P0, true);
}

#[test]
fn jubilant_mascot_targets_on_the_stack_and_pays_on_resolution() {
    cr!("603.3d", "601.2c", "701.41a");
    ruling!(
        "Jubilant Mascot",
        "You choose the target creatures to support when Jubilant Mascot's triggered ability is put onto the stack, but you don't choose whether to pay {3}{W} until the ability resolves."
    );
    supported("Jubilant Mascot");
    // Jubilant Mascot: "At the beginning of combat on your turn, you may pay {3}{W}. If
    // you do, support 2."
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Jubilant Mascot");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let giant = t.battlefield(P0, "Hill Giant");
    t.lands(P0, "Plains", 4);
    // When P0 decides whether to pay, the ability is on the stack with its targets.
    let seen = watch(
        &mut t,
        P0,
        |d| matches!(d, Decision::YesNo { .. } | Decision::OptionalCost { .. }),
        |g| {
            g.stack
                .last()
                .and_then(|id| g.obj(*id).stack.as_ref())
                .map(|si| si.chosen.iter().flat_map(|c| c.targets.concat()).count())
        },
    );
    t.answer_targets(P0, &[bears.into(), giant.into()]);
    let from = t.asked().len();
    t.advance_to(P0, Step::BeginningOfCombat);
    t.settle();
    assert_eq!(t.stack_len(), 1);
    let asked = t.asked()[from..].to_vec();
    assert!(asked
        .iter()
        .any(|(_, d)| matches!(d, Decision::ChooseTargets { .. })));
    t.answer_yes(P0, true);
    t.resolve_all();
    assert_eq!(*seen.lock().unwrap(), vec![Some(2)]);
    assert_eq!(t.counters(bears, counters::PLUS1), 1);
    assert_eq!(t.counters(giant, counters::PLUS1), 1);
}

#[test]
fn surge_doesnt_change_the_mana_cost_or_mana_value() {
    cr!("702.117a", "202.3", "118.9c");
    ruling!(
        "Boulder Salvo",
        "Casting a spell for its surge cost doesn’t change its mana cost or its mana value."
    );
    ruling!(
        "Fall of the Titans",
        "Fall of the Titan's mana value is based on its mana cost of {X}{X}{R}, even if you're casting it for its surge cost. For example, if you cast Fall of the Titans for its surge cost and choose 4 for X, its mana value will be 9."
    );
    supported("Boulder Salvo");
    supported("Fall of the Titans");
    let mut t = TestGame::new(2);
    bolt(&mut t, P0, Entity::Player(P1));
    t.resolve_all();
    // Boulder Salvo ({4}{R}, surge {1}{R}): mana value 5.
    let giant = t.battlefield(P1, "Hill Giant");
    let lands = t.lands(P0, "Mountain", 2);
    let c = t.hand(P0, "Boulder Salvo");
    let spell = t.cast(P0, c).method(SURGE).target(giant).go();
    assert!(lands.iter().all(|l| t.obj(*l).tapped));
    assert_eq!(mana_value(&t, spell), 5);
    t.resolve_all();
    // Fall of the Titans ({X}{X}{R}, surge {X}{R}) with X = 4: five mana paid, mana
    // value 9.
    let lands = t.lands(P0, "Mountain", 5);
    let c = t.hand(P0, "Fall of the Titans");
    let spell = t.cast(P0, c).method(SURGE).x(4).targets(&[P1.into()]).go();
    assert!(lands.iter().all(|l| t.obj(*l).tapped));
    assert_eq!(mana_value(&t, spell), 9);
    t.resolve_all();
    assert_eq!(t.life(P1), 20 - 3 - 4);
}

#[test]
fn surge_counts_a_countered_spell_or_one_still_on_the_stack() {
    cr!("702.117a");
    ruling!(
        "Boulder Salvo",
        "The other spell that you or a teammate cast can be one that’s resolved, one that was countered, or (for instants with surge) one that’s still on the stack."
    );
    // A spell that was countered.
    let mut t = TestGame::new(2);
    t.lands(P0, "Mountain", 2);
    let salvo = t.hand(P0, "Boulder Salvo");
    t.battlefield(P1, "Hill Giant");
    assert!(!can_cast(&mut t, P0, salvo, SURGE));
    let b = bolt(&mut t, P0, Entity::Player(P1));
    counterspell(&mut t, P1, b);
    t.resolve_all();
    assert_eq!(t.life(P1), 20);
    assert!(can_cast(&mut t, P0, salvo, SURGE));
    // A spell still on the stack, for an instant with surge (Fall of the Titans).
    let mut t = TestGame::new(2);
    t.lands(P0, "Mountain", 2);
    let fall = t.hand(P0, "Fall of the Titans");
    assert!(!can_cast(&mut t, P0, fall, SURGE));
    bolt(&mut t, P0, Entity::Player(P1));
    assert_eq!(t.stack_len(), 1);
    assert!(can_cast(&mut t, P0, fall, SURGE));
}

#[test]
fn swampwalk_cares_about_the_land_type_swamp() {
    cr!("702.14c", "305.6");
    ruling!(
        "Sheoldred, Whispering One",
        "Swampwalk is a keyword ability that means “This creature can't be blocked if defending player controls a Swamp.” Most nonbasic lands don't have basic land types, even if they produce colored mana. For example, Graven Cairns is neither a Swamp nor a Mountain, while Blood Crypt is both."
    );
    supported("Sheoldred, Whispering One");
    supported("Graven Cairns");
    supported("Blood Crypt");
    let blockable = |land: &str| {
        let mut t = TestGame::new(2);
        let sheoldred = t.battlefield(P0, "Sheoldred, Whispering One");
        t.battlefield(P1, land);
        let wall = t.battlefield(P1, "Wall of Wood");
        attack_with(&mut t, &[(sheoldred, Entity::Player(P1))]);
        t.g.can_block(t.g.current(wall), t.g.current(sheoldred))
    };
    assert!(blockable("Graven Cairns"));
    assert!(!blockable("Blood Crypt"));
    assert!(!blockable("Swamp"));
}

/// P0 attacks with a Crypt Feaster with `yard` cards in their graveyard; `exile` of them
/// are exiled while its threshold trigger (if any) is on the stack. Returns whether it
/// triggered and its power after the trigger resolved.
fn crypt_feaster(yard: usize, exile: usize) -> (bool, i32) {
    let mut t = TestGame::new(2);
    let feaster = t.battlefield(P0, "Crypt Feaster");
    let cards: Vec<ObjectId> = (0..yard).map(|_| t.graveyard(P0, "Swamp")).collect();
    attack_with(&mut t, &[(feaster, Entity::Player(P1))]);
    let triggered = t.stack_len() > 0;
    for c in cards.iter().take(exile) {
        move_to(&mut t, *c, Zone::Exile);
    }
    t.resolve_all();
    (triggered, t.pt(feaster).0)
}

#[test]
fn crypt_feasters_threshold_checks_on_trigger_and_on_resolution() {
    cr!("603.4");
    ruling!(
        "Crypt Feaster",
        "Crypt Feaster's threshold ability checks your graveyard at the moment it would trigger to see if you have seven or more cards in your graveyard. If you don't, the ability won't trigger at all. If it does trigger, the ability will check again as it tries to resolve. If you don't have seven or more cards in your graveyard at that time, the ability won't resolve and none of its effects will happen."
    );
    supported("Crypt Feaster");
    // "Threshold — Whenever this creature attacks, if there are seven or more cards in your
    // graveyard, this creature gets +2/+0 until end of turn."
    assert_eq!(crypt_feaster(6, 0), (false, 3));
    assert_eq!(crypt_feaster(7, 0), (true, 5));
    assert_eq!(crypt_feaster(7, 1), (true, 3));
}

#[test]
fn repentant_vampire_with_threshold_is_only_white() {
    cr!("105.3", "613.1e");
    ruling!(
        "Repentant Vampire",
        "If Threshold is met, this card is white. It is not white and black."
    );
    supported("Repentant Vampire");
    let mut t = TestGame::new(2);
    let vampire = t.battlefield(P0, "Repentant Vampire");
    for _ in 0..6 {
        t.graveyard(P0, "Swamp");
    }
    t.g.recompute();
    assert_eq!(colors(&t, vampire), ColorSet::single(Color::Black));
    t.graveyard(P0, "Swamp");
    t.g.recompute();
    assert_eq!(colors(&t, vampire), ColorSet::single(Color::White));
}
