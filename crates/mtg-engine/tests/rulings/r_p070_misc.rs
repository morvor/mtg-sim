//! Rulings batch P070 — the rest: values computed as an ability resolves (CR 608.2h) and
//! last known information (CR 113.7a), token copies (CR 707.2, 111.4), replacement effects
//! and spells that exile themselves (CR 614.1a, 608.2n), triggers that resolve before the
//! spell that caused them (CR 603.3), zone changes and new objects (CR 400.7), searches
//! (CR 701.23), choices made by other players (CR 101.4, 723.5), hidden information
//! (CR 406.3), counterspells against uncounterable spells (CR 101.2).

use crate::r_p070_common::*;
use mtg_engine::decision::{Answer, Decision};
use mtg_engine::facedown::can_look_at;
use mtg_engine::mana::ManaType::*;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::CardType;
use mtg_engine::*;

/// Casts `card` (followed across zone changes) for `p` with mana in its pool for its mana
/// cost (colored symbols as their colors, the rest colorless).
fn cast_paid(t: &mut TestGame, p: PlayerId, card: ObjectId, targets: &[Entity]) -> ObjectId {
    let c = t.g.current(card);
    let cost = t
        .obj_now(c)
        .chars
        .mana_cost
        .clone()
        .map(|m| format!("{m}"))
        .unwrap_or_default();
    for sym in cost.split('}').filter_map(|s| s.strip_prefix('{')) {
        let ty = match sym {
            "W" => W,
            "U" => U,
            "B" => B,
            "R" => R,
            "G" => G,
            n => {
                mana(t, p, C, n.parse::<u32>().unwrap_or(0));
                continue;
            }
        };
        mana(t, p, ty, 1);
    }
    t.g.turn.priority = Some(p);
    t.cast(p, c).targets(targets).go()
}

// ---------------------------------------------------------------------------
// Values computed as the ability resolves
// ---------------------------------------------------------------------------

#[test]
fn herald_of_amity_counts_auras_as_its_attack_trigger_resolves() {
    cr!("608.2h");
    ruling!(
        "Herald of Amity",
        "The value of X is calculated at the time Herald of Amity's last ability resolves."
    );
    supported("Herald of Amity");
    let mut t = TestGame::new(2);
    let h = t.battlefield(P0, "Herald of Amity");
    attack_with(&mut t, &[(h, Entity::Player(P1))]);
    assert_eq!(t.stack_len(), 1);
    // An Aura arrives with the trigger on the stack.
    let hs = t.battlefield(P0, "Holy Strength");
    assert!(t.g.attach(hs, o(h)));
    t.g.recompute();
    assert_eq!(t.pt(h), (3, 4));
    t.resolve();
    assert_eq!(t.pt(h), (4, 5), "X 1: {}", t.dump_log());
}

#[test]
fn taster_of_wares_counts_goblins_as_its_ability_resolves() {
    cr!("608.2h");
    ruling!(
        "Taster of Wares",
        "The value of X is calculated only once, as Taster of Wares's ability resolves."
    );
    supported("Taster of Wares");
    let mut t = TestGame::new(2);
    let hand: Vec<_> = ["Divination", "Opt", "Shock"]
        .iter()
        .map(|n| t.hand(P1, n))
        .collect();
    t.answer_targets(P0, &[Entity::Player(P1)]);
    t.enter(P0, "Taster of Wares");
    t.settle();
    assert_eq!(t.stack_len(), 1);
    // Another Goblin arrives before it resolves: X is 2.
    t.battlefield(P0, "Goblin Piker");
    let from = t.asked().len();
    t.answer_choose(P1, &[o(hand[0]), o(hand[1])]);
    t.answer_choose(P0, &[o(hand[1])]);
    t.resolve_all();
    let p0_choices: Vec<usize> = t.asked()[from..]
        .iter()
        .filter_map(|(p, d)| match d {
            Decision::ChooseEntities { candidates, .. } if *p == P0 => Some(candidates.len()),
            _ => None,
        })
        .collect();
    assert_eq!(p0_choices, vec![2], "two cards revealed: {}", t.dump_log());
    assert!(exiled(&t, hand[1]));
}

#[test]
fn pantlaza_uses_the_toughness_the_creature_last_had() {
    cr!("608.2h", "113.7a", "701.57a");
    ruling!(
        "Pantlaza, Sun-Favored",
        "The value of X is calculated only once, as the ability resolves. If the creature has left the battlefield, use its toughness as it last existed on the battlefield."
    );
    supported("Pantlaza, Sun-Favored");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Pantlaza, Sun-Favored");
    // Discover 6 skips nothing below 7: Hill Giant (4) is found.
    let giant = t.library_top(P0, "Hill Giant");
    let d = t.enter(P0, "Colossal Dreadmaw");
    t.settle();
    assert_eq!(t.stack_len(), 1);
    // The Dreadmaw (6 toughness) leaves before the trigger resolves.
    move_to(&mut t, d, Zone::Graveyard(P0));
    t.answer_yes(P0, true);
    t.answer_yes(P0, true);
    t.resolve_all();
    assert!(t.on_battlefield(t.g.current(giant)), "{}", t.dump_log());
}

