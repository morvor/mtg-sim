//! What's left of a prevention effect's division of simultaneous damage (CR 615.7, 616.1;
//! `Game::per_source_division`): the part its controller put on a recipient belongs to that
//! effect and that recipient's damage from that source, and it's used up as that effect
//! prevents the damage.
//!
//! - Two Covers of Winter ("If a creature would deal combat damage to you and/or one or
//!   more creatures you control, prevent X of that damage, where X is the number of age
//!   counters on this enchantment.") each prevent their own X of a trampler's damage as
//!   their controller divided it, whichever is applied first to each recipient's damage
//!   (register entry `per-source-prevention-division`).
//! - A shield divided among its recipients (Divine Deflection, register entry
//!   `shield-divided-among-recipients`) gives a recipient its part only once, even when
//!   more of the same source's damage is redirected to it (Pariah, CR 614.9), so the rest
//!   of the shield is left for the other recipients' parts.

use crate::r614_audit_common::*;
use mtg_engine::decision::{Agent, Answer, Decision, PassiveAgent};
use mtg_engine::game::Game;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

/// P0's Cover of Winter with `x` age counters.
fn cover_of_winter(t: &mut TestGame, x: u32) -> ObjectId {
    supported("Cover of Winter");
    let cover = t.battlefield(P0, "Cover of Winter");
    t.g.add_counters(obj(cover), "age", x, None);
    cover
}

#[test]
fn two_covers_of_winter_each_prevent_their_own_part_of_a_tramplers_damage() {
    cr!("615.1a", "616.1", "616.1e", "616.1f", "510.2", "702.19b");
    interp!("per-source-prevention-division");
    ruling!(
        "Cover of Winter",
        "you decide how to divide the prevention effect"
    );
    // Colossal Dreadmaw (6/6 trample) blocked by Wall of Wood (0/3) assigns 3 to the Wall and
    // 3 to P0. Each Cover has 2 age counters, and P0 divides each Cover's 2 as 1 to the Wall
    // and 1 to P0. P0 then chooses which Cover applies first to the Wall's damage and which
    // to P0's (every combination is tried): either way each Cover prevents 1 of each, so the
    // Wall is dealt 1 and P0 1. (Using up the wrong part as a Cover prevents, such as the
    // other Cover's part for the same recipient, or its own part for another recipient,
    // leaves the Cover applied second to some damage with nothing to prevent.)
    for (wall_first, p0_first) in [(0, 0), (0, 1), (1, 0), (1, 1)] {
        let mut t = TestGame::new(2);
        cover_of_winter(&mut t, 2);
        cover_of_winter(&mut t, 2);
        let wurm = t.battlefield(P1, "Colossal Dreadmaw");
        let wall = t.battlefield(P0, "Wall of Wood");
        t.set_step(P1, Step::PrecombatMain);
        for _ in 0..2 {
            t.answer(P0, DecisionKind::Divide, Answer::Numbers(vec![1, 1]));
        }
        t.answer(P0, DecisionKind::Replacement, Answer::Index(wall_first));
        t.answer(P0, DecisionKind::Replacement, Answer::Index(p0_first));
        t.attack(&[(wurm, Entity::Player(P0))], &[(wall, wurm)]);
        let order = format!("first options {wall_first} and {p0_first}");
        assert_eq!(
            asked_count(&t, P0, |d| matches!(d, Decision::Divide { .. })),
            2,
            "one division for each Cover"
        );
        let choices = replacement_choices(&t);
        assert_eq!(
            choices.len(),
            2,
            "one choice for each recipient: {choices:?}"
        );
        assert!(
            choices.iter().all(|(p, o)| *p == P0 && o.len() == 2),
            "{choices:?}"
        );
        assert!(t.on_battlefield(wall), "{order}");
        assert_eq!(damage_on(&t, wall), 1, "{order}");
        assert_eq!(t.life(P0), 19, "{order}");
    }
}

