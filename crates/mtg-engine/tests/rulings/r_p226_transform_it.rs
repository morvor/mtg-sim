//! Rulings batch P226 — cards whose transform instruction refers to the permanent with a
//! pronoun ("transform it", "transform her"): Avacynian Missionaries, Uninvited Geist,
//! Geier Reach Bandit, Casal, Foreboding Statue, Cecil and Vincent Valentine, each with
//! every ability on both faces (CR 701.27, 712.18).

use crate::r_s01_common::*;
use crate::r_s02_common::{can_attack, destroy};
use crate::r_s06_common::{attach_new, damage};
use crate::r_s08_common::is_tapped;
use crate::r_s13_common::add;
use crate::r_s17_common::*;
use crate::r_s21_common::legal_blocks;
use mtg_engine::keywords::KeywordKind;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

const MISSIONARIES: &str = "Avacynian Missionaries // Lunarch Inquisitors";
const GEIST: &str = "Uninvited Geist // Unimpeded Trespasser";
const BANDIT: &str = "Geier Reach Bandit // Vildin-Pack Alpha";
const CASAL: &str = "Casal, Lurkwood Pathfinder // Casal, Pathbreaker Owlbear";
const STATUE: &str = "Foreboding Statue // Forsaken Thresher";
const CECIL: &str = "Cecil, Dark Knight // Cecil, Redeemed Paladin";
const VINCENT: &str = "Vincent Valentine // Galian Beast";

/// `p` casts Lightning Bolt at `target` and it resolves.
fn bolt(t: &mut TestGame, p: PlayerId, target: PlayerId) {
    t.lands(p, "Mountain", 1);
    let b = t.hand(p, "Lightning Bolt");
    t.cast(p, b).target(target).go();
    t.resolve_all();
}

#[test]
fn avacynian_missionaries_transform_while_equipped_and_the_inquisitors_exile_a_creature() {
    cr!("603.4", "701.27a", "701.27e", "610.3");
    supported(MISSIONARIES);
    // "At the beginning of your end step, if this creature is equipped, transform it."
    // Unequipped: nothing.
    let mut t = TestGame::new(2);
    let missionaries = t.battlefield(P0, MISSIONARIES);
    t.advance_to(P0, Step::End);
    t.settle();
    assert_eq!(t.stack_len(), 0);
    t.resolve_all();
    assert_eq!(name_of(&t, missionaries), "Avacynian Missionaries");
    // Equipped: it transforms, and Lunarch Inquisitors' "When this creature transforms
    // into Lunarch Inquisitors, you may exile another target creature until this
    // creature leaves the battlefield." exiles P1's Bears.
    let mut t = TestGame::new(2);
    let missionaries = t.battlefield(P0, MISSIONARIES);
    attach_new(&mut t, P0, "Bonesplitter", missionaries);
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.answer_targets(P0, &[Entity::Object(bears)]);
    t.answer_yes(P0, true);
    t.advance_to(P0, Step::End);
    t.resolve_all();
    assert_eq!(name_of(&t, missionaries), "Lunarch Inquisitors");
    assert!(t.in_exile("Grizzly Bears"));
    // 4/4, +2/+0 from Bonesplitter.
    assert_eq!(t.pt(missionaries), (6, 4));
    // The Bears return when the Inquisitors leave the battlefield.
    destroy(&mut t, missionaries);
    assert_eq!(t.named_on_battlefield("Grizzly Bears").len(), 1);
}