#[test]
fn laughing_jasper_flint_counts_outlaws_as_its_ability_resolves() {
    cr!("608.2h");
    ruling!(
        "Laughing Jasper Flint",
        "The value of X is determined as Laughing Jasper Flint’s last ability resolves."
    );
    supported("Laughing Jasper Flint");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Laughing Jasper Flint");
    let lib = stack_library(&mut t, P1, &["Grizzly Bears", "Hill Giant", "Forest"]);
    t.answer_targets(P0, &[Entity::Player(P1)]);
    t.set_step(P1, Step::End);
    t.advance_to(P0, Step::Upkeep);
    t.settle();
    assert_eq!(t.stack_len(), 1, "{}", t.dump_log());
    // Another outlaw (a Rogue) arrives with the trigger on the stack: X is 2.
    t.battlefield(P0, "Robber of the Rich");
    t.resolve_all();
    assert!(exiled(&t, lib[0]) && exiled(&t, lib[1]), "{}", t.dump_log());
    assert_eq!(t.zone(lib[2]), Zone::Library(P1));
}

#[test]
fn evolving_door_counts_the_colors_the_creature_last_had() {
    cr!("113.7a", "707.2", "701.23a");
    ruling!(
        "Evolving Door",
        "You count the colors of the sacrificed creature as it last existed on the battlefield. Notably, if it was a double-faced card or a copy of another creature, that may be different from what its colors are in the graveyard."
    );
    supported("Evolving Door");
    let mut t = TestGame::new(2);
    let door = t.battlefield(P0, "Evolving Door");
    t.battlefield(P1, "Watchwolf");
    // Clone (blue) as a copy of Watchwolf (green and white).
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[]);
    let wolf = t.named_on_battlefield("Watchwolf")[0];
    t.clear_answers();
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[o(wolf)]);
    let clone = t.enter(P0, "Clone");
    t.settle();
    assert_eq!(t.obj_now(clone).chars.name, "Watchwolf", "{}", t.dump_log());
    let thrinax = t.library_top(P0, "Sprouting Thrinax");
    let ww = t.library_top(P0, "Watchwolf");
    mana(&mut t, P0, C, 1);
    let from = t.asked().len();
    t.answer_choose(P0, &[o(clone)]);
    t.answer_choose(P0, &[o(thrinax)]);
    t.answer_yes(P0, false);
    t.activate(P0, door, 0, &[]).unwrap();
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Clone"));
    // Two colors plus one: the three-color Thrinax, not the two-color Watchwolf.
    assert!(was_offered(&t, from, thrinax), "{}", t.dump_log());
    assert!(!was_offered(&t, from, ww));
    assert!(exiled(&t, thrinax), "{}", t.dump_log());
}

// ---------------------------------------------------------------------------
// Token copies
// ---------------------------------------------------------------------------

#[test]
fn maestros_gift_token_copies_only_the_copiable_values() {
    cr!("707.2", "111.4", "707.4");
    ruling!(
        "Inspired Skypainter // Maestro's Gift",
        "The token created by Maestro's Gift copies exactly what was printed on the original creature and nothing else (unless that permanent is copying something else or is a token; see below). It doesn't copy whether that permanent is tapped or untapped, whether it has any counters on it or Auras and Equipment attached to it, or any non-copy effects that have changed its power, toughness, types, color, or so on. If it is a Vehicle, it is not crewed."
    );
    supported("Inspired Skypainter // Maestro's Gift");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.g.obj_mut(bears).counters.insert("+1/+1".into(), 1);
    t.g.obj_mut(bears).tapped = true;
    let hs = t.hand(P0, "Holy Strength");
    cast_paid(&mut t, P0, hs, &[o(bears)]);
    t.resolve_all();
    let gg = t.hand(P0, "Giant Growth");
    cast_paid(&mut t, P0, gg, &[o(bears)]);
    t.resolve_all();
    assert_eq!(t.pt(bears), (7, 8));
    // A Vehicle, crewed.
    let cart = t.battlefield(P0, "Smuggler's Copter");
    let elves = t.battlefield(P0, "Llanowar Elves");
    t.answer_choose(P0, &[o(elves)]);
    t.activate(P0, cart, 0, &[]).unwrap();
    t.resolve_all();
    t.clear_answers();
    assert!(t.obj_now(cart).is(CardType::Creature), "crewed");
    let sky = t.battlefield(P0, "Inspired Skypainter // Maestro's Gift");
    for (target, pt) in [(bears, (2, 2)), (cart, (3, 3))] {
        mtg_engine::designations::become_prepared(&mut t.g, sky);
        let spell = t.obj_now(sky).prepared.expect("prepared");
        let before = t.g.battlefield.clone();
        cast_paid(&mut t, P0, spell, &[o(target)]);
        t.resolve_all();
        let token =
            *t.g.battlefield
                .iter()
                .find(|x| !before.contains(x))
                .expect("a token");
        let tok = t.obj_now(token);
        assert!(!tok.tapped && tok.counters.is_empty(), "{}", t.dump_log());
        if target == bears {
            assert_eq!(t.pt(token), pt);
            assert!(tok.is(CardType::Creature));
        } else {
            // Not crewed: not a creature.
            assert!(!tok.is(CardType::Creature), "{}", t.dump_log());
        }
        assert!(t
            .g
            .battlefield
            .iter()
            .all(|x| t.obj_now(*x).attached_to != Some(o(token))));
    }
}

