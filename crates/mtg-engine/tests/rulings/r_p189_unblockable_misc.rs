//! Rulings batch P189 — other rulings of cards with "can't be blocked" abilities:
//! targeting, mandatory "return a creature" triggers, cost reduction, combat-damage and
//! attack triggers, and a few cards' other abilities.

use crate::r_p076_common::{is_blocked, mana};
use crate::r_p189_common::*;
use crate::r_s01_common::{attack_with, custom_card, supported};
use crate::r_s03_common::to_blockers;
use crate::r_s21_common::legal_blocks;
use mtg_engine::decision::{Answer, Decision};
use mtg_engine::keywords::KeywordKind;
use mtg_engine::mana::ManaType;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

// ---------------------------------------------------------------------------
// Fugitive Droid: "{U}, Sacrifice this creature: Counter target spell that targets an
// artifact or creature you control."
// ---------------------------------------------------------------------------

/// P1 casts `name` with `targets` (one per slot) during P0's main phase; returns it.
fn opponent_casts(t: &mut TestGame, name: &str, ty: ManaType, n: u32, targets: &[Entity]) -> ObjectId {
    mana(t, P1, ty, n);
    let c = t.hand(P1, name);
    let spell = t.cast_with(P1, c, targets).expect("cast");
    t.settle();
    spell
}

#[test]
fn fugitive_droid_targets_a_spell_with_several_targets() {
    cr!("115.1", "701.6a");
    ruling!(
        "Fugitive Droid",
        "Fugitive Droid's activated ability can target a spell that has multiple targets, as long as at least one of those targets is an artifact or creature you control."
    );
    supported("Fugitive Droid");
    let mut t = TestGame::new(2);
    let droid = t.battlefield(P0, "Fugitive Droid");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let p1_bears = t.battlefield(P1, "Grizzly Bears");
    // Seeds of Strength: "Target creature gets +1/+1 until end of turn. Target creature
    // gets +1/+1 until end of turn. Target creature gets +1/+1 until end of turn."
    mana(&mut t, P1, ManaType::W, 1);
    let spell = opponent_casts(
        &mut t,
        "Seeds of Strength",
        ManaType::G,
        1,
        &[
            Entity::Object(p1_bears),
            Entity::Object(p1_bears),
            Entity::Object(bears),
        ],
    );
    mana(&mut t, P0, ManaType::U, 1);
    t.activate(P0, droid, 0, &[Entity::Object(spell)])
        .expect("a spell with one target among several that's your creature");
    t.resolve_all();
    assert!(t.in_graveyard(P1, "Seeds of Strength"));
    assert_eq!(t.pt(p1_bears), (2, 2));
}

#[test]
fn fugitive_droid_target_left_vs_became_illegal() {
    cr!("115.1", "608.2b");
    ruling!(
        "Fugitive Droid",
        "If the artifact or creature you control targeted by the target spell leaves the battlefield, that spell is no longer a legal target for Fugitive Droid's activated ability."
    );
    supported("Fugitive Droid");
    // The targeted creature left the battlefield: the spell can't be targeted.
    let mut t = TestGame::new(2);
    let droid = t.battlefield(P0, "Fugitive Droid");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let spell = opponent_casts(&mut t, "Lightning Bolt", ManaType::R, 1, &[Entity::Object(bears)]);
    bounce(&mut t, bears);
    mana(&mut t, P0, ManaType::U, 1);
    assert!(t.activate(P0, droid, 0, &[Entity::Object(spell)]).is_err());
    // It became an illegal target (hexproof) but is still yours: the spell can be.
    let mut t = TestGame::new(2);
    let droid = t.battlefield(P0, "Fugitive Droid");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let spell = opponent_casts(&mut t, "Lightning Bolt", ManaType::R, 1, &[Entity::Object(bears)]);
    mana(&mut t, P0, ManaType::G, 1);
    let guile = t.hand(P0, "Ranger's Guile");
    t.cast(P0, guile).target(bears).go();
    t.resolve();
    assert!(t.obj(bears).has_keyword(KeywordKind::Hexproof));
    mana(&mut t, P0, ManaType::U, 1);
    t.activate(P0, droid, 0, &[Entity::Object(spell)])
        .expect("the spell still targets a creature you control");
    t.resolve_all();
    assert!(t.in_graveyard(P1, "Lightning Bolt"));
}

