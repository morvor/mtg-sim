//! Rulings batch P056 — returning cards to the battlefield (Gruesome Menagerie, Yorion,
//! Retether, Rise to Glory and other effects putting Auras onto the battlefield, CR 303.4f,
//! 400.7, 603.6a), Bound // Determined (CR 101.2, 701.6), and choices made as a spell or
//! ability resolves (Council's Judgment's votes, Haphazard Bombardment's random choice,
//! Intrude on the Mind's piles, Coax from the Blind Eternities' card).

use crate::r_p017_common::choice_candidates;
use crate::r_p056_common::*;
use crate::r_s01_common::supported;
use crate::r_s06_common::{attach_new, attached_to};
use crate::r_s29_common::{cast_and_resolve, choose_modes, put_counters};
use mtg_engine::decision::{Answer, Decision};
use mtg_engine::object::{CastMethod, Zone};
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

fn priority_asked_since(t: &TestGame, from: usize) -> bool {
    t.asked()[from..]
        .iter()
        .any(|(_, d)| matches!(d, Decision::Priority { .. }))
}

// ---------------------------------------------------------------------------
// Gruesome Menagerie
// ---------------------------------------------------------------------------

#[test]
fn gruesome_menagerie_returns_the_cards_at_the_same_time() {
    cr!("603.6a", "608.2c");
    ruling!(
        "Gruesome Menagerie",
        "All of the returned cards enter the battlefield at the same time."
    );
    supported("Gruesome Menagerie");
    // Soul Warden: "Whenever another creature enters, you gain 1 life." It sees the other
    // two enter with it.
    let mut t = TestGame::new(2);
    let warden = t.graveyard(P0, "Soul Warden");
    let bears = t.graveyard(P0, "Grizzly Bears");
    let giant = t.graveyard(P0, "Centaur Courser");
    t.answer_choose(P0, &[obj(warden)]);
    t.answer_choose(P0, &[obj(bears)]);
    t.answer_choose(P0, &[obj(giant)]);
    cast_and_resolve(&mut t, P0, "Gruesome Menagerie", &[]);
    for c in [warden, bears, giant] {
        assert!(t.on_battlefield(c), "{:?}\n{}", t.asked(), t.dump_log());
    }
    assert_eq!(t.life(P0), 22);
}

#[test]
fn gruesome_menagerie_chooses_as_it_resolves() {
    cr!("608.2c", "115.10");
    ruling!(
        "Gruesome Menagerie",
        "Gruesome Menagerie doesn't target the cards to return. You choose them while it's resolving."
    );
    ruling!(
        "Gruesome Menagerie",
        "If you don't have a creature card in your graveyard with one mana value, you just continue on to the next."
    );
    // Cast with an empty graveyard; a creature card gets there in response; there's no
    // card with mana value 1.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let giant = t.graveyard(P0, "Centaur Courser");
    t.lands(P0, "Swamp", 5);
    let spell = t.hand(P0, "Gruesome Menagerie");
    t.cast(P0, spell).go();
    t.g.destroy(bears, None);
    t.g.flush_events();
    let bears_card = t.g.current(bears);
    t.answer_choose(P0, &[obj(bears_card)]);
    t.answer_choose(P0, &[obj(giant)]);
    t.resolve_all();
    assert!(t.on_battlefield(bears_card));
    assert!(t.on_battlefield(giant));
}

// ---------------------------------------------------------------------------
// Yorion, Sky Nomad
// ---------------------------------------------------------------------------

