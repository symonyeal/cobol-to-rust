# LinkedIn drafts

Drafts only. Not published. These working drafts await phase-two evidence. Replace the bracketed result fields and check each claim against the final report before use.

## 1. Choosing the next migration case

After the first three COBOL-to-Rust ports, I chose a small command-line calculator for the next case. Its operands use packed decimal, a representation common in COBOL business calculations.

The selection question was specific: could the same observation-based approach preserve argument conversion, unsigned arithmetic and division behavior under the pinned GnuCOBOL environment?

I inspected the source before choosing it and required a matching source revision and licence. Larger examples brought transaction-system or dialect dependencies that would make the next experiment harder to interpret.

[Insert admitted revision, scenario count and result link after validation.]

The scope remains one program and a finite corpus under one execution profile. Copybooks, indexed files and mainframe equivalence need their own evidence.

## 2. Let the reference settle numeric assumptions

A four-digit number looks straightforward until it passes through a legacy runtime's input conversion and an unsigned destination field.

For the next COBOL-to-Rust case, I captured the unchanged reference program before implementing the port. The corpus asks concrete questions about malformed arguments, subtraction below zero, fractional division and unsupported operators.

[Insert one observed behavior and the implementation choice it required. Link the retained evidence.]

The compatibility requirement includes the program's formatting and existing defects. A business correction would need a separate requirement and a separate comparison.

## 3. Make a passing migration check earn its result

A comparison that accepts a port should also reject a known defect.

For the packed-decimal calculator case, I deliberately changed [insert exact mutation] in the Rust implementation. The workbench returned [insert status] and identified [insert artifact and byte offset]. The reference capture remained unchanged.

After restoring the implementation, [insert checked local and clean Windows CI results, revision and link].

Those results support the declared corpus under the recorded environment. They do not establish universal equivalence or a performance improvement; performance was not measured.
