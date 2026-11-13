//! Rulings batch P202 — banding (CR 702.22), bargain (702.166), behold (701.4), bestow
//! (702.103), blight (701.68), blitz (702.152) and bloodrush (an ability word).

use crate::r_s01_common::*;
use crate::r_s02_common::*;
use mtg_engine::decision::Answer;
use mtg_engine::keywords::KeywordKind;
use mtg_engine::object::{CastMethod, Zone};
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

const FLASHBACK: CastMethod = CastMethod::Keyword(KeywordKind::Flashback);

fn band_of(t: &TestGame, a: ObjectId) -> Option<u32> {
    t.g.combat
        .as_ref()
        .and_then(|c| c.attacker(a))
        .and_then(|x| x.band)
}

fn declare_blocks(t: &mut TestGame, dp: PlayerId, blocks: &[(ObjectId, ObjectId)]) {
    use mtg_engine::turn::Stage;
    t.answer(
        dp,
        DecisionKind::Blockers,
        Answer::Blockers(blocks.to_vec()),
    );
    let ap = t.g.turn.active;
    let ok = t.g.run_until(10_000, |g| {
        g.turn.step == Step::DeclareBlockers
            && g.turn.stage == Stage::Priority
            && g.turn.priority == Some(ap)
    });
    assert!(ok, "blockers not declared");
    t.settle();
}

#[test]
fn chatzuk_bands_and_its_band_is_blocked_as_a_unit() {
    cr!("702.22a", "702.22c", "702.22h", "702.22k");
    ruling!(
        "Chatzuk, Mighty Guitarist",
        "If a creature with banding attacks, it can team up with any number of other attacking creatures with banding (and up to one nonbanding creature) and attack as a unit called a “band.” The band can be blocked by any creature that could block a single creature in the band. Blocking any creature in a band blocks the entire band. If a creature with banding is blocked, the attacking player chooses how the blockers’ damage is assigned."
    );
    // (Chatzuk's banding compiles; its other two abilities don't matter here.)
    let mut t = TestGame::new(2);
    let chatzuk = t.battlefield(P0, "Chatzuk, Mighty Guitarist");
    let hero = t.battlefield(P0, "Benalish Hero");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let maw = t.battlefield(P1, "Colossal Dreadmaw");
    let p1 = Entity::Player(P1);
    // Chatzuk bands with another banding creature and one nonbanding creature.
    t.answer_choose(P0, &[Entity::Object(hero), Entity::Object(bears)]);
    attack_with(&mut t, &[(chatzuk, p1), (hero, p1), (bears, p1)]);
    let band = band_of(&t, chatzuk);
    assert!(band.is_some());
    assert_eq!(band_of(&t, hero), band);
    assert_eq!(band_of(&t, bears), band);
    // A second nonbanding creature can't join (the band is rejected).
    let mut t2 = TestGame::new(2);
    let c2 = t2.battlefield(P0, "Chatzuk, Mighty Guitarist");
    let b2 = t2.battlefield(P0, "Grizzly Bears");
    let g2 = t2.battlefield(P0, "Hill Giant");
    t2.answer_choose(P0, &[Entity::Object(b2), Entity::Object(g2)]);
    attack_with(&mut t2, &[(c2, p1), (b2, p1), (g2, p1)]);
    assert_eq!(band_of(&t2, c2), None);

    // The Dreadmaw blocks the bears: the whole band is blocked, and P0 (the attacking
    // player) assigns the Dreadmaw's 6 damage — all to the Hero.
    declare_blocks(&mut t, P1, &[(maw, bears)]);
    let blocking = t.g.combat.as_ref().unwrap().blocking(maw);
    assert_eq!(blocking.len(), 3);
    let amounts: Vec<i64> = blocking
        .iter()
        .map(|a| if *a == hero { 6 } else { 0 })
        .collect();
    t.answer(P0, DecisionKind::Damage, Answer::Numbers(amounts));
    t.advance_to(P0, Step::EndOfCombat);
    assert!(!t.on_battlefield(hero));
    assert!(t.on_battlefield(chatzuk) && t.on_battlefield(bears));
    assert_eq!(t.life(P1), 20);
}

#[test]
fn bargained_brave_the_wilds_with_an_illegal_land_target_doesnt_resolve() {
    cr!("702.166b", "608.2b");
    ruling!(
        "Brave the Wilds",
        "If you bargained Brave the Wilds and the target land is an illegal target by the time it tries to resolve, the spell won't resolve. You won't search for a basic land card, and you won't shuffle."
    );
    supported("Brave the Wilds");
    let mut t = TestGame::new(2);
    let stone = t.battlefield(P0, "Mind Stone");
    let land = t.battlefield(P0, "Plains");
    stack_library(&mut t, P0, &["Forest", "Grizzly Bears", "Island"]);
    let library: Vec<ObjectId> = t.g.player(P0).library.clone();
    t.lands(P0, "Forest", 1);
    let c = t.hand(P0, "Brave the Wilds");
    t.answer(P0, DecisionKind::OptionalCost, Answer::Bool(true));
    t.answer_choose(P0, &[Entity::Object(stone)]);
    t.cast(P0, c).target(land).go();
    assert!(t.in_graveyard(P0, "Mind Stone"));
    let hand = t.hand_size(P0);
    destroy(&mut t, land);
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Brave the Wilds"));
    assert_eq!(t.hand_size(P0), hand);
    // Not searched, not shuffled: the library is in the same order.
    assert_eq!(t.g.player(P0).library, library);
}

