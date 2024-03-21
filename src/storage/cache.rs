use std::collections::HashMap;
use std::error;
use std::error::Error;
use std::fs;
use std::io;
use std::path;
use std::path::Path;
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::Mutex;
use std::thread;

use fsr::*;
use rand::distributions::Alphanumeric;
use rand::{thread_rng, Rng};
use redb::{ReadableTable, TableDefinition};
use reqwest::header as rh;
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc;
use tokio_util::codec::BytesCodec;
use tokio_util::codec::FramedRead;

/// All the information we have about a given URL.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct CacheRecord {
    /// The path to the cached response body on disk.
    pub path: String,
    /// The value of the Last-Modified header in the original response.
    pub last_modified: Option<String>,
    /// The value of the Etag header in the original response.
    pub etag: Option<String>,
    /// The value of the whether it has been uploaded to the server
    pub synchronized: bool,
}

pub struct Event {
    pub done: bool,
    /// The path to the cached file on disk.
    pub path: String,
    /// The url to update
    pub url: String,
}

impl redb::RedbValue for CacheRecord {
    type SelfType<'a> = CacheRecord
        where
            Self: 'a;
    type AsBytes<'a> = Vec<u8>
        where
            Self: 'a;

    fn fixed_width() -> Option<usize> {
        None
    }

    fn from_bytes<'a>(data: &'a [u8]) -> CacheRecord
    where
        Self: 'a,
    {
        let record: Result<CacheRecord, serde_json::Error> = serde_json::from_slice(data);
        match record {
            Ok(record) => record,
            Err(_) => CacheRecord {
                path: "".to_string(),
                last_modified: None,
                etag: None,
                synchronized: false,
            },
        }
    }

    fn as_bytes<'a, 'b: 'a>(value: &'a Self::SelfType<'b>) -> Vec<u8>
    where
        Self: 'a,
        Self: 'b,
    {
        serde_json::to_vec(value).unwrap_or_default()
    }

    fn type_name() -> redb::TypeName {
        redb::TypeName::new("name")
    }
}

fn canonicalize_db_path(path: path::PathBuf) -> Result<path::PathBuf, Box<dyn error::Error>> {
    Ok({
        let parent = path.parent().unwrap_or(path::Path::new("."));

        // Otherwise, canonicalize it so we can reliably compare instances.
        // The weird joining behaviour is because we require the path
        // to exist, but we don't require the filename to exist.
        parent
            .canonicalize()?
            .join(path.file_name().unwrap_or(std::ffi::OsStr::new("")))
    })
}

/// Represents the database that describes the contents of the cache.
pub struct CacheDB {
    path: path::PathBuf,
    db: Mutex<redb::Database>,
}

const TABLE: TableDefinition<&str, CacheRecord> = TableDefinition::new("cached");

impl CacheDB {
    /// Create a cache database in the given file.
    pub fn new(path: path::PathBuf) -> Result<CacheDB, Box<dyn error::Error>> {
        let path = canonicalize_db_path(path)?;
        debug!("Creating cache metadata in {:?}", path);
        let db = redb::Database::create(&path)?;
        // Package up the return value first, so we can use .query()
        // instead of wrangling sqlite directly.
        let res = CacheDB {
            path,
            db: Mutex::new(db),
        };
        Ok(res)
    }

    /// Return what the DB knows about a URL, if anything.
    pub fn get(&self, url: &str) -> Result<CacheRecord, Box<dyn error::Error>> {
        let db = self.db.lock().unwrap();
        let read_txn = db.begin_read()?;
        let table = read_txn.open_table(TABLE)?;
        let value = table.get(url)?.unwrap().value();
        Ok(value)
    }

    /// Record information about this information in the database.
    pub fn set(&self, url: &str, record: CacheRecord) -> Result<(), Box<dyn error::Error>> {
        let mut old_file = String::new();
        let db = self.db.lock().unwrap();
        let write_txn = db.begin_write()?;
        {
            let mut table = write_txn.open_table(TABLE)?;
            let old_value = table.insert(url, &record)?;
            if let Some(old_value) = old_value {
                old_file = old_value.value().path;
            }
        }
        write_txn.commit()?;

        if !old_file.is_empty() && !old_file.eq(&record.path) {
            // Remove expired cache files
            let _ = fs::remove_file(old_file);
        }
        Ok(())
    }
}

impl std::fmt::Debug for CacheDB {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "CacheDB {{path: {:?}}}", self.path)
    }
}