#[test]
fn restore_relic_token_copies_the_card_not_the_permanent_it_was() {
    cr!("707.2", "400.7", "111.4");
    ruling!(
        "Lorehold Archivist // Restore Relic",
        "The token created by Restore Relic copies exactly what was printed on the original card and nothing else. It doesn't copy any information about the object the card was before it was put into your graveyard."
    );
    supported("Lorehold Archivist // Restore Relic");
    let mut t = TestGame::new(2);
    // Grizzly Bears, with a counter and pumped, dies.
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.g.obj_mut(bears).counters.insert("+1/+1".into(), 2);
    let gg = t.hand(P0, "Giant Growth");
    cast_paid(&mut t, P0, gg, &[o(bears)]);
    t.resolve_all();
    assert_eq!(t.pt(bears), (7, 7));
    let card = move_to(&mut t, bears, Zone::Graveyard(P0));
    let a = t.battlefield(P0, "Lorehold Archivist // Restore Relic");
    mtg_engine::designations::become_prepared(&mut t.g, a);
    let spell = t.obj_now(a).prepared.expect("prepared");
    let before = t.g.battlefield.clone();
    cast_paid(&mut t, P0, spell, &[o(card)]);
    t.resolve_all();
    let token =
        *t.g.battlefield
            .iter()
            .find(|x| !before.contains(x))
            .expect("a token");
    assert_eq!(t.obj_now(token).chars.name, "Grizzly Bears");
    assert_eq!(t.pt(token), (2, 2), "{}", t.dump_log());
    assert!(t.obj_now(token).counters.is_empty());
}

// ---------------------------------------------------------------------------
// Replacement effects; what a spell does as it resolves
// ---------------------------------------------------------------------------

#[test]
fn goliath_daydreamer_spell_does_everything_then_is_exiled_unless_it_exiles_itself() {
    cr!("608.2n", "614.1a", "122.1");
    ruling!(
        "Goliath Daydreamer",
        "The spell will have all of its normal effects before being exiled. If the spell's normal effects include exiling itself, the effect of Goliath Daydreamer's first ability won't apply to it."
    );
    supported("Goliath Daydreamer");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Goliath Daydreamer");
    // Divination: draws two, then is exiled with a dream counter.
    let div = t.hand(P0, "Divination");
    let hand = t.hand_size(P0);
    cast_paid(&mut t, P0, div, &[]);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand - 1 + 2);
    assert!(exiled(&t, div));
    assert_eq!(t.counters(t.g.current(div), "dream"), 1);
    // Mnemonic Betrayal exiles itself: no dream counter.
    t.graveyard(P1, "Grizzly Bears");
    let mb = t.hand(P0, "Mnemonic Betrayal");
    cast_paid(&mut t, P0, mb, &[]);
    t.resolve_all();
    assert!(exiled(&t, mb), "{}", t.dump_log());
    assert_eq!(t.counters(t.g.current(mb), "dream"), 0, "{}", t.dump_log());
}

#[test]
fn planeswalkers_mischief_card_goes_to_its_owners_graveyard_after_resolving() {
    cr!("608.2n", "400.3");
    ruling!(
        "Planeswalker's Mischief",
        "When cast, the card goes to its owner’s graveyard after it resolves."
    );
    supported("Planeswalker's Mischief");
    let mut t = TestGame::new(2);
    let pm = t.battlefield(P0, "Planeswalker's Mischief");
    let div = t.hand(P1, "Divination");
    pool(&mut t, P0, &[U, C, C, C]);
    t.activate(P0, pm, 0, &[Entity::Player(P1)]).unwrap();
    t.resolve_all();
    assert!(exiled(&t, div), "{}", t.dump_log());
    let hand = t.hand_size(P0);
    cast_free(&mut t, P0, div);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand + 2);
    assert_eq!(t.zone(div), Zone::Graveyard(P1), "{}", t.dump_log());
}

