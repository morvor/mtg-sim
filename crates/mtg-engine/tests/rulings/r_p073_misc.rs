//! Rulings batch P073 — Bronzehide Lion's return (copies, tokens, control), Zack Fair's
//! counters, Final Showdown, Gift of Doom's morph, Dauntless Bodyguard's choice, Gyome's
//! count, Arms Race's delayed sacrifice, Spirit Bonds, Strefan, Chariot of the Sun and
//! Windborne Charge.

use crate::r_p073_common::*;
use crate::r_s01_common::supported;
use crate::r_s06_common::{activate_containing, give_control};
use crate::r_s13_common::add;
use mtg_engine::decision::Decision;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::{CardType, Color};
use mtg_engine::*;

// ---------------------------------------------------------------------------
// Bronzehide Lion
// ---------------------------------------------------------------------------

#[test]
fn bronzehide_lion_copy_returns_as_an_aura_with_only_the_granted_abilities() {
    cr!("707.2", "614.12", "303.4f");
    ruling!(
        "Bronzehide Lion",
        "If a nontoken permanent is a copy of Bronzehide Lion, it returns from its owner's graveyard as an Aura with the two abilities granted by Bronzehide Lion and none of its normal abilities. If it has any enters-the-battlefield replacement effects, those won't apply. The Aura keeps its name, colors, and any supertypes it may have."
    );
    ruling!(
        "Bronzehide Lion",
        "If a token is a copy of Bronzehide Lion, it won't return from its owner's graveyard."
    );
    supported("Bronzehide Lion");
    supported("Clone");
    let mut t = TestGame::new(2);
    let lion = t.battlefield(P0, "Bronzehide Lion");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let clone = crate::r_p023_common::enter_copying(&mut t, P0, "Clone", lion);
    assert_eq!(&*t.obj_now(clone).chars.name, "Bronzehide Lion");
    t.answer_choose(P0, &[obj(bears)]);
    destroy_now(&mut t, clone);
    t.resolve_all();
    let back = t.g.current(clone);
    assert_eq!(t.zone(back), Zone::Battlefield);
    let o = t.obj_now(back);
    assert_eq!(&*o.chars.name, "Clone", "Clone's copy effect didn't apply");
    assert!(o.chars.colors.contains(Color::Blue));
    assert!(o.is(CardType::Enchantment) && !o.is(CardType::Creature));
    assert_eq!(o.attached_to, Some(obj(bears)));
    assert!(o.chars.abilities.iter().any(|a| a.text.contains("Enchanted creature gains indestructible")));
    // A token copy doesn't return.
    let mut t = TestGame::new(2);
    let lion = t.battlefield(P0, "Bronzehide Lion");
    t.battlefield(P0, "Grizzly Bears");
    let tok = crate::r_s17_common::token_copy(&mut t, P0, lion)[0];
    destroy_now(&mut t, tok);
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Bronzehide Lion"), vec![lion]);
}

#[test]
fn bronzehide_lion_returns_under_its_controller() {
    cr!("603.3a", "110.2");
    ruling!(
        "Bronzehide Lion",
        "If you control but don't own Bronzehide Lion, you'll return it to the battlefield when it dies, not its owner. It'll enchant a creature you control."
    );
    let mut t = TestGame::new(2);
    let lion = t.battlefield(P1, "Bronzehide Lion");
    give_control(&mut t, lion, P0);
    let mine = t.battlefield(P0, "Grizzly Bears");
    t.battlefield(P1, "Hill Giant");
    t.answer_choose(P0, &[obj(mine)]);
    destroy_now(&mut t, lion);
    t.resolve_all();
    let back = t.g.current(lion);
    assert_eq!(t.zone(back), Zone::Battlefield);
    assert_eq!(t.obj_now(back).controller, P0);
    assert_eq!(t.obj_now(back).attached_to, Some(obj(mine)));
}

// ---------------------------------------------------------------------------
// Zack Fair
// ---------------------------------------------------------------------------