#[test]
fn kindle_the_inner_flame_beholds_any_mix_of_three_distinct_elementals() {
    cr!("701.4a", "702.34a", "601.2h");
    ruling!(
        "Kindle the Inner Flame",
        "To behold three Elementals, you can reveal three Elemental cards from your hand, choose three Elementals you control on the battlefield, or reveal and choose any combination that adds up to three. You can't behold the same object more than once to pay this cost."
    );
    supported("Kindle the Inner Flame");
    // One in hand and two on the battlefield.
    let mut t = TestGame::new(2);
    let in_hand = t.hand(P0, "Air Elemental");
    let e1 = t.battlefield(P0, "Air Elemental");
    let e2 = t.battlefield(P0, "Air Elemental");
    t.lands(P0, "Mountain", 2);
    let kindle = t.graveyard(P0, "Kindle the Inner Flame");
    assert!(can_cast(&mut t, P0, kindle, FLASHBACK));
    t.answer_choose(
        P0,
        &[
            Entity::Object(in_hand),
            Entity::Object(e1),
            Entity::Object(e2),
        ],
    );
    t.cast(P0, kindle).method(FLASHBACK).target(e1).go();
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Air Elemental").len(), 3);
    assert!(t.in_exile("Kindle the Inner Flame"));
    // The card in hand was revealed and stays in hand.
    assert_eq!(t.zone(in_hand), Zone::Hand(P0));

    // Three in hand.
    let mut t = TestGame::new(2);
    let target = t.battlefield(P0, "Grizzly Bears");
    let hs: Vec<Entity> = (0..3)
        .map(|_| Entity::Object(t.hand(P0, "Air Elemental")))
        .collect();
    t.lands(P0, "Mountain", 2);
    let kindle = t.graveyard(P0, "Kindle the Inner Flame");
    assert!(can_cast(&mut t, P0, kindle, FLASHBACK));
    t.answer_choose(P0, &hs);
    t.cast(P0, kindle).method(FLASHBACK).target(target).go();
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Grizzly Bears").len(), 2);

    // Only two Elementals: the same one can't be beheld twice, so it can't be cast.
    let mut t = TestGame::new(2);
    t.hand(P0, "Air Elemental");
    t.battlefield(P0, "Air Elemental");
    t.lands(P0, "Mountain", 2);
    let kindle = t.graveyard(P0, "Kindle the Inner Flame");
    assert!(!can_cast(&mut t, P0, kindle, FLASHBACK));
}

#[test]
fn hypnotic_siren_bestowed_triggers_the_stolen_creatures_constellation() {
    cr!("702.103b", "603.6a", "603.3a", "613.1b");
    ruling!(
        "Hypnotic Siren",
        "If you cast Hypnotic Siren for its bestow cost, and you gain control of a creature with a constellation ability, that ability will trigger when Hypnotic Siren enters the battlefield. You'll control that triggered ability."
    );
    supported("Hypnotic Siren");
    supported("Eidolon of Blossoms");
    let mut t = TestGame::new(2);
    // Eidolon of Blossoms: "Constellation — Whenever this creature or another enchantment
    // you control enters, draw a card."
    let eidolon = t.battlefield(P1, "Eidolon of Blossoms");
    t.lands(P0, "Island", 7);
    let siren = t.hand(P0, "Hypnotic Siren");
    t.cast(P0, siren)
        .method(CastMethod::Keyword(KeywordKind::Bestow))
        .target(eidolon)
        .go();
    let (h0, h1) = (t.hand_size(P0), t.hand_size(P1));
    t.resolve();
    assert_eq!(t.obj_now(eidolon).controller, P0);
    assert_eq!(triggers_on_stack(&t, "another enchantment you control enters"), 1);
    let trig = *t.g.stack.last().unwrap();
    assert_eq!(t.obj(trig).controller, P0);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), h0 + 1);
    assert_eq!(t.hand_size(P1), h1);
}

