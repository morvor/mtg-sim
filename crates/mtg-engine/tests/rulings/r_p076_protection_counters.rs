//! Rulings batch P076 — protection from creatures and from players (CR 702.16), "can't be
//! countered" (CR 101.2), riot granted by Rhythm of the Wild, and Jailbreak Scheme's
//! owner choosing the top or bottom of their library.

use crate::r_p076_common::*;
use crate::r_s01_common::{custom_card, supported};
use crate::r_s02_common::target_candidates;
use crate::r_s06_common::give_control;
use crate::r_s25_common::cast_new;
use mtg_engine::keywords::KeywordKind;
use mtg_engine::mana::ManaType;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

/// Whether `p` activating `source`'s `idx`th activated ability is offered `target`.
fn offered(t: &mut TestGame, p: PlayerId, source: ObjectId, idx: usize, target: ObjectId) -> bool {
    let from = t.asked().len();
    mana(t, p, ManaType::R, 4);
    mana(t, p, ManaType::C, 4);
    let target = t.g.current(target);
    let _ = t.activate(p, source, idx, &[Entity::Player(P0)]);
    t.g.players[p.idx()].mana_pool = Default::default();
    target_candidates(t, p, from)
        .iter()
        .any(|c| c.contains(&Entity::Object(target)))
}

#[test]
fn crypsis_protects_from_opponents_creature_cards_in_other_zones() {
    cr!("702.16b", "702.16e", "108.4a");
    ruling!(
        "Crypsis",
        "Abilities of creature cards not on the battlefield owned by an opponent can’t target that creature, and damage those cards would deal to the creature is prevented."
    );
    supported("Crypsis");
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let giant = t.battlefield(P0, "Hill Giant");
    let bears = t.battlefield(P0, "Grizzly Bears");
    cast_new(&mut t, P0, "Crypsis", &[Entity::Object(giant)]);
    t.resolve_all();
    // Ghost-Lit Raider's channel ability (from P1's hand) can't target the Giant.
    let raider = t.hand(P1, "Ghost-Lit Raider");
    assert!(!offered(&mut t, P1, raider, 1, giant));
    let raider = t.hand(P1, "Ghost-Lit Raider");
    assert!(offered(&mut t, P1, raider, 1, bears));
    // Damage dealt by a creature card in P1's hand is prevented.
    let blaster = custom_card(
        "Hand Blaster",
        "Creature — Elemental",
        "{R}",
        Some((1, 1)),
        "Channel — {R}, Discard Hand Blaster: It deals 2 damage to each creature.",
    );
    let b = t.custom(P1, blaster, Zone::Hand(P1));
    mana(&mut t, P1, ManaType::R, 1);
    t.activate(P1, b, 0, &[]).unwrap();
    t.resolve_all();
    assert_eq!(t.obj_now(giant).damage, 0);
    assert!(!t.on_battlefield(bears), "the unprotected Bears took 2");
}

#[test]
fn crypsis_follows_the_new_controller() {
    cr!("702.16b", "611.2c");
    ruling!(
        "Crypsis",
        "If another player gains control of the creature after Crypsis resolves, the creature will have protection from creatures controlled by opponents of its new controller."
    );
    supported("Crypsis");
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let giant = t.battlefield(P0, "Hill Giant");
    let mine = t.battlefield(P0, "Prodigal Sorcerer");
    let theirs = t.battlefield(P1, "Prodigal Sorcerer");
    cast_new(&mut t, P0, "Crypsis", &[Entity::Object(giant)]);
    t.resolve_all();
    assert!(offered(&mut t, P0, mine, 0, giant));
    assert!(!offered(&mut t, P1, theirs, 0, giant));
    t.g.untap(t.g.current(mine));
    t.g.untap(t.g.current(theirs));
    give_control(&mut t, giant, P1);
    // Now P0 is the opponent: P0's creatures can't target it; P1's can.
    assert!(!offered(&mut t, P0, mine, 0, giant));
    t.g.untap(t.g.current(mine));
    assert!(offered(&mut t, P1, theirs, 0, giant));
}

