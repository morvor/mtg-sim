//! Rulings batch P158 — tokens that are copies: the copy gets only the copiable values
//! (CR 707.2) — the original characteristics of a copied token (CR 111.3, 707.2) or what a
//! copying permanent copies (CR 707.3) — not counters, damage, tapped status or other
//! effects; the copy's abilities work (CR 707.5); a copy of Anax has its dies trigger.

use crate::r_p158_common::*;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

/// `p`'s green 2/2 Wolf creature token changed in every way a copy doesn't copy (tapped,
/// two +1/+1 counters, +3/+3 and green-blue until end of turn): a 7/7.
fn dressed_token(t: &mut TestGame, p: PlayerId) -> ObjectId {
    let tok = typed_token(t, p, "Wolf", Color::Green, 2);
    dress_up(t, tok);
    let tok = t.g.current(tok);
    t.g.recompute();
    assert_eq!(t.pt(tok), (7, 7));
    tok
}

/// `p`'s Grizzly Bears changed the same way: a 7/7.
fn dressed_bears(t: &mut TestGame, p: PlayerId) -> ObjectId {
    let bears = t.battlefield(p, "Grizzly Bears");
    dress_up(t, bears);
    t.g.recompute();
    assert_eq!(t.pt(bears), (7, 7));
    bears
}

/// Asserts `id` is a fresh (untapped, no counters) `name` with `pt` and no blue in its
/// colors: what a copy of a dressed-up permanent is.
fn assert_plain(t: &TestGame, id: ObjectId, name: &str, pt: (i32, i32)) {
    assert_eq!(name_of(t, id), name);
    assert!(fresh(t, id), "{name}: it copied the tapped status or counters");
    assert!(
        !t.obj_now(id).chars.colors.contains(Color::Blue),
        "{name}: it copied a color-changing effect"
    );
    assert_eq!(t.pt(id), pt, "{name}");
}

/// `p`'s Clone copying `p`'s Serra Angel.
fn clone_of_angel(t: &mut TestGame, p: PlayerId) -> ObjectId {
    let angel = t.battlefield(p, "Serra Angel");
    crate::r_p023_common::clone_of(t, p, angel)
}

#[test]
fn rite_of_replication_copies_only_the_copiable_values() {
    cr!("707.2", "111.3", "707.3");
    ruling!(
        "Rite of Replication",
        "The tokens copy exactly what was printed on the original creature and nothing else (unless that permanent is itself copying something else; see below). It doesn't copy whether that permanent is tapped or untapped, whether it has any counters on it or Auras attached to it, or any non-copy effects that have changed its power, toughness, types, color, and so on."
    );
    ruling!(
        "Rite of Replication",
        "If the copied creature is a token, the tokens created copy the original characteristics of that token as stated by the effect that created that token."
    );
    ruling!(
        "Rite of Replication",
        "If the copied creature is copying something else, then the tokens enter the battlefield as whatever that creature copied."
    );
    supported("Rite of Replication");
    for kicked in [false, true] {
        let n = if kicked { 5 } else { 1 };
        // A dressed-up Grizzly Bears: 2/2 Bears.
        let mut t = TestGame::new(2);
        let bears = dressed_bears(&mut t, P1);
        let before = t.g.battlefield.clone();
        lots(&mut t, P0);
        let r = t.hand(P0, "Rite of Replication");
        t.cast(P0, r).kicked(kicked).target(obj(bears)).go();
        t.resolve_all();
        let toks = new_tokens(&t, P0, &before);
        assert_eq!(toks.len(), n, "kicked {kicked}");
        for k in &toks {
            assert_plain(&t, *k, "Grizzly Bears", (2, 2));
        }
        // A dressed-up Wolf token: 2/2 green Wolves.
        let mut t = TestGame::new(2);
        let wolf = dressed_token(&mut t, P1);
        let before = t.g.battlefield.clone();
        lots(&mut t, P0);
        let r = t.hand(P0, "Rite of Replication");
        t.cast(P0, r).kicked(kicked).target(obj(wolf)).go();
        t.resolve_all();
        let toks = new_tokens(&t, P0, &before);
        assert_eq!(toks.len(), n, "kicked {kicked}");
        for k in &toks {
            assert_plain(&t, *k, "Wolf", (2, 2));
            assert!(t.obj_now(*k).chars.colors.contains(Color::Green));
        }
        // A Clone copying Serra Angel: Serra Angels.
        let mut t = TestGame::new(2);
        let clone = clone_of_angel(&mut t, P1);
        let before = t.g.battlefield.clone();
        lots(&mut t, P0);
        let r = t.hand(P0, "Rite of Replication");
        t.cast(P0, r).kicked(kicked).target(obj(clone)).go();
        t.resolve_all();
        let toks = new_tokens(&t, P0, &before);
        assert_eq!(toks.len(), n, "kicked {kicked}");
        for k in &toks {
            assert_plain(&t, *k, "Serra Angel", (4, 4));
        }
    }
}

