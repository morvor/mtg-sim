//! Rulings batch P012 — targets, damage and resolution: a spell or ability whose only
//! target is illegal doesn't resolve (CR 608.2b); excess damage (CR 120.4a); last known
//! information (CR 608.2h); emblems (CR 114); dice rolled as abilities resolve (CR 706);
//! the affected set of a resolving effect (CR 611.2c); and the remaining spot checks.

use crate::r_p012_common::*;
use crate::r_p127_common::{bounce, move_to};
use crate::r_s01_common::supported;
use mtg_engine::decision::Answer;
use mtg_engine::events::Event;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

/// The damage marked on the permanent now (0 if it left the battlefield).
fn dmg(t: &TestGame, id: ObjectId) -> u32 {
    if t.on_battlefield(id) {
        t.obj_now(id).damage
    } else {
        0
    }
}

#[test]
fn gandalfs_sanction_with_an_illegal_target_deals_no_damage() {
    cr!("608.2b");
    ruling!(
        "Gandalf's Sanction",
        "If the target creature is an illegal target by the time Gandalf's Sanction tries to resolve, the spell won't resolve. It won't deal damage to any player."
    );
    supported("Gandalf's Sanction");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    fill_graveyard(&mut t, P0, "Opt", 5);
    let gs = cast_card(&mut t, P0, "Gandalf's Sanction", &[obj(bears)]);
    bounce(&mut t, bears);
    t.resolve_all();
    assert_eq!(t.life(P1), 20);
    assert_eq!(t.zone(gs), Zone::Graveyard(P0));
}

#[test]
fn gandalfs_sanction_excess_damage_can_be_prevented() {
    cr!("120.4a", "615.1");
    ruling!(
        "Gandalf's Sanction",
        "Once you've determined how much damage is excess, Gandalf's Sanction simultaneously deals damage to the creature and to its controller. This damage may be modified by replacement or prevention effects."
    );
    // X = 5 at a 2/2: 2 to it and 3 to P1.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    fill_graveyard(&mut t, P0, "Opt", 5);
    cast_card(&mut t, P0, "Gandalf's Sanction", &[obj(bears)]);
    t.resolve_all();
    assert!(!t.on_battlefield(bears));
    assert_eq!(t.life(P1), 17);
    // P1's Healing Salve prevents the next 3 damage to P1: the excess is prevented.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    fill_graveyard(&mut t, P0, "Opt", 5);
    cast_card(&mut t, P0, "Gandalf's Sanction", &[obj(bears)]);
    t.lands(P1, "Plains", 1);
    let salve = t.hand(P1, "Healing Salve");
    t.cast(P1, salve).modes(&[1]).target(pl(P1)).go();
    t.resolve_all();
    assert!(!t.on_battlefield(bears));
    assert_eq!(t.life(P1), 20);
}