#[test]
fn zack_fair_puts_the_same_counters_it_had() {
    cr!("608.2h", "122.1");
    ruling!(
        "Zack Fair",
        "Zack Fair's last ability doesn't cause you to move counters from Zack Fair onto the target creature. Rather, you put the same number of each kind of counter Zack Fair had when it was sacrificed onto the target creature."
    );
    ruling!(
        "Zack Fair",
        "Zack Fair's last ability puts all counters that were on Zack Fair onto the target creature, not just its +1/+1 counters."
    );
    ruling!(
        "Zack Fair",
        "In some unusual cases, you may end up putting the appropriate counters on more than one permanent."
    );
    supported("Zack Fair");
    supported("The Ozolith");
    let mut t = TestGame::new(2);
    let ozolith = t.battlefield(P0, "The Ozolith");
    let zack = t.enter(P0, "Zack Fair");
    assert_eq!(t.counters(zack, "+1/+1"), 1);
    add(&mut t, zack, "+1/+1", 1);
    add(&mut t, zack, "flying", 1);
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.lands(P0, "Wastes", 1);
    t.answer_targets(P0, &[obj(bears)]);
    activate_containing(&mut t, P0, zack, "indestructible").unwrap();
    t.resolve_all();
    assert!(kw(&t, bears, INDESTRUCTIBLE));
    assert_eq!(t.counters(bears, "+1/+1"), 2);
    assert_eq!(t.counters(bears, "flying"), 1);
    // The Ozolith got them too.
    assert_eq!(t.counters(ozolith, "+1/+1"), 2);
    assert_eq!(t.counters(ozolith, "flying"), 1);
}

// ---------------------------------------------------------------------------
// Final Showdown
// ---------------------------------------------------------------------------

#[test]
fn final_showdown_later_abilities_survive_and_the_choice_is_on_resolution() {
    cr!("613.7", "613.1f", "702.172a");
    ruling!(
        "Final Showdown",
        "If an effect grants a creature an ability after Final Showdown causes all creatures to lose all abilities, that creature won’t lose that ability."
    );
    ruling!(
        "Final Showdown",
        "The second mode of Final Showdown doesn’t target the creature. You don’t choose which creature will gain indestructible until the spell is resolving."
    );
    supported("Final Showdown");
    supported("Jump");
    let mut t = TestGame::new(2);
    let angel = t.battlefield(P0, "Serra Angel");
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.lands(P0, "Plains", 3);
    let c = t.hand(P0, "Final Showdown");
    let from = t.asked().len();
    t.cast(P0, c).modes(&[0, 1]).go();
    assert!(crate::r_s25_common::targets_of(&t, c).is_empty(), "no targets");
    let chose_before = crate::r_s03_common::choice_candidates(&t, from, "");
    assert!(chose_before.is_empty(), "nothing chosen while casting");
    t.answer_choose(P0, &[obj(bears)]);
    t.resolve_all();
    assert!(!kw(&t, angel, FLYING), "lost its abilities");
    // Mode two's indestructible comes after mode one's ability loss.
    assert!(kw(&t, bears, INDESTRUCTIBLE));
    cast_resolve(&mut t, P0, "Jump", &[obj(bears)]);
    assert!(kw(&t, bears, FLYING));
}

// ---------------------------------------------------------------------------
// Gift of Doom, Dauntless Bodyguard
// ---------------------------------------------------------------------------

#[test]
fn gift_of_doom_is_attached_as_its_turned_face_up() {
    cr!("702.37e", "116.2b", "708.8");
    ruling!(
        "Gift of Doom",
        "No player may take any actions between the time you announce that you're turning Gift of Doom face up and the time it's attached to the chosen creature."
    );
    supported("Gift of Doom");
    let mut t = TestGame::new(2);
    let gift = crate::r_s12_common::morph(&mut t, P0, "Gift of Doom");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let fodder = t.battlefield(P0, "Llanowar Elves");
    t.answer_choose(P0, &[obj(fodder)]);
    t.answer_choose(P0, &[obj(bears)]);
    let from = t.asked().len();
    assert!(crate::r_s11_common::turn_face_up(&mut t, P0, gift));
    assert_eq!(t.stack_len(), 0);
    assert!(!t.on_battlefield(fodder));
    assert_eq!(t.obj_now(gift).attached_to, Some(obj(bears)));
    assert!(kw(&t, bears, INDESTRUCTIBLE));
    assert!(!t.asked()[from..]
        .iter()
        .any(|(_, d)| matches!(d, Decision::Priority { .. })));
}

