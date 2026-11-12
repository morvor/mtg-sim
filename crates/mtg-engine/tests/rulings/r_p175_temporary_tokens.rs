//! Rulings batch P175 — tokens exiled or sacrificed "at the beginning of the next end
//! step" (Valduk, Elemental Mastery, Rakdos Guildmage, Elemental Appeal, Salt Road
//! Skirmish), Chandra, Flamecaller's discard-and-draw, and Saheeli, the Gifted's affinity
//! for the next spell (CR 601.2f, 702.41).

use crate::r_p175_common::*;
use crate::r_s06_common::{attach_new, attached_to};
use crate::r_s26_common::new_tokens;
use mtg_engine::mana::ManaType;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

/// The number of `p`'s Elemental tokens on the battlefield.
fn elementals(t: &TestGame, p: PlayerId) -> usize {
    tokens(t, p)
        .into_iter()
        .filter(|id| t.g.obj(*id).chars.has_subtype("Elemental"))
        .count()
}

/// P0's Valduk with P0's Bonesplitter and P1's Holy Strength attached; P0's beginning of
/// combat step, with Valduk's trigger on the stack.
fn valduk() -> (TestGame, ObjectId) {
    let mut t = TestGame::new(2);
    let v = t.battlefield(P0, "Valduk, Keeper of the Flame");
    attach_new(&mut t, P0, "Bonesplitter", v);
    attach_new(&mut t, P1, "Holy Strength", v);
    t.advance_to(P0, Step::BeginningOfCombat);
    t.settle();
    assert_eq!(t.stack_len(), 1);
    (t, v)
}

#[test]
fn valduk_counts_every_aura_and_equipment_attached() {
    cr!("301.5", "303.4");
    ruling!(
        "Valduk, Keeper of the Flame",
        "Valduk counts all Auras and Equipment attached to it, not only Auras and Equipment you control."
    );
    supported("Valduk, Keeper of the Flame");
    let (mut t, _) = valduk();
    t.resolve_all();
    assert_eq!(elementals(&t, P0), 2);
}

#[test]
fn valduk_leaving_before_resolution_uses_what_was_last_attached() {
    cr!("608.2h", "113.7a");
    ruling!(
        "Valduk, Keeper of the Flame",
        "If Valduk leaves the battlefield after its ability has triggered but before it resolves, use the number of Auras and Equipment that were last attached to it before it left the battlefield to determine how many tokens to create."
    );
    let (mut t, v) = valduk();
    kill(&mut t, v);
    t.resolve_all();
    assert_eq!(elementals(&t, P0), 2);
    // An Aura that left before Valduk did wasn't attached to it as it last existed.
    let (mut t, v) = valduk();
    let aura = t.named_on_battlefield("Holy Strength")[0];
    kill(&mut t, aura);
    kill(&mut t, v);
    t.resolve_all();
    assert_eq!(elementals(&t, P0), 1);
    // The Bonesplitter attached to another creature after Valduk left (Magnetic Theft:
    // "Attach target Equipment to target creature.") was still attached to Valduk as
    // Valduk last existed.
    supported("Magnetic Theft");
    let (mut t, v) = valduk();
    let bears = t.battlefield(P0, "Grizzly Bears");
    let splitter = t.named_on_battlefield("Bonesplitter")[0];
    kill(&mut t, v);
    cast_at(&mut t, P0, "Magnetic Theft", &[obj(splitter), obj(bears)]);
    t.resolve();
    assert_eq!(attached_to(&t, splitter), Some(obj(bears)));
    t.resolve_all();
    assert_eq!(elementals(&t, P0), 2);
    // Control: an Equipment attached to another creature all along isn't counted.
    let (mut t, v) = valduk();
    let bears = t.battlefield(P0, "Grizzly Bears");
    attach_new(&mut t, P0, "Short Sword", bears);
    kill(&mut t, v);
    t.resolve_all();
    assert_eq!(elementals(&t, P0), 2);
}

#[test]
fn valduk_tokens_are_exiled_even_if_valduk_left() {
    cr!("603.7c", "603.7b");
    ruling!(
        "Valduk, Keeper of the Flame",
        "If Valduk leaves the battlefield after its ability has resolved, the tokens are still exiled at the beginning of the next end step."
    );
    let (mut t, v) = valduk();
    t.resolve_all();
    kill(&mut t, v);
    assert_eq!(elementals(&t, P0), 2);
    end_step(&mut t);
    assert_eq!(elementals(&t, P0), 0);
}