#[test]
fn throes_of_chaos_does_nothing_as_it_resolves() {
    cr!("702.85a", "608.2");
    ruling!(
        "Throes of Chaos",
        "Throes of Chaos does nothing as it resolves. All of its chaos happens before it resolves."
    );
    supported("Throes of Chaos");
    let mut t = TestGame::new(2);
    let bears = t.library_top(P0, "Grizzly Bears");
    let throes = t.hand(P0, "Throes of Chaos");
    t.answer_yes(P0, true);
    let spell = cast_paid(&mut t, P0, throes, &[]);
    // Cascade: the Bears are cast before Throes of Chaos resolves.
    t.resolve_all();
    assert!(t.on_battlefield(t.g.current(bears)), "{}", t.dump_log());
    let log = t.dump_log();
    let resolving = log
        .find(&format!("Resolving Throes of Chaos #{}", spell.0))
        .expect("resolved");
    let after: Vec<&str> = log[resolving..].lines().skip(1).collect();
    assert_eq!(after.len(), 1, "only its move to the graveyard: {log}");
    assert!(after[0].contains("Stack -> Graveyard"));
}

#[test]
fn tibalts_trickery_on_an_uncounterable_spell_does_the_rest() {
    cr!("101.2", "608.2b", "701.6b");
    ruling!(
        "Tibalt's Trickery",
        "Tibalt's Trickery can target a spell that can't be countered. If Tibalt's Trickery resolves, the target spell won't be countered, but all of the other effects will still happen."
    );
    supported("Tibalt's Trickery");
    let mut t = TestGame::new(2);
    let orni = t.battlefield(P0, "Ornithopter");
    for _ in 0..3 {
        t.library_top(P1, "Forest");
    }
    let bears = t.library_top(P1, "Grizzly Bears");
    for _ in 0..3 {
        t.library_top(P1, "Forest");
    }
    t.set_step(P1, Step::PrecombatMain);
    let decay = t.hand(P1, "Abrupt Decay");
    let spell = cast_paid(&mut t, P1, decay, &[o(orni)]);
    let tt = t.hand(P0, "Tibalt's Trickery");
    let lib = t.library_size(P1);
    t.answer_yes(P1, true);
    cast_paid(&mut t, P0, tt, &[o(spell)]);
    t.resolve();
    // Abrupt Decay is still on the stack; P1 milled, exiled cards and cast Grizzly Bears.
    assert!(is_on_stack(&t, decay), "{}", t.dump_log());
    assert!(t.graveyard_size(P1) >= 1, "{}", t.dump_log());
    assert!(is_on_stack(&t, bears), "{}", t.dump_log());
    assert!(t.library_size(P1) < lib);
    t.resolve_all();
    assert!(!t.on_battlefield(orni));
}

#[test]
fn spelljack_works_on_a_spell_cast_with_flashback() {
    cr!("702.34a", "614.1a", "616.1");
    ruling!(
        "Spelljack",
        "This spell can be used on a spell cast using Flashback."
    );
    supported("Spelljack");
    let mut t = TestGame::new(2);
    let tt = t.graveyard(P1, "Think Twice");
    t.set_step(P1, Step::PrecombatMain);
    mana(&mut t, P1, U, 1);
    mana(&mut t, P1, C, 2);
    let spell = t
        .cast(P1, tt)
        .method(mtg_engine::object::CastMethod::Keyword(
            mtg_engine::keywords::KeywordKind::Flashback,
        ))
        .go();
    let sj = t.hand(P0, "Spelljack");
    cast_paid(&mut t, P0, sj, &[o(spell)]);
    t.resolve();
    assert!(exiled(&t, tt), "{}", t.dump_log());
    // P0 may play it without paying its mana cost.
    t.set_step(P0, Step::PostcombatMain);
    let hand = t.hand_size(P0);
    cast_free(&mut t, P0, tt);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand + 1, "{}", t.dump_log());
}

#[test]
fn dauthi_voidwalker_tokens_still_die_nontoken_creatures_are_exiled_without_dying() {
    cr!("614.1a", "700.4", "111.7");
    ruling!(
        "Dauthi Voidwalker",
        "Tokens still die while Dauthi Voidwalker is on the battlefield."
    );
    ruling!(
        "Dauthi Voidwalker",
        "While Dauthi Voidwalker is on the battlefield, nontoken creatures your opponents control won't die. They'll be exiled instead. Abilities that would trigger when those creatures die won't trigger."
    );
    supported("Dauthi Voidwalker");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Dauthi Voidwalker");
    t.battlefield(P1, "Blood Artist");
    let bears = t.battlefield(P1, "Grizzly Bears");
    let token = crate::r_s02_common::create_token(&mut t, P1, "Grizzly Bears");
    t.answer_targets(P1, &[Entity::Player(P0)]);
    t.g.destroy(token, None);
    t.resolve_all();
    assert!(!t.g.is_live(token) || !t.on_battlefield(token));
    assert_eq!(
        t.life(P0),
        19,
        "Blood Artist saw the token die: {}",
        t.dump_log()
    );
    t.g.destroy(bears, None);
    t.resolve_all();
    assert!(exiled(&t, bears), "{}", t.dump_log());
    assert_eq!(t.counters(t.g.current(bears), "void"), 1);
    assert_eq!(t.life(P0), 19, "it didn't die");
}

