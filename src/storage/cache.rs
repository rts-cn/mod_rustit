use std::error;
use std::fs;
use std::io;
use std::path;
use std::path::Path;
use std::path::PathBuf;

use fsr::*;
use rand::distributions::Alphanumeric;
use rand::{thread_rng, Rng};
use redb::{ReadableTable, TableDefinition};
use reqwest::blocking::Client;
use reqwest::header as rh;
use serde::{Deserialize, Serialize};

/// All the information we have about a given URL.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct CacheRecord {
    /// The path to the cached response body on disk.
    pub path: String,
    /// The value of the Last-Modified header in the original response.
    pub last_modified: Option<String>,
    /// The value of the Etag header in the original response.
    pub etag: Option<String>,
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
        let json = String::from_utf8(data.to_vec()).unwrap_or_default();
        let record: Result<CacheRecord, serde_json::Error> = serde_json::from_str(&json);

        match record {
            Ok(record) => record,
            Err(_) => CacheRecord {
                path: "".to_string(),
                last_modified: None,
                etag: None,
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
    db: redb::Database,
}

const TABLE: TableDefinition<&str, CacheRecord> = TableDefinition::new("urls");

impl CacheDB {
    /// Create a cache database in the given file.
    pub fn new(path: path::PathBuf) -> Result<CacheDB, Box<dyn error::Error>> {
        let path = canonicalize_db_path(path)?;
        debug!("Creating cache metadata in {:?}", path);
        let db = redb::Database::create(path.clone())?;
        // Package up the return value first, so we can use .query()
        // instead of wrangling sqlite directly.
        let res = CacheDB { path, db };
        Ok(res)
    }

    /// Return what the DB knows about a URL, if anything.
    pub fn get(&self, mut url: reqwest::Url) -> Result<CacheRecord, Box<dyn error::Error>> {
        url.set_fragment(None);

        let read_txn = self.db.begin_read()?;
        let table = read_txn.open_table(TABLE)?;
        let key = url.as_str();
        let value = table.get(key)?.unwrap().value();
        Ok(value)
    }

    /// Record information about this information in the database.
    pub fn set(
        &mut self,
        mut url: reqwest::Url,
        record: CacheRecord,
    ) -> Result<redb::WriteTransaction, Box<dyn error::Error>> {
        url.set_fragment(None);

        let write_txn = self.db.begin_write()?;
        {
            let mut table = write_txn.open_table(TABLE)?;
            table.insert(url.as_str(), &record)?;
        }
        Ok(write_txn)
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
/// See [an example](index.html#first-example).
///
/// [`Cache`]: struct.Cache.html
#[derive(Debug)]
pub struct Cache {
    root: path::PathBuf,
    db: CacheDB,
    client: reqwest::blocking::Client,
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
    ///         reqwest::blocking::Client::new(),
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
    pub fn new(root: path::PathBuf, client: Client) -> Result<Cache, Box<dyn error::Error>> {
        fs::DirBuilder::new().recursive(true).create(&root)?;

        let db = CacheDB::new(root.join("cache.db"))?;

        Ok(Cache { root, db, client })
    }

    fn record_response(
        &mut self,
        url: reqwest::Url,
        response: &reqwest::blocking::Response,
    ) -> Result<(fs::File, path::PathBuf, redb::WriteTransaction), Box<dyn error::Error>> {
        let content_dir = self.root.join("content");
        fs::DirBuilder::new().recursive(true).create(&content_dir)?;

        let mut extension = "wav";
        let ext = Path::new(url.path()).extension();
        if let Some(ext) = ext {
            let ext = ext.to_str();
            if let Some(ext) = ext {
                extension = ext;
            }
        }

        let (handle, path) = make_random_file(&content_dir, extension)?;
        let trans = {
            // We can be sure the relative path is valid UTF-8, because
            // make_random_file() just generated it from ASCII.
            let path = path.strip_prefix(&self.root)?.to_str().unwrap().into();

            let last_modified = header_as_string(response.headers(), &rh::LAST_MODIFIED);

            let etag = header_as_string(response.headers(), &rh::ETAG);

            self.db.set(
                url,
                CacheRecord {
                    path,
                    last_modified,
                    etag,
                },
            )?
        };

        Ok((handle, path, trans))
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
    ///     #     reqwest::blocking::Client::new(),
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
    pub fn get(&mut self, mut url: reqwest::Url) -> Result<PathBuf, Box<dyn error::Error>> {
        use reqwest::StatusCode;

        url.set_fragment(None);

        let mut response = match self.db.get(url.clone()) {
            Ok(CacheRecord {
                path: p,
                last_modified: lm,
                etag: et,
            }) => {
                // We have a locally-cached copy, let's check whether the
                // copy on the server has changed.
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

                info!("Sending HTTP request: {:?}", request);

                let maybe_validation = self
                    .client
                    .execute(request)
                    .and_then(|resp| resp.error_for_status());

                match maybe_validation {
                    Ok(new_response) => {
                        info!("Got HTTP response: {:?}", new_response);

                        // If our existing cached data is still fresh...
                        if new_response.status() == StatusCode::NOT_MODIFIED {
                            // ... let's use it as is.
                            return Ok(self.root.join(p));
                        }

                        // Otherwise, we got a new response we need to cache.
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

        let (mut handle, path, trans) = self.record_response(url.clone(), &response)?;

        let count = io::copy(&mut response, &mut handle)?;

        debug!("Downloaded {} bytes", count);

        trans.commit()?;

        Ok(path)
    }

    pub fn load_cache_file(&mut self, url: &str) -> String {
        let url = reqwest::Url::parse(url);
        match url {
            Ok(url) => {
                let response = self.get(url);
                match response {
                    Ok(response) => {
                        return response.to_str().unwrap_or_default().to_string();
                    }
                    Err(e) => {
                        error!("{}", e);
                    }
                }
            }
            Err(e) => {
                error!("{}", e);
            }
        }
        "".to_string()
    }
}