#[test]
fn tokens_made_during_the_end_step_last_until_the_next_turns_end_step() {
    cr!("603.7a", "513.1");
    ruling!(
        "Elemental Mastery",
        "If the ability is activated after the current turn's End step has begun, the tokens won't be exiled until the next turn's End step."
    );
    ruling!(
        "Rakdos Guildmage",
        "If the second ability is activated during the end step, the token won't be exiled until the following turn's end step."
    );
    supported("Elemental Mastery");
    supported("Rakdos Guildmage");
    // (source to activate, the ability's index): Elemental Mastery on a 2/2; Rakdos
    // Guildmage's second ability.
    for guildmage in [false, true] {
        let mut t = TestGame::new(2);
        let (src, idx, n) = if guildmage {
            (t.battlefield(P0, "Rakdos Guildmage"), 1, 1)
        } else {
            let bears = t.battlefield(P0, "Grizzly Bears");
            attach_new(&mut t, P0, "Elemental Mastery", bears);
            (bears, 0, 2)
        };
        let kind = if guildmage { "Goblin" } else { "Elemental" };
        t.advance_to(P0, Step::End);
        t.resolve_all();
        act(&mut t, P0, src, idx, &[]);
        assert_eq!(with_subtype(&t, P0, kind).len(), n, "{kind}");
        // Next turn: still there until its end step begins.
        t.advance_to(P1, Step::PostcombatMain);
        assert_eq!(with_subtype(&t, P0, kind).len(), n, "{kind}");
        t.advance_to(P1, Step::End);
        t.resolve_all();
        assert!(with_subtype(&t, P0, kind).is_empty(), "{kind}");
    }
}

#[test]
fn elemental_appeal_kicked_token_is_a_7_1_with_a_pump() {
    cr!("707.2", "611.2a", "514.2", "702.33d");
    ruling!(
        "Elemental Appeal",
        "If Elemental Appeal was kicked, it creates a 7/1 token with trample and haste that gets +7/+0 until end of turn. It doesn't create a 14/1 token."
    );
    supported("Elemental Appeal");
    supported("Rootborn Defenses");
    // Kicked: a 14/1 this turn; a populated copy of it is a 7/1.
    let mut t = TestGame::new(2);
    lots(&mut t, P0);
    let spell = t.hand(P0, "Elemental Appeal");
    let before = t.g.battlefield.clone();
    t.cast(P0, spell).kicked(true).go();
    t.resolve_all();
    let tok = new_tokens(&t, P0, &before)[0];
    assert_eq!(t.pt(tok), (14, 1));
    let before = t.g.battlefield.clone();
    cast_at(&mut t, P0, "Rootborn Defenses", &[]);
    t.answer_choose(P0, &[obj(tok)]);
    t.resolve_all();
    let copy = new_tokens(&t, P0, &before)[0];
    assert_eq!(t.pt(copy), (7, 1));
    // Cast in the end step (with Leyline of Anticipation): the token survives that end
    // step and is a 7/1 next turn.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Leyline of Anticipation");
    t.advance_to(P0, Step::End);
    t.resolve_all();
    lots(&mut t, P0);
    let spell = t.hand(P0, "Elemental Appeal");
    let before = t.g.battlefield.clone();
    t.cast(P0, spell).kicked(true).go();
    t.resolve_all();
    let tok = new_tokens(&t, P0, &before)[0];
    assert_eq!(t.pt(tok), (14, 1));
    t.advance_to(P1, Step::Upkeep);
    assert_eq!(t.pt(tok), (7, 1));
    t.advance_to(P1, Step::End);
    t.resolve_all();
    assert!(!t.on_battlefield(tok));
}

#[test]
fn salt_road_skirmish_with_an_illegal_or_indestructible_target() {
    cr!("608.2b", "702.12b");
    ruling!(
        "Salt Road Skirmish",
        "If the target creature is an illegal target at the time Salt Road Skirmish tries to resolve, it won’t resolve and none of its effects will happen. However, if the target is still legal and the target isn’t destroyed (most likely because it has indestructible), you will still create two Warrior tokens."
    );
    supported("Salt Road Skirmish");
    for indestructible in [false, true] {
        let mut t = TestGame::new(2);
        let target = if indestructible {
            t.battlefield(P1, "Darksteel Myr")
        } else {
            t.battlefield(P1, "Grizzly Bears")
        };
        cast_at(&mut t, P0, "Salt Road Skirmish", &[obj(target)]);
        if !indestructible {
            bounce(&mut t, target);
        }
        t.resolve_all();
        let warriors = with_subtype(&t, P0, "Warrior").len();
        assert_eq!(warriors, if indestructible { 2 } else { 0 });
        if indestructible {
            assert!(t.on_battlefield(target));
        }
    }
}

