//! Item `rulings-residue-keywords` — triggered abilities and their timing: conditions
//! checked as an ability triggers or resolves (CR 603.4, 608.2h), abilities that need a
//! characteristic to keep triggering, sources that left (610.3c, 607.2a), illegal targets
//! (608.2b), sequential actions (701.16a), regeneration and its shields (701.19), scry by
//! several players (701.22, 101.4), and leaves-the-battlefield look-back (603.10a).

use crate::r_residue_keywords_common::*;
use mtg_engine::decision::{Answer, Decision};
use mtg_engine::game::{GameConfig, Variant};
use mtg_engine::keywords::KeywordKind;
use mtg_engine::object::{CastMethod, Zone};
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

/// The target creature loses all abilities until end of turn (as Turn to Frog would), as
/// an effect of P1's.
fn lose_abilities(t: &mut TestGame, id: ObjectId) {
    cast_new(t, P1, "Turn to Frog", &[obj(id)]);
    t.resolve();
}

#[test]
fn hooded_blightfang_needs_the_creature_to_have_deathtouch() {
    cr!("603.2", "702.2b", "613.1f");
    ruling!(
        "Hooded Blightfang",
        "Hooded Blightfang's triggered abilities trigger when it attacks and when it deals damage to a planeswalker, as long as it still has deathtouch."
    );
    supported("Hooded Blightfang");
    for frog in [false, true] {
        let mut t = TestGame::new(2);
        t.battlefield(P0, "Hooded Blightfang");
        let rats = t.battlefield(P0, "Typhoid Rats");
        let pw = t.battlefield(P1, "Garruk Relentless // Garruk, the Veil-Cursed");
        if frog {
            lose_abilities(&mut t, rats);
        }
        attack_with(&mut t, &[(rats, Entity::Object(pw))]);
        assert_eq!(t.stack_len(), if frog { 0 } else { 1 });
        t.resolve_all();
        block_and_finish(&mut t, P1, &[]);
        t.resolve_all();
        // Without deathtouch: no drain, and 1 damage leaves Garruk with 2 loyalty.
        assert_eq!(t.life(P1), if frog { 20 } else { 19 });
        assert_eq!(t.on_battlefield(pw), frog);
    }
}

#[test]
fn starving_revenant_surveils_before_drawing() {
    cr!("701.25a", "608.2c", "603.4");
    ruling!(
        "Starving Revenant",
        "For Starving Revenant's enters-the-battlefield ability, you put cards into your graveyard for surveil (if any) before drawing cards. If there are eight or more permanent cards in your graveyard by the time you draw cards, those draws will cause the descend 8 ability to trigger."
    );
    supported("Starving Revenant");
    let mut t = TestGame::new(2);
    for _ in 0..7 {
        t.graveyard(P0, "Forest");
    }
    let lib = stack_library(&mut t, P0, &["Grizzly Bears", "Hill Giant"]);
    // Surveil: the Bears to the graveyard (the eighth permanent card), the Giant on top.
    t.answer(
        P0,
        DecisionKind::Surveil,
        Answer::Split(vec![lib[1]], vec![lib[0]]),
    );
    t.answer_targets(P0, &[Entity::Player(P1)]);
    t.enter(P0, "Starving Revenant");
    t.resolve_all();
    assert_eq!(t.zone(lib[0]), Zone::Graveyard(P0));
    assert_eq!(t.zone(lib[1]), Zone::Hand(P0));
    // Drew one (lost 3), and descend 8 triggered for it (+1, and P1 lost 1).
    assert_eq!(t.life(P0), 18);
    assert_eq!(t.life(P1), 19);
}