#[test]
fn dauntless_bodyguard_choice_uses_no_stack_and_excludes_simultaneous_arrivals() {
    cr!("614.12", "603.6a");
    ruling!(
        "Dauntless Bodyguard",
        "Dauntless Bodyguard’s first ability isn’t a triggered ability and doesn’t use the stack. Players can’t respond to your choice of which creature it’s protecting."
    );
    ruling!(
        "Dauntless Bodyguard",
        "If Dauntless Bodyguard enters the battlefield at the same time as another creature, that creature can’t be chosen for its ability."
    );
    supported("Dauntless Bodyguard");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.answer_choose(P0, &[obj(bears)]);
    let guard = t.enter(P0, "Dauntless Bodyguard");
    assert_eq!(t.stack_len(), 0);
    activate_containing(&mut t, P0, guard, "Sacrifice").unwrap();
    t.resolve_all();
    assert!(kw(&t, bears, INDESTRUCTIBLE));
    // Entering together with the Giant: only the Bears may be chosen.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let from = t.asked().len();
    let ids = crate::r_s24_common::enter_together(
        &mut t,
        &[(P0, "Dauntless Bodyguard"), (P0, "Hill Giant")],
    );
    let cands: Vec<Vec<Entity>> = t.asked()[from..]
        .iter()
        .filter_map(|(_, d)| match d {
            Decision::ChooseEntities { candidates, .. } => Some(candidates.clone()),
            _ => None,
        })
        .collect();
    assert_eq!(cands, vec![vec![obj(bears)]], "{ids:?}");
}

// ---------------------------------------------------------------------------
// Gyome, Arms Race, Spirit Bonds, Strefan
// ---------------------------------------------------------------------------

#[test]
fn gyome_counts_every_nontoken_creature_that_entered_including_itself() {
    cr!("400.7");
    ruling!(
        "Gyome, Master Chef",
        "Gyome, Master Chef's second ability counts all nontoken creatures that entered the battlefield under your control this turn, even if some or all of those creatures are no longer on the battlefield, no longer creatures, or no longer under your control."
    );
    ruling!(
        "Gyome, Master Chef",
        "The second ability will count Gyome itself on the turn it enters the battlefield."
    );
    let mut t = TestGame::new(2);
    t.enter(P0, "Gyome, Master Chef");
    let gone = t.enter(P0, "Grizzly Bears");
    let given = t.enter(P0, "Hill Giant");
    t.enter(P1, "Llanowar Elves");
    crate::r_s02_common::create_token(&mut t, P0, "Warrior");
    destroy_now(&mut t, gone);
    give_control(&mut t, given, P1);
    t.advance_to(P0, Step::End);
    t.resolve_all();
    assert_eq!(crate::r_s05_common::tokens_with_subtype(&t, P0, "Food").len(), 3);
}

#[test]
fn arms_race_sacrifices_only_the_artifact_it_still_controls() {
    cr!("603.7c", "400.7");
    ruling!(
        "Arms Race",
        "You sacrifice the artifact only if you still control it at the beginning of the next end step. If that artifact has left the battlefield, even if it later came back, you don't sacrifice it."
    );
    supported("Arms Race");
    for case in ["stays", "flickered", "stolen"] {
        let mut t = TestGame::new(2);
        let race = t.battlefield(P0, "Arms Race");
        let thopter = t.hand(P0, "Ornithopter");
        t.lands(P0, "Mountain", 4);
        t.answer_yes(P0, true);
        t.answer_choose(P0, &[obj(thopter)]);
        activate_containing(&mut t, P0, race, "artifact card").unwrap();
        t.resolve_all();
        let thopter = t.g.current(thopter);
        assert!(t.on_battlefield(thopter), "{case}");
        assert!(kw(&t, thopter, HASTE));
        let watched = match case {
            "flickered" => {
                let e = crate::r_s05_common::move_to(&mut t, thopter, Zone::Exile).unwrap();
                crate::r_s05_common::move_to(&mut t, e, Zone::Battlefield).unwrap()
            }
            "stolen" => {
                give_control(&mut t, thopter, P1);
                thopter
            }
            _ => thopter,
        };
        t.advance_to(P0, Step::End);
        t.resolve_all();
        assert_eq!(t.on_battlefield(watched), case != "stays", "{case}");
    }
}

