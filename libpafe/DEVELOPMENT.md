DEVELOPMENT
===========

This document summarizes the development methodology used for the
libpafe Rust implementation and provides guidance for future
contributors on how to maintain the clean-room guarantees.

Clean-room principles
---------------------

1. Specification-first: Implementations must be based on publicly
   available specifications (FeliCa, PN532/PN533/RCS956) and original design
   notes — not by copying or translating GPL-licensed C source.

2. Independent design: Protocol encoders, parsers and state machines
   must be designed from written requirements and independent tests.

3. Documentation trail: When making design decisions influenced by
   external documents (datasheets, RFCs), cite the exact source in
   code comments or documentation.

4. No direct references to GPL sources: Developers should not read or
   open the original GPL C code during implementation; if reading is
   necessary for historical reasons, document the access and consider
   reassigning the work to a developer who did not view the GPL code.

Tests and validation
--------------------

- Use property-based tests (proptest) to validate mathematical
  invariants (checksums, framing). These tests demonstrate the
  independent correctness of the implementation.
- Keep test fixtures independent of sample values found in existing
  GPL projects; prefer deterministic synthetic values.