#[test]
fn airbender_ascension_returns_a_new_object() {
    cr!("400.7", "704.5m", "704.5n", "122.2");
    ruling!(
        "Airbender Ascension",
        "When the creature is exiled, Auras attached to the exiled creature will be put into their owners' graveyards. Any Equipment will become unattached and remain on the battlefield. Any counters on the exiled creature will cease to exist. When the card returns to the battlefield, it will be a new object with no connection to the card that was exiled."
    );
    supported("Airbender Ascension");
    let mut t = TestGame::new(2);
    let asc = t.battlefield(P0, "Airbender Ascension");
    t.g.obj_mut(asc).counters.insert("quest".into(), 4);
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.g.obj_mut(bears).counters.insert("+1/+1".into(), 1);
    let hs = t.battlefield(P0, "Holy Strength");
    assert!(t.g.attach(hs, o(bears)));
    let bs = t.battlefield(P0, "Bonesplitter");
    assert!(t.g.attach(bs, o(bears)));
    t.g.recompute();
    assert_eq!(t.pt(bears), (6, 5));
    t.answer_targets(P0, &[o(bears)]);
    t.advance_to(P0, Step::End);
    t.resolve_all();
    let back = t.g.current(bears);
    assert_ne!(back, bears, "a new object");
    assert!(t.on_battlefield(back), "{}", t.dump_log());
    assert_eq!(t.pt(back), (2, 2));
    assert!(t.obj_now(back).counters.is_empty());
    assert!(t.in_graveyard(P0, "Holy Strength"));
    assert!(t.on_battlefield(bs));
    assert_eq!(t.obj_now(bs).attached_to, None);
}

// ---------------------------------------------------------------------------
// Triggers that resolve before the spell
// ---------------------------------------------------------------------------

#[test]
fn urianger_gains_life_before_the_spell_resolves_even_if_its_countered() {
    cr!("603.3", "608.1", "305.1");
    ruling!(
        "Urianger Augurelt",
        "Urianger Augurelt's first ability resolves before the spell that caused it to trigger. It resolves even if that spell is countered or otherwise leaves the stack. (Playing a land doesn't use the stack.)"
    );
    supported("Urianger Augurelt");
    let mut t = TestGame::new(2);
    let u = t.battlefield(P0, "Urianger Augurelt");
    let lib = stack_library(&mut t, P0, &["Grizzly Bears"]);
    t.answer_yes(P0, true);
    t.activate(P0, u, 0, &[]).unwrap();
    t.resolve_all();
    t.g.untap(u);
    t.activate(P0, u, 1, &[]).unwrap();
    t.resolve_all();
    // The Bears ({1}{G} less {2}): the trigger is above it; P1 counters the Bears.
    mana(&mut t, P0, G, 1);
    let life = t.life(P0);
    let spell = t.cast(P0, t.g.current(lib[0])).go();
    t.settle();
    assert_eq!(t.stack_len(), 2, "{}", t.dump_log());
    let cs = t.hand(P1, "Counterspell");
    cast_paid(&mut t, P1, cs, &[o(spell)]);
    t.resolve();
    assert!(t.in_graveyard(P0, "Grizzly Bears"));
    t.resolve_all();
    assert_eq!(t.life(P0), life + 2, "{}", t.dump_log());
    // A land played from exile: no stack; the trigger is the only thing on it.
    let forest = t.library_top(P0, "Forest");
    t.g.untap(u);
    t.answer_yes(P0, true);
    t.activate(P0, u, 0, &[]).unwrap();
    t.resolve_all();
    t.g.untap(u);
    t.activate(P0, u, 1, &[]).unwrap();
    t.resolve_all();
    let life = t.life(P0);
    t.play_land(P0, t.g.current(forest)).unwrap();
    assert!(t.on_battlefield(t.g.current(forest)));
    t.settle();
    assert_eq!(
        stack_names(&t),
        ["Urianger Augurelt ability"],
        "{}",
        t.dump_log()
    );
    t.resolve_all();
    assert_eq!(t.life(P0), life + 2);
}