#[test]
fn yorion_exiled_permanents_return_as_new_objects() {
    cr!("400.7", "303.4c", "301.5c", "122.2");
    ruling!(
        "Yorion, Sky Nomad",
        "Auras attached to the exiled permanents will be put into their owners' graveyards. Equipment attached to the exiled permanents will become unattached and remain on the battlefield. Any counters on the exiled permanents will cease to exist."
    );
    supported("Yorion, Sky Nomad");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    put_counters(&mut t, bears, "+1/+1", 2);
    let aura = attach_new(&mut t, P0, "Holy Strength", obj(bears));
    let blade = attach_new(&mut t, P0, "Bonesplitter", obj(bears));
    t.answer_choose(P0, &[obj(bears)]);
    t.enter(P0, "Yorion, Sky Nomad");
    t.resolve_all();
    assert_eq!(t.zone(bears), Zone::Exile);
    assert!(t.in_graveyard(P0, "Holy Strength"));
    assert_eq!(t.zone(aura), Zone::Graveyard(P0));
    assert!(t.on_battlefield(blade));
    assert_eq!(attached_to(&t, blade), None);
    t.advance_to(P0, Step::End);
    t.resolve_all();
    let back = t.g.current(bears);
    assert!(t.on_battlefield(back));
    assert_ne!(back, bears);
    assert_eq!(t.counters(back, "+1/+1"), 0);
    assert_eq!(t.pt(back), (2, 2));
}

// ---------------------------------------------------------------------------
// Auras returned to the battlefield
// ---------------------------------------------------------------------------

#[test]
fn retether_returns_an_enchant_land_aura_to_an_animated_land() {
    cr!("303.4f", "303.4a");
    ruling!(
        "Retether",
        "Auras don't need to say \"enchant creature\" to return to the battlefield. For example, an Aura with \"enchant land\" will return to the battlefield if there's an animated land for it to enchant."
    );
    supported("Retether");
    // Wild Growth ("Enchant land"); Dryad Arbor is a land creature.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Forest");
    let arbor = t.battlefield(P0, "Dryad Arbor");
    let growth = t.graveyard(P0, "Wild Growth");
    t.answer_choose(P0, &[obj(arbor)]);
    cast_and_resolve(&mut t, P0, "Retether", &[]);
    assert!(t.on_battlefield(growth));
    assert_eq!(attached_to(&t, growth), Some(obj(arbor)));
    // With no land creature, it stays in the graveyard.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Forest");
    t.battlefield(P0, "Grizzly Bears");
    let growth = t.graveyard(P0, "Wild Growth");
    cast_and_resolve(&mut t, P0, "Retether", &[]);
    assert_eq!(t.zone(growth), Zone::Graveyard(P0));
}

#[test]
fn rise_to_glory_aura_can_enchant_the_creature_it_returned() {
    cr!("700.2", "608.2c", "303.4f");
    ruling!(
        "Rise to Glory",
        "Because modes are performed sequentially, an Aura returned to the battlefield with Rise to Glory can enchant a creature returned to the battlefield with Rise to Glory."
    );
    supported("Rise to Glory");
    let mut t = TestGame::new(2);
    let bears = t.graveyard(P0, "Grizzly Bears");
    let aura = t.graveyard(P0, "Holy Strength");
    choose_modes(&mut t, P0, &[0, 1]);
    let back = |t: &TestGame| t.g.current(bears);
    t.lands(P0, "Plains", 3);
    t.lands(P0, "Swamp", 2);
    let spell = t.hand(P0, "Rise to Glory");
    t.cast(P0, spell)
        .targets(&[obj(bears)])
        .targets(&[obj(aura)])
        .go();
    t.resolve_all();
    assert!(t.on_battlefield(bears));
    assert_eq!(attached_to(&t, aura), Some(obj(back(&t))));
    assert_eq!(t.pt(bears), (3, 4));
}

/// P1 controls Gladecover Scout (hexproof) as the only creature; `put` puts P0's
/// Pacifism (in `p`'s graveyard, or wherever `put` needs it) onto the battlefield without
/// casting it, P0 choosing the Scout. It ends up attached to the Scout.
fn pacifism_on_hexproof(name: &str, put: fn(&mut TestGame, ObjectId) -> ObjectId) {
    supported(name);
    let mut t = TestGame::new(2);
    let scout = t.battlefield(P1, "Gladecover Scout");
    let aura = t.graveyard(P0, "Pacifism");
    t.answer_choose(P0, &[obj(scout)]);
    let aura = put(&mut t, aura);
    assert!(t.on_battlefield(aura), "{name}: {}", t.dump_log());
    assert_eq!(attached_to(&t, aura), Some(obj(scout)), "{name}");
}