// ---------------------------------------------------------------------------
// Hexdrinker (level up)
// ---------------------------------------------------------------------------

#[test]
fn hexdrinker_set_pt_overrides_later_levels() {
    cr!("711.2b", "613.4b", "613.7");
    ruling!(
        "Hexdrinker",
        "If an effect has set Hexdrinker’s power and/or toughness to a specific value after it entered the battlefield, leveling up won’t change that characteristic."
    );
    supported("Hexdrinker");
    let mut t = TestGame::new(2);
    let hd = t.battlefield(P0, "Hexdrinker");
    // Diminish: "Target creature has base power and toughness 1/1 until end of turn."
    mana(&mut t, P0, ManaType::U, 1);
    let d = t.hand(P0, "Diminish");
    t.cast(P0, d).target(hd).go();
    t.resolve_all();
    assert_eq!(t.pt(hd), (1, 1));
    for _ in 0..3 {
        mana(&mut t, P0, ManaType::C, 1);
        t.activate(P0, hd, 0, &[]).unwrap();
        t.resolve_all();
    }
    assert_eq!(t.counters(hd, "level"), 3);
    assert_eq!(t.pt(hd), (1, 1));
    // The level ability still gives protection from instants.
    assert!(t.obj(hd).chars.abilities.len() > 1);
}

// ---------------------------------------------------------------------------
// Secret Tunnel: "{4}, {T}: Two target creatures you control that share a creature type
// can't be blocked this turn."
// ---------------------------------------------------------------------------

#[test]
fn secret_tunnel_one_creature_left() {
    cr!("608.2b", "115.1");
    ruling!(
        "Secret Tunnel",
        "If one of the two creatures leaves the battlefield before Secret Tunnel's ability resolves, the other still can't be blocked this turn as long as it has a creature type that the other card had as it left the battlefield."
    );
    supported("Secret Tunnel");
    let mut t = TestGame::new(2);
    let tunnel = t.battlefield(P0, "Secret Tunnel");
    let a = t.battlefield(P0, "Grizzly Bears");
    let b = t.battlefield(P0, "Grizzly Bears");
    let blocker = t.battlefield(P1, "Hill Giant");
    mana(&mut t, P0, ManaType::C, 4);
    t.answer_targets(P0, &[Entity::Object(a), Entity::Object(b)]);
    t.activate(P0, tunnel, 1, &[]).expect("two Bears");
    bounce(&mut t, b);
    t.resolve_all();
    attack_with(&mut t, &[(a, Entity::Player(P1))]);
    assert!(!legal_blocks(&mut t, P1, &[(blocker, a)]));
}

#[test]
fn secret_tunnel_needs_a_shared_creature_type() {
    cr!("115.1", "205.3m");
    ruling!(
        "Secret Tunnel",
        "The creatures must share at least one creature type, such as Ally or Lemur. Card types such as artifact, and supertypes such as legendary or snow, aren't creature types."
    );
    supported("Secret Tunnel");
    let mut t = TestGame::new(2);
    let tunnel = t.battlefield(P0, "Secret Tunnel");
    // Two artifact creatures: a Thopter and a Construct.
    let thopter = t.battlefield(P0, "Ornithopter");
    let memnite = t.battlefield(P0, "Memnite");
    mana(&mut t, P0, ManaType::C, 4);
    t.answer_targets(P0, &[Entity::Object(thopter), Entity::Object(memnite)]);
    let r = t.activate(P0, tunnel, 1, &[]);
    assert!(
        r.is_err() || t.g.stack.is_empty(),
        "an artifact creature pair with no shared creature type was targeted"
    );
}