#[test]
fn the_ancient_one_stays_in_combat_once_it_attacked() {
    cr!("506.4", "508.1c", "207.2c");
    ruling!(
        "The Ancient One",
        "Once The Ancient One has legally attacked or blocked, removing permanent cards from your graveyard so the descend 8 ability no longer applies won't remove The Ancient One from combat."
    );
    supported("The Ancient One");
    let mut t = TestGame::new(2);
    let a = t.battlefield(P0, "The Ancient One");
    let cards: Vec<ObjectId> = (0..8).map(|_| t.graveyard(P0, "Forest")).collect();
    attack_with(&mut t, &[(a, Entity::Player(P1))]);
    assert!(is_attacking(&t, a));
    for c in cards {
        move_to(&mut t, c, Zone::Exile);
    }
    assert!(is_attacking(&t, a));
    block_and_finish(&mut t, P1, &[]);
    assert_eq!(t.life(P1), 12);
}

#[test]
fn intellect_devourer_gone_before_its_ability_resolves_exiles_nothing() {
    cr!("610.3c", "610.3");
    ruling!(
        "Intellect Devourer",
        "If Intellect Devourer leaves the battlefield before the Devour Intellect ability resolves, opponents won't exile any cards from their hands."
    );
    supported("Intellect Devourer");
    let mut t = TestGame::new(2);
    t.hand(P1, "Grizzly Bears");
    let d = t.enter(P0, "Intellect Devourer");
    t.settle();
    assert_eq!(t.stack_len(), 1);
    move_to(&mut t, d, Zone::Hand(P0));
    t.resolve_all();
    assert_eq!(t.hand_size(P1), 1);
    assert!(!t.in_exile("Grizzly Bears"));
}

#[test]
fn slippery_bogbonder_moves_counters_onto_a_creature_with_hexproof() {
    cr!("702.11b", "115.1", "122.5");
    ruling!(
        "Slippery Bogbonder",
        "If Slippery Bogbonder's triggered ability puts a hexproof counter on a creature an opponent controls, you can still move counters onto that creature."
    );
    supported("Slippery Bogbonder");
    let mut t = TestGame::new(2);
    let mine = t.battlefield(P0, "Grizzly Bears");
    crate::r_p120_common::give_plus1(&mut t, mine, 2);
    let theirs = t.battlefield(P1, "Grizzly Bears");
    t.answer_targets(P0, &[obj(theirs)]);
    t.answer_choose(P0, &[obj(mine)]);
    t.answer(P0, DecisionKind::Number, Answer::Number(2));
    t.enter(P0, "Slippery Bogbonder");
    t.resolve_all();
    assert_eq!(t.counters(theirs, "hexproof"), 1);
    assert_eq!(p1(&t, theirs), 2, "{}", t.dump_log());
    assert_eq!(p1(&t, mine), 0);
}

#[test]
fn odric_counts_hexproof_from_a_color_as_hexproof_once() {
    cr!("702.11e", "702.11a");
    ruling!(
        "Odric, Blood-Cursed",
        "Variants of an ability are counted as that ability and are not counted multiple times. For example, if you control a creature with hexproof from blue and another creature with hexproof from black, you create a single Blood token."
    );
    supported("Odric, Blood-Cursed");
    let mut t = TestGame::new(2);
    // Hexproof from black, and hexproof from blue.
    t.battlefield(P0, "Garruk's Harbinger");
    t.battlefield(P0, "Sporeweb Weaver");
    t.enter(P0, "Odric, Blood-Cursed");
    t.resolve_all();
    // Hexproof (once) and reach (Sporeweb Weaver).
    assert_eq!(tokens_of(&t, P0, "Blood"), 2);
}

#[test]
fn sunder_the_gateway_doesnt_incubate_if_its_target_is_illegal() {
    cr!("608.2b", "701.53a");
    ruling!(
        "Sunder the Gateway",
        "For the first mode, if the nontoken artifact or enchantment is an illegal target at the time Sunder the Gateway tries to resolve, it won't resolve and none of its effects will happen. You won't incubate."
    );
    let mut t = TestGame::new(2);
    let mind_stone = t.battlefield(P1, "Mind Stone");
    lands_for_cost(&mut t, P0, "Sunder the Gateway");
    let s = t.hand(P0, "Sunder the Gateway");
    t.cast(P0, s).modes(&[0]).target(mind_stone).go();
    move_to(&mut t, mind_stone, Zone::Hand(P1));
    t.resolve_all();
    assert_eq!(tokens_of(&t, P0, "Incubator"), 0);
    assert!(t.in_graveyard(P0, "Sunder the Gateway"));
}

