//! Rulings batch P082 — spells: Ricochet Trap's target change (CR 115.7, 115.9a), modes
//! chosen on casting (CR 601.2b), colors chosen as a permanent enters or as a spell resolves
//! (CR 614.12, 608.2), Leadership Vacuum and the commander tax (CR 903.8), casting a card
//! from a graveyard with a trigger (CR 601.2f), and spells that can't be countered
//! (CR 101.2, 608.2c).

use crate::r_s01_common::supported;
use crate::r_s02_common::destroy;
use crate::r_s04_common::add_mana;
use crate::r_s21_common::castable;
use crate::r_s25_common::cast_new;
use mtg_engine::decision::{Answer, Decision};
use mtg_engine::mana::ManaType;
use mtg_engine::object::{CastMethod, Zone};
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

fn obj(id: ObjectId) -> Entity {
    Entity::Object(id)
}

// ---------------------------------------------------------------------------------
// Ricochet Trap: "Change the target of target spell with a single target."
// ---------------------------------------------------------------------------------

/// P0 casts Ricochet Trap (paying {3}{R}) targeting `spell`; returns the Trap.
fn ricochet(t: &mut TestGame, spell: ObjectId) -> ObjectId {
    supported("Ricochet Trap");
    cast_new(t, P0, "Ricochet Trap", &[obj(spell)])
}

/// Whether P0 could cast Ricochet Trap targeting `spell` now.
fn ricochet_can_target(t: &mut TestGame, spell: ObjectId) -> bool {
    t.lands(P0, "Mountain", 4);
    let c = t.hand(P0, "Ricochet Trap");
    let ok = t.cast_with(P0, c, &[obj(spell)]).is_ok();
    ok && t.g.stack.len() > 1
}

#[test]
fn ricochet_trap_needs_a_spell_with_exactly_one_target_instance() {
    cr!("115.9a", "115.10");
    ruling!(
        "Ricochet Trap",
        "If a spell targets multiple things, you can’t target it with Ricochet Trap, even if all but one of those targets have become illegal."
    );
    ruling!(
        "Ricochet Trap",
        "If a spell targets the same player or object multiple times, you can’t target it with Ricochet Trap."
    );
    supported("Electrolyze");
    supported("Seeds of Strength");
    // Electrolyze targeting two creatures, one of them gone since.
    let mut t = TestGame::new(2);
    let b1 = t.battlefield(P0, "Grizzly Bears");
    let b2 = t.battlefield(P0, "Llanowar Elves");
    t.lands(P1, "Island", 1);
    t.lands(P1, "Mountain", 1);
    t.lands(P1, "Wastes", 1);
    let c = t.hand(P1, "Electrolyze");
    t.answer(P1, DecisionKind::Divide, Answer::Numbers(vec![1, 1]));
    let el = t.cast(P1, c).targets(&[obj(b1), obj(b2)]).go();
    let el_targets: usize = t.obj(el).stack.as_ref().unwrap().chosen[0]
        .targets
        .iter()
        .map(|v| v.len())
        .sum();
    assert_eq!(el_targets, 2);
    destroy(&mut t, b2);
    assert!(!ricochet_can_target(&mut t, el));
    // Seeds of Strength targeting the same creature three times.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let seeds = cast_new(&mut t, P1, "Seeds of Strength", &[obj(bears), obj(bears), obj(bears)]);
    assert_eq!(
        t.obj(seeds).stack.as_ref().unwrap().chosen[0]
            .targets
            .iter()
            .flatten()
            .filter(|e| **e == obj(bears))
            .count(),
        3
    );
    assert!(!ricochet_can_target(&mut t, seeds));
    // A single-target spell can be targeted.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let blade = cast_new(&mut t, P1, "Doom Blade", &[obj(bears)]);
    assert!(ricochet_can_target(&mut t, blade));
}

