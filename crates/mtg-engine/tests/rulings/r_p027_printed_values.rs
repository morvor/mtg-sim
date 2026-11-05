//! Rulings batch P027 — a copy gets the copiable values of what it copies: its printed
//! values plus copy effects (and a merged permanent's characteristics), not its status,
//! counters, attachments or other effects (CR 707.2, 707.3, 613.2a, 721.2); a face-down
//! permanent's copiable values are those of a nameless 2/2 (CR 708.2, 707.2). Also Valki
//! and Tibalt, whose copy and play permissions use cards exiled with them (CR 607.2a).

use crate::r_p021_common::{animate_land, assert_printed_giant, dressed_giant, red, untap};
use crate::r_p027_common::*;
use crate::r_s01_common::{attack_with, supported};
use crate::r_s06_common::{activate_containing, attach_new};
use crate::r_s11_common::manifest_card;
use crate::r_s18_common::lands_for;
use crate::r_s25_common::cast_new;
use mtg_engine::card::card;
use mtg_engine::decision::Answer;
use mtg_engine::events::MoveCause;
use mtg_engine::keywords::KeywordKind;
use mtg_engine::mana::ManaType;
use mtg_engine::object::{CastMethod, Zone};
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

#[test]
fn tilonallis_skinshifter_copies_only_the_printed_values() {
    cr!("707.2", "613.1a");
    ruling!("Tilonalli's Skinshifter", "Tilonalli’s Skinshifter copies the printed values of the target creature, plus any copy effects that have been applied to it. It won’t copy counters on that creature or effects that have changed its power, toughness, types, color, or so on.");
    let mut t = TestGame::new(2);
    let skin = t.battlefield(P0, "Tilonalli's Skinshifter");
    // P0's Hill Giant: two +1/+1 counters, +3/+3 and green-blue, Bonesplitter.
    let giant = dressed_giant(&mut t, P0);
    untap(&mut t, giant);
    // A Clone copying it: a copy effect the Skinshifter copies too.
    let elves = t.battlefield(P0, "Llanowar Elves");
    t.answer_targets(P0, &[obj(giant)]);
    attack_with(
        &mut t,
        &[
            (skin, Entity::Player(P1)),
            (giant, Entity::Player(P1)),
            (elves, Entity::Player(P1)),
        ],
    );
    t.resolve_all();
    assert_printed_giant(&t, skin, Some((3, 3)), false);
    // Copying a Clone that copies the Hill Giant: a Hill Giant too.
    let mut t = TestGame::new(2);
    let skin = t.battlefield(P0, "Tilonalli's Skinshifter");
    let giant = t.battlefield(P1, "Hill Giant");
    t.answer_choose(P0, &[obj(giant)]);
    let clone = t.enter(P0, "Clone");
    t.settle();
    crate::r_p023_common::unsick(&mut t, clone);
    t.answer_targets(P0, &[obj(clone)]);
    attack_with(
        &mut t,
        &[(skin, Entity::Player(P1)), (clone, Entity::Player(P1))],
    );
    t.resolve_all();
    assert_printed_giant(&t, skin, Some((3, 3)), false);
}