#[test]
fn vaans_counters_come_before_the_spell_resolves_even_if_its_countered() {
    cr!("603.3", "608.1");
    ruling!(
        "Vaan, Street Thief",
        "Vaan's last ability resolves before the spell that caused it to trigger. It resolves even if that spell is countered or otherwise leaves the stack."
    );
    supported("Vaan, Street Thief");
    let mut t = TestGame::new(2);
    let v = t.battlefield(P0, "Vaan, Street Thief");
    let bears = t.library_top(P1, "Grizzly Bears");
    attack_with(&mut t, &[(v, Entity::Player(P1))]);
    t.resolve_all();
    t.advance_to(P0, Step::CombatDamage);
    t.settle();
    mana(&mut t, P0, G, 2);
    t.answer_yes(P0, true);
    t.resolve();
    assert!(is_on_stack(&t, bears), "{}", t.dump_log());
    t.settle();
    // The cast trigger is above the Bears; P1 counters the Bears.
    assert_eq!(t.stack_len(), 2, "{}", t.dump_log());
    let cs = t.hand(P1, "Counterspell");
    let target = t.g.current(bears);
    cast_paid(&mut t, P1, cs, &[o(target)]);
    t.resolve();
    assert!(t.in_graveyard(P1, "Grizzly Bears"), "{}", t.dump_log());
    t.resolve_all();
    assert_eq!(t.counters(v, "+1/+1"), 1, "{}", t.dump_log());
}

// ---------------------------------------------------------------------------
// Searches and choices
// ---------------------------------------------------------------------------

/// The number of times `p` shuffled its library this turn.
fn shuffles(t: &TestGame, p: PlayerId) -> usize {
    t.g.turn_events
        .iter()
        .filter(|e| matches!(e, mtg_engine::events::Event::Shuffled { player } if *player == p))
        .count()
}

#[test]
fn thada_adel_searches_and_shuffles_even_finding_nothing() {
    cr!("701.23a", "701.23b", "701.24a");
    ruling!(
        "Thada Adel, Acquisitor",
        "When the triggered ability resolves, you must search that player's library (and they must shuffle it) even if there are no artifact cards in it or you choose not to find one."
    );
    supported("Thada Adel, Acquisitor");
    for (artifact, find) in [(false, false), (true, false)] {
        let mut t = TestGame::new(2);
        let th = t.battlefield(P0, "Thada Adel, Acquisitor");
        let orni = artifact.then(|| t.library_top(P1, "Ornithopter"));
        if !find {
            t.answer_choose(P0, &[]);
        }
        attack_with(&mut t, &[(th, Entity::Player(P1))]);
        t.resolve_all();
        let before = shuffles(&t, P1);
        t.advance_to(P0, Step::CombatDamage);
        t.resolve_all();
        assert!(
            t.g.turn_events.iter().any(|e| matches!(e,
                mtg_engine::events::Event::Searched { player, library } if *player == P0 && *library == P1)),
            "{}",
            t.dump_log()
        );
        assert_eq!(shuffles(&t, P1), before + 1, "{}", t.dump_log());
        if let Some(orni) = orni {
            assert_eq!(t.zone(orni), Zone::Library(P1));
        }
    }
}

#[test]
fn opposition_agent_cant_find_cards_the_search_doesnt_ask_for() {
    cr!("723.5", "701.23a", "701.23d");
    ruling!(
        "Opposition Agent",
        "You can't have the player find cards in their library that aren't asked for by the search instruction. For example, if the opponent is searching their library for a basic land card, you can't have them find a different card, such as a creature card."
    );
    supported("Opposition Agent");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Opposition Agent");
    let bears = t.library_top(P1, "Grizzly Bears");
    let forest = t.library_top(P1, "Forest");
    t.set_step(P1, Step::PrecombatMain);
    let rg = t.hand(P1, "Rampant Growth");
    let from = t.asked().len();
    // P0 makes P1's choice: it tries the Bears, which isn't a basic land card.
    t.answer_choose(P0, &[o(bears)]);
    cast_paid(&mut t, P1, rg, &[]);
    t.resolve_all();
    assert!(!was_offered(&t, from, bears), "{}", t.dump_log());
    assert!(was_offered(&t, from, forest));
    assert_eq!(t.zone(bears), Zone::Library(P1), "{}", t.dump_log());
}

#[test]
fn opposition_agent_controller_cant_make_tournament_or_concession_decisions() {
    cr!("723.5b", "723.6");
    ruling!(
        "Opposition Agent",
        "While controlling an opponent, you can't make any choices or decisions for that player related to tournament rules. You can't make them concede or have them agree to an intentional draw."
    );
    supported("Opposition Agent");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Opposition Agent");
    t.g.search_finds_by_default = true;
    let forest = t.library_top(P1, "Forest");
    t.set_step(P1, Step::PrecombatMain);
    let rg = t.hand(P1, "Rampant Growth");
    let from = t.asked().len();
    // P0 controls P1 during the search: the only decisions are the search's own.
    t.answer(P0, DecisionKind::Priority, Answer::Action(Action::Concede));
    cast_paid(&mut t, P1, rg, &[]);
    t.resolve();
    assert!(exiled(&t, forest), "{}", t.dump_log());
    assert!(!t.has_lost(P1));
    assert!(t.asked()[from..].iter().all(|(_, d)| match d {
        Decision::Priority { actions } => !actions.contains(&Action::Concede),
        _ => true,
    }));
}

