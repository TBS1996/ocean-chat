<p align="center">
  <img src="assets/logo.png" alt="Logo" width="150" />
</p>
<h1 align="center">Ocean Chat</h1>

A program that will pair you up with the person who has the closest personality to your own!

Personality is defined by the big 5 test. You can take the test here: https://openpsychometrics.org/tests/IPIP-BFFM/

"Closest personality" is found by taking the euclidean distance between the 5 dimension scores of two personalities

**This project is w.i.p.**

## For developing locally:
Run the backend with `cargo run --features server`.

Open project in another terminal, do `dx serve`.

Open in yet another terminal, do `dx serve --port 8081` (or any free port other than first one)

> [!NOTE]
> Client-side code won't compile if you haven't run the backend, as it generates some textfiles that will be packaged into the client's binary

## For building locally:
To build the backend, run `cargo build --features server --profile release-server`

To build the frontend, run `dx build --release`






            div {
                width: "500px",
                margin_left: "100px",
                match peer_score() {
                    Some(score) => {
                        let more_similar = format!("{:.1}", scores.percentage_similarity(score));
                        rsx! {
                            div {
                                h4 { "Your peer's personality:" }
                                { score_cmp(scores, score) }
                                p {
                                    "{more_similar}% of people are more similar to you than your peer."
                                }
                            }
                        }
                    },
                    None => { rsx!{""} },
                }
            }