#[test]
fn ricochet_trap_for_r_can_target_any_spell() {
    cr!("118.9", "601.2b");
    ruling!(
        "Ricochet Trap",
        "If you cast Ricochet Trap for {R}, you don’t have to target the blue spell an opponent cast."
    );
    let mut t = TestGame::new(2);
    let mine = t.battlefield(P0, "Grizzly Bears");
    let theirs = t.battlefield(P1, "Grizzly Bears");
    cast_new(&mut t, P1, "Opt", &[]);
    t.resolve_all();
    let blade = cast_new(&mut t, P1, "Doom Blade", &[obj(mine)]);
    t.lands(P0, "Mountain", 1);
    let c = t.hand(P0, "Ricochet Trap");
    let alt = t
        .cast_options(P0, c)
        .into_iter()
        .find(|o| matches!(o.method, CastMethod::Alternative(_)))
        .expect("the {R} cost is available");
    t.answer_targets(P0, &[obj(blade)]);
    t.answer_targets(P0, &[obj(theirs)]);
    t.cast(P0, c).method(alt.method).go();
    t.resolve_all();
    assert!(t.on_battlefield(mine));
    assert!(!t.on_battlefield(theirs));
}

#[test]
fn ricochet_trap_targets_only_the_spell() {
    cr!("115.7", "702.16b");
    ruling!(
        "Ricochet Trap",
        "Ricochet Trap targets only the spell whose target will be changed. It doesn’t directly affect the original target of that spell or the new target of that spell."
    );
    supported("Kor Firewalker");
    // Doom Blade is moved onto a creature with protection from red: the red Trap doesn't
    // target it, so that's fine.
    let mut t = TestGame::new(2);
    let mine = t.battlefield(P0, "Grizzly Bears");
    let kor = t.battlefield(P1, "Kor Firewalker");
    let blade = cast_new(&mut t, P1, "Doom Blade", &[obj(mine)]);
    t.answer_targets(P0, &[obj(kor)]);
    ricochet(&mut t, blade);
    t.resolve_all();
    assert!(t.on_battlefield(mine));
    assert!(!t.on_battlefield(kor));
}

#[test]
fn ricochet_trap_chooses_on_resolution_must_change_if_possible() {
    cr!("115.7", "115.7a", "608.2b");
    ruling!(
        "Ricochet Trap",
        "You don’t choose the new target for the spell until Ricochet Trap resolves. You must change the target if possible. However, you can’t change the target to an illegal target. If there are no legal targets to choose from, the target isn’t changed. It doesn’t matter if the original target of that spell has somehow become illegal itself."
    );
    // The only other legal target appears after Ricochet Trap is cast; it's chosen.
    let mut t = TestGame::new(2);
    let mine = t.battlefield(P0, "Grizzly Bears");
    let blade = cast_new(&mut t, P1, "Doom Blade", &[obj(mine)]);
    ricochet(&mut t, blade);
    let late = t.battlefield(P1, "Grizzly Bears");
    t.resolve();
    assert_eq!(
        t.obj(blade).stack.as_ref().unwrap().chosen[0].targets[0],
        vec![obj(late)]
    );
    t.resolve_all();
    assert!(t.on_battlefield(mine) && !t.on_battlefield(late));
    // No other legal target (only a black creature): unchanged.
    let mut t = TestGame::new(2);
    let mine = t.battlefield(P0, "Grizzly Bears");
    t.battlefield(P1, "Typhoid Rats");
    let blade = cast_new(&mut t, P1, "Doom Blade", &[obj(mine)]);
    ricochet(&mut t, blade);
    t.resolve_all();
    assert!(!t.on_battlefield(mine));
    // The original target became illegal (it left): still changed.
    let mut t = TestGame::new(2);
    let mine = t.battlefield(P0, "Grizzly Bears");
    let theirs = t.battlefield(P1, "Grizzly Bears");
    let blade = cast_new(&mut t, P1, "Doom Blade", &[obj(mine)]);
    ricochet(&mut t, blade);
    destroy(&mut t, mine);
    t.resolve_all();
    assert!(!t.on_battlefield(theirs));
}

#[test]
fn ricochet_trap_can_redirect_a_counterspell_onto_itself_but_not_itself() {
    cr!("115.5", "115.7", "608.2b");
    ruling!(
        "Ricochet Trap",
        "If you cast Ricochet Trap targeting a spell that targets a spell on the stack (like Cancel does, for example), you can’t change that spell’s target to itself. You can, however, change that spell’s target to Ricochet Trap. If you do, that spell won’t resolve when it tries to resolve because Ricochet Trap will have left the stack by then."
    );
    supported("Cancel");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let bolt = cast_new(&mut t, P0, "Lightning Bolt", &[obj(bears)]);
    let cancel = cast_new(&mut t, P1, "Cancel", &[obj(bolt)]);
    let trap = ricochet(&mut t, cancel);
    let from = t.asked().len();
    t.resolve();
    // The only new target offered is Ricochet Trap itself (not Cancel, not the Bolt).
    let offered: Vec<Vec<Entity>> = t.asked()[from..]
        .iter()
        .filter_map(|(p, d)| match d {
            Decision::ChooseTargets { candidates, .. } if *p == P0 => Some(candidates.clone()),
            _ => None,
        })
        .collect();
    assert_eq!(offered, vec![vec![obj(trap)]]);
    t.resolve_all();
    assert!(t.in_graveyard(P1, "Cancel"));
    assert!(!t.on_battlefield(bears), "the Bolt resolved");
}

