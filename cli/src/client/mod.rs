use anyhow::Result;
use proto::api::{archive, inbox, upload};
use reqwest::{Body, Method, RequestBuilder, Url};
use tokio::io::{AsyncRead, AsyncWrite};
use tokio_util::codec::{BytesCodec, FramedRead, FramedWrite};

pub use auth::Auth;
use auth::Session;
use futures::{StreamExt, TryStreamExt, SinkExt};

pub mod auth;

pub struct Client {
    base_url: Url,
    session: Session,
    client: reqwest::Client,
}

impl Client {
    pub async fn new(base_url: &str, auth: Auth) -> Result<Self> {
        let base_url = if !base_url.ends_with('/') {
            Url::parse(&format!("{}/api/", base_url))
        } else {
            Url::parse(base_url)
        }?;

        let client = reqwest::Client::builder()
            .build()?;

        let session = Session::authenticate(auth, &base_url, &client).await?;

        return Ok(Self {
            base_url,
            session,
            client,
        });
    }

    fn request(&mut self,
               method: Method,
               path: &str) -> Result<RequestBuilder> {
        let url = self.base_url.join(path.trim_start_matches('/'))?;

        let request = self.client.request(method, url);

        return Ok(request);
    }

    pub async fn upload(&mut self, r: impl AsyncRead + Send + Sync + 'static) -> Result<upload::UploadResponse> {
        let request = self.request(Method::POST, "/upload")?;

        let r = FramedRead::new(r, BytesCodec::new());
        let r = Body::wrap_stream(r);
        let request = request.body(r)
            .header(reqwest::header::CONTENT_TYPE, "application/pdf");


        let response = self.session.send(request).await?
            .error_for_status()?;

        return Ok(response.json().await?);
    }

    pub async fn inbox_list(&mut self) -> Result<inbox::ListResponse> {
        let request = self.request(Method::GET, "/inbox")?;

        let response = self.session.send(request).await?
            .error_for_status()?;

        return Ok(response.json().await?);
    }

    pub async fn inbox_bundle(&mut self, id: &str) -> Result<inbox::GetResponse> {
        let request = self.request(Method::GET, &format!("/inbox/{}", id))?;

        let response = self.session.send(request).await?
            .error_for_status()?;

        return Ok(response.json().await?);
    }

    pub async fn inbox_fragment(&mut self, id: &str, kind: &str, w: impl AsyncWrite + Send + Sync + 'static) -> Result<()> {
        let request = self.request(Method::GET, &format!("/inbox/{}/{}", id, kind))?;

        let response = self.session.send(request).await?
            .error_for_status()?;

        let w = FramedWrite::new(w, BytesCodec::new());

        return response.bytes_stream()
            .err_into()
            .forward(w.sink_err_into()).await;
    }

    pub async fn inbox_delete(&mut self, id: &str) -> Result<()> {
        let request = self.request(Method::DELETE, &format!("/inbox/{}", id))?;

        self.session.send(request).await?
            .error_for_status()?;

        return Ok(());
    }

    pub async fn inbox_archive(&mut self, id: &str, data: &inbox::ArchiveRequest) -> Result<()> {
        let request = self.request(Method::POST, &format!("/inbox/{}", id))?;
        let request = request.json(data);

        self.session.send(request).await?
            .error_for_status()?;

        return Ok(());
    }

    pub async fn archive_bundle(&mut self, id: &str) -> Result<archive::BundleResponse> {
        let request = self.request(Method::GET, &format!("/archive/{}", id))?;

        let response = self.session.send(request).await?
            .error_for_status()?;

        return Ok(response.json().await?);
    }

    pub async fn archive_fragment(&mut self, id: &str, kind: &str, w: impl AsyncWrite + Send + Sync + 'static) -> Result<()> {
        let request = self.request(Method::GET, &format!("/archive/{}/{}", id, kind))?;

        let response = self.session.send(request).await?
            .error_for_status()?;

        let w = FramedWrite::new(w, BytesCodec::new());

        return response.bytes_stream()
            .err_into()
            .forward(w.sink_err_into()).await;
    }

    pub async fn archive_search(&mut self, query: &str) -> Result<archive::SearchResponse> {
        let request = self.request(Method::GET, "/archive")?;
        let request = request.query(&[("query", query)]);

        let response = self.session.send(request).await?
            .error_for_status()?;

        return Ok(response.json().await?);
    }
}