#[test]
fn cliffside_rescuer_protection_from_players() {
    cr!("702.16k", "108.4a");
    ruling!(
        "Cliffside Rescuer",
        "Protection from a player means that the permanent has protection from each object controlled by that player. If an object has no controller (such as a card in a graveyard), its owner is considered its controller for this purpose."
    );
    supported("Cliffside Rescuer");
    let mut t = TestGame::new(2);
    let rescuer = t.battlefield(P0, "Cliffside Rescuer");
    let giant = t.battlefield(P0, "Hill Giant");
    let sorcerer = t.battlefield(P1, "Prodigal Sorcerer");
    t.activate(P0, rescuer, 0, &[Entity::Object(giant)]).unwrap();
    t.resolve_all();
    // An object P1 controls can't target it.
    assert!(!offered(&mut t, P1, sorcerer, 0, giant));
    // Nor can a card in P1's hand (no controller: its owner counts).
    let raider = t.hand(P1, "Ghost-Lit Raider");
    assert!(!offered(&mut t, P1, raider, 1, giant));
}

#[test]
fn cliffside_rescuer_opponents_fixed_on_resolution() {
    cr!("702.16k", "611.2c");
    ruling!(
        "Cliffside Rescuer",
        "The players that the target permanent has protection from are determined as Cliffside Rescuer’s ability resolves. If an opponent somehow gains control of that permanent later in the turn, it still has protection from your opponents, including its new controller."
    );
    supported("Cliffside Rescuer");
    let mut t = TestGame::new(2);
    let rescuer = t.battlefield(P0, "Cliffside Rescuer");
    let giant = t.battlefield(P0, "Hill Giant");
    let sorcerer = t.battlefield(P1, "Prodigal Sorcerer");
    let mine = t.battlefield(P0, "Prodigal Sorcerer");
    t.activate(P0, rescuer, 0, &[Entity::Object(giant)]).unwrap();
    t.resolve_all();
    give_control(&mut t, giant, P1);
    assert_eq!(t.obj_now(giant).controller, P1);
    // Still protection from P1, its new controller; not from P0.
    assert!(!offered(&mut t, P1, sorcerer, 0, giant));
    assert!(offered(&mut t, P0, mine, 0, giant));
}

#[test]
fn jailbreak_scheme_the_owner_chooses_top_or_bottom() {
    cr!("401.4", "702.172a");
    ruling!(
        "Jailbreak Scheme",
        "The permanent’s owner chooses whether to put it on the top or bottom of their library."
    );
    supported("Jailbreak Scheme");
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let bears = t.battlefield(P1, "Grizzly Bears");
    crate::r_s29_common::choose_modes(&mut t, P0, &[1]);
    mana(&mut t, P0, ManaType::U, 1);
    mana(&mut t, P0, ManaType::C, 2);
    let card = t.hand(P0, "Jailbreak Scheme");
    t.answer_targets(P0, &[Entity::Object(bears)]);
    t.cast(P0, card).go();
    // P1 picks the bottom.
    t.answer(P1, DecisionKind::Option, Answer::Index(1));
    let from = t.asked().len();
    t.resolve_all();
    assert!(t.asked()[from..]
        .iter()
        .any(|(p, d)| *p == P1 && matches!(d, Decision::ChooseOption { .. })));
    assert!(!t.asked()[from..]
        .iter()
        .any(|(p, d)| *p == P0 && matches!(d, Decision::ChooseOption { .. })));
    let lib = &t.g.player(P1).library;
    assert_eq!(t.g.obj(lib[0]).chars.name, "Grizzly Bears", "{lib:?}");
}

/// P1 casts `counter` targeting `spell`; `spell` stays on the stack, and P1 still draws.
fn counter_fails_but_draws(t: &mut TestGame, counter: &str, spell: ObjectId) {
    let hand = hand_count(t, P1);
    cast_new(t, P1, counter, &[Entity::Object(spell)]);
    t.resolve();
    assert!(t.g.stack.contains(&spell), "{counter} countered the spell");
    assert_eq!(hand_count(t, P1), hand + 1);
}

#[test]
fn sphinx_of_the_final_word_counterspells_still_draw() {
    cr!("101.2", "608.2c");
    ruling!(
        "Sphinx of the Final Word",
        "A spell or ability that counters spells can still target Sphinx of the Final Word or any other spell that can't be countered. When that spell or ability resolves, the spell won't be countered, but any additional effects of the countering spell or ability will still happen."
    );
    supported("Sphinx of the Final Word");
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    t.battlefield(P0, "Sphinx of the Final Word");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let growth = cast_new(&mut t, P0, "Giant Growth", &[Entity::Object(bears)]);
    // Dismiss: "Counter target spell. Draw a card."
    counter_fails_but_draws(&mut t, "Dismiss", growth);
    t.resolve_all();
    assert_eq!(t.pt(bears), (5, 5));
}