/// Wraps P0's agent: each division (`Decision::Divide`) is answered by recipient from
/// `parts` (0 for the others), and each choice among replacement effects
/// (`Decision::ChooseReplacement`) takes the option containing `first`. Every decision is
/// still logged by the scripted agent, which answers the others.
fn p0_answers(t: &mut TestGame, parts: Vec<(Entity, i64)>, first: &'static str) {
    struct Picks {
        inner: Box<dyn Agent>,
        parts: Vec<(Entity, i64)>,
        first: &'static str,
    }
    impl Agent for Picks {
        fn decide(&mut self, g: &Game, p: PlayerId, d: &Decision) -> Answer {
            let scripted = self.inner.decide(g, p, d);
            match d {
                Decision::Divide { recipients, .. } => Answer::Numbers(
                    recipients
                        .iter()
                        .map(|r| {
                            self.parts
                                .iter()
                                .find(|(e, _)| e == r)
                                .map_or(0, |(_, n)| *n)
                        })
                        .collect(),
                ),
                Decision::ChooseReplacement { options } => {
                    match options.iter().position(|o| o.contains(self.first)) {
                        Some(i) => Answer::Index(i),
                        None => scripted,
                    }
                }
                _ => scripted,
            }
        }
    }
    let mut agents = t.g.agents.0.lock().unwrap();
    let inner = std::mem::replace(&mut agents[P0.idx()], Box::new(PassiveAgent));
    agents[P0.idx()] = Box::new(Picks {
        inner,
        parts,
        first,
    });
}

/// P0's Siege Mastodon (3/5), enchanted with Pariah ("All damage that would be dealt to you
/// is dealt to enchanted creature instead.").
fn mastodon_with_pariah(t: &mut TestGame) -> ObjectId {
    supported("Pariah");
    let mastodon = t.battlefield(P0, "Siege Mastodon");
    let pariah = t.battlefield(P0, "Pariah");
    assert!(
        t.g.attach(pariah, obj(mastodon)),
        "Pariah enchants the Mastodon"
    );
    t.g.recompute();
    mastodon
}

/// The divisions of a prevention effect P0 was asked for: their recipients.
fn divisions(t: &TestGame) -> Vec<Vec<Entity>> {
    t.asked()
        .into_iter()
        .filter_map(|(p, d)| match d {
            Decision::Divide { recipients, .. } if p == P0 => Some(recipients),
            _ => None,
        })
        .collect()
}

/// P1 attacks with Colossal Dreadmaw (6/6 trample), blocked by P0's Siege Mastodon with
/// Pariah, and Wild Elephant (3/3 trample), blocked by P0's Wall of Wood (0/3): the
/// Dreadmaw assigns 5 to the Mastodon and 1 to P0, the Elephant 3 to the Wall. P0's Divine
/// Deflection ("Prevent the next X damage that would be dealt to you and/or permanents you
/// control this turn. If damage is prevented this way, Divine Deflection deals that much
/// damage to any target.", X = 3, at P1) is divided as `parts` (the Mastodon's, P0's, the
/// Wall's), and P0 applies the effect named `first` first to the damage to P0 (the only
/// choice among effects: Pariah's redirection or the shield). Returns the game, the
/// Mastodon and the Wall.
fn deflection_and_pariah(parts: [i64; 3], first: &'static str) -> (TestGame, ObjectId, ObjectId) {
    supported("Divine Deflection");
    let mut t = TestGame::new(2);
    let mastodon = mastodon_with_pariah(&mut t);
    let wall = t.battlefield(P0, "Wall of Wood");
    let wurm = t.battlefield(P1, "Colossal Dreadmaw");
    let elephant = t.battlefield(P1, "Wild Elephant");
    t.set_step(P1, Step::PrecombatMain);
    t.lands(P0, "Plains", 1);
    t.lands(P0, "Wastes", 3);
    let deflection = t.hand(P0, "Divine Deflection");
    t.cast(P0, deflection).x(3).target(Entity::Player(P1)).go();
    t.resolve_all();
    p0_answers(
        &mut t,
        vec![
            (obj(mastodon), parts[0]),
            (Entity::Player(P0), parts[1]),
            (obj(wall), parts[2]),
        ],
        first,
    );
    // (Both attackers have trample: their damage is assigned, and replaced, in the order
    // they attack, the Dreadmaw's first.)
    t.attack(
        &[(wurm, Entity::Player(P0)), (elephant, Entity::Player(P0))],
        &[(mastodon, wurm), (wall, elephant)],
    );
    let asked = divisions(&t);
    assert_eq!(asked.len(), 1, "{asked:?}");
    for r in [obj(mastodon), Entity::Player(P0), obj(wall)] {
        assert!(asked[0].contains(&r), "{r:?} in {asked:?}");
    }
    let choices = replacement_choices(&t);
    assert_eq!(choices.len(), 1, "{choices:?}");
    assert!(
        choices[0].1.iter().any(|o| o.contains(first)),
        "{first}: {choices:?}"
    );
    (t, mastodon, wall)
}

