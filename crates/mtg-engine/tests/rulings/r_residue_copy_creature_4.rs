//! Rulings residue of batches P020, P021 and P023 — interactions of creatures that copy:
//! effects that applied before a permanent became a copy keep applying (CR 613.1a, 707.2),
//! becoming a copy isn't entering (CR 603.6a, 614.1c), damage stays marked (CR 120.6),
//! targeted and untargeted choices (CR 115.1, 614.12), copiable exceptions (CR 707.9b),
//! characteristic-defining abilities in graveyards (CR 604.3), and triggers that resolve
//! after their source leaves (CR 113.7a).

use crate::r_p020_common::RIFTWATCHER;
use crate::r_residue_copy_creature::*;
use crate::r_s01_common::supported;
use crate::r_s18_common::lands_for;
use crate::r_s26_common::{modify_until_eot, new_tokens};
use mtg_engine::ability::{Modification, Value};
use mtg_engine::keywords::KeywordKind;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

/// Gives the permanent +3/+3 until end of turn (as Giant Growth would: a non-copy effect).
pub fn pump(t: &mut TestGame, id: ObjectId) {
    modify_until_eot(
        t,
        id,
        vec![Modification::ModifyPT(Value::c(3), Value::c(3))],
    );
}

fn plus1(t: &mut TestGame, id: ObjectId) {
    let id = t.g.current(id);
    t.g.add_counters(obj(id), counters::PLUS1, 1, None);
    t.g.recompute();
}

/// Advances from the current turn to P0's next upkeep and puts the triggers on the stack.
fn to_p0_upkeep(t: &mut TestGame) {
    t.set_step(P1, Step::End);
    t.advance_to(P0, Step::Upkeep);
    t.settle();
}

// --- Characteristic-defining abilities in a graveyard -----------------------------------

#[test]
fn soul_separator_zombie_uses_a_cda_in_the_graveyard_and_the_spirit_is_1_1() {
    cr!("604.3", "604.3a", "707.9b", "707.9d");
    ruling!(
        "Soul Separator",
        "Abilities that define a creature’s power and toughness apply while that card is in your graveyard, but abilities that add or subtract from it don’t. For example, the ability of Sage of Ancient Lore applies to determine the Zombie token’s power and toughness, but the ability of Liliana’s Elite doesn’t."
    );
    ruling!(
        "Soul Separator",
        "If the copied creature card has an ability that defines its power and toughness, such as that of Sage of Ancient Lore, the Spirit token will still have base power and toughness 1/1."
    );
    // "{5}, {T}, Sacrifice this artifact: Exile target creature card from your graveyard.
    // Create a token that's a copy of that card, except it's 1/1, it's a Spirit in addition
    // to its other types, and it has flying. Create a black Zombie creature token with
    // power equal to that card's power and toughness equal to that card's toughness."
    supported("Soul Separator");
    supported("Sage of Ancient Lore // Werewolf of Ancient Hunger");
    supported("Liliana's Elite");
    let run = |name: &str| {
        let mut t = TestGame::new(2);
        let sep = t.battlefield(P0, "Soul Separator");
        let card = t.graveyard(P0, name);
        // At least three cards in hand, two other creature cards in the graveyard.
        for _ in 0..3 {
            t.hand(P0, "Island");
        }
        t.graveyard(P0, "Grizzly Bears");
        t.graveyard(P0, "Hill Giant");
        lands_for(&mut t, P0, "{5}");
        let hand = t.hand_size(P0) as i32;
        let before = t.g.battlefield.clone();
        t.answer_targets(P0, &[obj(card)]);
        crate::r_s06_common::activate_containing(&mut t, P0, sep, "Exile target").unwrap();
        t.resolve_all();
        let toks = new_tokens(&t, P0, &before);
        assert_eq!(toks.len(), 2, "{}", t.dump_log());
        // (Liliana's Elite is a Zombie too: the Spirit is the one with flying.)
        let spirit = toks
            .iter()
            .copied()
            .find(|id| has_kw(&t, *id, KeywordKind::Flying))
            .unwrap();
        let zombie = toks.iter().copied().find(|id| *id != spirit).unwrap();
        assert!(has_subtype(&t, zombie, "Zombie") && has_subtype(&t, spirit, "Spirit"));
        (hand, t.pt(zombie), t.pt(spirit))
    };
    // Sage of Ancient Lore: "Sage of Ancient Lore's power and toughness are each equal to
    // the number of cards in your hand." (The Spirit copy's "When this creature enters,
    // draw a card." draws one after the Zombie is created.)
    let (hand, zombie, spirit) = run("Sage of Ancient Lore // Werewolf of Ancient Hunger");
    assert!(hand >= 3);
    assert_eq!(zombie, (hand, hand));
    assert_eq!(spirit, (1, 1));
    // Liliana's Elite (1/1): "Liliana's Elite gets +1/+1 for each creature card in your
    // graveyard." — not in the graveyard.
    let (_, zombie, spirit) = run("Liliana's Elite");
    assert_eq!(zombie, (1, 1));
    // The Spirit copy is a 1/1 that has the ability, on the battlefield: +2/+2 for the
    // Grizzly Bears and Hill Giant cards.
    assert_eq!(spirit, (3, 3));
}