#[test]
fn lumarets_favor_infusion_checks_life_gained_as_it_resolves() {
    cr!("603.4", "608.2h", "707.10");
    ruling!(
        "Lumaret's Favor",
        "The infusion ability will trigger when you cast Lumaret's Favor regardless of whether you've gained life this turn. As that ability resolves, it will check to see if you've gained life this turn. If you have, Lumaret's Favor will be copied once."
    );
    supported("Lumaret's Favor");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    cast_new(&mut t, P0, "Lumaret's Favor", &[obj(bears)]);
    t.settle();
    // It triggered although no life was gained yet.
    assert_eq!(t.stack_len(), 2);
    crate::r_p120_common::gain_life(&mut t, P0, 1);
    t.answer_yes(P0, false);
    t.resolve_all();
    assert_eq!(pt_now(&mut t, bears), (6, 10));
}

#[test]
fn sharp_eyed_rookie_compares_as_each_ability_resolves() {
    cr!("603.4", "608.2h");
    ruling!(
        "Sharp-Eyed Rookie",
        "If multiple creatures enter the battlefield at the same time, Sharp-Eyed Rookie's triggered ability may trigger multiple times, although the stat comparison will take place each time one of those abilities tries to resolve. For example, if you control a 2/2 Sharp-Eyed Rookie and two 3/3 creatures enter the battlefield, Sharp-Eyed Rookie's triggered ability will trigger twice. The first ability will resolve, and you'll put a +1/+1 counter on Sharp-Eyed Rookie and investigate. When the second ability tries to resolve, neither the power nor the toughness of the new creature is greater than that of Sharp-Eyed Rookie, so that ability does nothing."
    );
    ruling!(
        "Sharp-Eyed Rookie",
        "When comparing the stats as Sharp-Eyed Rookie's ability resolves, it's possible that the stat that's greater changes from power to toughness or vice versa. If this happens, the ability will still resolve and you'll put a +1/+1 counter on Sharp-Eyed Rookie and investigate. For example, if you control a 2/2 Sharp-Eyed Rookie and a 1/3 creature enters the battlefield under your control, its toughness is greater, so Sharp-Eyed Rookie's triggered ability will trigger. In response, the 1/3 creature gets +2/-2. When Sharp-Eyed Rookie's ability tries to resolve, the other creature's power is greater, so you'll put a +1/+1 counter on Sharp-Eyed Rookie and investigate."
    );
    // Two 3/3s at once.
    let mut t = TestGame::new(2);
    let r = t.battlefield(P0, "Sharp-Eyed Rookie");
    enter_together(&mut t, &[(P0, "Hill Giant"), (P0, "Hill Giant")]);
    t.settle();
    assert_eq!(t.stack_len(), 2);
    t.resolve_all();
    assert_eq!(p1(&t, r), 1);
    assert_eq!(tokens_of(&t, P0, "Clue"), 1);
    // A 1/4 (toughness greater) that gets +2/-2 in response (power greater).
    let mut t = TestGame::new(2);
    let r = t.battlefield(P0, "Sharp-Eyed Rookie");
    let turtle = t.enter(P0, "Horned Turtle");
    t.settle();
    assert_eq!(t.stack_len(), 1);
    crate::r_p214_common::pump(&mut t, turtle, 2, -2);
    assert_eq!(pt_now(&mut t, turtle), (3, 2));
    t.resolve_all();
    assert_eq!(p1(&t, r), 1);
    assert_eq!(tokens_of(&t, P0, "Clue"), 1);
}