#[test]
fn auras_put_onto_the_battlefield_dont_target() {
    cr!("303.4f", "702.11b");
    ruling!(
        "Rise to Glory",
        "An Aura put onto the battlefield this way doesn’t target anything (so it could be attached to an opponent’s permanent with hexproof"
    );
    ruling!(
        "Brotherhood Outcast",
        "An Aura put onto the battlefield this way doesn’t target anything (so it could be attached to an opponent’s permanent with hexproof"
    );
    ruling!(
        "Calix, Destiny's Hand",
        "An Aura put onto the battlefield this way doesn’t target anything (so it could be attached to an opponent’s permanent with hexproof"
    );
    ruling!(
        "Flicker of Fate",
        "An Aura put onto the battlefield this way doesn’t target anything (so it could be attached to an opponent’s permanent with hexproof"
    );
    ruling!(
        "Rescue Skiff",
        "An Aura put onto the battlefield this way doesn’t target anything (so it could be attached to an opponent’s permanent with hexproof"
    );
    pacifism_on_hexproof("Rise to Glory", |t, aura| {
        choose_modes(t, P0, &[1]);
        cast_and_resolve(t, P0, "Rise to Glory", &[obj(aura)]);
        aura
    });
    pacifism_on_hexproof("Brotherhood Outcast", |t, aura| {
        choose_modes(t, P0, &[0]);
        t.answer_targets(P0, &[obj(aura)]);
        t.enter(P0, "Brotherhood Outcast");
        t.resolve_all();
        aura
    });
    pacifism_on_hexproof("Rescue Skiff", |t, aura| {
        t.answer_targets(P0, &[obj(aura)]);
        t.enter(P0, "Rescue Skiff");
        t.resolve_all();
        aura
    });
    pacifism_on_hexproof("Calix, Destiny's Hand", |t, aura| {
        let calix = t.battlefield(P0, "Calix, Destiny's Hand");
        put_counters(t, calix, "loyalty", 7);
        crate::r_s06_common::activate_containing(t, P0, calix, "Return all").expect("-7");
        t.resolve_all();
        aura
    });
    pacifism_on_hexproof("Flicker of Fate", |t, _| {
        // Another Pacifism is on the battlefield attached to P1's Grizzly Bears; it's
        // flickered and returns attached to the Scout.
        let bears = t.battlefield(P1, "Grizzly Bears");
        let aura = attach_new(t, P0, "Pacifism", obj(bears));
        cast_and_resolve(t, P0, "Flicker of Fate", &[obj(aura)]);
        aura
    });
}

#[test]
fn an_aura_with_nothing_to_enchant_stays_where_it_is() {
    cr!("303.4g");
    ruling!(
        "Rise to Glory",
        "If the Aura can’t legally be attached to anything, it remains in its current zone."
    );
    let mut t = TestGame::new(2);
    let aura = t.graveyard(P0, "Pacifism");
    choose_modes(&mut t, P0, &[1]);
    cast_and_resolve(&mut t, P0, "Rise to Glory", &[obj(aura)]);
    assert_eq!(t.zone(aura), Zone::Graveyard(P0));
}

// ---------------------------------------------------------------------------
// Bound // Determined
// ---------------------------------------------------------------------------

/// P0 casts Determined ("Other spells you control can't be countered this turn. Draw a
/// card.") with mana for it.
fn cast_determined(t: &mut TestGame) -> ObjectId {
    t.lands(P0, "Forest", 1);
    t.lands(P0, "Island", 1);
    let c = t.hand(P0, "Bound // Determined");
    t.cast(P0, c).method(CastMethod::Half(1)).go();
    c
}