// --- Volrath, the Shapestealer -----------------------------------------------------------

const VOLRATH: &str = "Volrath, the Shapestealer";

#[test]
fn volrath_copying_before_combat_loses_its_combat_trigger_and_can_copy_after_it() {
    cr!("707.9a", "613.1a", "507.1", "117.3a");
    ruling!(
        "Volrath, the Shapestealer",
        "Activating Volrath's last ability causes it to lose its first ability until your next turn, so if you activate it before the beginning of combat step, it won't trigger during that turn. Players can cast instant spells and activate abilities (such as the second ability) during the beginning of combat step after that first ability has triggered or resolved but before attackers are chosen in the declare attackers step."
    );
    // "At the beginning of combat on your turn, put a -1/-1 counter on up to one target
    // creature. / {1}: Until your next turn, Volrath becomes a copy of target creature with
    // a counter on it, except it's 7/5 and it has this ability."
    supported(VOLRATH);
    // Activated in the main phase: no trigger at the beginning of combat.
    let mut t = TestGame::new(2);
    let v = t.battlefield(P0, VOLRATH);
    t.lands(P0, "Wastes", 1);
    let bears = t.battlefield(P0, "Grizzly Bears");
    plus1(&mut t, bears);
    let giant = t.battlefield(P1, "Hill Giant");
    t.set_step(P0, Step::PrecombatMain);
    t.answer_targets(P0, &[obj(bears)]);
    crate::r_s06_common::activate_containing(&mut t, P0, v, "becomes a copy").unwrap();
    t.resolve_all();
    assert_eq!(name_of(&t, v), "Grizzly Bears");
    assert_eq!(abilities_with(&t, v, "-1/-1 counter"), 0);
    t.answer_targets(P0, &[obj(giant)]);
    t.advance_to(P0, Step::BeginningOfCombat);
    t.settle();
    assert_eq!(t.stack_len(), 0);
    t.resolve_all();
    assert_eq!(t.counters(giant, counters::MINUS1), 0);
    // Not activated: the trigger resolves (a -1/-1 counter on P1's Hill Giant), and then,
    // still in the beginning of combat step, Volrath copies the Giant, which now has a
    // counter on it.
    let mut t = TestGame::new(2);
    let v = t.battlefield(P0, VOLRATH);
    t.lands(P0, "Wastes", 1);
    let giant = t.battlefield(P1, "Hill Giant");
    t.set_step(P0, Step::PrecombatMain);
    t.answer_targets(P0, &[obj(giant)]);
    t.advance_to(P0, Step::BeginningOfCombat);
    t.resolve_all();
    assert_eq!(t.counters(giant, counters::MINUS1), 1);
    assert_eq!(t.g.turn.step, Step::BeginningOfCombat);
    t.answer_targets(P0, &[obj(giant)]);
    crate::r_s06_common::activate_containing(&mut t, P0, v, "becomes a copy").unwrap();
    t.resolve_all();
    assert_eq!(t.g.turn.step, Step::BeginningOfCombat);
    assert_eq!(name_of(&t, v), "Hill Giant");
    assert_eq!(t.pt(v), (7, 5));
}

// --- Vesuvan Doppelganger ----------------------------------------------------------------

const DOPPELGANGER: &str = "Vesuvan Doppelganger";