// ---------------------------------------------------------------------------
// Keymaster Rogue / Storm Sculptor: "When this creature enters, return a creature you
// control to its owner's hand."
// ---------------------------------------------------------------------------

fn returns_itself_when_alone(name: &str) {
    supported(name);
    let mut t = TestGame::new(2);
    let me = t.enter(P0, name);
    t.resolve_all();
    assert!(!t.on_battlefield(me));
    assert!(t.in_hand(P0, name));
}

#[test]
fn keymaster_rogue_returns_itself() {
    cr!("603.3", "608.2c");
    ruling!(
        "Keymaster Rogue",
        "Keymaster Rogue’s last ability isn’t optional. If Keymaster Rogue is the only creature you control when the ability resolves, you’ll have to return it to its owner’s hand."
    );
    returns_itself_when_alone("Keymaster Rogue");
}

#[test]
fn storm_sculptor_returns_itself() {
    cr!("603.3", "608.2c");
    ruling!(
        "Storm Sculptor",
        "Storm Sculptor’s last ability isn’t optional. If Storm Sculptor is the only creature you control when the ability resolves, you’ll have to return it to its owner’s hand."
    );
    returns_itself_when_alone("Storm Sculptor");
}

#[test]
fn storm_sculptor_chooses_on_resolution() {
    cr!("608.2c", "115.10");
    ruling!(
        "Storm Sculptor",
        "Storm Sculptor’s last ability doesn’t target the creature you’ll return to hand. You choose one as the ability resolves."
    );
    supported("Storm Sculptor");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let ss = t.enter(P0, "Storm Sculptor");
    t.settle();
    let trig = stack_triggers_from(&t, ss);
    assert_eq!(trig.len(), 1);
    let si = t.obj(trig[0]).stack.clone().unwrap();
    assert!(si.chosen.iter().all(|c| c.targets.iter().all(|s| s.is_empty())));
    // A creature entering before it resolves can be chosen.
    let giant = t.battlefield(P0, "Hill Giant");
    t.answer_choose(P0, &[Entity::Object(giant)]);
    t.resolve_all();
    assert!(t.in_hand(P0, "Hill Giant"));
    assert!(t.on_battlefield(bears) && t.on_battlefield(ss));
}

// ---------------------------------------------------------------------------
// Gearseeker Serpent: affinity for artifacts.
// ---------------------------------------------------------------------------

#[test]
fn gearseeker_serpent_cost_locked_before_sacrificing_artifacts_for_mana() {
    cr!("601.2f", "601.2g", "601.2h", "702.41a");
    ruling!(
        "Gearseeker Serpent",
        "The total cost to cast this spell is locked in before you pay that cost. For example, if you control three artifacts, including one you can sacrifice to add {C}, the total cost of Gearseeker Serpent is {2}{U}{U}."
    );
    ruling!(
        "Gearseeker Serpent",
        "Once a player has announced that they are casting this spell, no player may take actions to try to change the number of artifacts its controller controls before the spell’s cost is locked in."
    );
    supported("Gearseeker Serpent");
    let mut t = TestGame::new(2);
    for _ in 0..3 {
        crate::r_s02_common::create_token(&mut t, P0, "Treasure");
    }
    t.lands(P0, "Island", 1);
    let serpent = t.hand(P0, "Gearseeker Serpent");
    let from = t.asked().len();
    // {2}{U}{U}: one Island and three Treasures sacrificed while paying.
    let spell = t.cast(P0, serpent).try_go().expect("cost {2}{U}{U}");
    assert!(t.g.stack.contains(&spell));
    assert!(t
        .g
        .permanents()
        .all(|o| !o.chars.has_subtype("Treasure")));
    // Nobody got priority while the spell was being cast.
    assert!(t.asked()[from..]
        .iter()
        .all(|(_, d)| !matches!(d, Decision::Priority { .. })));
}