fn make_random_file<P: AsRef<path::Path>>(
    parent: P,
    extension: &str,
) -> Result<(fs::File, path::PathBuf), Box<dyn error::Error>> {
    let mut rng = thread_rng();

    loop {
        let mut new_path = parent.as_ref().join(
            (0..20)
                .map(|_| rng.sample(Alphanumeric) as char)
                .collect::<String>(),
        );

        new_path.set_extension(extension);
        match fs::OpenOptions::new()
            .create_new(true)
            .write(true)
            .open(&new_path)
        {
            Ok(handle) => return Ok((handle, new_path)),
            Err(e) => {
                if e.kind() != io::ErrorKind::AlreadyExists {
                    // An actual error, we'd better report it!
                    return Err(e.into());
                }

                // Otherwise, we just picked a bad name. Let's go back
                // around the loop and try again.
            }
        };
    }
}

fn header_as_string(headers: &rh::HeaderMap, key: &rh::HeaderName) -> Option<String> {
    headers.get(key).and_then(|value| match value.to_str() {
        Ok(s) => Some(s.into()),
        Err(err) => {
            warn!("Header {} contained weird value: {}", key, err);
            None
        }
    })
}

/// Represents a local cache of HTTP resources.
///
/// Whenever you ask it for the contents of a URL,
/// it will re-use a previously-downloaded copy
/// if the resource has not changed on the server.
/// Otherwise,
/// it will download the new version and use that instead.
///
/// [`Cache`]: struct.Cache.html
#[derive(Debug, Clone)]
pub struct Cache {
    root: path::PathBuf,
    db: Arc<CacheDB>,
    client: reqwest::blocking::Client,
    file_lock: Arc<Mutex<HashMap<String, bool>>>,
    event: tokio::sync::mpsc::Sender<Event>,
}

impl Cache {
    /// Returns a Cache that wraps `client` and caches data in `root`.
    ///
    /// If the directory `root` does not exist, it will be created.
    /// If multiple instances share the same `root`
    /// (concurrently or in series),
    /// each instance will be able to re-use resources downloaded by
    /// the others.
    ///
    /// For best results,
    /// choose a `root` that is directly attached to
    /// the computer running your program,
    /// such as somewhere inside the `%LOCALAPPDATA%` directory on Windows,
    /// or the `$XDG_CACHE_HOME` directory on POSIX systems.
    ///
    /// `client` should almost certainly be a `reqwest::Client`,
    /// but you can use any type that implements [`reqwest_mock::Client`]
    /// if you want to use a different HTTP client library
    /// or a test double of some kind.
    ///
    ///     # use std::error::Error;
    ///     # use std::fs::File;
    ///     # use std::path::PathBuf;
    ///     # fn get_my_resource() -> Result<(), Box<dyn Error>> {
    ///     let mut cache = Cache::new(
    ///         PathBuf::from("my_cache_directory"),
    ///     )?;
    ///     # Ok(())
    ///     # }
    ///
    ///
    /// Errors
    /// ======
    ///
    /// This method may return an error:
    ///
    ///   - if `root` cannot be created, or cannot be written to
    ///   - if the metadata database cannot be created or cannot be written to
    ///   - if the metadata database is corrupt
    ///
    /// In all cases, it should be safe to blow away the entire directory
    /// and start from scratch.
    /// It's only cached data, after all.
    pub fn new(root: &str) -> Result<Cache, Box<dyn error::Error>> {
        let root = Path::new(root).to_path_buf();

        fs::DirBuilder::new().recursive(true).create(&root)?;

        let db = CacheDB::new(root.join("cache.db"))?;

        let client = reqwest::blocking::Client::builder()
            .use_rustls_tls()
            .build()?;

        // Create content download dir
        let content_dir = root.join("download");
        fs::DirBuilder::new().recursive(true).create(&content_dir)?;

        // Create content upload dir
        let content_dir = root.join("upload");
        fs::DirBuilder::new().recursive(true).create(&content_dir)?;

        let (tx, rx) = mpsc::channel::<Event>(10);

        let cached = Cache {
            root,
            db: Arc::new(db),
            client,
            file_lock: Arc::new(Mutex::new(HashMap::new())),
            event: tx,
        };

        let worker = cached.clone();
        thread::spawn(move || {
            worker.worker_thread(rx);
        });
        Ok(cached)
    }

    pub fn close(&self) {
        let event = Event {
            done: true,
            path: "".to_string(),
            url: "".to_string(),
        };

        let result = self.event.blocking_send(event);
        match result {
            Ok(_) => (),
            Err(e) => {
                error!("{}", e);
            }
        }
    }

