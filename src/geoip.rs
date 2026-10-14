//! Optional MaxMind GeoIP lookups for hop addresses.

use anyhow::Context;
use itertools::Itertools;
use maxminddb::Reader;
use std::cell::RefCell;
use std::collections::HashMap;
use std::net::IpAddr;
use std::path::Path;
use std::rc::Rc;

/// The subset of a MaxMind city record the window shows.
#[derive(Debug, Clone, Default)]
pub struct City {
    latitude: Option<f64>,
    longitude: Option<f64>,
    accuracy_radius: Option<u16>,
    city: Option<String>,
    subdivision: Option<String>,
    subdivision_code: Option<String>,
    country: Option<String>,
    country_code: Option<String>,
    continent: Option<String>,
}

impl City {
    /// City, subdivision code and country code, e.g. `Los Angeles, CA, US`.
    pub fn short_name(&self) -> String {
        [
            self.city.as_ref(),
            self.subdivision_code.as_ref(),
            self.country_code.as_ref(),
        ]
        .into_iter()
        .flatten()
        .join(", ")
    }

    /// City, subdivision, country and continent.
    pub fn long_name(&self) -> String {
        [
            self.city.as_ref(),
            self.subdivision.as_ref(),
            self.country.as_ref(),
            self.continent.as_ref(),
        ]
        .into_iter()
        .flatten()
        .join(", ")
    }

    /// Latitude, longitude and accuracy radius.
    pub fn location(&self) -> String {
        format!(
            "{}, {} (~{}km)",
            self.latitude.unwrap_or_default(),
            self.longitude.unwrap_or_default(),
            self.accuracy_radius.unwrap_or_default(),
        )
    }

    /// The coordinates and accuracy radius, if known.
    pub const fn coordinates(&self) -> Option<(f64, f64, u16)> {
        match (self.latitude, self.longitude, self.accuracy_radius) {
            (Some(lat), Some(long), Some(radius)) => Some((lat, long, radius)),
            _ => None,
        }
    }
}

impl From<(maxminddb::geoip2::City<'_>, &str)> for City {
    fn from((value, locale): (maxminddb::geoip2::City<'_>, &str)) -> Self {
        let city = localized_name(&value.city.names, locale);
        let subdivision = value
            .subdivisions
            .first()
            .and_then(|c| localized_name(&c.names, locale));
        let subdivision_code = value
            .subdivisions
            .first()
            .and_then(|c| c.iso_code.map(ToString::to_string));
        Self {
            latitude: value.location.latitude,
            longitude: value.location.longitude,
            accuracy_radius: value.location.accuracy_radius,
            city,
            subdivision,
            subdivision_code,
            country: localized_name(&value.country.names, locale),
            country_code: value.country.iso_code.map(ToString::to_string),
            continent: localized_name(&value.continent.names, locale),
        }
    }
}

/// The fallback locale, as recommended by MaxMind.
const FALLBACK_LOCALE: &str = "en";

type Cache = RefCell<HashMap<IpAddr, Option<Rc<City>>>>;

/// Looks up [`City`] records, caching the results.
#[derive(Debug)]
pub struct Lookup {
    reader: Option<Reader<Vec<u8>>>,
    cache: Cache,
    locale: String,
}

impl Lookup {
    /// Opens a MaxMind database file.
    pub fn from_file(path: &Path) -> anyhow::Result<Self> {
        let reader = Reader::open_readfile(path).context(format!("{}", path.display()))?;
        Ok(Self {
            reader: Some(reader),
            cache: RefCell::new(HashMap::new()),
            locale: FALLBACK_LOCALE.to_string(),
        })
    }

    /// A lookup that returns `None` for every address.
    pub fn empty() -> Self {
        Self {
            reader: None,
            cache: RefCell::new(HashMap::new()),
            locale: FALLBACK_LOCALE.to_string(),
        }
    }

    /// Whether a database is loaded.
    pub const fn enabled(&self) -> bool {
        self.reader.is_some()
    }

    /// Looks up an address, caching the result.
    pub fn lookup(&self, addr: IpAddr) -> anyhow::Result<Option<Rc<City>>> {
        let Some(reader) = &self.reader else {
            return Ok(None);
        };
        if let Some(city) = self.cache.borrow().get(&addr) {
            return Ok(city.clone());
        }
        let city = reader
            .lookup(addr)?
            .decode::<maxminddb::geoip2::City<'_>>()?
            .map(|city| City::from((city, self.locale.as_str())))
            .map(Rc::new);
        self.cache.borrow_mut().insert(addr, city.clone());
        Ok(city)
    }
}

fn localized_name(names: &maxminddb::geoip2::Names<'_>, locale: &str) -> Option<String> {
    lookup_locale(names, locale)
        .or_else(|| lookup_locale(names, FALLBACK_LOCALE))
        .map(ToString::to_string)
}

/// Maps a locale code to the closest MaxMind name field.
fn lookup_locale<'a>(names: &maxminddb::geoip2::Names<'a>, code: &str) -> Option<&'a str> {
    if code.starts_with("pt") {
        names.brazilian_portuguese
    } else if code.starts_with("zh") {
        names.simplified_chinese
    } else {
        match code {
            "de" => names.german,
            "en" => names.english,
            "es" => names.spanish,
            "fr" => names.french,
            "ja" => names.japanese,
            "ru" => names.russian,
            _ => None,
        }
    }
}
