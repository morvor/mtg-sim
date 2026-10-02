//! Rulings batch P146 — "return a permanent you control to its owner's hand" engines
//! ("rescue"): returning a land as a cost (CR 601.2h, 602.2b), choices made as an ability
//! resolves (CR 608.2c), copies of permanent spells (CR 707.10, 111.1), putting lands
//! onto the battlefield without playing them (CR 305.4), and type-changing effects
//! (CR 305.7).

use crate::r_p146_common::*;
use crate::r_s32_common::{basic_now, options_offered, subtypes_now};
use mtg_engine::decision::{Answer, Decision};
use mtg_engine::keywords::KeywordKind;
use mtg_engine::object::{CastMethod, Zone};
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

fn foods(t: &TestGame, p: PlayerId) -> usize {
    t.g.permanents()
        .filter(|o| o.controller == p && o.chars.has_subtype("Food"))
        .count()
}

#[test]
fn drafna_copy_of_a_permanent_spell_isnt_created() {
    cr!("707.10", "707.10f", "111.1");
    ruling!(
        "Drafna, Founder of Lat-Nam",
        "A resolving copy of a permanent spell becomes a token, so the token isn't \"created.\""
    );
    supported("Drafna, Founder of Lat-Nam");
    supported("Peregrin Took");
    // "{3}, {T}: Copy target artifact spell you control. (The copy becomes a token.)"
    // Peregrin Took: "If one or more tokens would be created under your control, those
    // tokens plus an additional Food token are created instead."
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Peregrin Took");
    let d = t.battlefield(P0, "Drafna, Founder of Lat-Nam");
    t.lands(P0, "Wastes", 3);
    let o = t.hand(P0, "Ornithopter");
    let spell = t.cast(P0, o).go();
    t.activate(P0, d, 1, &[obj(spell)]).unwrap();
    t.resolve_all();
    let birds = t.named_on_battlefield("Ornithopter");
    assert_eq!(birds.len(), 2);
    assert_eq!(birds.iter().filter(|b| t.obj(**b).is_token()).count(), 1);
    // No token was created: no Food.
    assert_eq!(foods(&t, P0), 0);
}

#[test]
fn drafna_copy_of_a_prototyped_spell_is_prototyped() {
    cr!("718.3c", "707.10");
    ruling!(
        "Drafna, Founder of Lat-Nam",
        "If an effect copies a prototyped spell, that copy (as well as the token it becomes on the battlefield) will have the same characteristics as the prototyped spell."
    );
    // Blitz Automaton: {7} 6/4, "Prototype {2}{R} — 3/2".
    let mut t = TestGame::new(2);
    let d = t.battlefield(P0, "Drafna, Founder of Lat-Nam");
    t.lands(P0, "Mountain", 1);
    t.lands(P0, "Wastes", 5);
    let b = t.hand(P0, "Blitz Automaton");
    let spell = t
        .cast(P0, b)
        .method(CastMethod::Keyword(KeywordKind::Prototype))
        .go();
    t.activate(P0, d, 1, &[obj(spell)]).unwrap();
    // The copy on the stack is prototyped too.
    let copy = *t.g.stack.last().unwrap();
    assert_ne!(copy, spell);
    t.resolve();
    let copy = *t.g.stack.last().unwrap();
    assert_eq!(t.obj(copy).chars.power, Some(3));
    t.resolve_all();
    let all = t.named_on_battlefield("Blitz Automaton");
    assert_eq!(all.len(), 2);
    for a in all {
        assert_eq!(t.pt(a), (3, 2));
        assert_eq!(t.obj(a).chars.colors, ColorSet::single(Color::Red));
    }
}

#[test]
fn salvage_scuttler_with_no_artifacts_does_nothing() {
    cr!("608.2c", "609.3");
    ruling!(
        "Salvage Scuttler",
        "If you control no artifacts, you won’t return anything to your hand. There’s no penalty for being unable to do so."
    );
    supported("Salvage Scuttler");
    // "Whenever this creature attacks, return an artifact you control to its owner's hand."
    let mut t = TestGame::new(2);
    let s = t.battlefield(P0, "Salvage Scuttler");
    let land = t.battlefield(P0, "Forest");
    let hand = t.hand_size(P0);
    attack_with(&mut t, &[(s, Entity::Player(P1))]);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand);
    assert!(t.on_battlefield(s) && t.on_battlefield(land));
    block_and_finish(&mut t, P1, &[]);
    assert_eq!(t.life(P1), 16);
    // With an artifact, it's returned.
    let mut t = TestGame::new(2);
    let s = t.battlefield(P0, "Salvage Scuttler");
    let o = t.battlefield(P0, "Ornithopter");
    attack_with(&mut t, &[(s, Entity::Player(P1))]);
    t.resolve_all();
    assert_eq!(t.zone(o), Zone::Hand(P0));
}