    async fn reqwest_multipart_form(
        &self,
        client: reqwest::Client,
        file: &str,
        url: &str,
    ) -> Result<reqwest::Response, Box<dyn Error>> {
        let file_path = self.root.join(file);

        let file = tokio::fs::File::open(&file_path).await?;

        // read file body stream
        let stream = FramedRead::new(file, BytesCodec::new());
        let file_body = reqwest::Body::wrap_stream(stream);

        let mut file_name = "record.wav";
        if let Some(name) = file_path.file_name() {
            let name = name.to_str().unwrap_or_default();
            file_name = name;
        }

        // make form part of file
        let part = reqwest::multipart::Part::stream(file_body)
            .file_name(file_name.to_string())
            .mime_str("application/octet-stream")?;

        // create the multipart form
        let form = reqwest::multipart::Form::new().part("file", part);

        // send request
        let response = client.post(url).multipart(form).send().await?;
        Ok(response)
    }

    fn worker_thread(&self, mut rx: mpsc::Receiver<Event>) {
        let rt = tokio::runtime::Runtime::new().unwrap();
        let client = reqwest::Client::builder().use_rustls_tls().build().unwrap();
        rt.block_on(async {
            loop {
                let recv = rx.recv().await;
                match recv {
                    Some(recv) => {
                        if recv.done {
                            info!("shutdown");
                            break;
                        }
                        let response = self
                            .reqwest_multipart_form(client.clone(), &recv.path, &recv.url)
                            .await;
                        match response {
                            Ok(response) => {
                                info!("result: {:?}", response);
                                let last_modified =
                                    header_as_string(response.headers(), &rh::LAST_MODIFIED);
                                let etag = header_as_string(response.headers(), &rh::ETAG);
                                let _ = self.db.set(
                                    &recv.url,
                                    CacheRecord {
                                        path: recv.path,
                                        last_modified,
                                        etag,
                                        synchronized: true,
                                    },
                                );
                            }
                            Err(e) => {
                                error!("{}", e);
                            }
                        }
                    }
                    None => {
                        break;
                    }
                }
            }
        });
    }

    fn lock_file(&self, uri: &str, lock: bool) {
        if lock {
            loop {
                let lock = self.file_lock.lock().unwrap();
                let state = lock.get(uri);
                if state.is_none() {
                    break;
                }
                drop(lock);
                std::thread::sleep(std::time::Duration::from_millis(1000));
            }
            self.file_lock.lock().unwrap().insert(uri.to_string(), true);
        } else {
            self.file_lock.lock().unwrap().remove(uri);
        }
    }

    fn record_response(
        &self,
        url: &str,
        mut response: reqwest::blocking::Response,
    ) -> Result<PathBuf, Box<dyn error::Error>> {
        let content_dir = self.root.join("download");

        let mut extension = "wav";
        let ext = Path::new(url).extension();
        if let Some(ext) = ext {
            let ext = ext.to_str();
            if let Some(ext) = ext {
                extension = ext;
            }
        }

        let (mut handle, file_path) = make_random_file(&content_dir, extension)?;

        // We can be sure the relative path is valid UTF-8, because
        // make_random_file() just generated it from ASCII.
        let path = file_path.strip_prefix(&self.root)?.to_str().unwrap().into();

        let last_modified = header_as_string(response.headers(), &rh::LAST_MODIFIED);

        let etag = header_as_string(response.headers(), &rh::ETAG);

        let count = io::copy(&mut response, &mut handle)?;

        debug!("Downloaded {} bytes", count);

        self.db.set(
            url,
            CacheRecord {
                path,
                last_modified,
                etag,
                synchronized: true,
            },
        )?;

        Ok(file_path)
    }