/// P0's Vesuvan Doppelganger enters as a copy of `what`.
fn doppelganger(t: &mut TestGame, what: ObjectId) -> ObjectId {
    supported(DOPPELGANGER);
    t.answer_yes(P0, true);
    let d = crate::r_p023_common::enter_copying(t, P0, DOPPELGANGER, what);
    t.clear_answers();
    assert_eq!(name_of(t, d), name_of(t, what));
    crate::r_p023_common::unsick(t, d);
    d
}

#[test]
fn vesuvan_doppelganger_as_enters_choice_isnt_targeted_its_upkeep_trigger_is() {
    cr!("115.1", "115.10", "702.11b", "614.12");
    ruling!(
        "Vesuvan Doppelganger",
        "Although Vesuvan Doppelganger's triggered ability is targeted, its \"as this creature enters\" ability is not."
    );
    // Gladecover Scout has hexproof: the Doppelganger can enter as a copy of it, but its
    // upkeep trigger ("you may have this creature become a copy of target creature") can't
    // target it.
    let mut t = TestGame::new(2);
    let scout = t.battlefield(P1, "Gladecover Scout");
    let bears = t.battlefield(P1, "Grizzly Bears");
    let d = doppelganger(&mut t, scout);
    assert_eq!(name_of(&t, d), "Gladecover Scout");
    let from = t.asked().len();
    t.answer_targets(P0, &[obj(bears)]);
    t.answer_yes(P0, true);
    to_p0_upkeep(&mut t);
    let offered = crate::r_s02_common::target_candidates(&t, P0, from);
    assert!(!offered.is_empty(), "{}", t.dump_log());
    assert!(offered[0].contains(&obj(bears)) && !offered[0].contains(&obj(scout)));
    t.resolve_all();
    assert_eq!(name_of(&t, d), "Grizzly Bears");
}

#[test]
fn vesuvan_doppelganger_can_switch_to_the_same_creature_and_keeps_its_damage() {
    cr!("707.2", "120.6", "704.5g", "613.1a");
    ruling!(
        "Vesuvan Doppelganger",
        "Can switch to the same creature it is currently a copy of."
    );
    ruling!(
        "Vesuvan Doppelganger",
        "Damage is not removed when it changes forms."
    );
    // A copy of P1's Hill Giant targets that Hill Giant: still a (blue) Hill Giant with
    // the upkeep ability.
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P1, "Hill Giant");
    let d = doppelganger(&mut t, giant);
    let from = t.asked().len();
    t.answer_targets(P0, &[obj(giant)]);
    t.answer_yes(P0, true);
    to_p0_upkeep(&mut t);
    let offered = crate::r_s02_common::target_candidates(&t, P0, from);
    assert!(offered[0].contains(&obj(giant)));
    t.resolve_all();
    assert_eq!(name_of(&t, d), "Hill Giant");
    assert_eq!(t.obj_now(d).chars.colors, ColorSet::single(Color::Blue));
    assert_eq!(abilities_with(&t, d, "upkeep"), 1);
    // A copy of Serra Angel (4/4) is dealt 2 damage with its upkeep trigger waiting, then
    // becomes a copy of Grizzly Bears (2/2): the damage stays and it dies.
    let mut t = TestGame::new(2);
    let angel = t.battlefield(P1, "Serra Angel");
    let bears = t.battlefield(P1, "Grizzly Bears");
    let d = doppelganger(&mut t, angel);
    t.answer_targets(P0, &[obj(bears)]);
    t.answer_yes(P0, true);
    to_p0_upkeep(&mut t);
    assert_eq!(t.stack_len(), 1);
    let source = t.battlefield(P1, "Prodigal Pyromancer");
    crate::r_s06_common::damage(&mut t, source, 2, d);
    assert!(t.on_battlefield(d));
    assert_eq!(t.obj_now(d).damage, 2);
    t.resolve_all();
    assert!(!t.on_battlefield(d), "{}", t.dump_log());
    assert!(t.in_graveyard(P0, DOPPELGANGER));
}

// --- Non-copy effects that applied before it became a copy ---------------------------------

