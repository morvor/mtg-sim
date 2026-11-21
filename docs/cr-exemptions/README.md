# CR exemptions

One `.tsv` file per area (e.g. `2xx-card-parts.tsv`), each line `<rule id><TAB><reason>`.
Exempt only rules with no engine-testable behavior (purely informational or
presentational text, tournament/draft procedures). Everything else needs a test citing it
with `cr!(...)`. `mtg-tools cr-coverage` reads every file here.

`examples.tsv` is different: it exempts the rules' worked examples ("Example:" paragraphs),
each line `<rule id><TAB><distinctive substring><TAB><reason>`, where the substring must match
exactly one of that rule's examples. Tests cite examples with
`cr_example!("613.10", "distinctive substring")`, which also counts as citing the rule.
`mtg-tools cr-coverage --examples [--range 600-611]` lists each example with its status.