#[test]
fn lazav_doesnt_investigate_if_its_target_is_illegal() {
    cr!("608.2b");
    ruling!(
        "Lazav, Wearer of Faces",
        "If the target of Lazav, Wearer of Faces's first ability is illegal as the ability tries to resolve, it won't resolve and none of its effects will happen. You won't investigate."
    );
    supported("Lazav, Wearer of Faces");
    let mut t = TestGame::new(2);
    let l = t.battlefield(P0, "Lazav, Wearer of Faces");
    let card = t.graveyard(P1, "Grizzly Bears");
    t.answer_targets(P0, &[obj(card)]);
    attack_with(&mut t, &[(l, Entity::Player(P1))]);
    assert_eq!(t.stack_len(), 1);
    move_to(&mut t, card, Zone::Hand(P1));
    t.resolve_all();
    assert_eq!(tokens_of(&t, P0, "Clue"), 0);
}

#[test]
fn tamiyo_investigates_one_clue_at_a_time() {
    cr!("701.16a", "608.2c", "603.2c");
    interp!("sequential-investigations-are-separate-events");
    ruling!(
        "Tamiyo Meets the Story Circle",
        "If you're instructed to investigate multiple times, those actions are sequential, meaning you'll create that many Clue tokens one at a time."
    );
    supported("Tamiyo Meets the Story Circle");
    let mut t = TestGame::new(2);
    // Cloakwood Swarmkeeper: "Whenever one or more tokens you control enter, put a +1/+1
    // counter on this creature." Two Clues one at a time: it triggers twice.
    let sk = t.battlefield(P0, "Cloakwood Swarmkeeper");
    let saga = t.enter(P0, "Tamiyo Meets the Story Circle");
    t.resolve_all();
    let card = t.hand(P0, "Grizzly Bears");
    t.answer_choose(P0, &[obj(card)]);
    crate::r_s19_common::add_lore(&mut t, saga, 1);
    t.resolve_all();
    assert_eq!(tokens_of(&t, P0, "Clue"), 2);
    assert_eq!(p1(&t, sk), 2, "{}", t.dump_log());
}

#[test]
fn dihadas_ploy_counts_the_jump_start_discard() {
    cr!("702.133a", "701.9a", "601.2h");
    ruling!(
        "Dihada's Ploy",
        "Dihada's Ploy counts the total number of cards discarded, including the card you discard as part of its effect and any cards you discarded prior to its resolution this turn. If you cast it with jump-start, this includes the card you discarded to cast it."
    );
    supported("Dihada's Ploy");
    let mut t = TestGame::new(2);
    // One card discarded earlier this turn.
    let early = t.hand(P0, "Forest");
    t.g.discard(P0, early, None);
    t.settle();
    let ploy = t.graveyard(P0, "Dihada's Ploy");
    let cost = t.hand(P0, "Grizzly Bears");
    lands_for_cost(&mut t, P0, "Dihada's Ploy");
    t.answer_choose(P0, &[obj(cost)]);
    t.cast(P0, ploy)
        .method(CastMethod::Keyword(KeywordKind::JumpStart))
        .go();
    t.resolve_all();
    // Earlier discard + jump-start cost + the effect's discard.
    assert_eq!(t.life(P0), 23, "{}", t.dump_log());
    assert_eq!(t.zone(ploy), Zone::Exile);
}

// --- Regeneration (CR 701.19) ------------------------------------------------------------

/// P0's Spiny Starfish regenerates once now (from a {U} shield, destroyed); only the
/// shield ability resolves.
fn starfish_regenerates(t: &mut TestGame, s: ObjectId) {
    t.lands(P0, "Island", 1);
    t.activate(P0, s, 0, &[]).unwrap();
    t.resolve();
    destroy(t, s);
    assert!(t.on_battlefield(s));
}