#[test]
fn a_divided_shield_gives_a_recipient_its_part_once_when_more_damage_is_redirected_to_it() {
    cr!("615.7", "614.9", "616.1", "616.1e", "510.2", "615.5");
    interp!("shield-divided-among-recipients");
    ruling!(
        "Divine Deflection",
        "you choose which of that damage to prevent if the chosen value for X"
    );
    // Divided as 2 to the Mastodon, 0 to P0 and 1 to the Wall. The damage to P0 is
    // redirected to the Mastodon, before or after the shield is applied to it (both are
    // tried). Either way the Mastodon's part is all the shield prevents of the Dreadmaw's
    // damage to it: the Mastodon is dealt 6 - 2 = 4, and the Wall's part is still there for
    // the Wall, which is dealt 3 - 1 = 2. (Giving the Mastodon its part again for the
    // redirected 1 would leave the Wall nothing: it would be dealt 3 and die.)
    for first in ["Pariah", "Divine Deflection"] {
        let (t, mastodon, wall) = deflection_and_pariah([2, 0, 1], first);
        assert_eq!(t.life(P0), 20, "{first} first");
        assert!(t.on_battlefield(mastodon), "{first} first");
        assert_eq!(damage_on(&t, mastodon), 4, "{first} first");
        assert!(
            t.on_battlefield(wall),
            "{first} first: the Wall keeps its part"
        );
        assert_eq!(damage_on(&t, wall), 2, "{first} first");
        // The 3 prevented in all.
        assert_eq!(t.life(P1), 17, "{first} first");
    }
}

/// Fails until fix: a part put on damage that's redirected before the effect is applied to
/// it is lost (`per_source_division` is keyed by the damage's recipient, and the redirected
/// damage finds the new recipient's part, already used), so the shield prevents 2, not 3.
#[test]
fn a_divided_shields_part_goes_with_damage_redirected_before_the_shield_is_applied() {
    cr!("615.7", "614.9", "616.1", "616.1e", "510.2");
    interp!("shield-divided-among-recipients");
    ruling!(
        "Divine Deflection",
        "you choose which of that damage to prevent if the chosen value for X"
    );
    // Divided as 1 to each recipient. With the shield applied first to the damage to P0,
    // that 1 is prevented there; with Pariah first, it's dealt to the Mastodon instead, with
    // the part P0 put on it. Either way the whole shield is used: the Mastodon is dealt
    // 6 - 2 = 4 and survives, the Wall 3 - 1 = 2, and Divine Deflection deals 3 to P1.
    for first in ["Pariah", "Divine Deflection"] {
        let (t, mastodon, wall) = deflection_and_pariah([1, 1, 1], first);
        assert_eq!(t.life(P0), 20, "{first} first");
        assert!(t.on_battlefield(mastodon), "{first} first");
        assert_eq!(damage_on(&t, mastodon), 4, "{first} first");
        assert_eq!(damage_on(&t, wall), 2, "{first} first");
        assert_eq!(t.life(P1), 17, "{first} first: the whole shield is used");
    }
}

/// Fails until fix: as for the shield above, Cover of Winter's part for P0 is lost when the
/// damage to P0 is redirected before the Cover is applied to it, so 1 of the Dreadmaw's
/// damage is prevented, not X = 2.
#[test]
fn a_cover_of_winter_part_goes_with_damage_redirected_before_the_cover_is_applied() {
    cr!("615.1a", "614.9", "616.1", "616.1e", "702.19b");
    interp!("per-source-prevention-division");
    ruling!(
        "Cover of Winter",
        "as long as the total amount of that creature's damage that would be prevented is X"
    );
    // Colossal Dreadmaw blocked by the Mastodon with Pariah assigns 5 to it and 1 to P0, and
    // P0 divides the Cover's X = 2 as 1 to each. Whichever P0 applies first to the damage to
    // P0, 2 of the Dreadmaw's 6 damage is prevented: the Mastodon is dealt 4 and survives.
    for first in ["Pariah", "Cover of Winter"] {
        let mut t = TestGame::new(2);
        cover_of_winter(&mut t, 2);
        let mastodon = mastodon_with_pariah(&mut t);
        let wurm = t.battlefield(P1, "Colossal Dreadmaw");
        t.set_step(P1, Step::PrecombatMain);
        p0_answers(
            &mut t,
            vec![(obj(mastodon), 1), (Entity::Player(P0), 1)],
            first,
        );
        t.attack(&[(wurm, Entity::Player(P0))], &[(mastodon, wurm)]);
        assert_eq!(divisions(&t).len(), 1, "{:?}", divisions(&t));
        assert_eq!(t.life(P0), 20, "{first} first");
        assert!(t.on_battlefield(mastodon), "{first} first");
        assert_eq!(damage_on(&t, mastodon), 4, "{first} first");
    }
}