    fn load_cache(&self, mut url: reqwest::Url) -> Result<PathBuf, Box<dyn error::Error>> {
        use reqwest::StatusCode;

        url.set_fragment(None);

        let response = match self.db.get(url.as_str()) {
            Ok(CacheRecord {
                path: p,
                last_modified: lm,
                etag: et,
                synchronized: sync,
            }) => {
                // We have a locally-cached copy

                // The file is not synchronized, using the local cache
                if !sync {
                    debug!("Unsynchronized caches, using the local cache");
                    return Ok(self.root.join(p));
                }

                // let's check whether the copy on the server has changed.
                let mut request =
                    reqwest::blocking::Request::new(reqwest::Method::GET, url.clone());
                if let Some(timestamp) = lm {
                    request.headers_mut().append(
                        rh::IF_MODIFIED_SINCE,
                        rh::HeaderValue::from_str(&timestamp)?,
                    );
                }
                if let Some(etag) = et {
                    request
                        .headers_mut()
                        .append(rh::IF_NONE_MATCH, rh::HeaderValue::from_str(&etag)?);
                }

                debug!("Sending HTTP request: {:?}", request);

                debug!("validation file {} {:?}", url.path(), request.headers());

                let maybe_validation = self
                    .client
                    .execute(request)
                    .and_then(|resp| resp.error_for_status());

                match maybe_validation {
                    Ok(new_response) => {
                        debug!("Got HTTP response: {:?}", new_response);

                        // If our existing cached data is still fresh...
                        if new_response.status() == StatusCode::NOT_MODIFIED {
                            // ... let's use it as is.
                            debug!("Hit cache, using the local cache data");
                            return Ok(self.root.join(p));
                        }

                        // Otherwise, we got a new response we need to cache.
                        debug!("Cache expires, remove expired cache files, refresh cache!");
                        new_response
                    }
                    Err(e) => {
                        warn!("Could not validate cached response: {}", e);
                        // Let's just use the existing data we have.
                        return Ok(self.root.join(p));
                    }
                }
            }
            Err(_) => {
                // This URL isn't in the cache, or we otherwise can't find it.
                self.client
                    .execute(reqwest::blocking::Request::new(
                        reqwest::Method::GET,
                        url.clone(),
                    ))?
                    .error_for_status()?
            }
        };
        self.record_response(url.as_str(), response)
    }

    /// Retrieve the content of the given URL.
    ///
    /// If we've never seen this URL before,
    /// we will try to retrieve it
    /// (with a `GET` request)
    /// and store its data locally.
    ///
    /// If we have seen this URL before, we will ask the server
    /// whether our cached data is stale.
    /// If our data is stale,
    /// we'll download the new version
    /// and store it locally.
    /// If our data is fresh,
    /// we'll re-use the local copy we already have.
    ///
    /// If we can't talk to the server to see if our cached data is stale,
    /// we'll silently re-use the data we have.
    ///
    /// Returns a file-handle to the local copy of the data, open for
    /// reading.
    ///
    ///     # extern crate reqwest;
    ///     # use std::error::Error;
    ///     # use std::fs::File;
    ///     # use std::path::PathBuf;
    ///     # fn get_my_resource() -> Result<(), Box<dyn Error>> {
    ///     # let mut cache = Cache::new(
    ///     #     PathBuf::from("my_cache_directory"),
    ///     # )?;
    ///     let file = cache.get(reqwest::Url::parse("http://example.com/some-resource")?)?;
    ///     # Ok(())
    ///     # }
    ///
    /// Errors
    /// ======
    ///
    /// This method may return an error:
    ///
    ///   - if the cache metadata is corrupt
    ///   - if the requested resource is not cached,
    ///     and we can't connect to/download it
    ///   - if we can't update the cache metadata
    ///   - if the cache metadata points to a local file that no longer exists
    ///
    /// After returning a network-related or disk I/O-related error,
    /// this `Cache` instance should be OK and you may keep using it.
    /// If it returns a database-related error,
    /// the on-disk storage *should* be OK,
    /// so you might want to destroy this `Cache` instance
    /// and create a new one pointing at the same location.
    pub fn get(&self, uri: &str) -> String {
        let mut cache_file = String::new();
        self.lock_file(uri, true);
        let url = reqwest::Url::parse(uri);
        match url {
            Ok(url) => {
                let response = self.load_cache(url);
                match response {
                    Ok(response) => {
                        cache_file = response.display().to_string();
                    }
                    Err(e) => {
                        error!("Fetch file {}", e);
                    }
                }
            }
            Err(e) => {
                error!("Bad Url: {}, {}", uri, e);
            }
        }
        self.lock_file(uri, false);
        cache_file
    }

    pub fn create_cached_file(&self, file_path: &str) -> String {
        let mut rng = thread_rng();
        let content_dir = self.root.join("upload");
        loop {
            let mut new_path = content_dir.join(
                (0..20)
                    .map(|_| rng.sample(Alphanumeric) as char)
                    .collect::<String>(),
            );

            let mut extension = "wav";
            let ext = path::Path::new(file_path).extension();
            if let Some(ext) = ext {
                let ext = ext.to_str();
                if let Some(ext) = ext {
                    extension = ext;
                }
            }

            new_path.set_extension(extension);

            if !new_path.exists() {
                return new_path.display().to_string();
            }
        }
    }

    pub fn close_cached_file(&self, url: &str, path: &str) -> Result<(), Box<dyn Error>> {
        self.db.set(
            url,
            CacheRecord {
                path: path.to_string(),
                last_modified: None,
                etag: None,
                synchronized: false,
            },
        )?;

        let ev = Event {
            done: false,
            path: path.to_string(),
            url: url.to_string(),
        };

        self.event.blocking_send(ev)?;
        Ok(())
    }
}