#[test]
fn spiny_starfish_counts_regenerations_from_any_shield() {
    cr!("701.19a", "603.4");
    ruling!(
        "Spiny Starfish",
        "The second ability doesn’t care where the regeneration shields Spiny Starfish used that turn came from. If it regenerated due to a regeneration shield from Refresh, for example, the ability will still put a Starfish token onto the battlefield."
    );
    supported("Spiny Starfish");
    let mut t = TestGame::new(2);
    let s = t.battlefield(P0, "Spiny Starfish");
    cast_new(&mut t, P0, "Refresh", &[obj(s)]);
    t.resolve_all();
    destroy(&mut t, s);
    assert!(t.on_battlefield(s));
    t.advance_to(P0, Step::End);
    t.resolve_all();
    assert_eq!(tokens_of(&t, P0, "Starfish"), 1);
}

#[test]
fn spiny_starfish_counts_on_resolution_and_not_after() {
    cr!("603.4", "608.2h", "701.19a");
    ruling!(
        "Spiny Starfish",
        "The second ability has an “intervening ‘if’ clause,” so it won’t trigger at all unless Spiny Starfish had regenerated that turn. However, the number of tokens that are created isn’t determined until the ability resolves (in case it regenerates again before that time)."
    );
    ruling!(
        "Spiny Starfish",
        "If Spiny Starfish regenerates during the end step after its second ability has resolved, no tokens will be created as a result."
    );
    // Never regenerated: no trigger.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Spiny Starfish");
    t.advance_to(P0, Step::End);
    t.settle();
    assert_eq!(t.stack_len(), 0);
    // Regenerated once; again in response to the trigger: two tokens. Then again after it
    // resolved: none more.
    let mut t = TestGame::new(2);
    let s = t.battlefield(P0, "Spiny Starfish");
    starfish_regenerates(&mut t, s);
    t.advance_to(P0, Step::End);
    t.settle();
    assert_eq!(t.stack_len(), 1);
    starfish_regenerates(&mut t, s);
    t.resolve_all();
    assert_eq!(tokens_of(&t, P0, "Starfish"), 2);
    starfish_regenerates(&mut t, s);
    t.resolve_all();
    assert_eq!(tokens_of(&t, P0, "Starfish"), 2);
}

#[test]
fn debt_of_loyalty_needs_a_regeneration_this_turn() {
    cr!("701.19a", "701.19b", "514.2");
    ruling!(
        "Debt of Loyalty",
        "If you cast Debt of Loyalty but nothing happens to the targeted creature for the rest of the turn that would cause it to be destroyed, Debt of Loyalty has no visible effect. The regeneration shield wears off at the end of the turn. If the creature regenerates during a later turn, you won’t gain control of it."
    );
    supported("Debt of Loyalty");
    let mut t = TestGame::new(2);
    let boa = t.battlefield(P1, "River Boa");
    cast_new(&mut t, P0, "Debt of Loyalty", &[obj(boa)]);
    t.resolve_all();
    assert_eq!(controller(&mut t, boa), P1);
    t.advance_to(P1, Step::PrecombatMain);
    // A later turn: P1's own shield regenerates it; P0 doesn't gain control.
    t.lands(P1, "Forest", 1);
    t.activate(P1, boa, 0, &[]).unwrap();
    t.resolve_all();
    destroy(&mut t, boa);
    assert!(t.on_battlefield(boa));
    assert_eq!(controller(&mut t, boa), P1);
}

#[test]
fn debt_of_loyalty_on_your_own_creature() {
    cr!("701.19a", "108.4");
    ruling!(
        "Debt of Loyalty",
        "You may cast Debt of Loyalty targeting a creature you already control. If you do, and that creature would be destroyed later in the turn, it will regenerate and you’ll gain control of it (though gaining control of a creature you already control usually has no visible effect)."
    );
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    cast_new(&mut t, P0, "Debt of Loyalty", &[obj(bears)]);
    t.resolve_all();
    destroy(&mut t, bears);
    assert!(t.on_battlefield(bears));
    assert!(tapped(&t, bears));
    assert_eq!(controller(&mut t, bears), P0);
}