#[test]
fn effects_and_counters_from_before_becoming_a_copy_still_apply() {
    cr!("613.1a", "613.4c", "707.2", "122.1a", "301.5c");
    ruling!(
        "Lazav, Familiar Stranger",
        "Any effects that applied to Lazav before it becomes a copy of another card will continue to apply once it becomes a copy. The same is true of any counters that are on Lazav."
    );
    ruling!(
        "Vesuvan Drifter",
        "Any non-copy effect that began to apply to Vesuvan Drifter before it became a copy of a creature card will continue to apply to it after it becomes a copy. Auras and Equipment attached to Vesuvan Drifter will continue to work, as well as counters that change its power and toughness."
    );
    ruling!(
        "Assimilation Aegis",
        "Any non-copy effects that applied to the creature Assimilation Aegis is attached to before it becomes a copy of another card will continue to apply once it becomes a copy. The same is true of any counters that are on that creature."
    );
    ruling!(
        "Deceiver of Form",
        "Any noncopy effects that affected a creature you control, including ones that modified its power and/or toughness, will continue to apply."
    );
    // Lazav: "Whenever you commit a crime, put a +1/+1 counter on Lazav. Then you may
    // exile a card from a graveyard. If a creature card was exiled this way, you may have
    // Lazav become a copy of that card until end of turn." It had +3/+3 and a +1/+1
    // counter; Shock targeting P1 is a crime: a Hill Giant 3/3 with two counters and +3/+3.
    supported("Lazav, Familiar Stranger");
    let mut t = TestGame::new(2);
    let lazav = t.battlefield(P0, "Lazav, Familiar Stranger");
    plus1(&mut t, lazav);
    pump(&mut t, lazav);
    let card = t.graveyard(P1, "Hill Giant");
    t.answer_choose_moved(P0, &[obj(card)]);
    t.answer_yes(P0, true);
    crate::r_s25_common::cast_new(&mut t, P0, "Shock", &[Entity::Player(P1)]);
    t.resolve_all();
    assert_eq!(name_of(&t, lazav), "Hill Giant");
    assert_eq!(t.counters(lazav, counters::PLUS1), 2);
    assert_eq!(t.pt(lazav), (3 + 2 + 3, 3 + 2 + 3));
    // Vesuvan Drifter: "At the beginning of each combat, you may reveal the top card of
    // your library. If you reveal a creature card this way, this creature becomes a copy
    // of that card until end of turn, except it has flying." It has Rancor (+2/+0 and
    // trample), Bonesplitter (+2/+0), a +1/+1 counter and +3/+3.
    supported("Vesuvan Drifter");
    let mut t = TestGame::new(2);
    let drifter = t.battlefield(P0, "Vesuvan Drifter");
    crate::r_s06_common::attach_new(&mut t, P0, "Rancor", drifter);
    crate::r_s06_common::attach_new(&mut t, P0, "Bonesplitter", drifter);
    plus1(&mut t, drifter);
    t.set_step(P0, Step::PrecombatMain);
    pump(&mut t, drifter);
    t.library_top(P0, "Hill Giant");
    t.answer_yes(P0, true);
    t.advance_to(P0, Step::BeginningOfCombat);
    t.resolve_all();
    assert_eq!(name_of(&t, drifter), "Hill Giant");
    assert!(has_kw(&t, drifter, KeywordKind::Flying) && has_kw(&t, drifter, KeywordKind::Trample));
    assert_eq!(t.g.attachments_of(obj(drifter)).len(), 2);
    assert_eq!(t.pt(drifter), (3 + 2 + 2 + 1 + 3, 3 + 1 + 3));
    // Assimilation Aegis: "When this Equipment enters, exile up to one target creature
    // until this Equipment leaves the battlefield. / Whenever this Equipment becomes
    // attached to a creature, for as long as this Equipment remains attached to it, that
    // creature becomes a copy of a creature card exiled with this Equipment." It exiled
    // P1's Hill Giant; it's attached to a Grizzly Bears with a counter and +3/+3.
    supported("Assimilation Aegis");
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P1, "Hill Giant");
    let bears = t.battlefield(P0, "Grizzly Bears");
    plus1(&mut t, bears);
    pump(&mut t, bears);
    t.answer_targets(P0, &[obj(giant)]);
    let aegis = t.enter(P0, "Assimilation Aegis");
    t.settle();
    t.resolve_all();
    assert!(t.in_exile("Hill Giant"));
    t.set_step(P0, Step::PrecombatMain);
    t.lands(P0, "Wastes", 2);
    t.answer_targets(P0, &[obj(bears)]);
    crate::r_s06_common::activate_containing(&mut t, P0, aegis, "Equip").unwrap();
    t.resolve_all();
    assert_eq!(name_of(&t, bears), "Hill Giant");
    assert_eq!(t.pt(bears), (3 + 1 + 3, 3 + 1 + 3));
    // Deceiver of Form: "At the beginning of combat on your turn, reveal the top card of
    // your library. If a creature card is revealed this way, you may have creatures you
    // control other than this creature become copies of that card until end of turn. You
    // may put that card on the bottom of your library."
    supported("Deceiver of Form");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Deceiver of Form");
    let bears = t.battlefield(P0, "Grizzly Bears");
    plus1(&mut t, bears);
    t.set_step(P0, Step::PrecombatMain);
    pump(&mut t, bears);
    t.library_top(P0, "Hill Giant");
    t.answer_yes(P0, true);
    t.answer_yes(P0, false);
    t.advance_to(P0, Step::BeginningOfCombat);
    t.resolve_all();
    assert_eq!(name_of(&t, bears), "Hill Giant");
    assert_eq!(t.pt(bears), (3 + 1 + 3, 3 + 1 + 3));
}