#[test]
fn mythos_of_illuna_copies_only_the_copiable_values() {
    cr!("707.2", "111.3", "707.3");
    ruling!(
        "Mythos of Illuna",
        "The token copies exactly what is printed on the permanent and nothing else (unless that permanent is copying something else or is a token; see below). It doesn't copy whether that permanent is tapped or untapped, whether it has any counters on it or Auras and/or Equipment attached to it, or any non-copy effects that changed its power, toughness, types, color, and so on."
    );
    ruling!(
        "Mythos of Illuna",
        "If the copied permanent is itself a token, the token created by Mythos of Illuna copies the original characteristics of that token as stated by the effect that created it."
    );
    supported("Mythos of Illuna");
    // (what, its name as copied, its printed power and toughness)
    for which in ["bears", "token", "clone"] {
        let mut t = TestGame::new(2);
        let (target, name, pt) = match which {
            "bears" => (dressed_bears(&mut t, P1), "Grizzly Bears", (2, 2)),
            "token" => (dressed_token(&mut t, P1), "Wolf", (2, 2)),
            _ => (clone_of_angel(&mut t, P1), "Serra Angel", (4, 4)),
        };
        let before = t.g.battlefield.clone();
        cast_at(&mut t, P0, "Mythos of Illuna", &[obj(target)]);
        t.resolve_all();
        let toks = new_tokens(&t, P0, &before);
        assert_eq!(toks.len(), 1, "{which}");
        assert_plain(&t, toks[0], name, pt);
    }
}

#[test]
fn krothuss_copies_a_tokens_original_characteristics() {
    cr!("707.2", "111.3");
    ruling!(
        "Runo Stromkirk // Krothuss, Lord of the Deep",
        "If the copied creature is a token, the token created by Krothuss copies the original characteristics of that token as stated by the effect that put it onto the battlefield."
    );
    supported("Runo Stromkirk // Krothuss, Lord of the Deep");
    let mut t = TestGame::new(2);
    let k = t.battlefield(P0, "Runo Stromkirk // Krothuss, Lord of the Deep");
    crate::r_s17_common::transform(&mut t, k);
    let k = t.g.current(k);
    assert_eq!(name_of(&t, k), "Krothuss, Lord of the Deep");
    let wolf = dressed_token(&mut t, P0);
    t.g.untap(wolf);
    let before = t.g.battlefield.clone();
    t.answer_targets(P0, &[obj(wolf)]);
    attack_with(&mut t, &[(k, pl(P1)), (wolf, pl(P1))]);
    t.resolve_all();
    let toks = new_tokens(&t, P0, &before);
    assert_eq!(toks.len(), 1);
    let o = t.obj_now(toks[0]);
    assert_eq!(o.chars.name, "Wolf");
    assert!(o.counters.values().all(|n| *n == 0), "it copied the counters");
    assert!(o.tapped, "it enters tapped");
    assert!(!o.chars.colors.contains(Color::Blue));
    assert_eq!(t.pt(toks[0]), (2, 2));
    let c = t.g.combat.as_ref().unwrap();
    assert_eq!(c.attack_target(toks[0]), Some(pl(P1)));
}

#[test]
fn hazel_copies_a_tokens_original_characteristics_or_what_it_copies() {
    cr!("707.2", "111.3", "707.3");
    ruling!(
        "Hazel of the Rootbloom",
        "The token (or tokens) you create copies the original characteristics of the token as stated by the effect that created that token (unless that token is copying something else; see below). It doesn't copy whether that token is tapped or untapped, whether it has any counters on it or Auras and Equipment attached to it, or any non-copy effects that have changed its power, toughness, types, color, or so on."
    );
    supported("Hazel of the Rootbloom");
    // A dressed-up Wolf token: a plain 2/2 Wolf.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Hazel of the Rootbloom");
    let wolf = dressed_token(&mut t, P0);
    let before = t.g.battlefield.clone();
    t.answer_targets(P0, &[obj(wolf)]);
    t.advance_to(P0, Step::End);
    t.resolve_all();
    let toks = new_tokens(&t, P0, &before);
    assert_eq!(toks.len(), 1);
    assert_plain(&t, toks[0], "Wolf", (2, 2));
    // A token copy of a Serra Angel, dressed up: a plain Serra Angel.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Hazel of the Rootbloom");
    let angel = t.battlefield(P1, "Serra Angel");
    let copy = crate::r_s17_common::token_copy(&mut t, P0, angel)[0];
    dress_up(&mut t, copy);
    t.g.recompute();
    assert_eq!(t.pt(copy), (9, 9));
    let before = t.g.battlefield.clone();
    t.answer_targets(P0, &[obj(copy)]);
    t.advance_to(P0, Step::End);
    t.resolve_all();
    let toks = new_tokens(&t, P0, &before);
    assert_eq!(toks.len(), 1);
    assert_plain(&t, toks[0], "Serra Angel", (4, 4));
}

const LANDFALL: &str = "Whenever a land you control enters";