// ---------------------------------------------------------------------------------
// Parch.
// ---------------------------------------------------------------------------------

#[test]
fn parch_mode_is_chosen_on_casting_and_either_mode_can_hit_a_blue_creature() {
    cr!("601.2b", "700.2a");
    ruling!(
        "Parch",
        "You can choose to use the 2 damage mode on a blue creature if you want."
    );
    ruling!("Parch", "You choose the mode when you announce the spell.");
    supported("Parch");
    supported("Horned Turtle");
    let mut t = TestGame::new(2);
    let turtle = t.battlefield(P1, "Horned Turtle");
    t.lands(P0, "Mountain", 2);
    let c = t.hand(P0, "Parch");
    let spell = t.cast(P0, c).modes(&[0]).target(obj(turtle)).go();
    assert_eq!(t.obj(spell).stack.as_ref().unwrap().chosen[0].mode, Some(0));
    t.resolve_all();
    assert_eq!(t.obj_now(turtle).damage, 2);
}

// ---------------------------------------------------------------------------------
// Iona, Shield of Emeria and Persecute.
// ---------------------------------------------------------------------------------

/// The answer to "choose a color" for `c`.
fn choose_color(t: &mut TestGame, p: PlayerId, c: Color) {
    let i = Color::ALL.iter().position(|x| *x == c).unwrap();
    t.answer(p, DecisionKind::Option, Answer::Index(i));
}

#[test]
fn iona_stops_permanent_spells_but_not_lands_or_abilities() {
    cr!("101.2", "305.1", "602.1");
    ruling!(
        "Iona, Shield of Emeria",
        "Iona's third ability includes permanent spells (artifacts, creatures, enchantments, and planeswalkers), not just instant and sorcery spells. It does not include lands or abilities (such as cycling or unearth)."
    );
    supported("Iona, Shield of Emeria");
    let mut t = TestGame::new(2);
    choose_color(&mut t, P0, Color::Green);
    t.enter(P0, "Iona, Shield of Emeria");
    t.set_step(P1, Step::PrecombatMain);
    t.lands(P1, "Forest", 3);
    let bears = t.hand(P1, "Grizzly Bears");
    assert!(!castable(&mut t, P1, bears));
    let forest = t.hand(P1, "Forest");
    t.play_land(P1, forest).unwrap();
    let tusker = t.hand(P1, "Krosan Tusker");
    assert!(crate::r_s04_common::can_cycle(&mut t, P1, tusker));
}

#[test]
fn iona_color_is_chosen_with_no_chance_to_respond() {
    cr!("614.12", "614.1c");
    ruling!(
        "Iona, Shield of Emeria",
        "Once the color is chosen, it's too late for opponents to respond by casting spells of that color. Iona is not yet in the battlefield at the time the color is chosen, so, for example, there's no way for an opponent to destroy it by casting Doom Blade if the chosen color is black."
    );
    let mut t = TestGame::new(2);
    let blade = t.hand(P1, "Doom Blade");
    t.lands(P1, "Swamp", 2);
    let iona = cast_new(&mut t, P0, "Iona, Shield of Emeria", &[]);
    // While Iona is a spell, no color has been chosen and Doom Blade can't target her.
    assert!(t.obj(iona).choices.color.is_none());
    choose_color(&mut t, P0, Color::Black);
    t.resolve();
    let iona = t.g.current(iona);
    assert!(t.on_battlefield(iona));
    assert_eq!(t.obj(iona).choices.color, Some(Color::Black));
    assert!(!castable(&mut t, P1, blade));
}

