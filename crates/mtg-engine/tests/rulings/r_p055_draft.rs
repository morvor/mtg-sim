//! Rulings batch P055 — drafting additional cards (CR 905.2): Cogwork Librarian and
//! Leovold's Operative, one card at a time and several of them at once.

use crate::r_s01_common::supported;
use mtg_engine::card::card;
use mtg_engine::draft::{Draft, DraftError, DraftOption, DraftStyle};
use mtg_engine::testing::*;
use mtg_engine::*;

/// A one-player draft (the pack comes back to P0 after each pick) of one pack.
fn solo(pack: &[&str]) -> Draft {
    Draft::new(
        vec![vec![pack.iter().map(|n| card(n)).collect()]],
        DraftStyle::Conspiracy,
        1,
    )
}

fn additional(with: &str, extra: usize) -> DraftOption {
    DraftOption::Additional {
        with: with.into(),
        extra,
    }
}

fn drafted_names(d: &Draft) -> Vec<String> {
    d.drafted(P0)
        .iter()
        .map(|c| c.card.name.to_string())
        .collect()
}

#[test]
fn cogwork_librarians_two_cards_are_drafted_one_at_a_time() {
    cr!("905.2", "905.2c");
    ruling!(
        "Cogwork Librarian",
        "The two cards you draft are drafted one at a time, in case one of the cards cares about when it was drafted or what the next card you draft is."
    );
    supported("Cogwork Librarian");
    // Aether Searcher: "Reveal the next card you draft and note its name." Drafted first
    // of the two, it notes the second; drafted second, it notes the card after.
    for (picks, extra, noted) in [(0usize, 1usize, "Lightning Bolt"), (1, 0, "Shock")] {
        let mut d = solo(&[
            "Cogwork Librarian",
            "Aether Searcher",
            "Lightning Bolt",
            "Shock",
            "Opt",
        ]);
        d.pick(P0, &[0]).unwrap();
        d.pick_with(P0, &[picks], Some(additional("Cogwork Librarian", extra)))
            .unwrap();
        let first = if picks == 0 {
            "Aether Searcher"
        } else {
            "Lightning Bolt"
        };
        assert_eq!(drafted_names(&d)[0], first);
        // The next pick (the pack still has Shock, Opt and the Librarian).
        let shock = d.pack(P0).iter().position(|c| c.name == "Shock").unwrap();
        d.pick(P0, &[shock]).unwrap();
        let (_, info) = d.finish();
        let note = info
            .notes
            .iter()
            .find(|n| n.card == "Aether Searcher")
            .unwrap();
        assert_eq!(note.noted_name.as_deref(), Some(noted), "first: {first}");
    }
}

#[test]
fn several_cogwork_librarians_while_drafting_from_one_pack() {
    cr!("905.2", "905.2c");
    ruling!(
        "Cogwork Librarian",
        "You may use the ability of more than one Cogwork Librarian while drafting from one booster pack. For example you can draft three cards from one booster pack and put two Cogwork Librarians into that booster pack."
    );
    let mut d = solo(&[
        "Cogwork Librarian",
        "Cogwork Librarian",
        "Lightning Bolt",
        "Shock",
        "Opt",
        "Duress",
    ]);
    d.pick(P0, &[0]).unwrap();
    d.pick(P0, &[0]).unwrap();
    // Pack: Bolt, Shock, Opt, Duress. Three cards, two Librarians.
    d.pick_with_options(
        P0,
        &[0],
        vec![
            additional("Cogwork Librarian", 1),
            additional("Cogwork Librarian", 2),
        ],
    )
    .unwrap();
    assert_eq!(drafted_names(&d), vec!["Lightning Bolt", "Shock", "Opt"]);
    let mut pack: Vec<String> = d.pack(P0).iter().map(|c| c.name.to_string()).collect();
    pack.sort();
    assert_eq!(
        pack,
        vec!["Cogwork Librarian", "Cogwork Librarian", "Duress"]
    );
    // One Librarian can't be used twice in one pick.
    let mut d = solo(&["Cogwork Librarian", "Lightning Bolt", "Shock", "Opt"]);
    d.pick(P0, &[0]).unwrap();
    assert_eq!(
        d.pick_with_options(
            P0,
            &[0],
            vec![
                additional("Cogwork Librarian", 1),
                additional("Cogwork Librarian", 2),
            ],
        ),
        Err(DraftError::InvalidOption)
    );
    assert!(drafted_names(&d) == vec!["Cogwork Librarian"]);
}

#[test]
fn several_leovolds_operatives_skip_as_many_packs() {
    cr!("905.2", "905.2c");
    ruling!(
        "Leovold's Operative",
        "You can turn more than one Leovold's Operative face down to draft additional cards from one booster pack. However, for each Leovold's Operative you turn face down, you will skip drafting a card from an additional future booster pack. For example, if you turn three Leovold's Operatives face down as you draft a card from a booster pack, you may draft three additional cards from that booster pack. You then won't draft a card from the next three booster packs you are passed (and/or open)."
    );
    supported("Leovold's Operative");
    let mut d = solo(&[
        "Leovold's Operative",
        "Leovold's Operative",
        "Leovold's Operative",
        "Lightning Bolt",
        "Shock",
        "Opt",
        "Duress",
        "Grizzly Bears",
        "Hill Giant",
        "Craw Wurm",
        "Forest",
    ]);
    for _ in 0..3 {
        d.pick(P0, &[0]).unwrap();
    }
    // Pack: Bolt, Shock, Opt, Duress, Bears, Giant, Wurm, Forest. Four cards at once.
    d.pick_with_options(
        P0,
        &[0],
        vec![
            additional("Leovold's Operative", 1),
            additional("Leovold's Operative", 2),
            additional("Leovold's Operative", 3),
        ],
    )
    .unwrap();
    assert_eq!(d.drafted(P0).len(), 7);
    assert!(d.drafted(P0)[..3].iter().all(|c| !c.face_up));
    // The next three packs are passed without drafting from them.
    for _ in 0..3 {
        assert_eq!(d.pick(P0, &[0]), Err(DraftError::InvalidPick));
        d.pick(P0, &[]).unwrap();
    }
    assert_eq!(d.drafted(P0).len(), 7);
    // Then P0 drafts again.
    d.pick(P0, &[0]).unwrap();
    assert_eq!(d.drafted(P0).len(), 8);
}