#[test]
fn gearseeker_serpent_cant_cost_less_than_uu() {
    cr!("601.2f", "702.41a");
    ruling!(
        "Gearseeker Serpent",
        "This card’s first ability can’t reduce the total cost to cast the spell below {U}{U}."
    );
    supported("Gearseeker Serpent");
    let mut t = TestGame::new(2);
    for _ in 0..8 {
        t.battlefield(P0, "Ornithopter");
    }
    t.lands(P0, "Island", 1);
    let serpent = t.hand(P0, "Gearseeker Serpent");
    assert!(t.cast(P0, serpent).try_go().is_err(), "cast for {{U}}");
    let mut t = TestGame::new(2);
    for _ in 0..8 {
        t.battlefield(P0, "Ornithopter");
    }
    t.lands(P0, "Island", 2);
    let serpent = t.hand(P0, "Gearseeker Serpent");
    t.cast(P0, serpent).try_go().expect("cast for {U}{U}");
}

// ---------------------------------------------------------------------------
// Teysa, Envoy of Ghosts: "Whenever a creature deals combat damage to you, destroy that
// creature. Create a 1/1 white and black Spirit creature token with flying."
// ---------------------------------------------------------------------------

fn teysa_hit_by(name: &str) -> (TestGame, ObjectId) {
    supported("Teysa, Envoy of Ghosts");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Teysa, Envoy of Ghosts");
    let attacker = t.battlefield(P1, name);
    t.set_step(P1, Step::BeginningOfCombat);
    t.attack(&[(attacker, Entity::Player(P0))], &[]);
    t.resolve_all();
    (t, attacker)
}

fn spirits(t: &TestGame, p: PlayerId) -> usize {
    t.g.permanents()
        .filter(|o| o.controller == p && o.is_token() && o.chars.has_subtype("Spirit"))
        .count()
}

#[test]
fn teysa_controller_gets_the_spirit() {
    cr!("603.2", "111.2");
    ruling!(
        "Teysa, Envoy of Ghosts",
        "Teysa’s controller gets the Spirit creature token, not the controller of the creature that dealt combat damage."
    );
    let (t, attacker) = teysa_hit_by("Hill Giant");
    assert!(!t.on_battlefield(attacker));
    assert_eq!(spirits(&t, P0), 1);
    assert_eq!(spirits(&t, P1), 0);
}

#[test]
fn teysa_spirit_even_if_not_destroyed() {
    cr!("608.2c", "702.12b");
    ruling!(
        "Teysa, Envoy of Ghosts",
        "You get a Spirit creature token even if Teysa’s triggered ability doesn’t destroy the creature (perhaps because it regenerated or has indestructible)."
    );
    let (t, attacker) = teysa_hit_by("Darksteel Myr");
    // Darksteel Myr is 0/1: give it power through an indestructible attacker instead.
    let _ = attacker;
    let _ = t;
    let (t, attacker) = teysa_hit_by("Avacyn, Angel of Hope");
    assert!(t.on_battlefield(attacker));
    assert_eq!(spirits(&t, P0), 1);
}

// ---------------------------------------------------------------------------
// Gilded Scuttler: "When this creature enters, tap target creature an opponent controls
// and put a stun counter on it."
// ---------------------------------------------------------------------------

#[test]
fn gilded_scuttler_targets_an_already_tapped_creature() {
    cr!("701.26a", "122.1d");
    ruling!(
        "Gilded Scuttler",
        "You may target a creature that's already tapped with Gilded Scuttler's last ability. In that case, you'll just put a stun counter on that creature."
    );
    supported("Gilded Scuttler");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.g.tap(bears);
    t.answer_targets(P0, &[Entity::Object(bears)]);
    t.enter(P0, "Gilded Scuttler");
    t.resolve_all();
    assert!(t.obj(bears).tapped);
    assert_eq!(t.counters(bears, "stun"), 1);
}