/// P0 casts Grizzly Bears (with mana for it); P1 casts Counterspell targeting it.
fn bears_and_counterspell(t: &mut TestGame) -> (ObjectId, ObjectId) {
    t.lands(P0, "Forest", 2);
    let bears = t.hand(P0, "Grizzly Bears");
    t.cast(P0, bears).go();
    t.lands(P1, "Island", 2);
    let cs = t.hand(P1, "Counterspell");
    let spell = t.g.current(bears);
    t.cast(P1, cs).target(spell).go();
    (bears, cs)
}

#[test]
fn after_determined_counterspells_resolve_but_dont_counter() {
    cr!("101.2", "701.6a");
    ruling!(
        "Bound // Determined",
        "After Determined resolves, any spells or abilities that would counter a spell you control that turn will still resolve. They just won't counter that spell."
    );
    supported("Bound // Determined");
    let mut t = TestGame::new(2);
    cast_determined(&mut t);
    t.resolve_all();
    let (bears, _) = bears_and_counterspell(&mut t);
    t.resolve_all();
    assert!(t.in_graveyard(P1, "Counterspell"));
    assert!(t.on_battlefield(bears));
}

#[test]
fn spells_can_be_countered_while_determined_is_on_the_stack() {
    cr!("101.2", "608.2");
    ruling!(
        "Bound // Determined",
        "Spells you control may be countered by spells or abilities while Determined is on the stack, waiting to resolve."
    );
    let mut t = TestGame::new(2);
    t.lands(P0, "Forest", 2);
    let bears = t.hand(P0, "Grizzly Bears");
    t.cast(P0, bears).go();
    cast_determined(&mut t);
    t.lands(P1, "Island", 2);
    let cs = t.hand(P1, "Counterspell");
    let spell = t.g.current(bears);
    t.cast(P1, cs).target(spell).go();
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Grizzly Bears"));
}

/// P0 casts Bound with `sacrificed` (if any) on the battlefield, answering the choices
/// with `choose` (the creature to sacrifice, then the cards to return).
fn cast_bound(t: &mut TestGame, choose: &[Vec<Entity>]) {
    for c in choose {
        t.answer_choose(P0, c);
    }
    t.lands(P0, "Swamp", 3);
    t.lands(P0, "Forest", 2);
    let c = t.hand(P0, "Bound // Determined");
    t.cast(P0, c).method(CastMethod::Half(0)).go();
    t.resolve_all();
    assert!(t.in_exile("Bound // Determined"), "{}", t.dump_log());
}

#[test]
fn bound_chooses_the_returned_cards_as_it_resolves() {
    cr!("608.2c", "115.10");
    ruling!(
        "Bound // Determined",
        "The cards in the graveyard that you return as part of Bound's effect aren't targeted. They're chosen when the spell resolves. The creature you sacrificed may be one of them."
    );
    // Boros Swiftblade is red and white: up to two cards. P0 returns the Swiftblade
    // itself and Hill Giant (choosing among the cards in the graveyard as it resolves).
    let mut t = TestGame::new(2);
    let blade = t.battlefield(P0, "Boros Swiftblade");
    t.graveyard(P0, "Hill Giant");
    t.graveyard(P0, "Grizzly Bears");
    crate::r_s03_common::respond(&mut t, P0, |g, d| match d {
        Decision::ChooseEntities { candidates, .. }
            if candidates.iter().all(
                |c| matches!(c, Entity::Object(o) if matches!(g.obj(*o).zone, Zone::Graveyard(_))),
            ) =>
        {
            Some(Answer::Entities(
                candidates
                    .iter()
                    .copied()
                    .filter(|c| match c {
                        Entity::Object(o) => {
                            let n = &g.obj(*o).chars.name;
                            n == "Boros Swiftblade" || n == "Hill Giant"
                        }
                        _ => false,
                    })
                    .collect(),
            ))
        }
        _ => None,
    });
    cast_bound(&mut t, &[vec![obj(blade)]]);
    assert!(t.in_hand(P0, "Boros Swiftblade"), "{}", t.dump_log());
    assert!(t.in_hand(P0, "Hill Giant"));
    assert!(t.in_graveyard(P0, "Grizzly Bears"));
}