#[test]
fn true_polymorph_copies_printed_values_copy_and_mutate_effects() {
    cr!("707.2", "707.3", "730.2a", "702.140e", "613.1a");
    ruling!("True Polymorph", "True Polymorph causes the first target permanent to copy the printed values of the second target permanent, including any copy and mutate effects that have been applied to it. It won't copy counters on that permanent or effects that have changed its power, toughness, types, color, and so on.");
    supported("True Polymorph");
    // A dressed-up Hill Giant: just a Hill Giant.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let giant = dressed_giant(&mut t, P1);
    cast_new(&mut t, P0, "True Polymorph", &[obj(bears), obj(giant)]);
    t.resolve_all();
    assert_printed_giant(&t, bears, Some((3, 3)), true);
    // A Clone copying a Hill Giant: a Hill Giant.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let giant = t.battlefield(P1, "Hill Giant");
    t.answer_choose(P1, &[obj(giant)]);
    let clone = t.enter(P1, "Clone");
    t.settle();
    cast_new(&mut t, P0, "True Polymorph", &[obj(bears), obj(clone)]);
    t.resolve_all();
    assert_printed_giant(&t, bears, Some((3, 3)), true);
    // A Grizzly Bears with Dreamtail Heron mutated on top: a 3/4 flying Dreamtail Heron
    // with its mutate trigger.
    let mut t = TestGame::new(2);
    let target = t.battlefield(P1, "Llanowar Elves");
    let base = t.battlefield(P0, "Grizzly Bears");
    let heron = t.hand(P0, "Dreamtail Heron");
    t.g.players[0].mana_pool.add_type(ManaType::U, 1);
    t.g.players[0].mana_pool.add_type(ManaType::C, 3);
    t.cast(P0, heron)
        .method(CastMethod::Keyword(KeywordKind::Mutate))
        .target(obj(base))
        .go();
    t.answer(P0, DecisionKind::Option, Answer::Index(0));
    t.resolve_all();
    assert_eq!(name_of(&t, base), "Dreamtail Heron");
    cast_new(&mut t, P0, "True Polymorph", &[obj(target), obj(base)]);
    t.resolve_all();
    assert_eq!(name_of(&t, target), "Dreamtail Heron");
    assert_eq!(t.pt(target), (3, 4));
    assert!(t.obj_now(target).has_keyword(KeywordKind::Flying));
    assert_eq!(abilities_with(&t, target, "mutates"), 1);
}

#[test]
fn true_polymorph_has_no_duration() {
    cr!("611.2a", "400.7", "707.2");
    ruling!("True Polymorph", "True Polymorph's effect does not have a duration. The copy effect it creates lasts as long as the first target remains on the battlefield.");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let giant = t.battlefield(P1, "Hill Giant");
    cast_new(&mut t, P0, "True Polymorph", &[obj(bears), obj(giant)]);
    t.resolve_all();
    for p in [P1, P0, P1] {
        t.advance_to(p, Step::Upkeep);
        assert_eq!(name_of(&t, bears), "Hill Giant");
    }
    // Even after the Hill Giant leaves.
    kill(&mut t, giant);
    assert_eq!(name_of(&t, bears), "Hill Giant");
    // Once it leaves the battlefield, the effect is gone.
    let b = t.g.current(bears);
    t.g.move_object(b, Zone::Hand(P0), MoveCause::Effect, None);
    t.g.recompute();
    assert_eq!(name_of(&t, bears), "Grizzly Bears");
}

/// `name` enters for P0 copying P1's dressed-up Hill Giant (tapped, with counters, +3/+3,
/// green-blue, Bonesplitter and Holy Strength attached). Returns the game, the copy and
/// the Giant.
fn enters_copying_dressed_giant(name: &str) -> (TestGame, ObjectId, ObjectId) {
    supported(name);
    let mut t = TestGame::new(2);
    let giant = dressed_giant(&mut t, P1);
    attach_new(&mut t, P1, "Holy Strength", giant);
    assert_eq!(t.pt(giant), (11, 10));
    t.answer_choose(P0, &[obj(giant)]);
    let c = t.enter(P0, name);
    t.settle();
    (t, c, giant)
}

#[test]
fn vizier_of_many_faces_copies_only_what_was_printed() {
    cr!("707.2", "613.1a");
    ruling!("Vizier of Many Faces", "Vizier of Many Faces copies exactly what was printed on the original creature (unless that creature is copying something else or is a token; see below). It doesn't copy whether that creature is tapped or untapped, whether it has any counters on it or Auras attached to it, or any non-copy effects that have changed its power, toughness, types, color, or so on.");
    let (t, c, _) = enters_copying_dressed_giant("Vizier of Many Faces");
    assert_printed_giant(&t, c, Some((3, 3)), true);
}