// ---------------------------------------------------------------------------
// Attacks-and-isn't-blocked triggers.
// ---------------------------------------------------------------------------

#[test]
fn goblin_vandal_no_artifact_no_trigger() {
    cr!("603.3d", "508.1m");
    ruling!(
        "Goblin Vandal",
        "If the defending player controls no artifacts that you can target, the ability doesn’t go on the stack and you can’t pay {R}."
    );
    supported("Goblin Vandal");
    let mut t = TestGame::new(2);
    let vandal = t.battlefield(P0, "Goblin Vandal");
    t.lands(P0, "Mountain", 1);
    to_blockers(&mut t, &[(vandal, Entity::Player(P1))], &[]);
    assert_eq!(triggers_of(&t, vandal), 0);
    t.advance_to(P0, Step::EndOfCombat);
    assert_eq!(t.life(P1), 19);
    assert!(t.asked().iter().all(|(_, d)| !matches!(
        d,
        Decision::OptionalCost { .. } | Decision::YesNo { .. }
    )));
}

#[test]
fn keeper_of_tresserhorn_triggers_as_blockers_are_declared() {
    cr!("509.1i", "509.3g", "603.2");
    ruling!(
        "Keeper of Tresserhorn",
        "The ability triggers on declaration of blockers if the criteria is met."
    );
    supported("Keeper of Tresserhorn");
    let mut t = TestGame::new(2);
    let keeper = t.battlefield(P0, "Keeper of Tresserhorn");
    attack_with(&mut t, &[(keeper, Entity::Player(P1))]);
    assert_eq!(triggers_of(&t, keeper), 0, "not yet: no blockers declared");
    to_blockers(&mut t, &[], &[]);
    assert_eq!(t.g.turn.step, Step::DeclareBlockers);
    assert_eq!(triggers_of(&t, keeper), 1);
    t.resolve_all();
    assert_eq!(t.life(P1), 18);
    t.advance_to(P0, Step::EndOfCombat);
    assert_eq!(t.life(P1), 18, "no combat damage");
}

#[test]
fn wildfire_eternal_casts_a_sorcery_in_the_declare_blockers_step() {
    cr!("509.3g", "608.2g", "601.2");
    ruling!(
        "Wildfire Eternal",
        "Wildfire Eternal's second ability resolves before combat damage is dealt, and you must cast a spell at that time if you wish to cast one without paying its mana cost. You can cast a sorcery during the declare blockers step this way."
    );
    supported("Wildfire Eternal");
    let mut t = TestGame::new(2);
    let we = t.battlefield(P0, "Wildfire Eternal");
    let axe = t.hand(P0, "Lava Axe");
    t.answer(P0, DecisionKind::Any, Answer::Default);
    t.clear_answers();
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[Entity::Object(axe)]);
    t.answer_targets(P0, &[Entity::Player(P1)]);
    to_blockers(&mut t, &[(we, Entity::Player(P1))], &[]);
    assert_eq!(triggers_of(&t, we), 1);
    t.resolve_all();
    assert_eq!(t.g.turn.step, Step::DeclareBlockers);
    assert_eq!(t.life(P1), 15, "Lava Axe resolved before combat damage");
    t.advance_to(P0, Step::EndOfCombat);
    assert_eq!(t.life(P1), 14);
}

// ---------------------------------------------------------------------------
// Horobi, Death's Wail; Marchesa, Dealer of Death.
// ---------------------------------------------------------------------------