#[test]
fn nicol_bolas_each_opponent_chooses_and_two_or_fewer_exile_all() {
    cr!("101.4", "701.13a");
    ruling!(
        "Nicol Bolas, God-Pharaoh",
        "While resolving Nicol Bolas's second ability, each opponent chooses which cards to exile from their hand. Any opponent with two or fewer cards exiles their entire hand."
    );
    supported("Nicol Bolas, God-Pharaoh");
    let mut t = TestGame::new(3);
    let nb = t.battlefield(P0, "Nicol Bolas, God-Pharaoh");
    let p1: Vec<_> = ["Opt", "Shock", "Divination"]
        .iter()
        .map(|n| t.hand(P1, n))
        .collect();
    let p2 = t.hand(P2, "Opt");
    t.answer_choose(P1, &[o(p1[0]), o(p1[2])]);
    let from = t.asked().len();
    t.activate(P0, nb, 1, &[]).unwrap();
    t.resolve_all();
    assert!(exiled(&t, p1[0]) && exiled(&t, p1[2]), "{}", t.dump_log());
    assert_eq!(t.zone(p1[1]), Zone::Hand(P1));
    assert!(exiled(&t, p2));
    // P1 chose; P0 didn't choose for anyone.
    assert!(t.asked()[from..]
        .iter()
        .any(|(p, d)| *p == P1 && matches!(d, Decision::ChooseEntities { .. })));
    assert!(!t.asked()[from..]
        .iter()
        .any(|(p, d)| *p == P0 && matches!(d, Decision::ChooseEntities { .. })));
}

#[test]
fn dance_with_calamity_shows_each_card_before_the_next_choice() {
    cr!("406.3", "608.2c");
    ruling!(
        "Dance with Calamity",
        "You can look at each card you exile before deciding whether to push your luck or stop."
    );
    supported("Dance with Calamity");
    let mut t = TestGame::new(2);
    t.g.players[0].library.clear();
    stack_library(&mut t, P0, &["Hill Giant", "Grizzly Bears", "Forest"]);
    let dance = t.hand(P0, "Dance with Calamity");
    mana(&mut t, P0, R, 8);
    // At each yes/no: the face-up exiled cards P0 can see.
    let seen = crate::r_s01_common::watch(
        &mut t,
        P0,
        |d| matches!(d, Decision::YesNo { .. }),
        |g| {
            g.exile
                .iter()
                .filter(|x| !g.obj(**x).face_down && can_look_at(g, P0, **x))
                .count()
        },
    );
    t.answer_yes(P0, true);
    t.answer_yes(P0, true);
    t.answer_yes(P0, false);
    t.answer_yes(P0, false);
    t.answer_yes(P0, false);
    t.cast(P0, dance).go();
    t.resolve_all();
    let seen = seen.lock().unwrap().clone();
    assert!(seen.len() >= 3, "{seen:?}");
    assert_eq!(&seen[..3], &[0, 1, 2], "{seen:?}: {}", t.dump_log());
}

#[test]
fn siphon_insight_exiled_card_is_hidden_from_the_opponent() {
    cr!("406.3", "401.2");
    ruling!(
        "Siphon Insight",
        "You don't have to disclose which card you exiled and which card you put on the bottom of the library. For example, if your opponent knows what the top card of their library is and you cast Siphon Insight, you don't have to tell them where you put that card."
    );
    supported("Siphon Insight");
    let mut t = TestGame::new(2);
    let lib = stack_library(&mut t, P1, &["Grizzly Bears", "Hill Giant"]);
    let si = t.hand(P0, "Siphon Insight");
    t.answer_choose(P0, &[o(lib[0])]);
    cast_paid(&mut t, P0, si, &[Entity::Player(P1)]);
    t.resolve_all();
    let c = t.g.current(lib[0]);
    assert!(exiled(&t, c) && t.obj_now(c).face_down, "{}", t.dump_log());
    assert!(can_look_at(&t.g, P0, c));
    assert!(
        !can_look_at(&t.g, P1, c),
        "P1 isn't told which card was exiled"
    );
    assert_eq!(t.zone(lib[1]), Zone::Library(P1));
    assert!(!crate::r_p068_common::in_library(&t, c, P1));
}