#[test]
fn wall_of_stolen_identity_copies_only_what_was_printed() {
    cr!("707.2", "707.9a", "707.9b", "613.1a");
    ruling!("Wall of Stolen Identity", "Wall of Stolen Identity copies exactly what was printed on the original creature (unless that creature is copying something else or is a token; see below). It doesn’t copy whether that creature is tapped or untapped, whether it has any counters on it or any Auras and Equipment attached to it, or any non-copy effects that have changed its power, toughness, types, color, or so on.");
    let (t, c, _) = enters_copying_dressed_giant("Wall of Stolen Identity");
    // A Wall with defender in addition (the copy effect's exception).
    assert_printed_giant(&t, c, Some((3, 3)), true);
    assert!(has_subtype(&t, c, "Wall"));
    assert!(t.obj_now(c).has_keyword(KeywordKind::Defender));
}

#[test]
fn volrath_copies_printed_values_not_counters_or_animation() {
    cr!("707.2", "707.9b", "613.1a");
    ruling!("Volrath, the Shapestealer", "Volrath copies the printed values of the target creature, plus any copy effects that have been applied to it. It won't copy counters on that creature. It also won't copy any effects that have changed its power, toughness, types, color, or so on. Notably, it won't copy effects that made the target become a creature if it's not normally a creature.");
    // "{1}: Until your next turn, Volrath becomes a copy of target creature with a counter
    // on it, except it's 7/5 and it has this ability."
    supported("Volrath, the Shapestealer");
    let mut t = TestGame::new(2);
    let volrath = t.battlefield(P0, "Volrath, the Shapestealer");
    let giant = dressed_giant(&mut t, P1);
    t.lands(P0, "Wastes", 1);
    t.answer_targets(P0, &[obj(giant)]);
    activate_containing(&mut t, P0, volrath, "becomes a copy").expect("activation");
    t.resolve_all();
    let o = t.obj_now(volrath);
    assert_eq!(o.chars.name, "Hill Giant");
    assert_eq!(o.chars.colors, red());
    assert_eq!(t.pt(volrath), (7, 5));
    assert_eq!(t.counters(volrath, counters::PLUS1), 0);
    // A land made a creature (with a counter on it): Volrath becomes a land, not a
    // creature.
    let mut t = TestGame::new(2);
    let volrath = t.battlefield(P0, "Volrath, the Shapestealer");
    let forest = t.battlefield(P1, "Forest");
    animate_land(&mut t, forest);
    plus1(&mut t, forest, 1);
    t.lands(P0, "Wastes", 1);
    t.answer_targets(P0, &[obj(forest)]);
    activate_containing(&mut t, P0, volrath, "becomes a copy").expect("activation");
    t.resolve_all();
    let o = t.obj_now(volrath);
    assert_eq!(o.chars.name, "Forest");
    assert!(o.is(CardType::Land));
    assert!(!o.is(CardType::Creature));
}

#[test]
fn vesuvan_doppelganger_copying_a_face_down_creature() {
    cr!("707.2", "708.2a", "707.9a", "707.9c");
    ruling!("Vesuvan Doppelganger", "Vesuvan Doppelganger copies a face-down creature, it becomes a face-up 2/2 blue creature with no name and no abilities other than the one it gives itself.");
    let mut t = TestGame::new(2);
    let fd = manifest_card(&mut t, P1, "Serra Angel");
    t.answer_choose(P0, &[obj(fd)]);
    let dop = t.enter(P0, "Vesuvan Doppelganger");
    t.settle();
    let o = t.obj_now(dop);
    assert!(!o.face_down);
    assert!(o.chars.name.is_empty());
    assert_eq!(t.pt(dop), (2, 2));
    assert_eq!(o.chars.colors, ColorSet::single(Color::Blue));
    assert!(o.chars.subtypes.is_empty());
    assert_eq!(o.chars.abilities.len(), 1);
    assert!(o.chars.abilities[0].text.contains("upkeep"));
}