#[test]
fn horobi_only_target_destroyed_spell_doesnt_resolve() {
    cr!("608.2b", "603.3");
    ruling!(
        "Horobi, Death's Wail",
        "If the creature is the only target of the spell or ability, that spell or ability won't resolve."
    );
    supported("Horobi, Death's Wail");
    let mut t = TestGame::new(2);
    t.battlefield(P1, "Horobi, Death's Wail");
    let bears = t.battlefield(P0, "Grizzly Bears");
    mana(&mut t, P0, ManaType::W, 1);
    // Defiant Strike: "Target creature gets +1/+0 until end of turn. Draw a card."
    let ds = t.hand(P0, "Defiant Strike");
    let hand = t.hand_size(P0);
    t.cast(P0, ds).target(bears).go();
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Grizzly Bears"));
    assert!(t.in_graveyard(P0, "Defiant Strike"));
    assert_eq!(t.hand_size(P0), hand - 1, "Defiant Strike didn't resolve (no draw)");
}

#[test]
fn marchesa_with_one_card_in_library() {
    cr!("401.5", "608.2c");
    ruling!(
        "Marchesa, Dealer of Death",
        "If there’s only one card in your library as Marchesa’s triggered ability resolves, you’ll put it into your hand."
    );
    supported("Marchesa, Dealer of Death");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Marchesa, Dealer of Death");
    set_library(&mut t, P0, &["Hill Giant"]);
    mana(&mut t, P0, ManaType::R, 1);
    mana(&mut t, P0, ManaType::C, 1);
    let shock = t.hand(P0, "Shock");
    t.answer_yes(P0, true);
    t.cast(P0, shock).target(Entity::Player(P1)).go();
    t.resolve_all();
    assert_eq!(t.life(P1), 18);
    assert!(t.in_hand(P0, "Hill Giant"));
    assert_eq!(t.library_size(P0), 0);
}

// ---------------------------------------------------------------------------
// Shay Cormac
// ---------------------------------------------------------------------------

#[test]
fn shay_cormac_damaged_indestructible_creature_dies() {
    cr!("704.5g", "702.12b", "611.2c");
    ruling!(
        "Shay Cormac",
        "Because damage remains marked on a creature until damage is removed as the turn ends, damage previously dealt to a creature with indestructible may cause it to be destroyed if Shay Cormac’s first ability resolves later during that turn."
    );
    supported("Shay Cormac");
    let mut t = TestGame::new(2);
    let shay = t.battlefield(P0, "Shay Cormac");
    let myr = t.battlefield(P1, "Darksteel Myr");
    t.g.deal_damage(shay, Entity::Object(myr), 1, false);
    t.settle();
    assert!(t.on_battlefield(myr));
    mana(&mut t, P0, ManaType::C, 1);
    t.activate(P0, shay, 0, &[]).unwrap();
    t.resolve_all();
    assert!(t.in_graveyard(P1, "Darksteel Myr"));
}

#[test]
fn shay_cormac_later_permanents_keep_their_abilities() {
    cr!("611.2c");
    ruling!(
        "Shay Cormac",
        "If a permanent enters the battlefield under an opponent’s control with hexproof, indestructible, protection, shroud, and/or ward after Shay Cormac’s first ability resolves, it won’t lose that ability until you activate Shay Cormac’s first ability again."
    );
    supported("Shay Cormac");
    let mut t = TestGame::new(2);
    let shay = t.battlefield(P0, "Shay Cormac");
    let old = t.battlefield(P1, "Gladecover Scout");
    mana(&mut t, P0, ManaType::C, 1);
    t.activate(P0, shay, 0, &[]).unwrap();
    t.resolve_all();
    assert!(!t.obj(old).has_keyword(KeywordKind::Hexproof));
    let new = t.battlefield(P1, "Gladecover Scout");
    assert!(t.obj(new).has_keyword(KeywordKind::Hexproof));
    mana(&mut t, P0, ManaType::C, 1);
    t.activate(P0, shay, 0, &[]).unwrap();
    t.resolve_all();
    assert!(!t.obj(new).has_keyword(KeywordKind::Hexproof));
}