#[test]
fn uninvited_geist_skulks_and_transforms_into_an_unblockable_trespasser() {
    cr!("702.118b", "701.27a", "509.1b");
    supported(GEIST);
    let mut t = TestGame::new(2);
    let geist = t.battlefield(P0, GEIST);
    let giant = t.battlefield(P1, "Hill Giant");
    let bears = t.battlefield(P1, "Grizzly Bears");
    // Skulk: the 3/3 can't block the 2/2 Geist; the 2/2 Bears can.
    attack_with(&mut t, &[(geist, Entity::Player(P1))]);
    assert!(!legal_blocks(&mut t, P1, &[(giant, geist)]));
    assert!(legal_blocks(&mut t, P1, &[(bears, geist)]));
    // "When this creature deals combat damage to a player, transform it."
    block_and_finish(&mut t, P1, &[]);
    t.resolve_all();
    assert_eq!(t.life(P1), 18);
    assert_eq!(name_of(&t, geist), "Unimpeded Trespasser");
    assert_eq!(t.pt(geist), (3, 3));
    // "This creature can't be blocked." — not even by the Bears.
    t.advance_to(P1, Step::Upkeep);
    t.advance_to(P0, Step::BeginningOfCombat);
    attack_with(&mut t, &[(geist, Entity::Player(P1))]);
    assert!(!legal_blocks(&mut t, P1, &[(bears, geist)]));
    block_and_finish(&mut t, P1, &[]);
    assert_eq!(t.life(P1), 15);
}

#[test]
fn geier_reach_bandit_has_haste_and_its_vildin_pack_alpha_side_transforms_werewolves() {
    cr!("702.10b", "603.4", "701.27a", "603.6a");
    supported(BANDIT);
    // Haste: a 3/2 that can attack the turn it comes under P0's control.
    let mut t = TestGame::new(2);
    let bandit = t.battlefield_sick(P0, BANDIT);
    t.set_step(P0, Step::BeginningOfCombat);
    assert!(can_attack(&mut t, bandit));
    // "At the beginning of each upkeep, if no spells were cast last turn, transform this
    // creature."
    t.advance_to(P1, Step::Upkeep);
    t.resolve_all();
    assert_eq!(name_of(&t, bandit), "Vildin-Pack Alpha");
    assert_eq!(t.pt(bandit), (4, 3));
    // "Whenever a Werewolf you control enters, you may transform it."
    t.answer_yes(P0, true);
    let captive = t.enter(P0, "Wolfbitten Captive // Krallenhorde Killer");
    t.resolve_all();
    assert_eq!(name_of(&t, captive), "Krallenhorde Killer");
    t.answer_yes(P0, false);
    let other = t.enter(P0, "Wolfbitten Captive // Krallenhorde Killer");
    t.resolve_all();
    assert_eq!(name_of(&t, other), "Wolfbitten Captive");
    // Not a Werewolf: no trigger.
    t.enter(P0, "Grizzly Bears");
    t.settle();
    assert_eq!(t.stack_len(), 0);
    // "At the beginning of each upkeep, if a player cast two or more spells last turn,
    // transform this creature."
    t.advance_to(P1, Step::PrecombatMain);
    bolt(&mut t, P1, P0);
    bolt(&mut t, P1, P0);
    t.advance_to(P0, Step::Upkeep);
    t.resolve_all();
    assert_eq!(name_of(&t, bandit), "Geier Reach Bandit");
}