// --- Valki, God of Lies // Tibalt, Cosmic Impostor ------------------------------------------

const VALKI: &str = "Valki, God of Lies // Tibalt, Cosmic Impostor";

/// P0's Valki enters in a game of `n` players; each opponent's hand has the creature card
/// given, which Valki exiles. Returns the game, Valki and the exiled cards.
fn valki_exiling(n: usize, cards: &[&str]) -> (TestGame, ObjectId, Vec<ObjectId>) {
    supported(VALKI);
    let mut t = TestGame::new(n);
    let mut hand = vec![];
    for (i, c) in cards.iter().enumerate() {
        let p = PlayerId(i as u8 + 1);
        let h = t.hand(p, c);
        t.answer_choose(P0, &[obj(h)]);
        hand.push(h);
    }
    let valki = t.enter(P0, VALKI);
    t.resolve_all();
    let exiled: Vec<ObjectId> = hand.iter().map(|h| t.g.current(*h)).collect();
    for e in &exiled {
        assert_eq!(t.zone(*e), Zone::Exile);
    }
    (t, valki, exiled)
}

/// Activates Valki's "{X}: Choose a creature card exiled with Valki with mana value X.
/// Valki becomes a copy of that card." with X = `x` (paid with Wastes).
fn activate_valki(t: &mut TestGame, valki: ObjectId, x: i64) {
    t.lands(P0, "Wastes", x as usize);
    t.answer(P0, DecisionKind::X, Answer::Number(x));
    activate_containing(t, P0, valki, "becomes a copy").expect("activation");
}

#[test]
fn valki_copies_the_printed_card_and_loses_his_ability() {
    cr!("707.2", "607.2a");
    ruling!("Valki, God of Lies // Tibalt, Cosmic Impostor", "Valki copies the printed values of the exiled creature card. Notably, once Valki becomes a copy of another creature card, he won't have his own printed activated ability.");
    let (mut t, valki, _) = valki_exiling(2, &["Hill Giant"]);
    activate_valki(&mut t, valki, 4);
    t.resolve_all();
    assert_printed_giant(&t, valki, Some((3, 3)), true);
    assert_eq!(abilities_with(&t, valki, "becomes a copy"), 0);
}

#[test]
fn valki_chooses_the_card_as_the_ability_resolves() {
    cr!("607.2a", "608.2d");
    ruling!("Valki, God of Lies // Tibalt, Cosmic Impostor", "You don't choose which creature card exiled with Valki that Valki will become a copy of until that ability is resolving.");
    let (mut t, valki, exiled) = valki_exiling(3, &["Grizzly Bears", "Runeclaw Bear"]);
    let from = t.asked().len();
    activate_valki(&mut t, valki, 2);
    // Activated: nothing chosen yet.
    assert_eq!(t.stack_len(), 1);
    assert!(choices_since(&t, from).is_empty());
    t.answer_choose(P0, &[obj(exiled[1])]);
    t.resolve_all();
    let offered = choices_since(&t, from);
    assert_eq!(offered.len(), 1);
    assert!(offered[0].0.contains(&obj(exiled[0])) && offered[0].0.contains(&obj(exiled[1])));
    assert_eq!(name_of(&t, valki), "Runeclaw Bear");
}

#[test]
fn valki_can_activate_in_response_to_himself() {
    cr!("602.2", "117.3c", "607.2a", "113.7a");
    ruling!("Valki, God of Lies // Tibalt, Cosmic Impostor", "You can activate Valki's ability multiple times in response to one another. This may briefly allow Valki to copy different creature cards. You'll get priority to cast spells or activate abilities in between each of Valki's activated abilities.");
    let (mut t, valki, _) = valki_exiling(3, &["Grizzly Bears", "Hill Giant"]);
    activate_valki(&mut t, valki, 2);
    activate_valki(&mut t, valki, 4);
    assert_eq!(t.stack_len(), 2);
    t.resolve();
    assert_eq!(name_of(&t, valki), "Hill Giant");
    // The other activation is still waiting; players get priority now.
    assert_eq!(t.stack_len(), 1);
    t.resolve_all();
    assert_eq!(name_of(&t, valki), "Grizzly Bears");
}

