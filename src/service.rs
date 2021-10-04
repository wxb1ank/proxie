mod html;

use hyper::{
    http::uri::Scheme,
    {Body, Client, Method, Request, Response, StatusCode, Uri},
};
use hyper_rustls::HttpsConnector;

const MAX_REDIRECT_COUNT: usize = 5;
static USER_AGENT: &str = concat!(env!("CARGO_PKG_NAME"), "/", env!("CARGO_PKG_VERSION"));

pub async fn respond(req: Request<Body>) -> Result<Response<Body>, http::Error> {
    let (method, uri) = (req.method(), req.uri());
    tracing::debug!("{} {}", method, uri);

    match method {
        &Method::GET => get(uri).await,
        &Method::HEAD => get(uri).await.map(|mut res| {
            *res.body_mut() = Body::empty();
            res
        }),
        &Method::OPTIONS => Response::builder()
            .status(StatusCode::NO_CONTENT)
            .header("Allow", "GET, HEAD, OPTIONS")
            .body(Body::empty()),
        _ => Response::builder()
            .status(StatusCode::METHOD_NOT_ALLOWED)
            .body(Body::empty()),
    }
}

pub async fn get(uri: &Uri) -> Result<Response<Body>, http::Error> {
    if let Some(maybe_uri) = find_external_uri(uri) {
        get_external(&maybe_uri?, 0).await
    } else {
        get_internal()
    }
}

fn find_external_uri(uri: &Uri) -> Option<Result<Uri, http::Error>> {
    let mut path_components = uri.path().split('/').filter(|it| !it.is_empty());

    path_components.next().map(|host| {
        Uri::builder()
            .scheme(uri.scheme().cloned().unwrap_or(Scheme::HTTPS))
            .authority(host)
            .path_and_query({
                // After calling [`Iterator::next`], `path_components` no longer contains
                // `host`, and it now represents the full external path.
                std::iter::once("")
                    .chain(path_components)
                    .chain(uri.query())
                    .collect::<Vec<&str>>()
                    .join("/")
            })
            .build()
    })
}

async fn get_external(uri: &Uri, redirect_count: usize) -> Result<Response<Body>, http::Error> {
    if redirect_count >= MAX_REDIRECT_COUNT {
        return Response::builder()
            .status(StatusCode::GATEWAY_TIMEOUT)
            .body({
                // Document::default()
                //     .title("Too many redirects")
                //     .body(&indoc::formatdoc! {
                //         "
                //         {} couldn't find the requested resource after {} redirects.
                //         ",
                //         env!("CARGO_PKG_NAME"),
                //         MAX_REDIRECT_COUNT,
                //     })
                //     .render()
                //     .unwrap()
                //     .into()
                Body::empty()
            });
    }

    let res = get_external_impl(uri).await?;

    if !res.status().is_redirection() {
        return Ok(res);
    }

    if let Some(value) = res.headers().get("Location") {
        let maybe_uri = value.to_str().ok().and_then(|it| Uri::try_from(it).ok());

        if let Some(ref uri) = maybe_uri {
            get_external_impl(uri).await
        } else {
            Response::builder().status(StatusCode::BAD_GATEWAY).body({
                // Document::default()
                //     .title("Invalid redirect location")
                //     .body(&indoc::formatdoc! {
                //         "
                //             While locating your resource, {} was redirected to an invalid location.
                //             ",
                //         env!("CARGO_PKG_NAME"),
                //     })
                //     .render()
                //     .unwrap()
                //     .into()
                Body::empty()
            })
        }
    } else {
        Response::builder().status(StatusCode::BAD_GATEWAY).body({
            // Document::default()
            //     .title("Unknown redirect location")
            //     .body(&indoc::formatdoc! {
            //         "
            //             While locating your resource, {} was redirected to an unspecified location.
            //             ",
            //         env!("CARGO_PKG_NAME"),
            //     })
            //     .render()
            //     .unwrap()
            //     .into()
            Body::empty()
        })
    }
}

async fn get_external_impl(uri: &Uri) -> Result<Response<Body>, http::Error> {
    Client::builder()
        .build(HttpsConnector::with_native_roots())
        .request({
            let req = Request::builder()
                .method(Method::GET)
                .uri(uri)
                .header("User-Agent", USER_AGENT)
                .body(Body::empty())?;
            tracing::trace!("{:#?}", req);

            req
        })
        .await
        .or_else(|_| {
            Response::builder().status(StatusCode::NOT_FOUND).body({
                // Document::default()
                //     .title("Not found")
                //     .body(&indoc::formatdoc! {
                //         "
                //             {} couldn't find the requested resource at <code>{}</code>.
                //             ",
                //         env!("CARGO_PKG_NAME"),
                //         uri,
                //     })
                //     .render()
                //     .unwrap()
                //     .into()
                Body::empty()
            })
        })
}

fn get_internal() -> Result<Response<Body>, http::Error> {
    Response::builder()
        .status(StatusCode::OK)
        .body(html::home().into())
}