#[test]
fn bound_sacrifice_is_part_of_resolution_and_colorless_returns_nothing() {
    cr!("608.2c", "105.2c");
    ruling!(
        "Bound // Determined",
        "The creature is sacrificed when Bound resolves. It's not an additional cost. It's not optional. If you can't sacrifice a creature or a colorless creature is sacrificed, no cards are returned."
    );
    // No creature: it can be cast; nothing is returned.
    let mut t = TestGame::new(2);
    t.graveyard(P0, "Hill Giant");
    cast_bound(&mut t, &[]);
    assert!(t.in_graveyard(P0, "Hill Giant"));
    // Ornithopter (colorless): sacrificed, nothing returned.
    let mut t = TestGame::new(2);
    let thopter = t.battlefield(P0, "Ornithopter");
    let giant = t.graveyard(P0, "Hill Giant");
    cast_bound(&mut t, &[vec![obj(thopter)], vec![obj(giant)]]);
    assert!(t.in_graveyard(P0, "Ornithopter"));
    assert!(t.in_graveyard(P0, "Hill Giant"));
}

// ---------------------------------------------------------------------------
// Council's Judgment
// ---------------------------------------------------------------------------

#[test]
fn councils_judgment_every_player_votes_and_exile_follows_at_once() {
    cr!("701.38a", "608.2");
    ruling!(
        "Council's Judgment",
        "Each player must vote for one of the candidate permanents. They can't abstain."
    );
    ruling!(
        "Council's Judgment",
        "Players can't do anything after they finishing voting but before the \"winning\" permanent or permanents are exiled."
    );
    supported("Council's Judgment");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let giant = t.battlefield(P1, "Hill Giant");
    let land = t.battlefield(P1, "Forest");
    t.answer_choose(P0, &[obj(bears)]);
    // P1 tries to vote for a land (not a candidate): they still vote for a candidate.
    t.answer_choose(P1, &[obj(land)]);
    t.lands(P0, "Plains", 3);
    let j = t.hand(P0, "Council's Judgment");
    t.cast(P0, j).go();
    t.settle();
    let from = t.asked().len();
    t.g.resolve_top();
    assert!(!priority_asked_since(&t, from));
    let votes = choice_candidates(&t, from);
    assert_eq!(votes.len(), 2, "both players voted");
    assert!(t.zone(bears) == Zone::Exile);
    assert!(t.on_battlefield(land));
    let _ = giant;
}

#[test]
fn councils_judgment_with_no_votes_exiles_nothing() {
    cr!("701.38a");
    ruling!(
        "Council's Judgment",
        "In the unusual case that no permanents have any votes, nothing is exiled."
    );
    let mut t = TestGame::new(2);
    let mine = t.battlefield(P0, "Grizzly Bears");
    let land = t.battlefield(P1, "Forest");
    cast_and_resolve(&mut t, P0, "Council's Judgment", &[]);
    assert!(t.on_battlefield(mine) && t.on_battlefield(land));
    assert_eq!(t.g.exile.len(), 0);
}

// ---------------------------------------------------------------------------
// Haphazard Bombardment
// ---------------------------------------------------------------------------

#[test]
fn haphazard_bombardment_aim_counters_from_anywhere() {
    cr!("122.1", "608.2");
    ruling!(
        "Haphazard Bombardment",
        "Aim counters are interchangeable. Your Haphazard Bombardment can destroy any permanent you don’t control with an aim counter on it no matter how that counter got there"
    );
    ruling!(
        "Haphazard Bombardment",
        "Players can’t take actions between the time the permanent is randomly selected and the time it’s destroyed. Notably, if a land is randomly selected, it can’t be tapped for mana before being destroyed."
    );
    supported("Haphazard Bombardment");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Haphazard Bombardment");
    let lands = t.lands(P1, "Forest", 2);
    for l in &lands {
        put_counters(&mut t, *l, "aim", 1);
    }
    t.advance_to(P0, Step::End);
    t.settle();
    let from = t.asked().len();
    t.g.resolve_top();
    assert!(!priority_asked_since(&t, from));
    let left = lands.iter().filter(|l| t.on_battlefield(**l)).count();
    assert_eq!(left, 1, "{}", t.dump_log());
    // The destroyed land wasn't tapped for mana first.
    assert!(lands.iter().all(|l| !tapped(&t, *l)));
}