/// P0 casts Tibalt, Cosmic Impostor (Valki's back face). Returns the planeswalker.
fn tibalt(t: &mut TestGame) -> ObjectId {
    supported(VALKI);
    lands_for(t, P0, "{B}{R}");
    t.lands(P0, "Wastes", 5);
    let c = t.hand(P0, VALKI);
    t.cast(P0, c).method(CastMethod::Half(1)).go();
    t.resolve_all();
    t.named_on_battlefield("Tibalt, Cosmic Impostor")[0]
}

#[test]
fn lands_exiled_with_tibalt_use_the_land_play() {
    cr!("305.2", "607.2a");
    ruling!("Valki, God of Lies // Tibalt, Cosmic Impostor", "Unless an effect allows you to play additional lands that turn, you can play land cards exiled with Tibalt only if you haven't played a land yet that turn.");
    let mut t = TestGame::new(2);
    let tib = tibalt(&mut t);
    let a = t.library_top(P1, "Forest");
    let b = t.library_top(P0, "Mountain");
    t.activate(P0, tib, 0, &[]).expect("+2");
    t.resolve_all();
    let (a, b) = (t.g.current(a), t.g.current(b));
    assert_eq!(t.zone(a), Zone::Exile);
    assert_eq!(t.zone(b), Zone::Exile);
    // P0 plays one of them as the land for the turn, and not the other.
    t.play_land(P0, a).expect("the land play");
    assert!(t.on_battlefield(a));
    assert!(t.play_land(P0, b).is_err());
    assert_eq!(t.zone(b), Zone::Exile);
    // Next turn, it can be played.
    t.advance_to(P1, Step::Upkeep);
    t.advance_to(P0, Step::PrecombatMain);
    t.play_land(P0, b).expect("the next land play");
    // A land played from hand first uses the land play.
    let mut t = TestGame::new(2);
    let tib = tibalt(&mut t);
    let e = t.library_top(P1, "Forest");
    t.activate(P0, tib, 0, &[]).expect("+2");
    t.resolve_all();
    let hand = t.hand(P0, "Plains");
    t.play_land(P0, hand).expect("land from hand");
    assert!(t.play_land(P0, t.g.current(e)).is_err());
}

#[test]
fn tibalts_last_ability_adds_rrr_with_no_cards_exiled() {
    cr!("606.3", "106.4");
    ruling!("Valki, God of Lies // Tibalt, Cosmic Impostor", "While resolving Tibalt's last ability, you'll add {R}{R}{R} even if you don't exile any cards.");
    let mut t = TestGame::new(2);
    let tib = tibalt(&mut t);
    t.g.add_counters(obj(tib), counters::LOYALTY, 3, None);
    assert_eq!(t.counters(tib, counters::LOYALTY), 8);
    assert_eq!(t.graveyard_size(P0) + t.graveyard_size(P1), 0);
    activate_containing(&mut t, P0, tib, "Exile all graveyards").expect("-8");
    t.resolve();
    assert_eq!(crate::r_s24_common::pool(&t, P0, ManaType::R), 3);
}

// --- Volatile Chimera ----------------------------------------------------------------------