/// P0's Scute Swarm (or a Clone copying it when `clone`) with `lands` lands; then a land
/// enters. Returns the Swarm (or Clone) and the tokens made.
fn landfall(lands: usize, clone: bool, dress: bool) -> (TestGame, ObjectId, Vec<ObjectId>) {
    supported("Scute Swarm");
    let mut t = TestGame::new(2);
    let mut swarm = t.battlefield(P0, "Scute Swarm");
    if clone {
        let original = swarm;
        swarm = crate::r_p023_common::clone_of(&mut t, P0, original);
        crate::r_p198_common::exile(&mut t, original);
    }
    if dress {
        dress_up(&mut t, swarm);
        damage(&mut t, swarm, obj(swarm), 1);
        t.g.recompute();
        assert_eq!(t.pt(swarm), (6, 6));
    }
    t.lands(P0, "Forest", lands);
    let before = t.g.battlefield.clone();
    t.set_step(P0, Step::PrecombatMain);
    let f = t.hand(P0, "Forest");
    t.play_land(P0, f).expect("land");
    t.resolve_all();
    let toks = new_tokens(&t, P0, &before);
    (t, swarm, toks)
}

#[test]
fn scute_swarm_token_copies_have_the_ability_and_only_the_copiable_values() {
    cr!("707.2", "707.5", "707.3");
    ruling!(
        "Scute Swarm",
        "The token copy will have Scute Swarm's ability. It will also be able to create copies of itself."
    );
    ruling!(
        "Scute Swarm",
        "The token copy won't copy counters or damage marked on Scute Swarm, nor will it copy other effects that have changed Scute Swarm's power, toughness, types, color, and so on. Normally, this means the token will simply be a Scute Swarm, but if any copy effects have affected the original Scute Swarm, the token will take those into account."
    );
    // Five lands: an Insect; six (five plus the one entering): a copy.
    let (t, _, toks) = landfall(4, false, false);
    assert_eq!(toks.len(), 1);
    assert_eq!(name_of(&t, toks[0]), "Insect Token");
    let (mut t, _, toks) = landfall(5, false, true);
    assert_eq!(toks.len(), 1);
    assert_plain(&t, toks[0], "Scute Swarm", (1, 1));
    assert_eq!(t.obj_now(toks[0]).damage, 0, "it copied the damage");
    assert!(has_ability(&t, toks[0], LANDFALL));
    // The copy makes copies of itself (and the original of itself): two more.
    let before = t.g.battlefield.clone();
    let f = t.hand(P0, "Forest");
    t.g.players[0].lands_played_this_turn = 0;
    t.play_land(P0, f).expect("land");
    t.resolve_all();
    let more = new_tokens(&t, P0, &before);
    assert_eq!(more.len(), 2);
    assert!(more.iter().all(|k| name_of(&t, *k) == "Scute Swarm"));
    // A Clone copying Scute Swarm: its token is a Scute Swarm, not a Clone.
    let (t, _, toks) = landfall(5, true, false);
    assert_eq!(toks.len(), 1);
    assert_plain(&t, toks[0], "Scute Swarm", (1, 1));
    assert!(has_ability(&t, toks[0], LANDFALL));
}

#[test]
fn anax_power_is_defined_in_all_zones_and_a_token_copy_triggers_when_it_dies() {
    cr!("604.3", "707.2", "603.2", "603.10a");
    ruling!(
        "Anax, Hardened in the Forge",
        "The ability that defines Anax's power works in all zones, not just the battlefield."
    );
    ruling!(
        "Anax, Hardened in the Forge",
        "If you create a token that's a copy of Anax, its last ability triggers when it dies."
    );
    supported("Anax, Hardened in the Forge");
    // Devotion to red 3 (Gray Ogre {2}{R}, Hill Giant {3}{R}, Mogg Fanatic {R}): Anax's
    // power is 3 in the graveyard and in the hand too.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Gray Ogre");
    t.battlefield(P0, "Hill Giant");
    t.battlefield(P0, "Mogg Fanatic");
    let in_gy = t.graveyard(P0, "Anax, Hardened in the Forge");
    let in_hand = t.hand(P0, "Anax, Hardened in the Forge");
    t.g.recompute();
    assert_eq!(t.pt(in_gy).0, 3);
    assert_eq!(t.pt(in_hand).0, 3);
    assert_eq!(t.zone(in_gy), Zone::Graveyard(P0));
    // A token copy of Anax dies (here to the legend rule, the original kept, CR 704.5j):
    // its own "Anax ... dies" ability triggers. Its power as it last existed was 5
    // (devotion: Anax and the copy, {1}{R}{R} each, plus Mogg Fanatic), so two Satyrs.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Mogg Fanatic");
    let anax = t.battlefield(P0, "Anax, Hardened in the Forge");
    let before = t.g.battlefield.clone();
    crate::r_s17_common::token_copy(&mut t, P0, anax);
    assert!(t.on_battlefield(anax));
    assert!(new_tokens(&t, P0, &before).is_empty(), "the copy died to the legend rule");
    t.resolve_all();
    assert_eq!(with_subtype(&t, P0, "Satyr").len(), 2);
}