#[test]
fn grinning_totem_finds_any_card_and_must_find_one() {
    cr!("701.23b", "701.23a");
    ruling!(
        "Grinning Totem",
        "You may choose to find any kind of card, including a land. You must exile a card (unless the opponent’s library is empty); you can’t fail to find anything."
    );
    supported("Grinning Totem");
    // A land can be found and played.
    let mut t = TestGame::new(2);
    let gt = t.battlefield(P0, "Grinning Totem");
    let forest = t.library_top(P1, "Forest");
    t.answer_choose(P0, &[o(forest)]);
    mana(&mut t, P0, C, 2);
    t.activate(P0, gt, 0, &[Entity::Player(P1)]).unwrap();
    t.resolve_all();
    assert!(exiled(&t, forest), "{}", t.dump_log());
    assert!(playable_land(&mut t, P0, forest));
    // Answering "nothing": a card is exiled anyway.
    let mut t = TestGame::new(2);
    let gt = t.battlefield(P0, "Grinning Totem");
    t.library_top(P1, "Forest");
    t.answer_choose(P0, &[]);
    mana(&mut t, P0, C, 2);
    let lib = t.library_size(P1);
    t.activate(P0, gt, 0, &[Entity::Player(P1)]).unwrap();
    t.resolve_all();
    assert_eq!(t.library_size(P1), lib - 1, "{}", t.dump_log());
    assert_eq!(t.g.exile.len(), 1);
}

#[test]
fn kotose_may_leave_cards_with_the_same_name() {
    cr!("701.23b", "701.23d");
    ruling!(
        "Kotose, the Silent Spider",
        "You may leave any cards with the same name as the target card in the zones they're in if you want."
    );
    supported("Kotose, the Silent Spider");
    let mut t = TestGame::new(2);
    let target = t.graveyard(P1, "Lightning Bolt");
    let in_hand = t.hand(P1, "Lightning Bolt");
    let in_gy = t.graveyard(P1, "Lightning Bolt");
    let in_lib = t.library_top(P1, "Lightning Bolt");
    t.answer_targets(P0, &[o(target)]);
    // Exile only the one in the library (searched after the graveyard and the hand).
    t.answer_choose(P0, &[]);
    t.answer_choose(P0, &[]);
    t.answer_choose(P0, &[o(in_lib)]);
    t.enter(P0, "Kotose, the Silent Spider");
    t.resolve_all();
    assert!(exiled(&t, target), "{}", t.dump_log());
    assert!(exiled(&t, in_lib), "{}", t.dump_log());
    assert_eq!(t.zone(in_hand), Zone::Hand(P1));
    assert_eq!(t.zone(in_gy), Zone::Graveyard(P1));
}

#[test]
fn brass_tunnel_grinder_discards_any_number_even_none() {
    cr!("608.2c", "701.9a");
    ruling!(
        "Brass's Tunnel-Grinder // Tecutlan, the Searing Rift",
        "You choose how many cards to discard as the first ability of Brass's Tunnel-Grinder resolves. You may choose to discard no cards and just draw a card."
    );
    supported("Brass's Tunnel-Grinder // Tecutlan, the Searing Rift");
    for n in [0usize, 2] {
        let mut t = TestGame::new(2);
        let hand: Vec<_> = (0..3).map(|_| t.hand(P0, "Opt")).collect();
        let chosen: Vec<_> = hand.iter().take(n).map(|c| o(*c)).collect();
        t.answer_choose(P0, &chosen);
        t.enter(P0, "Brass's Tunnel-Grinder // Tecutlan, the Searing Rift");
        t.resolve_all();
        assert_eq!(t.graveyard_size(P0), n, "{}", t.dump_log());
        assert_eq!(t.hand_size(P0), 3 - n + n + 1);
    }
}

#[test]
fn kheru_spellsnatcher_can_counter_your_own_spell_and_cast_it_later() {
    cr!("702.37e", "608.2b");
    ruling!(
        "Kheru Spellsnatcher",
        "You can target a spell you control with Kheru Spellsnatcher's triggered ability. This will give you the ability to cast the card later without paying its mana cost."
    );
    supported("Kheru Spellsnatcher");
    let mut t = TestGame::new(2);
    let k = t.hand(P0, "Kheru Spellsnatcher");
    mana(&mut t, P0, C, 3);
    let fd = t
        .cast(P0, k)
        .method(mtg_engine::object::CastMethod::FaceDown(
            mtg_engine::keywords::KeywordKind::Morph,
        ))
        .go();
    t.resolve_all();
    let fd = t.g.current(fd);
    assert!(t.obj_now(fd).face_down);
    let bears = t.hand(P0, "Grizzly Bears");
    let spell = cast_paid(&mut t, P0, bears, &[]);
    pool(&mut t, P0, &[U, U, C, C, C, C]);
    t.answer_targets(P0, &[o(spell)]);
    assert!(
        crate::r_s11_common::turn_face_up(&mut t, P0, fd),
        "{}",
        t.dump_log()
    );
    t.resolve();
    assert!(exiled(&t, bears), "{}", t.dump_log());
    t.resolve_all();
    // Later: cast for free.
    t.set_step(P0, Step::PostcombatMain);
    cast_free(&mut t, P0, bears);
    t.resolve_all();
    assert!(t.on_battlefield(t.g.current(bears)), "{}", t.dump_log());
}