#[test]
fn spellbreaker_behemoth_counterspells_still_draw() {
    cr!("101.2", "608.2c");
    ruling!(
        "Spellbreaker Behemoth",
        "Any spells or abilities that would counter Spellbreaker Behemoth or (once it’s on the battlefield) a creature spell you control with power 5 or greater can still be cast or activated and will still resolve. They just won’t counter that spell. Any other effects they have will happen as normal."
    );
    supported("Spellbreaker Behemoth");
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let behemoth = cast_new(&mut t, P0, "Spellbreaker Behemoth", &[]);
    // Exclude: "Counter target creature spell. Draw a card."
    counter_fails_but_draws(&mut t, "Exclude", behemoth);
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Spellbreaker Behemoth").len(), 1);
    // Craw Wurm (6/4) while Spellbreaker Behemoth is on the battlefield.
    let wurm = cast_new(&mut t, P0, "Craw Wurm", &[]);
    counter_fails_but_draws(&mut t, "Exclude", wurm);
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Craw Wurm").len(), 1);
}

#[test]
fn spellbreaker_behemoth_anthems_dont_apply_on_the_stack() {
    cr!("101.2", "109.2");
    ruling!(
        "Spellbreaker Behemoth",
        "Effects that affect a creature’s power (such as the one from Glorious Anthem, for example) apply only to creatures on the battlefield, not to creature spells on the stack."
    );
    supported("Spellbreaker Behemoth");
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    t.battlefield(P0, "Spellbreaker Behemoth");
    t.battlefield(P0, "Glorious Anthem");
    // Rumbling Baloth: 4/4, 5/5 once on the battlefield, but 4 power on the stack.
    let baloth = cast_new(&mut t, P0, "Rumbling Baloth", &[]);
    cast_new(&mut t, P1, "Exclude", &[Entity::Object(baloth)]);
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Rumbling Baloth"));
}

#[test]
fn spellbreaker_behemoth_star_power_works_on_the_stack() {
    cr!("101.2", "604.3");
    ruling!(
        "Spellbreaker Behemoth",
        "If a creature card has “*” in its power, the ability that defines “*” works in all zones, including the stack."
    );
    supported("Spellbreaker Behemoth");
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    t.battlefield(P0, "Spellbreaker Behemoth");
    // Six card types among cards in graveyards: Tarmogoyf is 6/7 on the stack.
    for n in [
        "Grizzly Bears",
        "Giant Growth",
        "Divination",
        "Forest",
        "Ornithopter",
        "Glorious Anthem",
    ] {
        t.graveyard(P1, n);
    }
    let goyf = cast_new(&mut t, P0, "Tarmogoyf", &[]);
    counter_fails_but_draws(&mut t, "Exclude", goyf);
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Tarmogoyf").len(), 1);
}

#[test]
fn gaeas_herald_protects_opponents_creature_spells_too() {
    cr!("101.2");
    ruling!(
        "Gaea's Herald",
        "The ability applies to your opponent’s spells as well as your own."
    );
    supported("Gaea's Herald");
    let mut t = TestGame::new(2);
    t.set_step(P1, Step::PrecombatMain);
    t.battlefield(P0, "Gaea's Herald");
    let bears = cast_new(&mut t, P1, "Grizzly Bears", &[]);
    let hand = hand_count(&t, P0);
    cast_new(&mut t, P0, "Exclude", &[Entity::Object(bears)]);
    t.resolve_all();
    assert_eq!(hand_count(&t, P0), hand + 1);
    assert_eq!(t.named_on_battlefield("Grizzly Bears").len(), 1);
}

/// Whether the permanent got riot's +1/+1 counter or haste.
fn rioted(t: &TestGame, id: ObjectId) -> bool {
    t.counters(id, "+1/+1") > 0 || crate::r_s06_common::has_kw(t, id, KeywordKind::Haste)
}

#[test]
fn rhythm_of_the_wild_riot_for_what_enters_as_a_creature() {
    cr!("702.136a", "614.12");
    ruling!(
        "Rhythm of the Wild",
        "A noncreature card that happens to be entering the battlefield as a creature will have riot (for example, Rusted Relic while you control three other artifacts). Similarly, a creature card entering the battlefield as a noncreature permanent won't have riot (for example, Thassa, God of the Sea while your other permanents contribute only four to your devotion to blue)."
    );
    supported("Rhythm of the Wild");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Rhythm of the Wild");
    for _ in 0..3 {
        t.battlefield(P0, "Mind Stone");
    }
    let relic = t.enter(P0, "Rusted Relic");
    t.resolve_all();
    assert!(t.obj_now(relic).is(mtg_engine::types::CardType::Creature));
    assert!(rioted(&t, relic));
    let thassa = t.enter(P0, "Thassa, God of the Sea");
    t.resolve_all();
    assert!(!t.obj_now(thassa).is(mtg_engine::types::CardType::Creature));
    assert!(!rioted(&t, thassa));
}