#[test]
fn persecute_chooses_its_color_on_resolution() {
    cr!("608.2c", "608.2d");
    ruling!("Persecute", "The color is chosen on resolution.");
    supported("Persecute");
    let mut t = TestGame::new(2);
    t.hand(P1, "Lightning Bolt");
    t.hand(P1, "Grizzly Bears");
    let from = t.asked().len();
    let p = cast_new(&mut t, P0, "Persecute", &[Entity::Player(P1)]);
    let color_asked = |t: &TestGame, from: usize| {
        t.asked()[from..].iter().any(|(p, d)| {
            *p == P0 && matches!(d, Decision::ChooseOption { options, .. } if options.len() == 5)
        })
    };
    assert!(!color_asked(&t, from), "not chosen while casting");
    choose_color(&mut t, P0, Color::Red);
    t.resolve_all();
    assert!(color_asked(&t, from));
    assert!(t.in_graveyard(P1, "Lightning Bolt"));
    assert!(t.in_hand(P1, "Grizzly Bears"));
    assert!(t.in_graveyard(P0, "Persecute"));
    let _ = p;
}

// ---------------------------------------------------------------------------------
// Leadership Vacuum.
// ---------------------------------------------------------------------------------

/// A Commander game with P1's commander `name` on the battlefield, cast `casts` times
/// from the command zone.
fn commander_on_battlefield(name: &str, casts: u32) -> (TestGame, ObjectId) {
    let mut t = crate::r_s13_common::commander_game();
    let c = t.battlefield(P1, name);
    t.g.objects[c.0 as usize].is_commander = true;
    t.g.players[1].commander_names.push(name.into());
    if casts > 0 {
        t.g.players[1].commander_casts.insert(name.into(), casts);
    }
    (t, c)
}

#[test]
fn leadership_vacuum_targets_the_player_not_the_commander() {
    cr!("115.1", "702.11b", "903.9a");
    ruling!(
        "Leadership Vacuum",
        "Leadership Vacuum targets the player, not their creatures. A commander with hexproof or protection from blue may be returned this way."
    );
    supported("Leadership Vacuum");
    let (mut t, sigarda) = commander_on_battlefield("Sigarda, Host of Herons", 0);
    cast_new(&mut t, P0, "Leadership Vacuum", &[Entity::Player(P1)]);
    t.resolve_all();
    assert_eq!(t.zone(sigarda), Zone::Command);
}

#[test]
fn leadership_vacuum_doesnt_add_commander_tax() {
    cr!("903.8");
    ruling!(
        "Leadership Vacuum",
        "The \"commander tax\" increases based on how many times a commander was cast from the command zone, not how many times it was put there as a result of Commander rules. The returned commander will cost more to cast again even though it was put directly into the command zone."
    );
    // Cast once before: {1}{G} plus {2}.
    let (mut t, bears) = commander_on_battlefield("Grizzly Bears", 1);
    cast_new(&mut t, P0, "Leadership Vacuum", &[Entity::Player(P1)]);
    t.resolve_all();
    let bears = t.g.current(bears);
    assert_eq!(t.zone(bears), Zone::Command);
    t.set_step(P1, Step::PrecombatMain);
    t.lands(P1, "Forest", 3);
    assert!(!castable(&mut t, P1, bears), "costs more than {{1}}{{G}}");
    t.lands(P1, "Forest", 1);
    assert!(castable(&mut t, P1, bears), "but only {{2}} more");
}

#[test]
fn leadership_vacuum_draws_even_without_commanders() {
    cr!("608.2c");
    ruling!(
        "Leadership Vacuum",
        "You'll draw a card even if the target player controls no commanders as Leadership Vacuum resolves."
    );
    let mut t = crate::r_s13_common::commander_game();
    let hand = t.hand_size(P0);
    cast_new(&mut t, P0, "Leadership Vacuum", &[Entity::Player(P1)]);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand + 1);
}

// ---------------------------------------------------------------------------------
// Toshiro Umezawa.
// ---------------------------------------------------------------------------------

#[test]
fn toshiro_still_needs_the_instants_mana_cost() {
    cr!("601.2f", "601.2h", "603.3d");
    ruling!(
        "Toshiro Umezawa",
        "You must still pay the mana cost of the target instant in your graveyard to cast it using Toshiro’s ability."
    );
    supported("Toshiro Umezawa");
    for pay in [false, true] {
        let mut t = TestGame::new(2);
        t.battlefield(P0, "Toshiro Umezawa");
        let bolt = t.graveyard(P0, "Lightning Bolt");
        let victim = t.battlefield(P1, "Grizzly Bears");
        if pay {
            t.lands(P0, "Mountain", 1);
        }
        t.answer_targets(P0, &[obj(bolt)]);
        t.answer_yes(P0, true);
        t.answer_targets(P0, &[Entity::Player(P1)]);
        destroy(&mut t, victim);
        t.resolve_all();
        if pay {
            assert_eq!(t.life(P1), 17, "cast for {{R}}");
            assert!(t.in_exile("Lightning Bolt"));
        } else {
            assert_eq!(t.life(P1), 20, "no mana: not cast");
            assert!(t.in_graveyard(P0, "Lightning Bolt"));
        }
    }
}