#[test]
fn spirit_bonds_pay_on_resolution_once_per_creature() {
    cr!("603.5");
    ruling!(
        "Spirit Bonds",
        "You decide whether to pay {W} as the triggered ability resolves. You may only pay {W} and create a Spirit token once per nontoken creature."
    );
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Spirit Bonds");
    t.lands(P0, "Plains", 3);
    let from = t.asked().len();
    crate::r_s05_common::enter(&mut t, P0, "Grizzly Bears");
    assert_eq!(t.stack_len(), 1);
    assert_eq!(
        crate::r_s07_common::count_asked(&t, from, |d| matches!(d, Decision::YesNo { .. } | Decision::OptionalCost { .. })),
        0,
        "nothing decided as it triggers"
    );
    t.answer_yes(P0, true);
    t.answer_yes(P0, true);
    t.resolve_all();
    assert_eq!(crate::r_s05_common::tokens_with_subtype(&t, P0, "Spirit").len(), 1);
    assert_eq!(crate::r_s01_common::tapped_lands(&t, P0), 1);
}

#[test]
fn strefan_counts_players_who_lost_life_even_if_they_gained_more() {
    cr!("119.3");
    ruling!(
        "Strefan, Maurer Progenitor",
        "Strefan's first triggered ability counts the number of players that lost any amount of life, even if they also gained that much or more life this turn."
    );
    supported("Strefan, Maurer Progenitor");
    let mut t = TestGame::new(3);
    t.battlefield(P0, "Strefan, Maurer Progenitor");
    t.g.lose_life(P1, 2);
    t.g.gain_life(P1, 5);
    t.g.lose_life(P0, 1);
    t.g.gain_life(P2, 3);
    t.advance_to(P0, Step::End);
    t.resolve_all();
    assert_eq!(crate::r_s05_common::tokens_with_subtype(&t, P0, "Blood").len(), 2);
}

// ---------------------------------------------------------------------------
// Chariot of the Sun, Windborne Charge
// ---------------------------------------------------------------------------

#[test]
fn chariot_of_the_sun_whole_effect_lasts_until_end_of_turn() {
    cr!("611.2a", "514.2");
    ruling!(
        "Chariot of the Sun",
        "The entire effect lasts until end of turn, not just the toughness reduction."
    );
    supported("Chariot of the Sun");
    let mut t = TestGame::new(2);
    let chariot = t.battlefield(P0, "Chariot of the Sun");
    let giant = t.battlefield(P0, "Hill Giant");
    t.lands(P0, "Wastes", 2);
    t.answer_targets(P0, &[obj(giant)]);
    activate_containing(&mut t, P0, chariot, "flying").unwrap();
    t.resolve_all();
    assert!(kw(&t, giant, FLYING));
    assert_eq!(t.pt(giant), (3, 1));
    t.advance_to(P1, Step::Upkeep);
    assert!(!kw(&t, giant, FLYING));
    assert_eq!(t.pt(giant), (3, 3));
}

#[test]
fn windborne_charge_needs_two_creatures_you_control() {
    cr!("601.2c", "115.1");
    ruling!(
        "Windborne Charge",
        "You must target two creatures you control as you cast Windborne Charge. If you can’t (because you control just one creature, perhaps), you can’t cast the spell."
    );
    supported("Windborne Charge");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.battlefield(P1, "Hill Giant");
    crate::r_s25_common::lands_for_cost(&mut t, P0, "Windborne Charge");
    let c = t.hand(P0, "Windborne Charge");
    assert!(!crate::r_p079_common::can_cast_now(&mut t, P0, c));
    let elves = t.battlefield(P0, "Llanowar Elves");
    assert!(crate::r_p079_common::can_cast_now(&mut t, P0, c));
    t.cast(P0, c).targets(&[obj(bears), obj(elves)]).go();
    t.resolve_all();
    assert_eq!(t.pt(bears), (4, 4));
    assert!(kw(&t, elves, FLYING));
}