#[test]
fn rhythm_of_the_wild_leaving_as_the_creature_enters() {
    cr!("614.12", "603.10");
    ruling!(
        "Rhythm of the Wild",
        "If Rhythm of the Wild leaves the battlefield at the same time that a nontoken creature enters the battlefield (most likely because that creature has a replacement effect, such as that of Rescuer Sphinx), that creature still gets a +1/+1 counter or haste."
    );
    supported("Rhythm of the Wild");
    let mut t = TestGame::new(2);
    let rhythm = t.battlefield(P0, "Rhythm of the Wild");
    // Rescuer Sphinx: "As this creature enters, you may return a nonland permanent you
    // control to its owner's hand. If you do, it enters with a +1/+1 counter on it."
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[Entity::Object(rhythm)]);
    let sphinx = t.enter(P0, "Rescuer Sphinx");
    t.resolve_all();
    assert!(t.in_hand(P0, "Rhythm of the Wild"));
    assert!(
        t.counters(sphinx, "+1/+1") == 2
            || (t.counters(sphinx, "+1/+1") == 1
                && crate::r_s06_common::has_kw(&t, sphinx, KeywordKind::Haste))
    );
}

#[test]
fn rhythm_of_the_wild_a_permanent_that_becomes_a_creature_later() {
    cr!("702.136a", "614.12");
    ruling!(
        "Rhythm of the Wild",
        "If a nontoken, noncreature permanent becomes a creature after it's already on the battlefield, it will have riot but it will be too late for the replacement effect to have any effect."
    );
    supported("Rhythm of the Wild");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Rhythm of the Wild");
    let relic = t.enter(P0, "Rusted Relic");
    t.resolve_all();
    assert!(!t.obj_now(relic).is(mtg_engine::types::CardType::Creature));
    for _ in 0..3 {
        t.battlefield(P0, "Mind Stone");
    }
    assert!(t.obj_now(relic).is(mtg_engine::types::CardType::Creature));
    assert!(crate::r_s06_common::has_kw(&t, relic, KeywordKind::Riot));
    assert!(!rioted(&t, relic));
}

#[test]
fn jailbreak_scheme_on_a_merged_permanent_moves_all_its_cards_together() {
    cr!("730.3", "730.3a", "401.4");
    ruling!(
        "Jailbreak Scheme",
        "If multiple cards are put into the library this way (such as when the spell targets a melded permanent), that permanent’s owner puts all the cards on top or all the cards on the bottom. They put them in whatever order they wish, and do not need to reveal the order."
    );
    supported("Jailbreak Scheme");
    supported("Gemrazer");
    use mtg_engine::keywords::KeywordKind;
    use mtg_engine::object::CastMethod;
    let mut t = TestGame::new(2);
    t.set_step(P1, Step::PrecombatMain);
    // P1 mutates Gemrazer onto their Grizzly Bears: one permanent, two cards.
    let bears = t.battlefield(P1, "Grizzly Bears");
    let gem = t.hand(P1, "Gemrazer");
    mana(&mut t, P1, ManaType::G, 2);
    mana(&mut t, P1, ManaType::C, 1);
    t.cast(P1, gem)
        .method(CastMethod::Keyword(KeywordKind::Mutate))
        .target(bears)
        .go();
    t.resolve_all();
    let merged = t.g.current(bears);
    assert!(!t.obj(merged).merged_with.is_empty());
    // P0 casts Jailbreak Scheme's second mode on it; P1 picks the bottom.
    t.set_step(P0, Step::PrecombatMain);
    crate::r_s29_common::choose_modes(&mut t, P0, &[1]);
    mana(&mut t, P0, ManaType::U, 1);
    mana(&mut t, P0, ManaType::C, 2);
    let card = t.hand(P0, "Jailbreak Scheme");
    t.answer_targets(P0, &[Entity::Object(merged)]);
    t.cast(P0, card).go();
    t.answer(P1, DecisionKind::Option, Answer::Index(1));
    t.resolve_all();
    let lib = &t.g.player(P1).library;
    let mut bottom: Vec<String> = lib[..2]
        .iter()
        .map(|c| t.g.obj(*c).chars.name.to_string())
        .collect();
    bottom.sort();
    assert_eq!(bottom, vec!["Gemrazer".to_string(), "Grizzly Bears".to_string()]);
}