// --- Becoming a copy isn't entering -------------------------------------------------------

#[test]
fn becoming_a_copy_doesnt_apply_the_copied_enters_abilities() {
    cr!("707.2", "603.6a", "614.1c", "718.2a");
    ruling!(
        "March from Velis Vel",
        "Because the lands aren't entering the battlefield when they become copies of the creature, any \"When [this creature] enters the battlefield\" or \"[This creature] enters the battlefield with\" abilities of the copied creature won't apply."
    );
    ruling!(
        "The Everflowing Well // The Myriad Pools",
        "Because the target permanent isn't entering the battlefield when it becomes a copy of a permanent spell, any \"When [this permanent] enters the battlefield\" or \"[This permanent] enters the battlefield with\" abilities of the copied spell won't apply for the target permanent."
    );
    // March from Velis Vel: the Gate becomes a copy of P0's Aven Riftwatcher: no time
    // counters, no life.
    let mut t = TestGame::new(2);
    let rift = t.battlefield(P0, RIFTWATCHER);
    let life = t.life(P0);
    let gate = copy_with(&mut t, "March from Velis Vel", rift)[0];
    assert_eq!(name_of(&t, gate), RIFTWATCHER);
    assert_eq!(t.counters(gate, counters::TIME), 0);
    assert_eq!(t.life(P0), life);
    // The Myriad Pools: "Whenever you cast a permanent spell using mana produced by The
    // Myriad Pools, up to one other target permanent you control becomes a copy of that
    // spell until end of turn." P0 casts Aven Riftwatcher ({2}{W}) with the Pools' mana;
    // its Hill Giant becomes a copy: no time counters on it, and only the real
    // Riftwatcher's enters trigger gains life.
    supported("The Everflowing Well // The Myriad Pools");
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let well = t.battlefield(P0, "The Everflowing Well // The Myriad Pools");
    assert!(mtg_engine::dfc::transform(&mut t.g, well));
    t.g.recompute();
    let giant = t.battlefield(P0, "Hill Giant");
    t.lands(P0, "Plains", 1);
    t.lands(P0, "Wastes", 1);
    let life = t.life(P0);
    let card = t.hand(P0, RIFTWATCHER);
    t.answer_targets(P0, &[obj(giant)]);
    t.cast(P0, card).go();
    t.settle();
    t.resolve();
    t.g.recompute();
    assert_eq!(t.stack_len(), 1, "{}", t.dump_log());
    assert_eq!(name_of(&t, giant), RIFTWATCHER);
    t.resolve_all();
    assert_eq!(name_of(&t, giant), RIFTWATCHER);
    assert_eq!(t.counters(giant, counters::TIME), 0);
    assert_eq!(t.life(P0), life + 2);
    let real = t.g.current(card);
    assert_eq!(t.counters(real, counters::TIME), 3);
}

// --- Copiable exceptions ------------------------------------------------------------------