#[test]
fn volatile_chimeras_reveal_one_or_three_each() {
    cr!("607.2n", "707.9a");
    ruling!("Volatile Chimera", "Unlike the other two cards with this style of ability (Arcane Savant and Caller of the Untamed), if you have more than one Volatile Chimera in your deck, you need only reveal one of them because you can exile as many creature cards that aren't in your deck as you want, with a minimum of three. If you do reveal more than one Volatile Chimera before shuffling, you'll have to exile at least three creature cards for each of them.");
    supported("Volatile Chimera");
    let side = [
        "Hill Giant",
        "Grizzly Bears",
        "Craw Wurm",
        "Llanowar Elves",
        "Serra Angel",
        "Runeclaw Bear",
    ];
    // Revealing one: P0 exiles five, and the other Chimera can copy them too.
    let mut t = TestGame::new(2);
    let chimeras = [
        t.library_top(P0, "Volatile Chimera"),
        t.library_top(P0, "Volatile Chimera"),
    ];
    let cards =
        t.g.add_to_sideboard(P0, side.iter().map(|n| card(n)).collect());
    let from = t.asked().len();
    t.answer_yes(P0, true);
    t.answer_choose(P0, &cards[..5].iter().map(|c| obj(*c)).collect::<Vec<_>>());
    t.answer_yes(P0, false);
    mtg_engine::opening_hand::before_shuffle_actions(&mut t.g);
    let mins: Vec<u32> = choices_since(&t, from).iter().map(|c| c.1).collect();
    assert_eq!(mins, vec![3]);
    let exiled = side[..5].iter().filter(|n| t.in_exile(n)).count();
    assert_eq!(exiled, 5);
    assert!(!t.in_exile("Runeclaw Bear"));
    // Each Chimera, the revealed one and the other, can become a copy of a card exiled
    // with cards named Volatile Chimera (CR 607.2n): "{1}{R}: Choose a card at random you
    // exiled with cards named Volatile Chimera. This creature becomes a copy of that card,
    // except it has this ability."
    for c in chimeras {
        let c =
            t.g.move_object(c, Zone::Battlefield, MoveCause::Effect, None)
                .expect("onto the battlefield");
        t.g.flush_events();
        t.settle();
        lands_for(&mut t, P0, "{1}{R}");
        t.activate(P0, c, 0, &[]).expect("activation");
        t.resolve_all();
        let now = name_of(&t, c);
        assert!(side[..5].contains(&now.as_str()), "it became {now}");
        assert_eq!(abilities_with(&t, c, "exiled with cards named"), 1);
    }
    // Revealing both: at least three for each.
    let mut t = TestGame::new(2);
    t.library_top(P0, "Volatile Chimera");
    t.library_top(P0, "Volatile Chimera");
    let cards =
        t.g.add_to_sideboard(P0, side.iter().map(|n| card(n)).collect());
    let from = t.asked().len();
    t.answer_yes(P0, true);
    t.answer_choose(P0, &cards[..3].iter().map(|c| obj(*c)).collect::<Vec<_>>());
    t.answer_yes(P0, true);
    t.answer_choose(P0, &cards[3..].iter().map(|c| obj(*c)).collect::<Vec<_>>());
    mtg_engine::opening_hand::before_shuffle_actions(&mut t.g);
    let mins: Vec<u32> = choices_since(&t, from).iter().map(|c| c.1).collect();
    assert_eq!(mins, vec![3, 3]);
    assert!(side.iter().all(|n| t.in_exile(n)));
    // Trying to exile fewer than three for the second one isn't allowed.
    let mut t = TestGame::new(2);
    t.library_top(P0, "Volatile Chimera");
    t.library_top(P0, "Volatile Chimera");
    let cards =
        t.g.add_to_sideboard(P0, side.iter().map(|n| card(n)).collect());
    t.answer_yes(P0, true);
    t.answer_choose(P0, &cards[..4].iter().map(|c| obj(*c)).collect::<Vec<_>>());
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[obj(cards[4])]);
    mtg_engine::opening_hand::before_shuffle_actions(&mut t.g);
    // Only two creature cards were left: the second one couldn't be revealed.
    let exiled = side.iter().filter(|n| t.in_exile(n)).count();
    assert_eq!(exiled, 4);
}
