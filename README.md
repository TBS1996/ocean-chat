# Oceanchat


for developing:

run the backend with `cargo run --features server`.

open project in another terminal, do `dx serve`.
open in yet another terminal, do `dx serve --port 8081` (or any free port other than first one)

atm theres just basic pairing and communication. click submit on both, and they'll go to a 'chat page'. write something in the field, hit enter, then inspect the other terminal's console, the message should have arrived there.


w.i.p.

A program that will pair you up with the person who has the closest personality to your own!

Personality is defined by the big 5 test. You can take the test here: https://openpsychometrics.org/tests/IPIP-BFFM/

"closest personality" is found by taking the euclidean distance between the 5 dimension scores of two personalities