#[test]
fn casal_fetches_a_forest_and_transforms_when_she_attacks_if_you_pay() {
    cr!("701.27a", "701.27e", "702.20b", "702.19b", "603.5");
    supported(CASAL);
    // "When Casal enters, search your library for a Forest card, put it onto the
    // battlefield tapped, then shuffle."
    let mut t = TestGame::new(2);
    let forest = t.library_top(P0, "Forest");
    let casal = t.enter(P0, CASAL);
    t.answer_choose(P0, &[Entity::Object(forest)]);
    t.resolve_all();
    let forests = t.named_on_battlefield("Forest");
    assert_eq!(forests.len(), 1);
    assert!(is_tapped(&t, forests[0]));
    // "Whenever Casal attacks, you may pay {1}{G}. If you do, transform her." Vigilance:
    // she doesn't tap.
    t.g.objects[casal.0 as usize].summoning_sick = false;
    let isamaru = t.battlefield(P0, "Isamaru, Hound of Konda");
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.lands(P0, "Forest", 2);
    t.answer_yes(P0, true);
    attack_with(&mut t, &[(casal, Entity::Player(P1))]);
    t.resolve_all();
    assert!(!is_tapped(&t, casal));
    assert_eq!(name_of(&t, casal), "Casal, Pathbreaker Owlbear");
    // "When this creature transforms into Casal, Pathbreaker Owlbear, other legendary
    // creatures you control get +2/+2 and gain trample until end of turn."
    t.resolve_all();
    assert_eq!(t.pt(isamaru), (4, 4));
    assert!(t.obj_now(isamaru).has_keyword(KeywordKind::Trample));
    assert_eq!(t.pt(bears), (2, 2));
    assert_eq!(t.pt(casal), (6, 6));
    let o = t.obj_now(casal);
    assert!(o.has_keyword(KeywordKind::Trample) && o.has_keyword(KeywordKind::Vigilance));
    block_and_finish(&mut t, P1, &[]);
    assert_eq!(t.life(P1), 14);
    // "At the beginning of your upkeep, transform Casal."
    t.advance_to(P1, Step::Upkeep);
    assert_eq!(t.pt(isamaru), (2, 2));
    t.advance_to(P0, Step::Upkeep);
    t.resolve_all();
    assert_eq!(name_of(&t, casal), "Casal, Lurkwood Pathfinder");
    // Not paying: she stays herself.
    t.advance_to(P0, Step::BeginningOfCombat);
    t.answer_yes(P0, false);
    attack_with(&mut t, &[(casal, Entity::Player(P1))]);
    t.resolve_all();
    assert_eq!(name_of(&t, casal), "Casal, Lurkwood Pathfinder");
}

#[test]
fn foreboding_statue_gathers_omens_then_untaps_and_transforms_into_forsaken_thresher() {
    cr!("603.4", "608.2c", "701.27a", "505.1");
    supported(STATUE);
    // "{T}: Add one mana of any color. Put an omen counter on this creature."
    let mut t = TestGame::new(2);
    let statue = t.battlefield(P0, STATUE);
    t.activate(P0, statue, 0, &[]).unwrap();
    assert_eq!(t.g.player(P0).mana_pool.total(), 1);
    assert_eq!(t.counters(statue, "omen"), 1);
    // Two omen counters at the end step: nothing.
    add(&mut t, statue, "omen", 1);
    t.advance_to(P0, Step::End);
    t.settle();
    assert_eq!(t.stack_len(), 0);
    // Three: "At the beginning of your end step, if there are three or more omen counters
    // on this creature, untap it, then transform it."
    t.advance_to(P1, Step::Upkeep);
    t.advance_to(P0, Step::PrecombatMain);
    t.activate(P0, statue, 0, &[]).unwrap();
    assert_eq!(t.counters(statue, "omen"), 3);
    assert!(is_tapped(&t, statue));
    t.advance_to(P0, Step::End);
    t.resolve_all();
    assert_eq!(name_of(&t, statue), "Forsaken Thresher");
    assert!(!is_tapped(&t, statue));
    assert_eq!(t.pt(statue), (5, 5));
    // "At the beginning of your first main phase, add one mana of any color."
    t.advance_to(P1, Step::Upkeep);
    t.advance_to(P0, Step::PrecombatMain);
    t.resolve_all();
    assert_eq!(t.g.player(P0).mana_pool.total(), 1);
}