// ---------------------------------------------------------------------------------
// Spells that can't be countered.
// ---------------------------------------------------------------------------------

/// P1 casts Dismiss ("Counter target spell. Draw a card.") targeting `spell`: the spell
/// isn't countered, but P1 draws.
fn dismiss_fails_but_draws(t: &mut TestGame, spell: ObjectId) {
    supported("Dismiss");
    let hand = t.hand_size(P1);
    cast_new(t, P1, "Dismiss", &[obj(spell)]);
    t.resolve();
    assert!(t.g.stack.contains(&spell), "the spell was countered");
    assert_eq!(t.hand_size(P1), hand + 1);
}

#[test]
fn uncounterable_spells_can_be_targeted_by_counterspells() {
    cr!("101.2", "608.2c", "701.6a");
    ruling!(
        "Gaea's Revenge",
        "A Gaea's Revenge spell can be targeted by spells and abilities that would counter it. The part of their effect that would counter Gaea's Revenge won't do anything, but any other effects those spells or abilities may have will still happen, if applicable."
    );
    ruling!(
        "Abrupt Decay",
        "A spell or ability that counters spells can still target Abrupt Decay. When that spell or ability resolves, Abrupt Decay won't be countered, but any additional effects of the countering spell or ability will still happen."
    );
    ruling!(
        "Commence the Endgame",
        "A spell or ability that counters spells can still target Commence the Endgame. When that spell or ability resolves, Commence the Endgame won't be countered, but any additional effects of that spell or ability will still happen."
    );
    ruling!(
        "Cragplate Baloth",
        "A spell or ability that counters spells can still target Cragplate Baloth. When that spell or ability resolves, Cragplate Baloth won't be countered, but any additional effects of the countering spell or ability will still happen."
    );
    ruling!(
        "Dovin's Veto",
        "A spell or ability that counters spells can still target Dovin's Veto. When that spell or ability resolves, Dovin's Veto won't be countered, but any additional effects of the countering spell or ability will still happen."
    );
    ruling!(
        "Void Rend",
        "A spell or ability that counters spells can still target Void Rend. When that spell or ability resolves, Void Rend won't be countered, but any additional effects of the countering spell or ability will still happen."
    );
    for name in [
        "Gaea's Revenge",
        "Abrupt Decay",
        "Commence the Endgame",
        "Cragplate Baloth",
        "Void Rend",
    ] {
        supported(name);
        let mut t = TestGame::new(2);
        let bears = t.battlefield(P1, "Grizzly Bears");
        let targets: Vec<Entity> = match name {
            "Abrupt Decay" | "Void Rend" => vec![obj(bears)],
            _ => vec![],
        };
        let spell = cast_new(&mut t, P0, name, &targets);
        dismiss_fails_but_draws(&mut t, spell);
        t.resolve_all();
        match name {
            "Abrupt Decay" | "Void Rend" => assert!(!t.on_battlefield(bears), "{name}"),
            "Commence the Endgame" => assert!(t.in_graveyard(P0, name)),
            _ => assert_eq!(t.named_on_battlefield(name).len(), 1, "{name}"),
        }
    }
    // Dovin's Veto countering P1's Opt; P1's Dismiss can't counter the Veto.
    supported("Dovin's Veto");
    let mut t = TestGame::new(2);
    let opt = cast_new(&mut t, P1, "Opt", &[]);
    let veto = cast_new(&mut t, P0, "Dovin's Veto", &[obj(opt)]);
    dismiss_fails_but_draws(&mut t, veto);
    let hand = t.hand_size(P1);
    t.resolve_all();
    assert!(t.in_graveyard(P1, "Opt"));
    assert_eq!(t.hand_size(P1), hand, "Opt was countered: no draw");
    assert!(t.in_graveyard(P0, "Dovin's Veto"));
}