#[test]
fn pirated_copys_exceptions_are_copied() {
    cr!("707.9b", "707.3", "707.2");
    ruling!(
        "Pirated Copy",
        "Being a Pirate and having the granted ability are part of the copiable values of Pirated Copy. If another creature enters the battlefield as or becomes a copy of Pirated Copy, it will copy whatever Pirated Copy is copying, be a Pirate in addition to its other types, and have \"Whenever this creature or another creature with the same name deals combat damage to a player, you draw a card.\""
    );
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P1, "Hill Giant");
    let pc = copy_with(&mut t, "Pirated Copy", giant)[0];
    let clone = crate::r_p023_common::clone_of(&mut t, P0, pc);
    assert_eq!(name_of(&t, clone), "Hill Giant");
    assert!(has_subtype(&t, clone, "Pirate") && has_subtype(&t, clone, "Giant"));
    assert_eq!(abilities_with(&t, clone, "you draw a card"), 1);
    // The Clone has the ability: it deals combat damage and P0 draws (twice: the Clone's
    // and Pirated Copy's abilities both see a creature with the same name).
    crate::r_p023_common::unsick(&mut t, clone);
    let hand = t.hand_size(P0);
    for _ in 0..3 {
        t.library_top(P0, "Island");
    }
    t.set_step(P0, Step::BeginningOfCombat);
    t.attack(&[(clone, Entity::Player(P1))], &[]);
    t.resolve_all();
    assert_eq!(t.life(P1), 17);
    assert_eq!(t.hand_size(P0), hand + 2);
}

// --- Cast triggers ------------------------------------------------------------------------

#[test]
fn benthic_anomaly_trigger_resolves_first_and_even_if_the_spell_is_countered() {
    cr!("603.2", "601.2i", "113.7a", "405.5");
    ruling!(
        "Benthic Anomaly",
        "Benthic Anomaly's triggered ability will resolve before Benthic Anomaly does. If Benthic Anomaly is countered or otherwise leaves the stack in response to that triggered ability, the triggered ability will still resolve as normal."
    );
    supported("Benthic Anomaly");
    for counter in [false, true] {
        let mut t = TestGame::new(2);
        let giant = t.battlefield(P1, "Hill Giant");
        lands_for(&mut t, P0, "{6}{U}");
        let c = t.hand(P0, "Benthic Anomaly");
        t.answer_choose(P0, &[obj(giant)]);
        let spell = t.cast(P0, c).go();
        t.settle();
        assert_eq!(t.stack_len(), 2);
        if counter {
            let spell = t.g.current(spell);
            assert!(t.g.counter(spell, None));
            assert!(t.in_graveyard(P0, "Benthic Anomaly"));
        }
        let before = t.g.battlefield.clone();
        t.resolve();
        let toks = new_tokens(&t, P0, &before);
        assert_eq!(toks.len(), 1, "{}", t.dump_log());
        assert_eq!(name_of(&t, toks[0]), "Hill Giant");
        // The spell (if it wasn't countered) is still on the stack.
        assert_eq!(t.stack_len(), usize::from(!counter));
        assert_eq!(t.named_on_battlefield("Benthic Anomaly").len(), 0);
    }
}

// --- Sakashima's Will ---------------------------------------------------------------------

#[test]
fn sakashimas_will_doesnt_change_whether_creatures_are_tokens() {
    cr!("707.2", "111.1", "613.1a");
    ruling!(
        "Sakashima's Will",
        "Choosing a token won't cause other nontoken creatures you control to become tokens. If you choose a nontoken creature, other creature tokens you control won't stop being tokens."
    );
    // "Choose a creature you control. Each other creature you control becomes a copy of
    // that creature until end of turn." (the second mode)
    supported("Sakashima's Will");
    let will = |t: &mut TestGame, chosen: ObjectId| {
        lands_for(t, P0, "{3}{U}");
        let c = t.hand(P0, "Sakashima's Will");
        t.answer_choose(P0, &[obj(chosen)]);
        t.cast(P0, c).modes(&[1]).go();
        t.resolve_all();
    };
    // A Wolf token is chosen: the Grizzly Bears becomes a Wolf, not a token.
    let mut t = TestGame::new(2);
    let wolf = crate::r_p023_common::wolf_token(&mut t, P0);
    let bears = t.battlefield(P0, "Grizzly Bears");
    will(&mut t, wolf);
    assert_eq!(name_of(&t, bears), "Wolf");
    assert!(!t.obj_now(bears).is_token());
    // A Hill Giant is chosen: the Wolf token becomes a Hill Giant and is still a token.
    let mut t = TestGame::new(2);
    let wolf = crate::r_p023_common::wolf_token(&mut t, P0);
    let giant = t.battlefield(P0, "Hill Giant");
    will(&mut t, giant);
    assert_eq!(name_of(&t, wolf), "Hill Giant");
    assert!(t.obj_now(wolf).is_token());
}