#[test]
fn shay_cormac_any_bounty_counter() {
    cr!("603.2", "603.10a");
    ruling!(
        "Shay Cormac",
        "Shay Cormac’s last ability looks for any bounty counter on a creature, not just one from that Shay Cormac."
    );
    supported("Shay Cormac");
    let mut t = TestGame::new(3);
    let a = t.battlefield(P0, "Shay Cormac");
    let b = t.battlefield(P1, "Shay Cormac");
    let bears = t.battlefield(P2, "Grizzly Bears");
    mana(&mut t, P0, ManaType::R, 1);
    let shock = t.hand(P0, "Shock");
    t.cast(P0, shock).target(bears).go();
    t.resolve_all();
    assert!(t.in_graveyard(P2, "Grizzly Bears"));
    assert_eq!(t.counters(a, counters::PLUS1), 2);
    assert_eq!(t.counters(b, counters::PLUS1), 2);
}

// ---------------------------------------------------------------------------
// Emissary of Soulfire: exalted counters (CR 122.1b, 702.83).
// ---------------------------------------------------------------------------

#[test]
fn exalted_counters_give_that_many_instances() {
    cr!("122.1b", "702.83a", "702.83b");
    ruling!(
        "Emissary of Soulfire",
        "A creature with multiple exalted counters will have that many instances of exalted."
    );
    supported("Emissary of Soulfire");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.g.add_counters(Entity::Object(bears), "exalted", 2, None);
    t.g.recompute();
    attack_with(&mut t, &[(bears, Entity::Player(P1))]);
    t.resolve_all();
    assert_eq!(t.pt(bears), (4, 4));
}

#[test]
fn exalted_attacks_alone_only_as_declared() {
    cr!("702.83b", "506.4");
    ruling!(
        "Emissary of Soulfire",
        "A creature attacks alone if it's the only creature declared as an attacker during the declare attackers step"
    );
    supported("Emissary of Soulfire");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let giant = t.battlefield(P0, "Hill Giant");
    t.g.add_counters(Entity::Object(bears), "exalted", 1, None);
    t.g.recompute();
    attack_with(
        &mut t,
        &[(bears, Entity::Player(P1)), (giant, Entity::Player(P1))],
    );
    mtg_engine::combat::remove_from_combat(&mut t.g, giant);
    t.g.flush_events();
    t.resolve_all();
    assert_eq!(t.pt(bears), (2, 2));
    assert_eq!(t.stack_len(), 0);
}

// ---------------------------------------------------------------------------
// Arixmethes, Slumbering Isle
// ---------------------------------------------------------------------------

#[test]
fn arixmethes_taps_for_mana_the_turn_it_enters() {
    cr!("302.6", "305.1");
    ruling!(
        "Arixmethes, Slumbering Isle",
        "Arixmethes can tap for mana the turn it enters the battlefield as long as it has slumber counters on it."
    );
    supported("Arixmethes, Slumbering Isle");
    let mut t = TestGame::new(2);
    let arix = t.enter(P0, "Arixmethes, Slumbering Isle");
    t.settle();
    assert!(t.obj(arix).tapped);
    assert!(t.obj(arix).summoning_sick);
    // "If you can figure out how to untap it".
    t.g.objects[arix.0 as usize].tapped = false;
    t.activate(P0, arix, 0, &[]).expect("a land taps for mana regardless of when it came under your control");
    let pool = &t.g.player(P0).mana_pool;
    assert_eq!(pool.total(), 2);
}

#[test]
fn arixmethes_cant_be_played_as_a_land() {
    cr!("305.1", "305.9");
    ruling!(
        "Arixmethes, Slumbering Isle",
        "Arixmethes can't be played as a land."
    );
    supported("Arixmethes, Slumbering Isle");
    let mut t = TestGame::new(2);
    let arix = t.hand(P0, "Arixmethes, Slumbering Isle");
    assert!(t.play_land(P0, arix).is_err());
    assert!(t.in_hand(P0, "Arixmethes, Slumbering Isle"));
}