#[test]
fn murasa_rootgrazer_putting_a_land_isnt_playing_it() {
    cr!("305.4", "305.2");
    ruling!(
        "Murasa Rootgrazer",
        "Murasa Rootgrazer’s middle ability doesn’t count as playing a land. It can put a land card onto the battlefield even if you’ve already played your land for the turn."
    );
    supported("Murasa Rootgrazer");
    // "{T}: You may put a basic land card from your hand onto the battlefield."
    let mut t = TestGame::new(2);
    let m = t.battlefield(P0, "Murasa Rootgrazer");
    let a = t.hand(P0, "Forest");
    t.play_land(P0, a).unwrap();
    let b = t.hand(P0, "Plains");
    yes(&mut t, P0);
    t.answer_choose(P0, &[obj(b)]);
    t.activate(P0, m, 0, &[]).unwrap();
    t.resolve_all();
    assert!(t.on_battlefield(b));
    assert_eq!(t.g.player(P0).lands_played_this_turn, 1);
    // Still one land play used: no other land can be played.
    let c = t.hand(P0, "Forest");
    assert!(!can_play_land(&mut t, P0, c));
}

#[test]
fn murasa_rootgrazer_stays_attacking_after_activating() {
    cr!("506.4", "702.20b", "510.1");
    ruling!(
        "Murasa Rootgrazer",
        "Once Murasa Rootgrazer has attacked, you can activate either of its activated abilities. Doing so doesn’t remove it from combat."
    );
    // Vigilance; "{T}: Return target basic land you control to its owner's hand."
    let mut t = TestGame::new(2);
    let m = t.battlefield(P0, "Murasa Rootgrazer");
    let f = t.battlefield(P0, "Forest");
    attack_with(&mut t, &[(m, Entity::Player(P1))]);
    t.activate(P0, m, 1, &[obj(f)]).unwrap();
    t.resolve_all();
    assert_eq!(t.zone(f), Zone::Hand(P0));
    assert!(t.obj(m).tapped);
    assert!(t.g.is_attacking(m));
    block_and_finish(&mut t, P1, &[]);
    assert_eq!(t.life(P1), 18);
}

/// Whether any player received priority between decision `from` and now.
fn priority_since(t: &TestGame, from: usize) -> bool {
    t.asked()[from..]
        .iter()
        .any(|(_, d)| matches!(d, Decision::Priority { .. }))
}

#[test]
fn mina_and_denn_the_land_is_returned_while_activating() {
    cr!("602.2b", "601.2h", "601.2i");
    ruling!(
        "Mina and Denn, Wildborn",
        "Once you announce that you’re activating the last ability, it’s too late for anyone to interrupt you by trying to remove the land you returned."
    );
    supported("Mina and Denn, Wildborn");
    // "{R}{G}, Return a land you control to its owner's hand: Target creature gains
    // trample until end of turn."
    let mut t = TestGame::new(2);
    let m = t.battlefield(P0, "Mina and Denn, Wildborn");
    t.lands(P0, "Mountain", 1);
    t.lands(P0, "Forest", 1);
    let extra = t.battlefield(P0, "Plains");
    t.answer_choose(P0, &[obj(extra)]);
    let from = n_asked(&t);
    t.activate(P0, m, 0, &[obj(m)]).unwrap();
    // Paid while activating, before anyone got priority.
    assert_eq!(t.zone(extra), Zone::Hand(P0));
    assert!(!priority_since(&t, from));
    t.resolve_all();
    assert!(t.obj(m).has_keyword(KeywordKind::Trample));
}

#[test]
fn moonbow_illusionist_changes_only_the_target_and_only_its_subtype() {
    cr!("305.7", "608.2c");
    ruling!("Moonbow Illusionist", "Only the targeted land is affected.");
    ruling!(
        "Moonbow Illusionist",
        "The land’s name as well as any supertype it might have (such as legendary) remains unchanged."
    );
    supported("Moonbow Illusionist");
    // "{2}, Return a land you control to its owner's hand: Target land becomes the basic
    // land type of your choice until end of turn."
    let mut t = TestGame::new(2);
    let m = t.battlefield(P0, "Moonbow Illusionist");
    t.lands(P0, "Wastes", 2);
    let returned = t.battlefield(P0, "Plains");
    let snow = t.battlefield(P1, "Snow-Covered Forest");
    let other = t.battlefield(P1, "Forest");
    t.answer_choose(P0, &[obj(returned)]);
    let from = n_asked(&t);
    t.activate(P0, m, 0, &[obj(snow)]).unwrap();
    // Island.
    let islands = || Answer::Index(1);
    t.answer(P0, DecisionKind::Option, islands());
    t.resolve_all();
    let opts = options_offered(&t, from);
    assert_eq!(opts.len(), 1);
    assert_eq!(opts[0][1], "Island");
    assert_eq!(subtypes_now(&t, snow), vec!["Island"]);
    let o = t.obj(snow);
    assert_eq!(o.chars.name, "Snow-Covered Forest");
    assert!(o.chars.supertypes.contains(Supertype::Snow));
    assert!(basic_now(&t, snow));
    // Only the target.
    assert_eq!(subtypes_now(&t, other), vec!["Forest"]);
}