#[test]
fn grub_blighting_to_zero_toughness_kills_only_after_the_ability_resolves() {
    cr!("701.68a", "704.3", "608.2");
    ruling!(
        "Grub, Storied Matriarch // Grub, Notorious Auntie",
        "If a creature's toughness is reduced to 0 while blighting during the resolution of Grub, Notorious Auntie's second ability, that creature won't die until after the ability finishes resolving. The same is true if the -1/-1 counter would cause damage already marked on that creature to become lethal."
    );
    supported("Grub, Storied Matriarch");
    for damaged in [false, true] {
        let mut t = TestGame::new(2);
        let grub = t.battlefield(P0, "Grub, Storied Matriarch");
        mtg_engine::dfc::transform(&mut t.g, grub);
        t.g.recompute();
        let victim = if damaged {
            let giant = t.battlefield(P0, "Hill Giant");
            t.g.objects[giant.0 as usize].damage = 2;
            giant
        } else {
            t.battlefield(P0, "Llanowar Elves")
        };
        t.answer_yes(P0, true);
        t.answer_choose(P0, &[Entity::Object(victim)]);
        attack_with(&mut t, &[(grub, Entity::Player(P1))]);
        assert_eq!(triggers_on_stack(&t, "blight 1"), 1);
        // Resolve the trigger without checking state-based actions afterwards.
        t.g.resolve_top();
        t.g.recompute();
        assert!(t.on_battlefield(victim));
        assert_eq!(t.counters(victim, counters::MINUS1), 1);
        let copies: Vec<ObjectId> = t
            .g
            .permanents()
            .filter(|o| o.is_token() && o.controller == P0)
            .map(|o| o.id)
            .collect();
        assert_eq!(copies.len(), 1);
        // Now it dies.
        t.settle();
        assert!(!t.on_battlefield(victim));
        assert!(t.on_battlefield(copies[0]));
    }
}

#[test]
fn warren_torchmasters_reflexive_trigger_targets_as_it_goes_on_the_stack() {
    cr!("603.12", "701.68a", "115.1");
    ruling!(
        "Warren Torchmaster",
        "You don't choose a target for Warren Torchmaster's ability at the time it triggers. Rather, a second \"reflexive\" ability triggers when you blight 1 this way. You choose a target for that ability as it goes on the stack. Each player may respond to this triggered ability as normal."
    );
    supported("Warren Torchmaster");
    let mut t = TestGame::new(2);
    let torch = t.battlefield(P0, "Warren Torchmaster");
    let giant = t.battlefield_sick(P0, "Hill Giant");
    let from = t.asked().len();
    t.advance_to(P0, Step::BeginningOfCombat);
    t.settle();
    // The trigger is on the stack; no target was chosen.
    assert_eq!(triggers_on_stack(&t, "you may blight 1"), 1);
    assert!(target_candidates(&t, P0, from).is_empty());
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[Entity::Object(torch)]);
    t.answer_targets(P0, &[Entity::Object(giant)]);
    t.resolve();
    // The reflexive trigger went on the stack with its target; P1 may respond.
    assert_eq!(t.counters(torch, counters::MINUS1), 1);
    assert_eq!(t.stack_len(), 1);
    assert_eq!(target_candidates(&t, P0, from).len(), 1);
    assert!(!t.obj_now(giant).chars.has_keyword(KeywordKind::Haste));
    // P1 responds by destroying the target: the reflexive ability does nothing.
    destroy(&mut t, giant);
    t.resolve_all();
    assert!(t.stack_len() == 0);
}

#[test]
fn star_athletes_blitz_draw_triggers_whenever_it_dies() {
    cr!("702.152a");
    ruling!(
        "Star Athlete",
        "If the creature was cast using its blitz ability, the triggered ability that lets its controller draw a card triggers when it dies for any reason, not just when you sacrifice it during the end step."
    );
    // (Star Athlete's blitz compiles; its attack trigger doesn't matter here.)
    let mut t = TestGame::new(2);
    t.lands(P0, "Mountain", 4);
    let c = t.hand(P0, "Star Athlete");
    t.cast(P0, c)
        .method(CastMethod::Keyword(KeywordKind::Blitz))
        .go();
    t.resolve_all();
    assert!(t.obj_now(c).chars.has_keyword(KeywordKind::Haste));
    let hand = t.hand_size(P0);
    destroy(&mut t, c);
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Star Athlete"));
    assert_eq!(t.hand_size(P0), hand + 1);
}

#[test]
fn rubblehulks_bloodrush_counts_lands_as_it_resolves() {
    cr!("207.2c", "608.2h");
    ruling!(
        "Rubblehulk",
        "If you activate Rubblehulk's bloodrush ability, the value of X is the number of lands you control when that ability resolves."
    );
    supported("Rubblehulk");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.lands(P0, "Mountain", 2);
    t.lands(P0, "Forest", 1);
    let hulk = t.hand(P0, "Rubblehulk");
    attack_with(&mut t, &[(bears, Entity::Player(P1))]);
    t.activate(P0, hulk, 0, &[Entity::Object(bears)]).unwrap();
    assert!(t.in_graveyard(P0, "Rubblehulk"));
    // Two more lands arrive before the ability resolves: X = 5.
    t.lands(P0, "Forest", 2);
    t.resolve_all();
    assert_eq!(t.pt(bears), (7, 7));
}