// --- Scry --------------------------------------------------------------------------------

#[test]
fn zaffai_uses_the_triggering_spells_mana_value() {
    cr!("608.2h", "701.22a");
    ruling!(
        "Zaffai, Thunder Conductor",
        "\"That spell's mana value\" refers to the spell that caused the ability to trigger, not the card you look at when you scry."
    );
    supported("Zaffai, Thunder Conductor");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Zaffai, Thunder Conductor");
    t.library_top(P0, "Colossal Dreadmaw");
    // Opt (mana value 1): the scried Dreadmaw (6) doesn't make an Elemental.
    cast_new(&mut t, P0, "Opt", &[]);
    t.resolve_all();
    assert_eq!(tokens_of(&t, P0, "Elemental"), 0);
    // Lava Axe (mana value 5) makes one.
    cast_new(&mut t, P0, "Lava Axe", &[Entity::Player(P1)]);
    t.resolve_all();
    assert_eq!(tokens_of(&t, P0, "Elemental"), 1);
}

#[test]
fn banes_contingency_only_counts_commanders_on_the_battlefield() {
    cr!("903.3", "608.2c");
    ruling!(
        "Bane's Contingency",
        "A \"commander you control\" means a permanent on the battlefield that is a commander and is under your control. It can be your commander or one owned by another player. It doesn't refer to your commander in any other zone. Notably, if a player uses Bane's Contingency to counter a spell that would counter their commander, they do not scry or draw a card."
    );
    supported("Bane's Contingency");
    let commander_game = || {
        TestGame::with_config(
            2,
            GameConfig {
                variant: Variant::Commander,
                ..Default::default()
            },
        )
    };
    let make_commander = |t: &mut TestGame, p: PlayerId, id: ObjectId| {
        t.g.objects[id.0 as usize].is_commander = true;
        let name = t.g.obj(id).chars.name.clone();
        t.g.players[p.idx()].commander_names.push(name);
        t.g.recompute();
    };
    // P1's commander, controlled by P0, targeted by P1's Lightning Bolt: draw.
    let mut t = commander_game();
    let c = t.battlefield(P0, "Hill Giant");
    make_commander(&mut t, P1, c);
    t.set_step(P1, Step::PrecombatMain);
    t.lands(P1, "Mountain", 1);
    let bolt = t.hand(P1, "Lightning Bolt");
    let spell = t.cast(P1, bolt).target(c).go();
    let before = t.hand_size(P0);
    cast_new(&mut t, P0, "Bane's Contingency", &[obj(spell)]);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), before + 1);
    // P0's commander spell targeted by P1's Cancel; P0 counters the Cancel: no draw.
    let mut t = commander_game();
    let card = t.hand(P0, "Hill Giant");
    make_commander(&mut t, P0, card);
    t.lands(P0, "Mountain", 4);
    let cmd = t.cast(P0, card).go();
    t.lands(P1, "Island", 3);
    let cancel = t.hand(P1, "Cancel");
    let counter = t.cast(P1, cancel).target(cmd).go();
    let before = t.hand_size(P0);
    cast_new(&mut t, P0, "Bane's Contingency", &[obj(counter)]);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), before);
    assert_eq!(t.named_on_battlefield("Hill Giant").len(), 1);
}