#[test]
fn oboro_envoy_counts_the_returned_land() {
    cr!("601.2h", "608.2h");
    ruling!(
        "Oboro Envoy",
        "The X will usually include the land returned to pay the ability’s cost."
    );
    supported("Oboro Envoy");
    // "{2}, Return a land you control to its owner's hand: Target creature gets -X/-0
    // until end of turn, where X is the number of cards in your hand."
    let mut t = TestGame::new(2);
    let e = t.battlefield(P0, "Oboro Envoy");
    t.lands(P0, "Wastes", 2);
    let land = t.battlefield(P0, "Island");
    let giant = t.battlefield(P1, "Hill Giant");
    assert_eq!(t.hand_size(P0), 0);
    t.answer_choose(P0, &[obj(land)]);
    t.activate(P0, e, 0, &[obj(giant)]).unwrap();
    t.resolve_all();
    assert_eq!(t.pt(giant), (2, 3));
}

#[test]
fn zell_dincht_extra_land_is_cumulative() {
    cr!("305.2", "305.2a");
    ruling!(
        "Zell Dincht",
        "The effect of Zell Dincht's first ability is cumulative with similar effects."
    );
    supported("Zell Dincht");
    supported("Exploration");
    // "You may play an additional land on each of your turns."
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Zell Dincht");
    t.battlefield(P0, "Exploration");
    let mut played = 0;
    for _ in 0..5 {
        let land = t.hand(P0, "Forest");
        if t.play_land(P0, land).is_ok() {
            played += 1;
        }
    }
    assert_eq!(played, 3);
}

#[test]
fn floodbringer_can_return_the_targeted_land() {
    cr!("601.2c", "601.2h", "602.2b", "608.2b");
    ruling!(
        "Floodbringer",
        "The land Floodbringer returns to its owner’s hand can be the targeted land."
    );
    supported("Floodbringer");
    // "{2}, Return a land you control to its owner's hand: Tap target land." Targets are
    // chosen before the cost is paid.
    let mut t = TestGame::new(2);
    let f = t.battlefield(P0, "Floodbringer");
    t.lands(P0, "Wastes", 2);
    let land = t.battlefield(P0, "Island");
    t.answer_choose(P0, &[obj(land)]);
    t.activate(P0, f, 0, &[obj(land)]).unwrap();
    assert_eq!(t.zone(land), Zone::Hand(P0));
    assert_eq!(t.stack_len(), 1);
    // Its only target is gone: it doesn't resolve.
    t.resolve_all();
    assert_eq!(t.zone(land), Zone::Hand(P0));
}

#[test]
fn cache_raiders_chooses_the_permanent_on_resolution() {
    cr!("603.3d", "608.2c", "115.1");
    ruling!(
        "Cache Raiders",
        "This ability isn’t targeted. You choose a permanent to return when the ability resolves. No one will be able to respond to the choice."
    );
    supported("Cache Raiders");
    // "At the beginning of your upkeep, return a permanent you control to its owner's
    // hand."
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Cache Raiders");
    let land = t.battlefield(P0, "Forest");
    let from = n_asked(&t);
    into_upkeep(&mut t, P0);
    assert_eq!(t.stack_len(), 1);
    // No target was chosen as it was put on the stack.
    assert!(!t.asked()[from..]
        .iter()
        .any(|(_, d)| matches!(d, Decision::ChooseTargets { .. })));
    assert!(t
        .g
        .obj(t.g.stack[0])
        .stack
        .as_ref()
        .unwrap()
        .chosen
        .iter()
        .all(|c| c.targets.iter().all(|s| s.is_empty())));
    t.answer_choose(P0, &[obj(land)]);
    let from = n_asked(&t);
    t.g.resolve_top();
    assert!(!priority_since(&t, from));
    assert_eq!(t.zone(land), Zone::Hand(P0));
}

#[test]
fn esperzoa_returns_itself_if_its_the_only_artifact() {
    cr!("608.2c");
    ruling!(
        "Esperzoa",
        "When Esperzoa's ability resolves, if you control no other artifacts, you'll have to return Esperzoa itself."
    );
    supported("Esperzoa");
    // "At the beginning of your upkeep, return an artifact you control to its owner's
    // hand."
    let mut t = TestGame::new(2);
    let e = t.battlefield(P0, "Esperzoa");
    t.battlefield(P0, "Forest");
    into_upkeep(&mut t, P0);
    t.resolve_all();
    assert_eq!(t.zone(e), Zone::Hand(P0));
    // With another artifact, P0 may return that one instead.
    let mut t = TestGame::new(2);
    let e = t.battlefield(P0, "Esperzoa");
    let o = t.battlefield(P0, "Ornithopter");
    into_upkeep(&mut t, P0);
    t.answer_choose(P0, &[obj(o)]);
    t.resolve_all();
    assert!(t.on_battlefield(e));
    assert_eq!(t.zone(o), Zone::Hand(P0));
    let _ = Step::Upkeep;
}