// ---------------------------------------------------------------------------
// Intrude on the Mind
// ---------------------------------------------------------------------------

#[test]
fn intrude_on_the_mind_you_choose_the_opponent_and_a_pile_may_be_empty() {
    cr!("700.3d", "700.3a");
    ruling!(
        "Intrude on the Mind",
        "You decide which opponent chooses the pile while resolving Intrude on the Mind."
    );
    ruling!(
        "Intrude on the Mind",
        "You may choose to put all of the cards into one pile and leave the other pile empty."
    );
    supported("Intrude on the Mind");
    let mut t = TestGame::new(3);
    let library: Vec<ObjectId> = (0..6).map(|_| t.library_top(P0, "Island")).collect();
    t.lands(P0, "Island", 5);
    // P0 chooses P2 to choose; all five cards in the first pile.
    t.answer_choose(
        P0,
        &library[1..].iter().map(|c| obj(*c)).collect::<Vec<_>>(),
    );
    t.answer_choose(P0, &[Entity::Player(P2)]);
    // P2 chooses the empty pile: all five go to the graveyard.
    t.answer(P2, DecisionKind::Option, Answer::Index(1));
    let spell = t.hand(P0, "Intrude on the Mind");
    let hand = t.hand_size(P0);
    let from = t.asked().len();
    t.cast(P0, spell).go();
    t.resolve_all();
    let asked = t.asked()[from..].to_vec();
    assert!(
        asked
            .iter()
            .any(|(p, d)| *p == P2 && matches!(d, Decision::ChooseOption { .. })),
        "{asked:?}"
    );
    assert!(!asked
        .iter()
        .any(|(p, d)| *p == P1 && matches!(d, Decision::ChooseOption { .. })));
    assert_eq!(t.hand_size(P0), hand - 1);
    assert_eq!(t.graveyard_size(P0), 6, "the five cards and the spell");
}

// ---------------------------------------------------------------------------
// Coax from the Blind Eternities
// ---------------------------------------------------------------------------

#[test]
fn coax_chooses_only_face_up_eldrazi_cards() {
    cr!("406.3", "708.2", "205.3j");
    ruling!(
        "Coax from the Blind Eternities",
        "A card that is exiled face down has no characteristics. You can’t choose it, even if you’re allowed to look at it."
    );
    ruling!(
        "Coax from the Blind Eternities",
        "An Eldrazi card is one with the subtype Eldrazi. Noncreature cards with devoid from the Battle for Zendikar block are Eldrazi-themed, but they are not Eldrazi cards."
    );
    supported("Coax from the Blind Eternities");
    let mut t = TestGame::new(2);
    let up = t.exile(P0, "Ulamog's Crusher");
    let down = t.exile(P0, "Eldrazi Devastator");
    t.g.objects[down.0 as usize].face_down = true;
    let devoid = t.exile(P0, "Grip of Desolation");
    t.g.recompute();
    let from = t.asked().len();
    t.answer_choose(P0, &[obj(up)]);
    cast_and_resolve(&mut t, P0, "Coax from the Blind Eternities", &[]);
    let cands: Vec<Entity> = choice_candidates(&t, from).concat();
    assert!(cands.contains(&obj(up)), "{cands:?}");
    assert!(!cands.contains(&obj(down)));
    assert!(!cands.contains(&obj(devoid)));
    assert!(t.in_hand(P0, "Ulamog's Crusher"));
}