#[test]
fn sunflare_shaman_with_an_illegal_target_doesnt_damage_itself() {
    cr!("608.2b");
    ruling!(
        "Sunflare Shaman",
        "If the target leaves the battlefield or otherwise becomes an illegal target before the ability resolves, the ability doesn't resolve and Sunflare Shaman won't deal any damage to itself."
    );
    supported("Sunflare Shaman");
    let mut t = TestGame::new(2);
    let ss = t.battlefield(P0, "Sunflare Shaman");
    t.graveyard(P0, "Fire Elemental");
    t.lands(P0, "Mountain", 2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.activate(P0, ss, 0, &[obj(bears)]).unwrap();
    bounce(&mut t, bears);
    t.resolve_all();
    assert!(t.on_battlefield(ss));
    assert_eq!(dmg(&t, ss), 0);
    // With a legal target it deals X = 1 to each.
    let bears = t.battlefield(P1, "Hill Giant");
    crate::r_p127_common::untap(&mut t, ss);
    t.lands(P0, "Mountain", 2);
    t.activate(P0, ss, 0, &[obj(bears)]).unwrap();
    t.resolve_all();
    assert_eq!(dmg(&t, bears), 1);
    assert!(!t.on_battlefield(ss), "the 2/1 dealt itself 1 damage");
}

#[test]
fn ral_emblem_with_an_illegal_target_draws_nothing() {
    cr!("608.2b", "114.4");
    ruling!(
        "Ral, Izzet Viceroy",
        "If the target permanent or player becomes an illegal target for the triggered ability of Ral's emblem, the ability doesn't resolve. You don't draw two cards."
    );
    let mut t = TestGame::new(2);
    ral_emblem(&mut t);
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.answer_targets(P0, &[obj(bears)]);
    cast_card(&mut t, P0, "Opt", &[]);
    bounce(&mut t, bears);
    t.resolve();
    assert_eq!(t.hand_size(P0), 0, "the trigger drew nothing");
    t.resolve_all();
    assert_eq!(t.hand_size(P0), 1, "Opt's draw");
}

#[test]
fn ral_emblem_is_a_colorless_source() {
    cr!("114.3", "105.2c");
    ruling!(
        "Ral, Izzet Viceroy",
        "The emblem created by Ral's last ability is colorless. The damage it deals is from a colorless source."
    );
    let mut t = TestGame::new(2);
    ral_emblem(&mut t);
    let emblem = t.g.command[0];
    assert!(t.obj(emblem).chars.colors.is_colorless());
    t.answer_targets(P0, &[pl(P1)]);
    let from = t.g.turn_events.len();
    cast_card(&mut t, P0, "Opt", &[]);
    t.resolve_all();
    assert_eq!(t.life(P1), 16);
    let src = t.g.turn_events[from..].iter().find_map(|e| match e {
        Event::Damage { source, .. } => Some(*source),
        _ => None,
    });
    assert_eq!(src, Some(emblem));
}

#[test]
fn ral_puts_the_only_card_of_the_library_into_hand() {
    cr!("609.3");
    ruling!(
        "Ral, Izzet Viceroy",
        "If you have only one card in your library, Ral's first ability puts it into your hand. Nothing is put into your graveyard."
    );
    let mut t = TestGame::new(2);
    let lib: Vec<ObjectId> = t.g.player(P0).library.clone();
    for c in &lib[1..] {
        move_to(&mut t, *c, Zone::Exile);
    }
    assert_eq!(t.library_size(P0), 1);
    let ral = t.battlefield(P0, "Ral, Izzet Viceroy");
    t.activate(P0, ral, 0, &[]).unwrap();
    t.resolve_all();
    assert_eq!(t.hand_size(P0), 1);
    assert_eq!(t.graveyard_size(P0), 0);
    assert_eq!(t.library_size(P0), 0);
}

#[test]
fn olorins_searing_light_uses_the_exiled_creatures_last_known_power() {
    cr!("608.2h");
    ruling!(
        "Olórin's Searing Light",
        "Use the power of each exiled creature as it last existed on the battlefield to determine how much damage is dealt to each opponent."
    );
    supported("Olórin's Searing Light");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    // Giant Growth makes the Bears 5/5 until end of turn; exiled, it's a 2/2 card.
    t.lands(P1, "Forest", 1);
    let gg = t.hand(P1, "Giant Growth");
    t.cast(P1, gg).target(bears).go();
    t.resolve_all();
    fill_graveyard(&mut t, P0, "Opt", 2);
    cast_card(&mut t, P0, "Olórin's Searing Light", &[]);
    t.resolve_all();
    assert_eq!(t.zone(bears), Zone::Exile);
    assert_eq!(t.life(P1), 15);
}

#[test]
fn great_teachers_decree_affects_only_creatures_you_control_as_it_resolves() {
    cr!("611.2c");
    ruling!(
        "Great Teacher's Decree",
        "Great Teacher’s Decree affects only creatures you control at the time it resolves. It won’t affect creatures that come under your control later in the turn."
    );
    supported("Great Teacher's Decree");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    cast_card(&mut t, P0, "Great Teacher's Decree", &[]);
    t.resolve_all();
    let later = t.battlefield(P0, "Hill Giant");
    t.g.recompute();
    assert_eq!(t.pt(bears), (4, 3));
    assert_eq!(t.pt(later), (3, 3));
}

#[test]
fn blessings_of_nature_gives_each_target_at_least_one_counter() {
    cr!("601.2d");
    ruling!(
        "Blessings of Nature",
        "You may choose up to four target creatures. You declare how many counters will be placed on each target creature as you cast Blessings of Nature. Each target must receive at least one counter."
    );
    supported("Blessings of Nature");
    // Four targets, one counter each.
    let mut t = TestGame::new(2);
    let cs: Vec<ObjectId> = (0..4).map(|_| t.battlefield(P0, "Grizzly Bears")).collect();
    let es: Vec<Entity> = cs.iter().map(|c| obj(*c)).collect();
    t.answer_targets(P0, &es);
    t.answer(P0, DecisionKind::Divide, Answer::Numbers(vec![1, 1, 1, 1]));
    cast_card(&mut t, P0, "Blessings of Nature", &[]);
    t.resolve_all();
    for c in &cs {
        assert_eq!(t.counters(*c, "+1/+1"), 1);
    }
    // Two targets: a division leaving one of them without a counter isn't accepted.
    let mut t = TestGame::new(2);
    let a = t.battlefield(P0, "Grizzly Bears");
    let b = t.battlefield(P0, "Hill Giant");
    t.answer_targets(P0, &[obj(a), obj(b)]);
    t.answer(P0, DecisionKind::Divide, Answer::Numbers(vec![4, 0]));
    cast_card(&mut t, P0, "Blessings of Nature", &[]);
    t.resolve_all();
    assert!(t.counters(a, "+1/+1") >= 1 && t.counters(b, "+1/+1") >= 1);
    assert_eq!(t.counters(a, "+1/+1") + t.counters(b, "+1/+1"), 4);
}

#[test]
fn tarmogoyf_nest_creates_a_tarmogoyf_token() {
    cr!("111.4", "604.3");
    ruling!(
        "Tarmogoyf Nest",
        "Tarmogoyf Nest's activated ability creates a token that's a copy of the card Tarmogoyf in the Oracle card reference."
    );
    supported("Tarmogoyf Nest");
    let mut t = TestGame::new(2);
    let land = t.battlefield(P0, "Forest");
    t.lands(P0, "Forest", 2);
    let nest = cast_card(&mut t, P0, "Tarmogoyf Nest", &[obj(land)]);
    t.resolve_all();
    assert!(t.on_battlefield(nest));
    // Card types in graveyards: creature, instant, land.
    t.graveyard(P1, "Grizzly Bears");
    t.graveyard(P1, "Opt");
    t.graveyard(P1, "Forest");
    t.lands(P0, "Forest", 2);
    crate::r_p127_common::untap(&mut t, land);
    t.activate(P0, land, 1, &[]).unwrap();
    t.resolve_all();
    let toks = crate::r_s01_common::tokens(&t, P0);
    assert_eq!(toks.len(), 1);
    let tok = t.obj(toks[0]);
    assert_eq!(tok.chars.name, "Tarmogoyf");
    assert!(tok.chars.has_subtype("Lhurgoyf"));
    // Tarmogoyf Nest itself (a kindred enchantment) is on the battlefield, not counted.
    assert_eq!(t.pt(toks[0]), (3, 4));
}

#[test]
fn ruric_thar_doesnt_have_to_attack_if_it_cant_or_it_costs() {
    cr!("508.1d");
    ruling!(
        "Ruric Thar, the Unbowed",
        "If Ruric Thar can't attack for any reason (such as being tapped or having come under that player's control that turn), then it doesn't attack. If there's a cost associated with having a creature attack, the player isn't forced to pay that cost, so Ruric Thar doesn't have to attack in that case either."
    );
    supported("Ruric Thar, the Unbowed");
    // (setup, attacks)
    type Setup = fn(&mut TestGame, ObjectId);
    let cases: [(&str, Setup, bool); 3] = [
        ("untapped", |_, _| {}, true),
        ("tapped", |t, r| crate::r_p127_common::tap(t, r), false),
        ("Ghostly Prison", |t, _| {
            t.battlefield(P1, "Ghostly Prison");
            t.lands(P0, "Wastes", 2);
        }, false),
    ];
    for (what, setup, attacks) in cases {
        let mut t = TestGame::new(2);
        let ruric = t.battlefield(P0, "Ruric Thar, the Unbowed");
        setup(&mut t, ruric);
        t.set_step(P0, Step::BeginningOfCombat);
        // P0 declares no attackers.
        t.answer(P0, DecisionKind::Attackers, Answer::Attackers(vec![]));
        t.answer_yes(P0, false);
        t.advance_to(P0, Step::EndOfCombat);
        assert_eq!(t.life(P1) < 20, attacks, "{what}");
    }
}

#[test]
fn a_die_is_rolled_as_the_ability_resolves_after_targets_were_chosen() {
    cr!("602.2b", "608.2b");
    ruling!(
        "Spiked Pit Trap",
        "An effect that says \"choose a target, then roll a d20\" or similar still uses the normal process of putting an ability on the stack and resolving it. Choosing targets is part of putting the ability on the stack and rolling the d20 happens later, as the ability resolves."
    );
    supported("Spiked Pit Trap");
    let mut t = TestGame::new(2);
    let trap = t.battlefield(P0, "Spiked Pit Trap");
    t.lands(P0, "Wastes", 5);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let from = t.g.turn_events.len();
    t.activate(P0, trap, 0, &[obj(bears)]).unwrap();
    let rolled = |t: &TestGame| {
        t.g.turn_events[from..]
            .iter()
            .filter(|e| matches!(e, Event::DieRolled { .. }))
            .count()
    };
    assert_eq!(rolled(&t), 0, "no roll while the ability is on the stack");
    t.resolve_all();
    assert_eq!(rolled(&t), 1);
    assert!(!t.on_battlefield(bears));
    // With its target gone, the ability doesn't resolve: no roll.
    let mut t = TestGame::new(2);
    let trap = t.battlefield(P0, "Spiked Pit Trap");
    t.lands(P0, "Wastes", 5);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let from = t.g.turn_events.len();
    t.activate(P0, trap, 0, &[obj(bears)]).unwrap();
    bounce(&mut t, bears);
    t.resolve_all();
    let n = t.g.turn_events[from..]
        .iter()
        .filter(|e| matches!(e, Event::DieRolled { .. }))
        .count();
    assert_eq!(n, 0);
}