#[test]
fn kaya_opponents_choose_to_scry_in_turn_order() {
    cr!("701.22a", "101.4", "606.3");
    ruling!(
        "Kaya, Intangible Slayer",
        "For the middle loyalty ability, each opponent in turn order chooses whether or not to scry. Those who do (likely all of them) look at the top card of their library at the same time, then they decide in turn order where their card goes. Each opponent will know the choices of previous players in turn order before making their own choices."
    );
    supported("Kaya, Intangible Slayer");
    let mut t = TestGame::new(3);
    let k = t.battlefield(P0, "Kaya, Intangible Slayer");
    t.answer_yes(P1, true);
    t.answer_yes(P2, true);
    let from = t.asked().len();
    t.activate(P0, k, 1, &[]).unwrap();
    t.resolve_all();
    let order: Vec<(PlayerId, &'static str)> = asked_since(&t, from)
        .iter()
        .filter_map(|(p, d)| match d {
            Decision::YesNo { .. } => Some((*p, "may")),
            Decision::Scry { .. } => Some((*p, "scry")),
            _ => None,
        })
        .collect();
    assert_eq!(
        order,
        vec![(P1, "may"), (P2, "may"), (P1, "scry"), (P2, "scry")],
        "{}",
        t.dump_log()
    );
}

#[test]
fn taeko_scries_for_creatures_that_left_with_it_but_gets_no_counters() {
    cr!("603.10a", "603.6c", "701.22a");
    ruling!(
        "Taeko, the Patient Avalanche",
        "If Taeko leaves the battlefield or dies at the same time as one or more other creatures you control leave the battlefield without dying, its second ability triggers once for each of those other creatures. You will scry for each of those other creatures, but Taeko will not be on the battlefield to receive +1/+1 counters."
    );
    supported("Taeko, the Patient Avalanche");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Taeko, the Patient Avalanche");
    t.battlefield(P0, "Grizzly Bears");
    t.battlefield(P0, "Hill Giant");
    let from = t.asked().len();
    cast_new(&mut t, P0, "Evacuation", &[]);
    t.resolve();
    assert_eq!(triggers_on_stack(&t, "scry"), 2, "{}", t.dump_log());
    t.resolve_all();
    let scries = count_asked(&t, from, |d| matches!(d, Decision::Scry { .. }));
    assert_eq!(scries, 2);
    assert!(t.in_hand(P0, "Taeko, the Patient Avalanche"));
}

#[test]
fn lapis_orb_mana_triggers_once_per_mana_spent() {
    cr!("106.6", "603.2");
    ruling!(
        "Lapis Orb of Dragonkind",
        "If more than one {U} produced by a Lapis Orb of Dragonkind is spent to cast a single Dragon creature spell, the delayed triggered ability associated with each mana spent will trigger. You will scry 2 that many times."
    );
    supported("Lapis Orb of Dragonkind");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Lapis Orb of Dragonkind");
    t.battlefield(P0, "Lapis Orb of Dragonkind");
    t.lands(P0, "Mountain", 4);
    let from = t.asked().len();
    let d = t.hand(P0, "Shivan Dragon");
    t.cast(P0, d).go();
    t.settle();
    // The Dragon and the two Orbs' delayed triggers.
    assert_eq!(t.stack_len(), 3, "{}", t.dump_log());
    t.resolve_all();
    assert_eq!(
        count_asked(&t, from, |d| matches!(d, Decision::Scry { .. })),
        2
    );
}

#[test]
fn model_of_unity_scries_look_together_then_decide_in_turn_order() {
    cr!("701.22a", "101.4", "701.38a");
    ruling!(
        "Model of Unity",
        "If multiple players are instructed to scry at once, those players each look at the top cards of their library at the same time, then they choose in turn order whether to put those cards on the top or bottom of their library and in what order."
    );
    supported("Model of Unity");
    supported("Truth or Consequences");
    let mut t = TestGame::new(3);
    t.battlefield(P0, "Model of Unity");
    for p in [P0, P1, P2] {
        // Everyone votes truth; each may scry 2.
        t.answer(p, DecisionKind::Option, Answer::Index(0));
        t.answer_yes(p, true);
    }
    let from = t.asked().len();
    cast_new(&mut t, P0, "Truth or Consequences", &[]);
    t.resolve_all();
    let scries: Vec<PlayerId> = asked_since(&t, from)
        .iter()
        .filter(|(_, d)| matches!(d, Decision::Scry { .. }))
        .map(|(p, _)| *p)
        .collect();
    assert_eq!(scries, vec![P0, P1, P2], "{}", t.dump_log());
}
