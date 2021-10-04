pub fn home() -> String {
    Page {
        title: "",
        body: markup::new! {
            p {
                "Welcome to " @env!("CARGO_PKG_NAME") "! This is a special HTTP proxy. To get started, " a[href = "/www.google.com"] { "try me" } "!"
            }
            h2 { "API" }

            h3 { .request "GET /" }
            .response {
                "Returns this page."
            }

            h3 { .request "GET /<target-uri>" }
            .response {
                "Returns the external resource at " code { "target-uri" } " via the scheme from the proxy URL and with the user agent " code { @super::USER_AGENT } ". " @env!("CARGO_PKG_NAME") " will follow up to "  @super::MAX_REDIRECT_COUNT " redirects."
            }

            h3 { .request "HEAD <uri>" }
            .response {
                "Returns the result of an equivalent " code { "GET" } " request without the message body. See " a[href = "https://httpwg.org/specs/rfc7231.html#rfc.section.4.3.2"] { "RFC 7231 §4.3.2" } "."
            }

            h3 { .request "OPTIONS /<target-uri>" }
            .response {
                "Returns an " code { "Allow" } " header that lists all methods supported by the external resource at " code { "target-uri" } ". See " a[href = "https://httpwg.org/specs/rfc7231.html#rfc.section.4.3.7"] { "RFC 7231 §4.3.7" } "."
            }

            h3 { .request "OPTIONS /" }
            h3 { .request "OPTIONS *" }
            .response {
                "Returns an " code { "Allow" } " header that lists all methods supported by " @env!("CARGO_PKG_NAME") ". See " a[href = "https://httpwg.org/specs/rfc7231.html#rfc.section.4.3.7"] { "RFC 7231 §4.3.7" } "."
            }
        }
    }
    .to_string()
}

markup::define! {
    Page<'a, B: markup::Render>(title: &'a str, body: B) {
        @markup::doctype()
        html[lang = "en"] {
            head {
                meta[charset = "utf-8"] {}
                title {
                    @if !title.is_empty() {
                        @title " - "
                    }

                    @env!("CARGO_PKG_NAME")
                }
                style {
                    @indoc::indoc! {"
                        body {
                            background-color: White;
                            font-family: sans-serif;
                        }

                        code, .request {
                            background-color: GhostWhite;
                            border-radius: 5%;
                            color: RebeccaPurple;
                            font-family: monospace;
                        }

                        .api_def {
                            padding:
                        }

                        .request {
                            display: inline-block;
                        }

                        .response {
                            padding-left: 20pt;
                        }
                    "}
                }
            }
            body {
                h1 {
                    @if title.is_empty() {
                        @env!("CARGO_PKG_NAME")
                    } else {
                        @title
                    }
                }
                @body
            }
        }
    }
}