#[test]
fn cecil_pays_life_for_his_damage_and_transforms_at_half_life() {
    cr!("702.2b", "608.2c", "701.27a", "702.15b");
    supported(CECIL);
    // "Darkness — Whenever Cecil deals damage, you lose that much life. Then if your life
    // total is less than or equal to half your starting life total, untap Cecil and
    // transform it."
    let mut t = TestGame::new(2);
    let cecil = t.battlefield(P0, CECIL);
    let bears = t.battlefield(P1, "Grizzly Bears");
    // Deathtouch: 1 damage destroys the Bears. P0 loses 1 life: 19, not half.
    damage(&mut t, cecil, 1, bears);
    t.resolve_all();
    assert!(t.in_graveyard(P1, "Grizzly Bears"));
    assert_eq!(t.life(P0), 19);
    assert_eq!(name_of(&t, cecil), "Cecil, Dark Knight");
    // At 12 life, tapped: 2 damage makes it 10, half of 20.
    t.g.players[0].life = 12;
    t.g.objects[cecil.0 as usize].tapped = true;
    let giant = t.battlefield(P1, "Hill Giant");
    damage(&mut t, cecil, 2, giant);
    t.resolve_all();
    assert_eq!(t.life(P0), 10);
    assert_eq!(name_of(&t, cecil), "Cecil, Redeemed Paladin");
    assert!(!is_tapped(&t, cecil));
    // Cecil, Redeemed Paladin: 4/4 lifelink; "Protect — Whenever Cecil attacks, other
    // attacking creatures gain indestructible until end of turn."
    assert_eq!(t.pt(cecil), (4, 4));
    let ally = t.battlefield(P0, "Grizzly Bears");
    attack_with(
        &mut t,
        &[(cecil, Entity::Player(P1)), (ally, Entity::Player(P1))],
    );
    t.resolve_all();
    assert!(t.obj_now(ally).has_keyword(KeywordKind::Indestructible));
    assert!(!t.obj_now(cecil).has_keyword(KeywordKind::Indestructible));
    block_and_finish(&mut t, P1, &[]);
    assert_eq!(t.life(P1), 14);
    assert_eq!(t.life(P0), 14);
}

#[test]
fn vincent_valentine_grows_and_turns_into_galian_beast_which_returns_as_vincent() {
    cr!("701.27a", "603.10a", "712.14", "702.19b", "702.15b");
    supported(VINCENT);
    // "Whenever a creature an opponent controls dies, put a number of +1/+1 counters on
    // Vincent Valentine equal to that creature's power."
    let mut t = TestGame::new(2);
    let vincent = t.battlefield(P0, VINCENT);
    let giant = t.battlefield(P1, "Hill Giant");
    destroy(&mut t, giant);
    t.resolve_all();
    assert_eq!(t.counters(vincent, counters::PLUS1), 3);
    // P0's own creature: no counters.
    let mine = t.battlefield(P0, "Grizzly Bears");
    destroy(&mut t, mine);
    t.resolve_all();
    assert_eq!(t.counters(vincent, counters::PLUS1), 3);
    // "Whenever Vincent Valentine attacks, you may transform it." Galian Beast: 3/2
    // trample, lifelink (6/5 with the counters).
    t.answer_yes(P0, true);
    attack_with(&mut t, &[(vincent, Entity::Player(P1))]);
    t.resolve_all();
    assert_eq!(name_of(&t, vincent), "Galian Beast");
    assert_eq!(t.pt(vincent), (6, 5));
    let blocker = t.battlefield(P1, "Grizzly Bears");
    t.answer(
        P0,
        DecisionKind::Damage,
        mtg_engine::decision::Answer::Numbers(vec![2, 4]),
    );
    block_and_finish(&mut t, P1, &[(blocker, vincent)]);
    assert_eq!(t.life(P1), 16);
    assert_eq!(t.life(P0), 26);
    // "When Galian Beast dies, return it to the battlefield tapped (front face up)."
    destroy(&mut t, vincent);
    t.resolve_all();
    let back = t.named_on_battlefield("Vincent Valentine");
    assert_eq!(back.len(), 1);
    assert!(is_tapped(&t, back[0]));
    // Declining to transform.
    let mut t = TestGame::new(2);
    let vincent = t.battlefield(P0, VINCENT);
    t.answer_yes(P0, false);
    attack_with(&mut t, &[(vincent, Entity::Player(P1))]);
    t.resolve_all();
    assert_eq!(name_of(&t, vincent), "Vincent Valentine");
}