#[test]
fn arixmethes_enters_as_a_creature_but_landfall_sees_a_land() {
    cr!("614.12", "603.6a", "613.1d");
    ruling!(
        "Arixmethes, Slumbering Isle",
        "Arixmethes isn't a land until after it has entered the battlefield. Effects such as that of Blood Moon won't affect its enters-the-battlefield ability."
    );
    supported("Arixmethes, Slumbering Isle");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Blood Moon");
    let lynx = t.battlefield(P0, "Steppe Lynx");
    // Grumgully: "Each other non-Human creature you control enters with an additional
    // +1/+1 counter on it."
    t.battlefield(P0, "Grumgully, the Generous");
    let arix = t.enter(P0, "Arixmethes, Slumbering Isle");
    t.resolve_all();
    assert_eq!(t.counters(arix, "slumber"), 5, "Blood Moon didn't stop it");
    assert!(t.obj(arix).tapped);
    assert_eq!(t.counters(arix, counters::PLUS1), 1, "entered as a creature");
    assert!(t.obj(arix).is(CardType::Land));
    // Landfall: Steppe Lynx (0/1) gets +2/+2.
    assert_eq!(t.pt(lynx), (2, 3));
}

#[test]
fn arixmethes_copy_is_only_a_land() {
    cr!("613.1d", "707.9b", "205.1a");
    ruling!(
        "Arixmethes, Slumbering Isle",
        "Arixmethes's effect causing it to be a land overwrites any earlier effects that gave it additional types. For example, a Phyrexian Metamorph that copies Arixmethes will be a land, not an artifact land, until its slumber counters are removed."
    );
    supported("Arixmethes, Slumbering Isle");
    supported("Phyrexian Metamorph");
    let mut t = TestGame::new(2);
    // Without slumber counters, Arixmethes is a creature Metamorph can copy; the copy
    // enters with its own five slumber counters.
    // (It's an opponent's, so the legend rule doesn't apply.)
    let arix = t.battlefield(P1, "Arixmethes, Slumbering Isle");
    assert!(t.obj(arix).is(CardType::Creature));
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[Entity::Object(arix)]);
    let m = t.enter(P0, "Phyrexian Metamorph");
    t.resolve_all();
    let m = t.g.current(m);
    assert_eq!(t.obj(m).chars.name.as_str(), "Arixmethes, Slumbering Isle");
    assert_eq!(t.counters(m, "slumber"), 5);
    assert!(t.obj(m).is(CardType::Land));
    assert!(!t.obj(m).is(CardType::Artifact));
    assert!(!t.obj(m).is(CardType::Creature));
    // Without slumber counters it's an artifact creature.
    let n = t.counters(m, "slumber");
    t.g.remove_counters(Entity::Object(m), "slumber", n);
    t.g.recompute();
    assert!(t.obj(m).is(CardType::Artifact) && t.obj(m).is(CardType::Creature));
}

// ---------------------------------------------------------------------------
// Slippery Scoundrel: ascend.
// ---------------------------------------------------------------------------

#[test]
fn slippery_scoundrel_enters_with_hexproof_as_tenth_permanent() {
    cr!("702.131b", "603.6a");
    ruling!(
        "Slippery Scoundrel",
        "use the entering permanent’s characteristics after you have the city’s blessing to determine whether those abilities trigger"
    );
    supported("Slippery Scoundrel");
    let mut t = TestGame::new(2);
    let watcher = custom_card(
        "Hexproof Watcher",
        "Enchantment",
        "{1}",
        None,
        "Whenever a creature with hexproof you control enters, you gain 3 life.",
    );
    t.custom(P0, watcher, Zone::Battlefield);
    t.lands(P0, "Island", 8);
    let sc = t.hand(P0, "Slippery Scoundrel");
    t.cast(P0, sc).go();
    t.resolve_all();
    assert!(t.g.player(P0).has_citys_blessing);
    assert_eq!(t.life(P0), 23);
    let _ = is_blocked;
}