#[test]
fn chandra_discards_and_draws_counting_madness_cards() {
    cr!("614.1a", "702.35a", "121.2");
    ruling!(
        "Chandra, Flamecaller",
        "For Chandra's second ability, it doesn't matter if the cards don't go to your graveyard because of a replacement effect. For example, cards with madness discarded this way will still count toward the number of cards you discarded."
    );
    ruling!(
        "Chandra, Flamecaller",
        "You can activate Chandra's second ability while you have an empty hand. If you do, you'll draw one card."
    );
    supported("Chandra, Flamecaller");
    // A madness card (exiled instead) and a Grizzly Bears: three cards drawn.
    let mut t = TestGame::new(2);
    let c = t.battlefield(P0, "Chandra, Flamecaller");
    t.hand(P0, "Fiery Temper");
    t.hand(P0, "Grizzly Bears");
    t.answer_yes(P0, false);
    act(&mut t, P0, c, 1, &[]);
    assert_eq!(t.hand_size(P0), 3);
    assert_eq!(drawn(&t, P0), 3);
    // An empty hand: one card drawn.
    let mut t = TestGame::new(2);
    let c = t.battlefield(P0, "Chandra, Flamecaller");
    assert_eq!(t.hand_size(P0), 0);
    act(&mut t, P0, c, 1, &[]);
    assert_eq!(t.hand_size(P0), 1);
}

/// P0's Saheeli, the Gifted with the given artifacts: its second +1 ("The next spell you
/// cast this turn has affinity for artifacts.") has resolved; P0's mana pool is empty.
fn saheeli_affinity(t: &mut TestGame, artifacts: &[&str]) {
    for a in artifacts {
        t.battlefield(P0, a);
    }
    let s = t.battlefield(P0, "Saheeli, the Gifted");
    act(t, P0, s, 1, &[]);
    t.g.players[P0.idx()].mana_pool = Default::default();
}

#[test]
fn saheeli_affinity_reduces_only_generic_mana() {
    cr!("702.41a", "601.2f");
    ruling!(
        "Saheeli, the Gifted",
        "Saheeli's second ability reduces only generic mana in the cost of the next spell you cast."
    );
    ruling!(
        "Saheeli, the Gifted",
        "To determine the total cost of a spell, start with the mana cost or alternative cost you're paying, add any cost increases, then apply any cost reductions (such as that of Saheeli's second ability). The mana value of the spell remains unchanged, no matter what the total cost to cast it was."
    );
    // Five artifacts: Hill Giant ({3}{R}) costs {R}, not less.
    crate::r_p194_common::assert_total_cost(
        &|t| saheeli_affinity(t, &["Ornithopter"; 5]),
        P0,
        "Hill Giant",
        &[(ManaType::R, 1)],
        0,
    );
    assert!(!crate::r_p194_common::casts_for(
        &|t| saheeli_affinity(t, &["Ornithopter"; 5]),
        P0,
        "Hill Giant",
        &[],
        0
    ));
    // With Thalia, Guardian of Thraben ("Noncreature spells cost {1} more"), Divination
    // ({2}{U}) and two artifacts: {2}{U} + {1} − {2} = {1}{U}. Its mana value is still 3.
    crate::r_p194_common::assert_total_cost(
        &|t| {
            t.battlefield(P1, "Thalia, Guardian of Thraben");
            saheeli_affinity(t, &["Ornithopter"; 2]);
        },
        P0,
        "Divination",
        &[(ManaType::U, 1)],
        1,
    );
    let mut t = TestGame::new(2);
    t.battlefield(P1, "Thalia, Guardian of Thraben");
    saheeli_affinity(&mut t, &["Ornithopter"; 2]);
    let card = t.hand(P0, "Divination");
    mana(&mut t, P0, ManaType::U, 2);
    let spell = t.cast(P0, card).go();
    assert_eq!(t.obj(spell).chars.mana_value(), 3);
}

#[test]
fn saheeli_affinity_total_cost_is_locked_in_before_mana_abilities() {
    cr!("601.2f", "601.2g", "601.2h");
    ruling!(
        "Saheeli, the Gifted",
        "The total cost to cast a spell is locked in before you pay that cost. For example, if you control three artifacts, including one you can sacrifice to add {C}, Saheeli's second ability will reduce the cost of a spell you cast by {3}. Then you can sacrifice the artifact when you activate mana abilities just before paying the cost, and it remains reduced by {3}."
    );
    // Two Ornithopters and a Treasure: Hill Giant costs {R}, paid by sacrificing the
    // Treasure (leaving two artifacts).
    let mut t = TestGame::new(2);
    let treasure = make_token(
        &mut t,
        P0,
        mtg_engine::tokens_predefined::predefined("Treasure").unwrap(),
    );
    saheeli_affinity(&mut t, &["Ornithopter"; 2]);
    let giant = t.hand(P0, "Hill Giant");
    t.cast(P0, giant).go();
    assert!(!t.on_battlefield(treasure), "sacrificed for mana");
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Hill Giant").len(), 1);
}
